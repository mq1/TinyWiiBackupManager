// Copyright 2009 oggzee
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
#include "splits.h"

#include "platform.h"

u64 OPT_split_size = DEF_SPLIT_SIZE;
extern int OPT_verbose;


split_info_t* split_new()
{
	return calloc(1, sizeof(split_info_t));
}

void split_delete(split_info_t *s)
{
	//printf("split_close(%s)\n", s->fname);
	split_close(s);
	free(s);
}

void split_get_fname(split_info_t *s, int idx, char *fname)
{
    strcpy(fname, s->fname);
    if (idx == 0 && s->create_mode) {
        strcat(fname, ".tmp");
    } else if (idx > 0) {
        char *c = fname + strlen(fname) - 1;
        *c = '0' + idx;
    }
}

FILE *split_open_file(split_info_t *s, int idx)
{
    FILE *f = s->f[idx];
    if (f) return f;
    char fname[1024];
    split_get_fname(s, idx, fname);
    char *mode = s->create_mode ? "wb+" : "rb+";
    f = fopen(fname, mode);
    if (!f) {
        if (s->create_mode) {
            printf("ERROR: creating %s\n", fname);
            perror("fopen");
        }
        return NULL;
    }
    if (idx > 0) {
        if (OPT_verbose) {
            printf("%s Split: %d %s          \n",
                s->create_mode ? "Create" : "Read",
                idx, fname);
        }
    }
    s->f[idx] = f;
    return f;
}

int split_fill(split_info_t *s, int idx, off64_t size)
{
    FILE *f = split_open_file(s, idx);
    fseeko(f, 0, SEEK_END);
    off64_t fsize = ftello(f);
    if (fsize < size) {
        printf("TRUNC %d "FMT_lld"\n", idx, size);
        file_truncate(fileno(f), size);
        return 1;
    }
    return 0;
}

FILE *split_get_file(split_info_t *s, u32 lba, u32 *sec_count, int fill)
{
    FILE *f;
    if (lba >= s->total_sec) {
        printf("SPLIT(%s): invalid sector %u / %u\n", s->fname, lba, (u32)s->total_sec);
        return NULL;
    }
    int idx;
    idx = lba / s->split_sec;
    if (idx >= s->max_split) {
        printf("SPLIT: invalid split %d / %d\n", idx, s->max_split - 1);
        return NULL;
    }
    f = s->f[idx];
    if (!f) {
        // opening new, make sure all previous are full
        int i;
        for (i=0; i<idx; i++) {
            if (split_fill(s, i, s->split_size)) {
                //printf("FILL %d\n", i);
            }
        }
        f = split_open_file(s, idx);
    }
    if (!f) {
        printf("SPLIT %d: no file\n", idx);
        return NULL;
    }
    u32 sec = lba % s->split_sec; // inside file
    off64_t off = (off64_t)sec * 512;
    // num sectors till end of file
    u32 to_end = s->split_sec - sec;
    if (*sec_count > to_end) *sec_count = to_end;
    if (s->create_mode && fill) {
        // extend, so that read will be succesfull
        split_fill(s, idx, off + 512 * (*sec_count));
    }
    fseeko(f, off, SEEK_SET);
    return f;
}

int split_read_sector(void *_fp,u32 lba,u32 count,void*buf)
{
    split_info_t *s = _fp;
    FILE*f;                                 
    u64 off = lba;
    off *= 512ULL;
    int i;
    u32 chunk;
    size_t ret;
    //fprintf(stderr,"READ %d %d\n", lba, count);
    for (i=0; i<(int)count; i+=chunk) {
        chunk = count - i;
        f = split_get_file(s, lba+i, &chunk, 1);
        if (!f) {
            fprintf(stderr,"\n\n"FMT_lld" %d %p\n",off,count,_fp);
            wbfs_error("error seeking in disc partition");
            return 1;
        }
        ret = fread(buf+i*512, 512ULL, chunk, f);
        if (ret != chunk) {
            printf("error reading %u %u [%u] %u = %d\n",
                    lba, count, i, chunk, (int)ret);
            wbfs_error("error reading disc");
            return 1;
        }
    }
    return 0;
}

