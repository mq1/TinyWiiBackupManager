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

static u8 used[143432];

int build_disc_usage_table(FILE* fp,u8*table, u64 offset);

int main(int argc, char **argv)
{
        int i;
        FILE* disc_fp;
        FILE* disc2_fp;
        char buf[0x80000];
	if (argc != 3) {
		fprintf(stderr, "Usage: %s <disc file> <out>\n", argv[0]);
		return 1;
	}

	disc_fp = fopen(argv[1], "rb");
	if (disc_fp == 0)
		fatal("open disc file");
	disc2_fp = fopen(argv[2], "wb");
	if (disc2_fp == 0)
		fatal("open disc file");
        
        build_disc_usage_table(disc_fp,used, 0);
        fseeko(disc2_fp,4699979775ULL,SEEK_SET);
        fwrite("\0",1,1,disc2_fp);//write 0 byte at the end to force file size
        for (i=0;i<143432;i++)
        {
                if(used[i]){
                        if(i==0 || !used[i-1]){
                                fseeko(disc_fp,0x8000ULL*i,SEEK_SET);
                                fseeko(disc2_fp,0x8000ULL*i,SEEK_SET);
                        }
                        fread(buf,0x8000,1,disc_fp);
                        if(fwrite(buf,0x8000,1,disc2_fp)!=1)
                                fatal("writing");
                        spinner(i,143432);
                }
        }
        return 0;
}
