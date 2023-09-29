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
#include "libwbfs.h"
static int just_a_partition = 0;
static int wbfs = 0;
static int dump_partition_data = 0;
static u32 max_size_to_auto_analyse = 0x1000000;
static int uncompress_yaz0 = 1;
static int unpack_rarc = 1;

static FILE *disc_fp;
static FILE *disc_fp2;

static u64 partition_raw_offset;
static u64 partition_data_offset;
static u64 partition_data_size;
static u8 h3[0x18000];

u8 disc_key[16];
static char* extract_one_file = 0;
static u32 errors = 0;
static int try_read_partition=0;
wbfs_disc_t *wbfs_disc;
static void seek(u64 offset)
{
	if (fseeko(disc_fp, offset, SEEK_SET))
		fatal("error seeking in disc file");
}
static void disc_read(u64 offset, u8 *data, u32 len)
{
        if(wbfs){
                wbfs_disc_read(wbfs_disc,offset/4, data, len);

        }
        else{
                seek(offset);
                if (fread(data, len, 1, disc_fp) != 1 && !try_read_partition)
                        fatal("error reading disc");
        }
}
static int read_sector(void *_fp,u32 lba,u32 count,void*buf)
{
        FILE*fp =_fp;
//        printf("read sector %X %X\n",lba,count);
	if (fseeko(fp, lba*512ULL, SEEK_SET))
		fatal("error seeking in disc file");
        if (fread(buf, count*512ULL, 1, fp) != 1)
                fatal("error reading disc");
//        hexdump(buf,count*512);
        return 0;
  
}
static void partition_raw_read(u64 offset, u8 *data, u32 len)
{
	disc_read(partition_raw_offset + offset, data, len);
}

static void partition_read_block(u64 blockno, u8 *block)
{
	u8 raw[0x8000];
	u8 iv[16];
	u8 h[20];
	u8 *h0, *h1, *h2;
	u32 b1, b2, b3;
	u64 offset;
	u32 i;

	offset = partition_data_offset + 0x8000 * blockno;
	partition_raw_read(offset, raw, 0x8000);

	// decrypt data
	memcpy(iv, raw + 0x3d0, 16);
	aes_cbc_dec(disc_key, iv, raw + 0x400, 0x7c00, block);

	// decrypt hashes
	memset(iv, 0, 16);
	aes_cbc_dec(disc_key, iv, raw, 0x400, raw);

	h0 = raw;
	h1 = raw + 0x280;
	h2 = raw + 0x340;
	b1 = blockno & 7;
	b2 = (blockno >> 3) & 7;
	b3 = blockno >> 6;

	// check H0s
	for (i = 0; i < 31; i++) {
		sha(block + 0x400*i, 0x400, h);
		if (memcmp(h0 + 20*i, h, 20)) {
			//fprintf(stderr, "H0 mismatch for %llx.%02x\n", blockno, i);
			errors |= 1;
		}
	}

	// check H1
	sha(h0, 620, h);
	if (memcmp(h1 + 20*b1, h, 20)) {
		fprintf(stderr, "H1 mismatch for %llx\n", blockno);
		errors |= 2;
	}

	// check H2
	sha(h1, 160, h);
	if (memcmp(h2 + 20*b2, h, 20)) {
		fprintf(stderr, "H2 mismatch for %llx\n", blockno);
		errors |= 4;
	}

	// check H3
	sha(h2, 160, h);
	if (memcmp(h3 + 20*b3, h, 20)) {
		fprintf(stderr, "H3 mismatch for %llx\n", blockno);
		errors |= 8;
	}

/*        if(errors&0xf)
                fatal("stop");
*/}

