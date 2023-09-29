// Copyright 2009 Kwiirk
// Licensed under the terms of the GNU GPL, version 2
// http://www.gnu.org/licenses/old-licenses/gpl-2.0.txt

#include <stdio.h>     /* for printf */
#include <stdlib.h>    /* for exit */
#include <getopt.h>
#include <sys/stat.h>
#include <unistd.h>
#include <string.h>
#include <ctype.h>

#include "tools.h"
#include "libwbfs.h"

#include "platform.h"

// 4gb - 32kb (1 wii sector)
#define SPLIT_SIZE_4 ((u64)4LL * 1024 * 1024 * 1024 - 32 * 1024)

// 2gb - 32kb (1 wii sector)
#define SPLIT_SIZE_2 ((u64)2LL * 1024 * 1024 * 1024 - 32 * 1024)

// no split
#define SPLIT_SIZE_0 ((u64)10000000000LL)

#define DEF_SPLIT_SIZE SPLIT_SIZE_4

extern u64 OPT_split_size;
extern int OPT_split_verbose;

#define MAX_SPLIT 10

typedef struct split_info
{
	char fname[1024];
	FILE *f[MAX_SPLIT];
	u32 split_sec;
	u32 total_sec;
	u64 split_size;
	u64 total_size;
	int create_mode;
	int max_split;
} split_info_t;

split_info_t* split_new();
void split_delete(split_info_t*s);
void split_get_fname(split_info_t *s, int idx, char *fname);
FILE *split_open_file(split_info_t *s, int idx);
int  split_fill(split_info_t *s, int idx, off64_t size);
FILE *split_get_file(split_info_t *s, u32 lba, u32 *sec_count, int fill);
int  split_read_sector(void *_fp,u32 lba,u32 count,void*buf);
int  split_write_sector(void *_fp,u32 lba,u32 count,void*buf);
void split_init(split_info_t *s, char *fname);
void split_set_size(split_info_t *s, u64 split_size, u64 total_size);
int  split_create(split_info_t *s, char *fname, u64 split_size, u64 total_size);
int  split_open(split_info_t *s, char *fname);
int  split_truncate(split_info_t *s, off64_t full_size);
void split_close(split_info_t *s);

