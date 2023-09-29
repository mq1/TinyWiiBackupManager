#ifndef LIBWBFS_H
#define LIBWBFS_H

#ifdef __cplusplus
   extern "C" {
#endif /* __cplusplus */


#define likely(x)       __builtin_expect(!!(x), 1)
#define unlikely(x)     __builtin_expect(!!(x), 0)
 
#define blk_sz 512
#define blk_sz_shift 7
#define off_mask ((1<<blk_sz_shift)-1)

typedef struct wbfs_head
{
        u32 magic;
        u32 nsblk;	 // total number of 512B block in this partition
        u8  blk_sz_log2; // number of 32KB blocks
        u8  padding3[3];
        u8  disc_table[512-(4+4+4)];
}wbfs_head_t;
#define MAX_MAXDISC (512-4+4+4)

#define MAGIC (('W'<<24)|('B'<<16)|('F'<<8)|('S'))
#define GB (1024*1024*1024.)


void wbfs_open(void (*read_sector)(u32 lba,u32 count,void*buf));
void wbfs_open_disc(u8*id);
void wbfs_disc_read(u32 offset, u8 *data, u32 len);
u32 wbfs_get_list( u8 **headers,u32 num);

#ifdef __cplusplus
   }
#endif /* __cplusplus */

#endif