static void partition_read(u64 offset, u8 *data, u32 len)
{
	u8 block[0x8000];
	u32 offset_in_block;
	u32 len_in_block;

	if (just_a_partition)
		disc_read(offset, data, len);
	else while(len) {
		offset_in_block = offset % 0x7c00;
		len_in_block = 0x7c00 - offset_in_block;
		if (len_in_block > len)
			len_in_block = len;
		partition_read_block(offset / 0x7c00, block);
		memcpy(data, block + offset_in_block, len_in_block);
		data += len_in_block;
		offset += len_in_block;
		len -= len_in_block;
	}
}


static void do_data(u64 size)
{
	u8 data[0x7c00];
	u64 offset;
	u64 remaining_size;
	u32 block_size;
	FILE *fp;

	size = (size / 0x8000) * 0x7c00;

	fp = fopen("###dat###", "wb");
	if (fp == 0)
		fatal("cannot open partition output file");

	fprintf(stderr, "\nDumping partition contents...\n");
	offset = 0;
	remaining_size = size;
	while (remaining_size) {
		spinner(offset, size);

		block_size = 0x7c00;
		if (block_size > remaining_size)
			block_size = remaining_size;

		partition_read(offset, data, block_size);
		if (fwrite(data, block_size, 1, fp) != 1)
			fatal("error writing partition");

		offset += block_size;
		remaining_size -= block_size;
	}
	spinner(0, 0);

	fclose(fp);
}

static void copy_file(const char *name, u64 offset, u64 size)
{
	u8 data[0x80000];
	FILE *fp;
	u32 block_size;

	fp = fopen(name, "wb");
	if (fp == 0)
		fatal("cannot open output file");

	while (size) {
		block_size = sizeof data;
		if (block_size > size)
			block_size = size;

		partition_read(offset, data, block_size);
		if (fwrite(data, block_size, 1, fp) != 1)
			printf("error writing output file");

		offset += block_size;
		size -= block_size;
	}

	fclose(fp);
}

