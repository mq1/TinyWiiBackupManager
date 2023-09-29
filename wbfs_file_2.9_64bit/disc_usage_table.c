// Copyright 2007,2008  Segher Boessenkool  <segher@kernel.crashing.org>
// Licensed under the terms of the GNU GPL, version 2
// http://www.gnu.org/licenses/old-licenses/gpl-2.0.txt

#include <sys/types.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

#include "tools.h"
#include "platform.h"

static FILE *disc_fp;

static u64 disc_raw_offset=0;
static u64 partition_raw_offset;
static u64 partition_data_offset;
static u64 partition_data_size;
static u64 partition_block;
static u8 h3[0x18000];

static u8 *used;

u8 disc_key[16];
int dont_decrypt = 0;

static void seek(u64 offset)
{
	if (fseeko(disc_fp, disc_raw_offset+offset, SEEK_SET))
		fatal("error seeking in disc file");
}

static void disc_read(u64 offset, u8 *data, u32 len)
{
        u32 blockno = offset>>15;
        used[blockno]=1;
	seek(offset);
	if (fread(data, len, 1, disc_fp) != 1)
		fatal("error reading disc");
}

static void partition_raw_read(u64 offset, u8 *data, u32 len)
{
	disc_read(partition_raw_offset + offset, data, len);
}

static void partition_read_block(u64 blockno, u8 *block)
{
	u8 raw[0x8000];
	u8 iv[16];
	u64 offset;
        used[partition_block+blockno]=1;
        offset = partition_data_offset + 0x8000 * blockno;
        partition_raw_read(offset, raw, 0x8000);

        // decrypt data
        memcpy(iv, raw + 0x3d0, 16);
        aes_cbc_dec(disc_key, iv, raw + 0x400, 0x7c00, block);
}

static void partition_read(u64 offset, u8 *data, u32 len,int fake)
{
	u8 block[0x8000];
	u32 offset_in_block;
	u32 len_in_block;

	while(len) {
		offset_in_block = offset % 0x7c00;
		len_in_block = 0x7c00 - offset_in_block;
		if (len_in_block > len)
			len_in_block = len;
                if(!fake){
                        partition_read_block(offset / 0x7c00, block);
                        memcpy(data, block + offset_in_block, len_in_block);
                }else
                        used[partition_block+(offset/0x7c00)]=1;
		data += len_in_block;
		offset += len_in_block;
		len -= len_in_block;
	}
}


static void copy_file(u64 offset, u64 size)
{
	u8 data[0x80000];
	u32 block_size;


	while (size) {
		block_size = sizeof data;
		if (block_size > size)
			block_size = size;

		partition_read(offset, data, block_size,1);

		offset += block_size;
		size -= block_size;
	}

}


static u32 do_fst(u8 *fst, const char *names, u32 i, char *indent, int is_last)
{
	u64 offset;
	u32 size;
	const char *name;
	u32 parent;
	u32 j;

	name = names + (be32(fst + 12*i) & 0x00ffffff);
	size = be32(fst + 12*i + 8);

	if (i == 0) {
		for (j = 1; j < size; )
			j = do_fst(fst, names, j, indent, (j == size - 1));
		return size;
	}

	if (fst[12*i]) {
		parent = be32(fst + 12*i + 4);
		is_last = (be32(fst + 12*parent + 8) == size);
	}


	if (fst[12*i]) {

		for (j = i + 1; j < size; )
			j = do_fst(fst, names, j, indent, (j == size - 1));

		indent[strlen(indent) - 4] = 0;
		return size;
	} else {
		offset = be34(fst + 12*i + 4);
		copy_file(offset, size);
		return i + 1;
	}
}

static void do_files(void)
{
	u8 b[0x480]; // XXX: determine actual header size
	u64 dol_offset;
	u64 fst_offset;
	u32 fst_size;
	u64 apl_offset;
	u32 apl_size;
	u8 apl_header[0x20];
	u8 *fst;
	char indent[999];
	u32 n_files;

	partition_read(0, b, sizeof b,0);

	dol_offset = be34(b + 0x0420);
	fst_offset = be34(b + 0x0424);
	fst_size = be34(b + 0x0428);

	apl_offset = 0x2440;
	partition_read(apl_offset, apl_header, sizeof apl_header,0);
	apl_size = 0x20 + be32(apl_header + 0x14) + be32(apl_header + 0x18);
	copy_file(apl_offset, apl_size);
	copy_file(dol_offset, fst_offset - dol_offset);
		// XXX: read the header for getting the size

	fst = malloc(fst_size);
	if (fst == 0)
		fatal("malloc fst");
	partition_read(fst_offset, fst, fst_size,0);
	n_files = be32(fst + 8);

	indent[0] = 0;
	if (n_files > 1)
		do_fst(fst, (char *)fst + 12*n_files, 0, indent, 0);

	free(fst);
}

static void do_partition(void)
{
	u8 tik[0x2a4];
	u8 b[0x1c];
	u64 title_id;
	u64 tmd_offset;
	u32 tmd_size;
	u8 *tmd;
	u64 cert_offset;
	u32 cert_size;
	u8 *cert;
	u64 h3_offset;

	// read ticket, and read some offsets and sizes
	partition_raw_read(0, tik, sizeof tik);
	partition_raw_read(0x2a4, b, sizeof b);

	tmd_size = be32(b);
	tmd_offset = be34(b + 4);
	cert_size = be32(b + 8);
	cert_offset = be34(b + 0x0c);
	h3_offset = be34(b + 0x10);
	partition_data_offset = be34(b + 0x14);
	partition_data_size = be34(b + 0x18);
        partition_block = (partition_raw_offset+partition_data_offset)>>15;
	title_id = be64(tik + 0x01dc);
	tmd = malloc(tmd_size);
	if (tmd == 0)
		fatal("malloc tmd");
	partition_raw_read(tmd_offset, tmd, tmd_size);

	cert = malloc(cert_size);
	if (cert == 0)
		fatal("malloc cert");
	partition_raw_read(cert_offset, cert, cert_size);


	decrypt_title_key(tik, disc_key);

	partition_raw_read(h3_offset, h3, 0x18000);

	free(cert);
	free(tmd);

	do_files();

}

static void do_disc(void)
{
	u8 b[0x100];
	u64 partition_offset[32]; // XXX: don't know the real maximum
	u64 partition_type[32]; // XXX: don't know the real maximum
	u32 n_partitions;
        u32 magic;
	u32 i;

	disc_read(0, b, sizeof b);
        magic=be32(b+24);
        if(magic!=0x5D1C9EA3)
                return ;
	//fprintf(stderr, "Name: %s\t\t", b + 0x20);
        //fflush(stderr);
        
	disc_read(0x40000, b, sizeof b);
	n_partitions = be32(b);

	disc_read(be34(b + 4), b, sizeof b);
	for (i = 0; i < n_partitions; i++){
		partition_offset[i] = be34(b + 8 * i);
		partition_type[i] = be32(b + 8 * i+4);
        }
	for (i = 0; i < n_partitions; i++) {
                partition_raw_offset = partition_offset[i];
                if(partition_type[i]!=1)//dont copy update partition
                {
                        do_partition();
                }
                else partition_read(0, 0, 0x8000,1);// fake read partition header, just in case it is needed

		//break; // XXX SII: for testing
	}
}
int build_disc_usage_table(FILE* fp,u8*table, u64 offset)
{
        disc_fp = fp;
        used = table;
        disc_raw_offset = offset;
        memset(used,0,143432);
        do_disc();
        return 0;
}