int split_write_sector(void *_fp,u32 lba,u32 count,void*buf)
{
    split_info_t *s = _fp;
    FILE*f;                                 
    u64 off = lba;
    off*=512ULL;
    int i;
    u32 chunk;
    //printf("WRITE %d %d\n", lba, count);
    for (i=0; i<(int)count; i+=chunk) {
        chunk = count - i;
        f = split_get_file(s, lba+i, &chunk, 0);
        //if (chunk != count)
        //	printf("WRITE CHUNK %d %d/%d\n", lba+i, chunk, count);
        if (!f || !chunk) {
            fprintf(stderr,"\n\n"FMT_lld" %d %p\n",off,count,_fp);
            wbfs_error("error seeking in disc partition");
            return 1;
        }
        if (fwrite(buf+i*512, 512ULL, chunk, f) != chunk) {
            wbfs_error("error writing disc");
            return 1;
        }
    }
    return 0;
}

void split_init(split_info_t *s, char *fname)
{
    char *p;
    //printf("SPLIT_INIT %s\n", fname);
    memset(s, 0, sizeof(*s));
    strcpy(s->fname, fname);
    s->max_split = 1;
    p = strrchr(fname, '.');
    if (p && (strcasecmp(p, ".wbfs") == 0)) {
        s->max_split = MAX_SPLIT;
    }
}

void split_set_size(split_info_t *s, u64 split_size, u64 total_size)
{
    s->total_size = total_size;
    s->split_size = split_size;
    s->total_sec  = total_size / 512;
    s->split_sec  = split_size / 512;
}

void split_close(split_info_t *s)
{
    int i;
    char fname[1024];
    char tmpname[1024];
    for (i=0; i<s->max_split; i++) {
        if (s->f[i]) fclose(s->f[i]);
    }
    if (s->create_mode) {
        split_get_fname(s, -1, fname);
        split_get_fname(s, 0, tmpname);
        rename(tmpname, fname);
    }
    memset(s, 0, sizeof(*s));
}

int split_create(split_info_t *s, char *fname, u64 split_size, u64 total_size)
{
    int i;
    FILE *f;
    char sname[1024];
    int error = 0;
    split_init(s, fname);
    s->create_mode = 1;
    // check if any file already exists
    for (i=-1; i<s->max_split; i++) {
        split_get_fname(s, i, sname);
        f = fopen(sname, "rb");
        if (f) {
            printf("Error: file already exists: %s\n", sname);
            fclose(f);
            error = 1;
        }
    }
    if (error) {
        split_init(s, "");
        return -1;
    }
    split_set_size(s, split_size, total_size);
    return 0;
}

int split_open(split_info_t *s, char *fname)
{
    int i;
    u64 size = 0;
    u64 total_size = 0;
    u64 split_size = 0;
    FILE *f;
    split_init(s, fname);
    for (i=0; i<s->max_split; i++) {
        f = split_open_file(s, i);
        if (!f) {
            if (i==0) goto err;
            break;
        }
        // check previous size - all splits except last must be same size
        if (i > 0 && size != split_size) {
            printf("split %d: invalid size "FMT_lld"", i, size);
            goto err;
        }
        // get size
        fseeko(f, 0, SEEK_END);
        size = ftello(f);
        // check sector alignment
        if (size % 512) {
            printf("split %d: size ("FMT_lld") not sector (512) aligned!", i, size);
        }
        // first sets split size
        if (i==0) {
            split_size = size;
        }
        total_size += size;
    }
    split_set_size(s, split_size, total_size);
    return 0;
err:
    split_close(s);
    return -1;
}

int split_truncate(split_info_t *s, off64_t full_size)
{
    FILE *f;
    off64_t size;
    int i, ret;
    char fname[1024];
    for (i=0; i<s->max_split; i++) {
        size = full_size;
        if (size > (off64_t)s->split_size) {
            size = s->split_size;
        }
        if (size) {
            // truncate
            f = split_open_file(s, i);
            // flush & seek because we're mixing FILE/fd/fh
            fflush(f);
            fseeko(f, 0, SEEK_SET);
            //printf("TRUNC %d "FMT_lld"\n", i, size);
            ret = file_truncate(fileno(f), size);
            if (ret) {
                split_get_fname(s, i, fname);
                printf("ERROR: TRUNCATE %s %d "FMT_lld"\n", fname, i, size);
                return -1;
            }
        } else {
            // remove empty
            f = s->f[i];
            if (f) {
                fclose(f);
                s->f[i] = NULL;
            }
            split_get_fname(s, i, fname);
            remove(fname);
        }
        full_size -= size;
    }
    return 0;
}