static void do_fst_file(const char *name, u64 offset, u64 size)
{
	FILE *fp;
	u8 *data;

	if (size > max_size_to_auto_analyse) {
		copy_file(name, offset, size);

		return;
	}

	data = malloc(size);
	if (data == 0)
		fatal("malloc");
	partition_read(offset, data, size);

	if (uncompress_yaz0 && size >= 8 && memcmp(data, "Yaz0", 4) == 0) {
		u8 *dec;
		u32 dec_size;

		fprintf(stderr, " [Yaz0]");

		dec_size = be32(data + 4);
		dec = malloc(dec_size);
		if (dec == 0)
			fatal("malloc");

		do_yaz0(data, size, dec, dec_size);

		free(data);
		data = dec;
		size = dec_size;
	}

	if (unpack_rarc && size >= 8 && memcmp(data, "RARC", 4) == 0) {
		fprintf(stderr, " [RARC]");
	}

	fp = fopen(name, "wb");
	if (fp == 0)
		fatal("cannot open output file");
	if (size!=0 && fwrite(data, size, 1, fp) != 1){
                fatal("error writing output file \n");

        }
	fclose(fp);

	free(data);
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
		fprintf(stderr, "/\n");

		for (j = 1; j < size; )
			j = do_fst(fst, names, j, indent, (j == size - 1));

		return size;
	}

	if (fst[12*i]) {
		parent = be32(fst + 12*i + 4);
		is_last = (be32(fst + 12*parent + 8) == size);
	}

	fprintf(stderr, "%s%c-- %s", indent, "|+"[is_last], name);

	if (fst[12*i]) {
                if(!extract_one_file){
                        if (mkdir(name, 0777))
                                fatal("mkdir");
                        if (chdir(name))
                                fatal("chdir");
                }
		fprintf(stderr, "\n");

		if (is_last)
			strcat(indent, "    ");
		else
			strcat(indent, "|   ");

		for (j = i + 1; j < size; )
			j = do_fst(fst, names, j, indent, (j == size - 1));

		indent[strlen(indent) - 4] = 0;

                if(!extract_one_file){
                        if (chdir(".."))
                                fatal("chdir up");
                }
		return size;
	} else {
		offset = be34(fst + 12*i + 4);
                if(!extract_one_file || strcmp(name,extract_one_file)==0){
                        do_fst_file(name, offset, size);
                }

		fprintf(stderr, "\n");

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

	partition_read(0, b, sizeof b);

	fprintf(stderr, "\n");
	fprintf(stderr, "Title id: %c%c%c%c\n", b[0], b[1], b[2], b[3]);
	fprintf(stderr, "Group id: %c%c\n", b[4], b[5]);
	fprintf(stderr, "Name: %s\n", b + 0x20);
	fprintf(stderr, "\n");

	dol_offset = be34(b + 0x0420);
	fst_offset = be34(b + 0x0424);
	fst_size = be34(b + 0x0428);

	apl_offset = 0x2440;
	partition_read(apl_offset, apl_header, sizeof apl_header);
	apl_size = 0x20 + be32(apl_header + 0x14) + be32(apl_header + 0x18);

	fprintf(stderr, "\tDOL @ %09llx\n", dol_offset);
	fprintf(stderr, "\tFST @ %09llx (size %08x)\n", fst_offset, fst_size);
	fprintf(stderr, "\tAPL @ %09llx (size %08x)\n", apl_offset, apl_size);

	copy_file("###apl###", apl_offset, apl_size);
	copy_file("###dol###", dol_offset, fst_offset - dol_offset);
		// XXX: read the header for getting the size

	fst = malloc(fst_size);
	if (fst == 0)
		fatal("malloc fst");
	partition_read(fst_offset, fst, fst_size);
	n_files = be32(fst + 8);

	fprintf(stderr, "%d entries\n", n_files);
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
	u8 h[20];
	u32 ret;
	char dirname[17];

        try_read_partition=1;
	// read ticket, and read some offsets and sizes
	partition_raw_read(0, tik, sizeof tik);
        memset(b,0, sizeof b);
	partition_raw_read(0x2a4, b, sizeof b);

        try_read_partition=1;

	tmd_size = be32(b);
        if (tmd_size==0){
          fprintf(stderr, "partition removed\n");
          return;
        }
	tmd_offset = be34(b + 4);
	cert_size = be32(b + 8);
	cert_offset = be34(b + 0x0c);
	h3_offset = be34(b + 0x10);
	partition_data_offset = be34(b + 0x14);
	partition_data_size = be34(b + 0x18);

	title_id = be64(tik + 0x01dc);
	fprintf(stderr, "\ttitle id = %016llx\n", title_id);

	fprintf(stderr, "\ttmd size    =  %08x\n", tmd_size);
	fprintf(stderr, "\ttmd offset  = %09llx\n", tmd_offset);
	fprintf(stderr, "\tcert size   =  %08x\n", cert_size);
	fprintf(stderr, "\tcert offset = %09llx\n", cert_offset);
	fprintf(stderr, "\tdata offset = %09llx\n", partition_data_offset);
	fprintf(stderr, "\tdata size   = %09llx %.3fGb\n", partition_data_size,(double)partition_data_size/(1024*1024*1024.));
//        return;
	tmd = malloc(tmd_size);
	if (tmd == 0)
		fatal("malloc tmd");
	partition_raw_read(tmd_offset, tmd, tmd_size);

	cert = malloc(cert_size);
	if (cert == 0)
		fatal("malloc cert");
	partition_raw_read(cert_offset, cert, cert_size);

	ret = check_cert_chain(tik, sizeof tik, cert, cert_size);
	if (ret) {
		fprintf(stderr, "ticket cert failure (%d)\n", ret);
		errors |= 0x20;
	}
	ret = check_cert_chain(tmd, tmd_size, cert, cert_size);
	if (ret) {
		fprintf(stderr, "tmd cert failure (%d)\n", ret);
		errors |= 0x40;
	}

	decrypt_title_key(tik, disc_key);

	partition_raw_read(h3_offset, h3, 0x18000);

	sha(h3, 0x18000, h);
	if (memcmp(tmd + 0x1f4, h, 20)) {
		fprintf(stderr, "H4 mismatch\n");
		errors |= 0x10;
	}

	free(cert);
	free(tmd);

	snprintf(dirname, sizeof dirname, "%016llx", title_id);
        
        if(!extract_one_file){
                if (mkdir(dirname, 0777))
                        fatal("mkdir partition");
                if (chdir(dirname))
                        fatal("chdir partition");
        }
	if (dump_partition_data)
		do_data(partition_data_size);

	do_files();

	if (chdir(".."))
		fatal("chdir up out of partition");
}

