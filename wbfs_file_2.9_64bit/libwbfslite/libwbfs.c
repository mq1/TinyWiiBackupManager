// Copyright 2009 Kwiirk
// Licensed under the terms of the GNU GPL, version 2
// http://www.gnu.org/licenses/old-licenses/gpl-2.0.txt

#include "libwbfs_glue.h" // build dependant file

#include "libwbfs.h"

static wbfs_head_t *head;

static u8  wblk_sz_shift;
static u8  iwlba_shift;
static u32 nsblk;
static u32 lba_mask;
static u32 nblk;
static u32 disc_nblk;
static u32 max_disc;
static u32 table_size;
static u32 header_size;
static u8  *header;
static u8  *tmp_buffer;
static u32 freeblks_o;
static u32 freeblks_size;

static void (*read_sectors)(u32 lba,u32 num_sector,void *buf);

#define D(a,b) a=b
#define ALIGN512(x) (((x)+511)&~511)
static void wbfs_load_header(void)
{
        head = wbfs_malloc(sizeof(*head));
        read_sectors(0,1,head);
        if(wbfs_ntohl(head->magic)!=MAGIC)
                wbfs_fatal("not a WBFS partition!");

        D(  wblk_sz_shift,     head->blk_sz_log2-2+15		);
        D(  iwlba_shift,     head->blk_sz_log2-9+15		);
        D(  lba_mask,	     ((1<<(wblk_sz_shift-blk_sz_shift))-1));
        D(  nsblk ,	     wbfs_ntohl(head->nsblk)			);
        D(  nblk ,	     nsblk*512LL/(blk_sz*0x8000)	);
        D(  freeblks_size,   ALIGN512(nblk/8)			);
        D(  disc_nblk,	     (143432/(1<<head->blk_sz_log2))+1	);
        D(  table_size,	     (disc_nblk)*2			);
        D(  header_size,     ALIGN512(table_size+0x100)		);
        D(  freeblks_o,      blk_sz*0x8000-freeblks_size	); // end of first block
        D(  max_disc,	     (freeblks_o-512)/header_size	);

        if(max_disc > MAX_MAXDISC)
                max_disc = MAX_MAXDISC;

        header = wbfs_malloc(header_size);
        tmp_buffer = wbfs_malloc(blk_sz);
}
void wbfs_open(void (*_read_sectors)(u32 lba,u32 count,void*buf))
{
        read_sectors = _read_sectors;

        wbfs_load_header();

}
u32 wbfs_get_list( u8 **headers,u32 num)
{
        u32 i;
        u32 count = 0;
        for(i=0;i<max_disc;i++)
        {
                if (head->disc_table[i])
                {
                        read_sectors(1+i*header_size/512,header_size/512,header);
                        u32 magic=wbfs_be32(header+24);
                        if(magic!=0x5D1C9EA3)
                                continue;
                        headers[i] = wbfs_malloc(header_size);
                        wbfs_memcpy(headers[i],header,header_size);
                        if(++count == num)
                                return count;
                }
        }
        return count;
}
void wbfs_open_disc(u8*discid)
{
        u32 i;
        for(i=0;i<max_disc;i++)
        {
                if (head->disc_table[i]){
                        read_sectors(1+i*header_size/512,1,tmp_buffer);
                        u32 magic=wbfs_be32(tmp_buffer+24);
                        if(magic!=0x5D1C9EA3)
                                continue;
                        if(wbfs_memcmp(discid,tmp_buffer,6)==0){
                                read_sectors(1+i*header_size/512,header_size/512,header);
                                return;
                        }
                }
        }
        
}
#define LOOKUP_READ(head,blk)  wbfs_be16(head+0x100+blk*2)

/* read a wiidisc with offset addressing words! 

 offset is seen like:
 -----------------------------------------------------------
|      wlba        |          lba              |  off       |
 -----------------------------------------------------------

*/

void wbfs_disc_read(u32 offset, u8 *data, u32 len)
{
        u16 wlba = offset>>(wblk_sz_shift);
        u32 lba = (offset>>(blk_sz_shift))&(lba_mask);
        u32 off = offset&(off_mask);
        u16 iwlba = LOOKUP_READ(header,wlba);
        u32 len_copied;
        u8  *ptr = data;
        if(unlikely(off)){
                off*=4;
                read_sectors((iwlba<<iwlba_shift) + lba, 1, tmp_buffer);
                len_copied = blk_sz - off;
                if(likely(len < len_copied))
                        len_copied = len;
                wbfs_memcpy(ptr, tmp_buffer + off, len_copied);
                len -= len_copied;
                ptr += len_copied;
                lba++;
                if(unlikely(lba>lba_mask && len)){
                        lba=0;
                        wlba++;
                        iwlba = LOOKUP_READ(header,wlba);
                }
        }
        while(likely(len>=blk_sz))
        {
                u32 nlba = len>>(blk_sz_shift+2);
                
                if(unlikely(nlba > 1+lba_mask-lba)) // dont cross wide block boundary..
                        nlba = 1+lba_mask-lba;
                read_sectors((iwlba<<iwlba_shift) + lba, nlba, ptr);
                len -= nlba<<(blk_sz_shift+2);
                ptr += nlba<<(blk_sz_shift+2);
                lba += nlba;
                if(unlikely(lba>lba_mask && len)){
                        lba = 0;
                        iwlba = LOOKUP_READ(header,++wlba);
                }
        }
        if(unlikely(len)){
                read_sectors((iwlba<<iwlba_shift) + lba, 1, tmp_buffer);
                wbfs_memcpy(ptr, tmp_buffer, len);
        }
}