static void do_disc(void)
{
	u8 b[0x100];
	u64 partition_offset[32]; // XXX: don't know the real maximum
	u64 partition_type[32]; // XXX: don't know the real maximum
	u32 n_partitions;
	u32 i;

	disc_read(0, b, sizeof b);

	fprintf(stderr, "Title id: %c%c%c%c\n", b[0], b[1], b[2], b[3]);
	fprintf(stderr, "Group id: %c%c\n", b[4], b[5]);
	fprintf(stderr, "Name: %s\n", b + 0x20);
	fprintf(stderr, "\n");

	disc_read(0x40000, b, sizeof b);
	n_partitions = be32(b);

	disc_read(be34(b + 4), b, sizeof b);
	for (i = 0; i < n_partitions; i++){
		partition_offset[i] = be34(b + 8 * i);
		partition_type[i] = be32(b + 8 * i+4);
        }
	fprintf(stderr, "%d partitions:\n", n_partitions);
	for (i = 0; i < n_partitions; i++)
		fprintf(stderr, "\tpartition #%d @ %09llx\n", i,
		        partition_offset[i]);

	for (i = 0; i < n_partitions; i++) {
		fprintf(stderr, "\nDoing partition %d...\n", i);
		fprintf(stderr, "--------------------------------\n");

		partition_raw_offset = partition_offset[i];
                //if(partition_type[i]!=1 || wbfs==0)//dont copy update partition if wbfs
                do_partition();

		//break; // XXX SII: for testing
	}
}

int main(int argc, char **argv)
{
        wbfs_t *wbfs_p=0;
	if (argc != 2 && argc != 3 && argc != 4) {
		fprintf(stderr, "Usage: %s <disc file> %d\n", argv[0],argc);
		return 1;
	}
        if (argc >= 3)
                wbfs = 1;
        if (argc >= 4)
                extract_one_file = argv[3];

        disc_fp = fopen(argv[1], "rb");
        disc_fp2 = fopen("/backup/ptar01/wbfs/test.iso", "rb");
        if (disc_fp == 0)
          fatal("open disc file");
        if(wbfs){
          wbfs_p = wbfs_open_partition(read_sector,NULL,disc_fp,0,0,0,0);
                wbfs_disc = wbfs_open_disc(wbfs_p,(u8*)argv[2]);
        }
	if (just_a_partition)
		do_files();
	else
		do_disc();
        
        if(wbfs){
                wbfs_close_disc(wbfs_disc);
                wbfs_close(wbfs_p);
        }
        fclose(disc_fp);

	if (errors)
		fprintf(stderr, "\n\nErrors detected:\n");
	if (errors & 1)
		fprintf(stderr, "H0 mismatch\n");
	if (errors & 2)
		fprintf(stderr, "H1 mismatch\n");
	if (errors & 4)
		fprintf(stderr, "H2 mismatch\n");
	if (errors & 8)
		fprintf(stderr, "H3 mismatch\n");
	if (errors & 0x10)
		fprintf(stderr, "H4 mismatch\n");
	if (errors & 0x20)
		fprintf(stderr, "ticket cert failure\n");
	if (errors & 0x40)
		fprintf(stderr, "tmd cert failure\n");

	return 0;
}
