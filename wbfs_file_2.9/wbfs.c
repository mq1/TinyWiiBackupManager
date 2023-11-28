// Copyright 2023  Manuel Quarneti  <manuelquarneti@protonmail.com>
// Copyright 2009 Kwiirk
// Licensed under the terms of the GNU GPL, version 2
// http://www.gnu.org/licenses/old-licenses/gpl-2.0.txt

// 10-2009 oggzee: additional commands, split file support

#ifdef WIN32
#include <windows.h>
#include <winioctl.h>
#include <io.h>
#define PATH_SEP_CHAR '\\'
#define PATH_SEP_STR  "\\"
#else
#define PATH_SEP_CHAR '/'
#define PATH_SEP_STR  "/"
#endif

#include <stdio.h>     /* for printf */
#include <stdlib.h>    /* for exit */
#include <getopt.h>
#include <sys/stat.h>
#include <unistd.h>
#include <string.h>
#include <ctype.h>
#include <libgen.h>

#include "tools.h"
#include "libwbfs.h"
#include "splits.h"

#include "platform.h"

char tool_version[] = "2.9";
char invalid_path[] = "/\\:|<>?*\"'";

wbfs_t *wbfs_try_open(char *disc,char *partition, int reset);
wbfs_t *wbfs_try_open_partition(char *fn,int reset);

#define GB (1024*1024*1024.)
#define DVD_SECT_SIZE 2048

int OPT_verbose = 1;
struct { char*opt; char *desc; } layout_desc[] =
{
    { "f0", "file:ID.ext" },
    { "f1", "file:ID_TITLE.ext" },
    { "f2", "file:TITLE [ID].ext" },
    { "d0", "dir:ID/ID.ext" },
    { "d1", "dir:ID_TITLE/ID.ext" },
    { "d2", "dir:TITLE [ID]/ID.ext" },
};
#define LAY_NUM 6
#define LAY_FILE_ID       0
#define LAY_FILE_ID_TITLE 1
#define LAY_FILE_TITLE_ID 2
#define LAY_DIR_ID        3
#define LAY_DIR_ID_TITLE  4
#define LAY_DIR_TITLE_ID  5
int OPT_layout = LAY_DIR_TITLE_ID;
int OPT_layout_spec = 0;
// copy all partitions
int OPT_part_all = 1;
int OPT_copy_1_1 = 0;
int OPT_overwrite = 0;
int OPT_trim = 0;
int OPT_scrub_size = 1; // 1 wii sector
int OPT_zero_sparse = 0;
int OPT_title_txt = 0;
int OPT_force = 0;
char *OPT_filename = 0; // first filename argument
char *OPT_arg3 = NULL;

static u32 _be32(const u8 *p)
{
    return (p[0] << 24) | (p[1] << 16) | (p[2] << 8) | p[3];
}


#ifdef WIN32

#include <conio.h>

int con_readc(int *c)
{
    int ret;
    char ch;
    HANDLE fh = (HANDLE)_get_osfhandle(0);
    if (fh == INVALID_HANDLE_VALUE) {
        //print_error("_get_osfhandle");
        return 0;
    }
    int ftype = GetFileType(fh);
    //printf("ftype: %d\n", ftype);

    if (ftype == FILE_TYPE_CHAR) {
        // maybe: SetConsoleMode !ENABLE_LINE_INPUT
        // console
        ret = kbhit();
        //printf("kbhit: %d\n", ret);
        if (ret) {
            *c = getch();
            return 1;
        }

    } else if (ftype == FILE_TYPE_PIPE) {
        // process or cygwin terminal
        DWORD avail;
        ret = PeekNamedPipe(fh, NULL, 0, NULL, &avail, NULL);
        //printf("peek: %d, %d\n", ret, avail);
        if (ret && avail) {
read_char:
            ret = read(0, &ch, 1);
            if (ret == 1) {
                *c = ch;
                return 1;
            }
        }

    } else if (ftype == FILE_TYPE_DISK) {
        // file
        ret = WaitForSingleObject(fh, 0);
        //printf("wait: %d\n", ret);
        if (ret == 0) {
            goto read_char;
        }

    }
    // else unknown
    return 0;
}

#else

#include <sys/select.h>

int con_readc(int *c)
{
    int ret;
    char ch;
    fd_set fds;
    struct timeval tv = {0,0};
    //printf("isatty: %d\n", isatty(0));
    FD_ZERO(&fds);
    FD_SET(0, &fds); // stdin
    ret = select(1, &fds, NULL, NULL, &tv);
    if (ret == 1) {
        if (FD_ISSET(0, &fds)) {
            ret = read(0, &ch, 1);
            if (ret == 1) {
                *c = ch;
                return 1;
            }
        }
    }
    return 0;
}

#endif


int check_abort()
{
    static char cmd[16] = "";
    int c, len;
    while (con_readc(&c))
    {
        //printf("read: %d %d %x '%c' %s\n", ret, c, c, c, cmd);
        if (c == '\r' || c == '\n' || c == ' ') {
            // end of line or word
            if (strcasecmp(cmd, "abort") == 0) {
                *cmd = 0;
                return 1;
            }
            *cmd = 0;
        } else {
            // append
            len = strlen(cmd);
            if (len < (int)sizeof(cmd) - 1) {
                cmd[len] = c;
                cmd[len+1] = 0;
            }
        }
    }
    //Sleep(100);//debug

    return 0;
}

FILE *fopen_dev(const char *filename, const char *mode)
{
    char *name = get_dev_name((char*)filename);
    if (is_device(name)) {
        u32 sec_size = 0, sec_count = 0;
        get_capacity(name, &sec_size, &sec_count);
        off64_t size = (off64_t)sec_size * sec_count;
        static int first_time = 1;
        if (first_time) {
            printf("%s: "FMT_lld" (%d * %d)\n",
                    (sec_size == 2048) ? "DVD" : "DEV",
                    size, sec_size, sec_count);
            first_time = 0;
        }
    }
    return fopen(name, mode);
}

// offset is 4 bytes, count is bytes
int read_wii_file(void *_fp, u32 offset, u32 count, void *iobuf)
{
    FILE*fp =_fp;
    u64 off = offset;
    off <<= 2;
    size_t ret;
    if (fseeko(fp, off, SEEK_SET))
    {
        printf("error seeking disc %u\n", offset);
        return 1;
    }
    ret = fread(iobuf, count, 1, fp);
    if (ret != 1){
        //printf("ERROR: read (%u,%d) : %d\n", offset, count, ret);
        //printf("error reading disc %u "FMT_llu" %u = %u\n",
        //        offset, off, count, ret);
        //wbfs_error("error reading disc");
        static char tmpbuf[DVD_SECT_SIZE];
        u32 chunk;
        if (off % DVD_SECT_SIZE) {
            u64 noff = (off / DVD_SECT_SIZE) * DVD_SECT_SIZE;
            if (fseeko(fp, noff, SEEK_SET))
            {
                printf("ERROR seeking disc ("FMT_llu")\n", noff);
                return 1;
            }
            ret = fread(tmpbuf, DVD_SECT_SIZE, 1, fp);
            if (ret != 1){
                printf("ERROR: read ("FMT_llu", %d) : %d\n", noff, DVD_SECT_SIZE, (int)ret);
                return 1;
            }
            u32 delta = off - noff;
            u32 chunk = DVD_SECT_SIZE - delta;
            if (chunk > count) chunk = count;
            memcpy(iobuf, tmpbuf + delta, chunk);
            iobuf += chunk;
            count -= chunk;
            off   += chunk;
        }
        while (count) {
            if (fseeko(fp, off, SEEK_SET))
            {
                printf("ERROR seeking disc ("FMT_llu")\n", off);
                return 1;
            }
            ret = fread(tmpbuf, DVD_SECT_SIZE, 1, fp);
            if (ret != 1){
                printf("ERROR: read ("FMT_llu", %d) : %d\n", off, DVD_SECT_SIZE, (int)ret);
                return 1;
            }
            chunk = count;
            if (chunk > DVD_SECT_SIZE) chunk = DVD_SECT_SIZE;
            memcpy(iobuf, tmpbuf, chunk);
            iobuf += chunk;
            count -= chunk;
            off   += chunk;
        }
    }
    return 0;
}

// offset is 4 bytes, count is bytes
int write_wii_file(void *_fp, u32 offset, u32 count, void *iobuf)
{
    FILE*fp =_fp;
    u64 off = offset;
    off <<= 2;
    size_t ret;
    if (fseeko(fp, off, SEEK_SET))
    {
        printf("error seeking disc %u\n", offset);
        return 1;
    }
    ret = fwrite(iobuf, count, 1, fp);
    if (ret != 1){
        printf("write error (%u, %u)\n", offset, count);
        return 1;
    }
    return 0;
}

// offset and count is in 32k wii sector
int write_wii_sector_file(void *_fp, u32 lba, u32 count, void *iobuf)
{
    FILE*fp=_fp;
    u64 off = lba;
    off *=0x8000;
    //printf("w %u %u\n",lba,count);
    if (fseeko(fp, off, SEEK_SET))
    {
        printf("\n\n"FMT_lld" %p\n",off,_fp);
        wbfs_error("error seeking in written disc file");
        return 1;
    }
    if (fwrite(iobuf, count*0x8000, 1, fp) != 1){
        wbfs_error("error writing disc file");
        return 1;
    }
    return 0;
}


wbfs_t *wbfs_split_create_partition(split_info_t **sp, char *fn, int reset)
{
    wbfs_t *wbfs_p = NULL;
    split_info_t *s = NULL;
    u32 sector_size = 512;
    // max dual layer:
    u64 size = (u64)143432*2*0x8000ULL + 0x4000000;
    // +0x4000000 because freeblks size is n_wbfs_sec
    // and has to be aligned to 32 (num of bits in an int)
    // using exact file size as max dual layer is not enough for the worst case:
    // doing a 1:1 copy of a dual layer disc, because we need some space for the
    // headers. And the dual layer size is not wbfs sector aligned anyway.
    // So the minimum amount that has to be added is 32 wbfs sectors,
    // which is 32*2MB = 64MB = 0x4000000
    u32 n_sector = size / 512;
    int ret;
    //printf("OPEN_PART %s\n", fn);
    s = split_new();
    if (!s) return NULL;
    ret = split_create(s, fn, OPT_split_size, size);
    if (ret) return NULL;
    wbfs_p = wbfs_open_partition(
            split_read_sector, split_write_sector, s,
            sector_size, n_sector,0,reset);
    if (wbfs_p) {
        wbfs_p->close_hd = (close_callback_t)split_delete;
        if (sp) *sp = s;
    } else {
        split_delete(s);
    }
    return wbfs_p;
}

wbfs_t *wbfs_split_open_partition(split_info_t **sp, char *fn,int reset)
{
    wbfs_t *wbfs_p = NULL;
    split_info_t *s = NULL;
    u32 sector_size = 512;
    u32 n_sector;
    int ret;
    //printf("OPEN_PART %s\n", fn);
    s = split_new();
    if (!s) return NULL;
    ret = split_open(s, fn);
    n_sector = s->total_sec;
    if (ret) return NULL;
    wbfs_p = wbfs_open_partition(
            split_read_sector, split_write_sector, s,
            sector_size, n_sector,0,reset);
    if (wbfs_p) {
        wbfs_p->close_hd = (close_callback_t)split_delete;
        if (sp) *sp = s;
    } else {
        split_delete(s);
    }
    return wbfs_p;
}

wbfs_t *wbfs_auto_open_partition(char *fn,int reset)
{
    wbfs_t *p = NULL;
    if (is_device(fn)) {
        p = wbfs_try_open_partition(fn, reset);
    } else {
        p = wbfs_split_open_partition(NULL, fn, reset);
    }
    if (!p) {
        printf("Error opening WBFS '%s'\n", fn);
    }
    return p;
}

int get_first_disc_hdr(wbfs_t *p, u8 hdr[0x100])
{
    int count = wbfs_count_discs(p);
    if(count==0) {
        printf("wbfs empty\n");
        return -1;
    }
    u32 size;
    if(wbfs_get_disc_info(p,0,hdr,0x100,&size)) return -1;
    return 0;
}

int get_first_disc_id(wbfs_t *p, char *discid)
{
    u8 b[0x100];
    if (get_first_disc_hdr(p, b)) return -1;
    memcpy(discid, b, 6);
    discid[6] = 0;
    return 0;
}



int wbfs_applet_df(wbfs_t *p)
{
    u32 count = wbfs_count_usedblocks(p);
    printf("wbfs total: %.2fG used: %.2fG free: %.2fG\n",
            (float)p->n_wbfs_sec*p->wbfs_sec_sz/GB,
            (float)(p->n_wbfs_sec-count)*p->wbfs_sec_sz/GB,
            (float)(count)*p->wbfs_sec_sz/GB);
    /*printf("bytes tot:"FMT_lld" used:"FMT_lld"  free:"FMT_lld"\n",
      (u64)p->n_wbfs_sec*p->wbfs_sec_sz,
      (u64)(p->n_wbfs_sec-count)*p->wbfs_sec_sz,
      (u64)(count)*p->wbfs_sec_sz);*/
    return p ? 0 : -1;
}

int wbfs_applet_ls(wbfs_t *p)
{
    int count = wbfs_count_discs(p);
    if(count==0)
        printf("wbfs empty\n");
    else{
        int i;
        u32 size;
        u8 *b = wbfs_ioalloc(0x100);
        for (i=0;i<count;i++)
        {
            if(!wbfs_get_disc_info(p,i,b,0x100,&size))
                printf("%.6s : %-40s %.2fG\n", b, b + 0x20, size*4ULL/(GB));
            //printf("("FMT_lld")\n", (u64)size*4ULL);
        }
        wbfs_iofree(b);
    }   
    printf("\n");
    return wbfs_applet_df(p);
}

int wbfs_applet_mkhbc(wbfs_t *p)
{
    int count = wbfs_count_discs(p);
    char filename[7];
    FILE *xml;
    if(count==0)
        printf("wbfs empty\n");
    else{
        int i;
        u32 size;
        u8 *b = wbfs_ioalloc(0x100);
        for (i=0;i<count;i++)
        {
            wbfs_get_disc_info(p,i,b,0x100,&size);
            snprintf(filename,7,"%c%c%c%c%c%c",b[0], b[1], b[2], b[3], b[4], b[5]);
            mkdir(filename, 0777);
            printf("%s\n",filename);
            if (chdir(filename))
                wbfs_fatal("chdir");
            system("cp ../boot.dol .");
            system("cp ../icon.png .");
            xml = fopen("meta.xml","wb");
            fprintf(xml,"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
            fprintf(xml,"<app>\n\t<name>%s</name>\n", b+0x20);
            fprintf(xml,"<short_description>%.2fGB on USB HD </short_description>\n",size*4ULL/(GB));
            fprintf(xml,"<long_description>This launches the yal wbfs game loader by Kwiirk for discid %s</long_description>\n",filename);
            fprintf(xml,"</app>");
            fclose(xml);
            if (chdir(".."))
                wbfs_fatal("chdir");
        }
        wbfs_iofree(b);

    }   
    return p ? 0 : -1;
}
int wbfs_applet_init(wbfs_t *p)
{
    // nothing to do actually..
    // job already done by the reset flag of the wbfs_open_partition
    return p ? 0 : -1;

}

static void _spinner(int x,int y){ spinner(x,y);}

int wbfs_applet_addiso_gethdr(wbfs_t *p,char*argv, u8 *hdr)
{
    //printf("ADD\n");
    FILE *f = fopen_dev(argv, "rb");
    u8 discinfo[0x100];
    u8 discid[8];
    wbfs_disc_t *d;
    int ret;
    if(!f)
        wbfs_error("unable to open disc file");
    else
    {
        fread(discinfo, sizeof(discinfo), 1,f);
        if (hdr) memcpy(hdr, discinfo, sizeof(discinfo));
        memcpy(discid, discinfo, 6);
        discid[6] = 0;
        d = wbfs_open_disc(p,discid);
        fflush(stdout);
        if(d)
        {
            discinfo[6]=0;
            printf("%s already in disc..\n",discid);
            wbfs_close_disc(d);
            fclose(f);
            return 1;
        } else {
            int part = OPT_part_all ? ALL_PARTITIONS : ONLY_GAME_PARTITION;
            ret = wbfs_add_disc(p,read_wii_file,f,_spinner,part, OPT_copy_1_1);
            fclose(f);
            return ret;
        }
    }
    return 1;
}

int wbfs_applet_add_iso(wbfs_t *p,char*argv)
{
    return wbfs_applet_addiso_gethdr(p, argv, NULL);
}

int wbfs_applet_rm(wbfs_t *p,char*argv)
{
    return wbfs_rm_disc(p,(u8*)argv);
}

int do_extract(wbfs_disc_t *d, char *destname)
{
    int ret = 1;
    if(!d) return -1;
    FILE *f = fopen(destname,"rb");
    if (f) {
        fclose(f);
        if (OPT_overwrite) {
            printf("\nNote: file already exists: %s (overwriting)\n", destname);
        } else {
            printf("\nError: file already exists: %s\n", destname);
            return -1;
        }
    }

    f = fopen(destname,"wb");
    if(!f)
        wbfs_fatal("unable to open dest file");
    else{
        printf("writing to %s\n",destname);

        // check if the game is DVD9..
        u32 comp_blk;
        u32 last_blk;
        comp_blk = wbfs_sector_used2(d->p, d->header, &last_blk);
        u64 real_size = (u64)(last_blk+1) * (u64)d->p->wbfs_sec_sz;
        //u64 dual_size = (d->p->n_wii_sec_per_disc)*0x8000ULL;
        u64 single_size = (d->p->n_wii_sec_per_disc/2)*0x8000ULL;
        u64 size;
        //printf("allocating file\n");
        // write a zero at the end of the iso to ensure the correct size
        //fseeko(f,size-1ULL,SEEK_SET);
        //fwrite("",1,1,f);
        if (OPT_trim || real_size > single_size) {
            size = real_size;
        } else {
            size = single_size;
        }
        ret = file_truncate(fileno(f), size);
        if (ret) {
            printf("ERROR: TRUNCATE %s "FMT_lld"\n", destname, size);
            remove(destname);
            return -1;
        }

        ret = wbfs_extract_disc(d,write_wii_sector_file,f,_spinner);

        fflush(f);
        // sparse last block
        file_zero_data(fileno(f), real_size, size - real_size);
        fclose(f);
        printf("\n");
    }
    return ret;
}

void get_id_title(u8 *hdr, char *id, char *title, int re_space)
{
    int i, len;
    char *s;
    /* get ID */
    strncpy(id, (char*)hdr, 6);
    id[6] = 0;
    // trim leading space
    s = (char*)hdr+0x20;
    for (i=0; *s == ' ' && i<64; i++) s++;
    memset(title, 0, 65);
    strncpy(title, s, 64-i);
    title[64-i] = 0;
    len = strlen(title);
    // trim trailing space - not allowed on windows directories
    while (len && title[len-1] == ' ') {
        title[len-1] = 0;
        len--;
    }
    // replace silly chars with '_'
    for (i = 0; i < len; i++) {
        if(strchr(invalid_path, title[i]) || iscntrl(title[i])) {
            title[i] = '_';
        }
        if (re_space && title[i] == ' ') {
            title[i] = '_';
        }
    }
}

void layout_fmt(char *dest, char *id, char *title, int layout)
{
    *dest = 0;
    if (layout >= LAY_DIR_ID) layout -= LAY_DIR_ID;
    switch (layout) {
        case LAY_FILE_ID:
            sprintf(dest, "%s", id);
            break;
        case LAY_FILE_ID_TITLE:
            sprintf(dest, "%s_%s", id, title);
            break;
        case LAY_FILE_TITLE_ID:
            sprintf(dest, "%s [%s]", title, id);
            break;
    }
}

void get_game_id_title(char *buf, u8 *hdr, int re_space, int layout)
{
    char id[8];
    char title[65];
    get_id_title(hdr, id, title, re_space);
    layout_fmt(buf, id, title, layout);
}

void get_hdr_titlename(u8 *hdr, char *fname, char *path)
{
    int len;
    char *buf;
    // add path
    strcpy(fname, path);
    len = strlen(fname);
    if (len && fname[len-1] != '/' && fname[len-1] != '\\') {
        strcat(fname, PATH_SEP_STR);
    }
    len = strlen(fname);
    buf = fname + len;
    get_game_id_title(buf, hdr, 1, LAY_FILE_ID_TITLE);
}

int get_dest_name(char *dst_name, char *src_name, u8 *hdr, char *dest_arg, char *ext, int layout)
{
    char dest_dir[1024] = "";
    char id[8];
    char title[65];
    char *c;
    int ret;
    // get id and title
    get_id_title(hdr, id, title, 0);
    // is dest_arg a dir or file or none?
    if (dest_arg == NULL || *dest_arg == 0) {
        // no destination specified, use same as source
        strcpy(dest_dir, src_name);
        c = strrchr(dest_dir, '/');
        if (!c) c = strrchr(dest_dir, '\\');
        if (c) c++; else c = dest_dir;
        *c = 0;
    } else {
        c = strrchr(dest_arg, '.');
        if (c && strcasecmp(c, ext) == 0) {
            // dest filename specified
            strcpy(dst_name, dest_arg);
            return 0;
        }
        // dest_arg is dir
        strcpy(dest_dir, dest_arg);
        c = &dest_dir[strlen(dest_dir)-1];
        if (*c != '/' && *c != '\\') {
            strcat(c, PATH_SEP_STR);
        }
    }
    strcpy(dst_name, dest_dir);
    c = dst_name + strlen(dst_name);
    layout_fmt(c, id, title, layout);

    if (layout >= LAY_DIR_ID) {
        struct stat st;
        if (stat(dst_name, &st) != 0) {
            ret = mkdir(dst_name, 0777);
            if (ret != 0) {
                printf("error creating: %s\n", dst_name);
                perror("mkdir");
                return -1;
            }
        }
        strcat(c, PATH_SEP_STR);
        strcat(c, id);
    }
    strcat(c, ext);
    return 0;
}

void mk_title_txt(char *fname_wbfs, u8 *hdr)
{
    char fname[1024];
    char path[1024];
    char dname[1024];
    char id[8], title[64+1];
    FILE *f;

    if (OPT_title_txt == 0) return;

    // dirname() might modify, so we need a tmp copy
    strcpy(dname, fname_wbfs);
    strcpy(path, dirname(dname));
    get_hdr_titlename(hdr, fname, path);
    strcat(fname, ".txt");

    memcpy(id, hdr, 6);
    id[6] = 0;
    memcpy(title, hdr+0x20, 64);
    title[64] = 0;
    f = fopen(fname, "wb");
    if (!f) return;
    fprintf(f, "%.6s = %.64s\n", id, title);
    fclose(f);
    printf("Info file: %s\n", fname);
}

off64_t estimate_size(wbfs_t *p, read_wiidisc_callback_t read_disc, void *cb_data)
{
    off64_t size;
    u32 comp_sec, last_sec;
    int part = OPT_part_all ? ALL_PARTITIONS : ONLY_GAME_PARTITION;
    int ret;
    //u32 wbfs_size_disc(wbfs_t*p,read_wiidisc_callback_t read_src_wii_disc,
    //              void *callback_data,partition_selector_t sel,
    //              u32 *comp_size, u32 *real_size);
    ret = wbfs_size_disc(p, read_disc, cb_data, part, &comp_sec, &last_sec);
    if (ret) return -1;
    size = (off64_t)comp_sec * p->wii_sec_sz;
    // round up to wbfs sect size alignment
    size = (size + p->wbfs_sec_sz - 1) / p->wbfs_sec_sz * p->wbfs_sec_sz;
    // add 1 wbfs sect for header
    size += p->wbfs_sec_sz;
    return size;
}

int wbfs_applet_extract_iso(wbfs_t *p, char*discid, char *path)
{
    int ret = 1;
    wbfs_disc_t *d = wbfs_open_disc(p,(u8*)discid);
    if(d)
    {
        char isoname[1024];
        // if no layout specified use ID_TITLE naming
        // if (!OPT_layout_spec) OPT_layout = LAY_FILE_ID_TITLE;
        ret = get_dest_name(isoname, "", d->header->disc_header_copy, path, ".iso", OPT_layout);
        if (ret) {
            printf("ERROR: invalid dest path (%s)\n", path);
        } else {
            ret = do_extract(d, isoname);
        }
        wbfs_close_disc(d);
    }
    else
        printf("%s not in disc..\n",discid);
    return ret;
}

int wbfs_applet_extract_wbfs(wbfs_t *p, char*arg, char *path)
{
    int ret = 1;
    if (strlen(arg)!=6) {
        printf("invalid DISCID: '%s'\n", arg);
        return -1;
    }
    wbfs_disc_t *d = wbfs_open_disc(p,(u8*)arg);
    if(!d) {
        printf("%s not in disc..\n",arg);
        return -1;
    }

    u8 b[0x100];
    wbfs_disc_read(d, 0, b, 0x100);
    u32 magic = _be32(b+24);
    if(magic!=0x5D1C9EA3){
        printf("SKIP: Not a wii disc - bad magic (%08x)\n\n", magic);
        goto err;
    }

    char destname[1024];
    if (get_dest_name(destname, "", b, path, ".wbfs", OPT_layout)) {
        goto err;
    }

    printf("Writing '%s' to: '%s'\n", arg, destname);
    mk_title_txt(destname, b);
    fflush(stdout);

    split_info_t *dest_split;
    wbfs_t *dest_p = wbfs_split_create_partition(&dest_split, destname, 1);
    fflush(stdout);
    if (!dest_p) goto err;

    // estimate size
    off64_t size = estimate_size(dest_p, read_wiidisc_wbfsdisc_errcheck, d);
    // preallocate
    ret = split_truncate(dest_split, size);
    if (ret) {
        printf("Error pre-allocating!\n");
        return -1;
    }

    int part = OPT_part_all ? ALL_PARTITIONS : ONLY_GAME_PARTITION;
    ret = wbfs_add_disc(dest_p, read_wiidisc_wbfsdisc, d, _spinner, part, 0);
    fflush(stdout);

    wbfs_close_disc(d);
    wbfs_trim(dest_p);
    split_truncate(dest_split, dest_p->n_hd_sec*512ULL);
    wbfs_close(dest_p);
    fflush(stdout);

    return ret;

err:
    if (d) wbfs_close_disc(d);
    return -1;
}

int wbfs_applet_extract_wbfs_all(wbfs_t *p, char *path)
{
    // make sure path is not a file
    char *dot = strrchr(path, '.');
    if (dot && strcasecmp(dot, ".wbfs")==0) {
        printf("ERROR: specify DIR! (%s)\n", path);
        return -1;
    }
    int count = wbfs_count_discs(p);
    if(count==0) {
        printf("wbfs empty\n");
        return -1;
    }
    wbfs_applet_ls(p);
    printf("\nExtracting ALL games to: '%s'\n", path);

    int i, r, ret = 0;
    u32 size;
    u8 b[0x100];
    char discid[8];
    for (i=0;i<count;i++) {
        if(!wbfs_get_disc_info(p,i,b,0x100,&size)) {
            printf("\n%d / %d : ", i+1, count);
            printf("%.6s : %-40s %.2fG\n", b, b + 0x20, size*4ULL/(GB));
            //printf("("FMT_lld")\n", (u64)size*4ULL);
            // check magic
            u32 magic = _be32(b+24);
            if (magic != 0x5D1C9EA3){
                printf("SKIP: Not a wii disc - bad magic (%08x)\n\n", magic);
                continue;
            }
            memcpy(discid, b, 6);
            discid[6] = 0;
            r = wbfs_applet_extract_wbfs(p, discid, path);
            if (r) {
                printf("\nERROR: extract (%.6s) = %d\n\n", discid, r);
                ret = -1;
            }
        }
    }
    printf("Done.\n");
    return ret;
}

int wbfs_copy(wbfs_t *src_p, wbfs_t *dest_p, char *discid)
{
    wbfs_disc_t *d;
    // check if disc present on targer
    d = wbfs_open_disc(dest_p, (u8*)discid);
    fflush(stdout);
    if (d)
    {
        printf("%s already in disc..\n", discid);
        wbfs_close_disc(d);
        return -1;
    }

    // open disc in source
    d = wbfs_open_disc(src_p, (u8*)discid);
    if (!d)    {
        printf("Error: %s not found\n", discid);
        return -1;
    }

    fflush(stdout);
    int part = OPT_part_all ? ALL_PARTITIONS : ONLY_GAME_PARTITION;
    int ret = wbfs_add_disc(dest_p, read_wiidisc_wbfsdisc, d, _spinner, part, 0);
    fflush(stdout);

    wbfs_close_disc(d);
    fflush(stdout);

    return ret;
}

int wbfs_applet_add_wbfs(wbfs_t *p, char *fname)
{
    split_info_t *src_split;
    int ret;
    wbfs_t *src_p = wbfs_split_open_partition(&src_split, fname, 0);
    fflush(stdout);
    if (!src_p) return -1;

    char discid[8];
    if (get_first_disc_id(src_p, discid)) {
        printf("error finding ID in %s\n", fname);
        wbfs_close(src_p);
        return -1;
    }
    printf("Adding [%s] %s to WBFS\n", discid, fname);
    ret = wbfs_copy(src_p, p, discid);
    wbfs_close(src_p);

    return ret;
}

int wbfs_applet_wbfs_copy(wbfs_t *src_p, char *discid, char *dest_name)
{
    wbfs_t *dest_p;
    int ret;
    printf("WBFS COPY [%s] from %s to %s\n", discid, OPT_filename, dest_name);
    dest_p = wbfs_auto_open_partition(dest_name, 0);
    if (!src_p || !dest_p) {
        return -1;
    }
    ret = wbfs_copy(src_p, dest_p, discid);
    //wbfs_close(src_p);
    wbfs_close(dest_p);
    if (ret) {
        printf("COPY ERROR %d\n", ret);
    }
    return ret;
}

int wbfs_applet_make_info(wbfs_t *p)
{
    char *name_wbfs = OPT_filename;
    if (is_device(name_wbfs)) name_wbfs = "";
    int count = wbfs_count_discs(p);
    if(count==0)
        printf("wbfs empty\n");
    else{
        int i;
        u32 size;
        u8 *b = wbfs_ioalloc(0x100);
        for (i=0;i<count;i++)
        {
            if(!wbfs_get_disc_info(p,i,b,0x100,&size))
                printf("%.6s : %-40s %.2fG\n", b, b + 0x20, size*4ULL/(GB));
            //printf("("FMT_lld")\n", (u64)size*4ULL);
            mk_title_txt(name_wbfs, b);
        }
        wbfs_iofree(b);
    }   
    printf("\n");
    return wbfs_applet_df(p);
}

int wbfs_applet_id_title(wbfs_t *p)
{
    char id_title[100]; // required: 6+1+64+1
    u8 hdr[0x100];

    if (get_first_disc_hdr(p, hdr)) {
        return -1;
    }
    if (!OPT_layout_spec) OPT_layout = LAY_FILE_ID_TITLE;
    get_game_id_title(id_title, hdr, 0, OPT_layout);
    printf("%s\n", id_title);
    return 0;
}

int iso_id_title(char *filename)
{
    char id_title[100]; // required: 6+1+64+1
    u8 hdr[0x100];

    FILE *f = fopen_dev(filename, "rb");
    if (!f) {
        printf("ERROR: open(%s)\n", filename);
        return -1;
    }
    fread(hdr, sizeof(hdr), 1, f);
    fclose(f);

    if (!OPT_layout_spec) OPT_layout = LAY_FILE_ID_TITLE;
    get_game_id_title(id_title, hdr, 0, OPT_layout);
    printf("%s\n", id_title);
    return 0;
}

int wbfs_applet_extract_file(wbfs_t *p, char*argv, char *arg2)
{
    wbfs_disc_t *d;
    void *data = NULL;
    int size = 0;
    d = wbfs_open_disc(p,(u8*)argv);
    if(!d)
    {
        printf("Disc not found: %s\n", argv);
        return -1;
    }
    size = wbfs_extract_file(d, arg2, &data);
    wbfs_close_disc(d);
    if (!data || size <= 0) {
        printf("File: %s not found in disc %s\n",arg2, argv);
        return -1;
    }
    FILE *f;
    char *outfile = OPT_arg3;
    if (!outfile) outfile = arg2;
    if (!*outfile) outfile = "fst.dat";
    f = fopen(outfile, "wb");
    if (!f) {
        perror("fopen");
        return -1; 
    }
    if (fwrite(data, size, 1, f) != 1) {
        perror("write");
        return -1; 
    }
    fclose(f);
    printf("extracted: (%.6s) '%s' -> '%s'\n", argv, arg2, outfile);
    return 0;
}

typedef struct {
    u8 filetype;
    char name_offset[3];
    u32 fileoffset;
    u32 filelen;
} __attribute__((packed)) FST_ENTRY;


char *fstfilename2(FST_ENTRY *fst, u32 index)
{
    u32 count = _be32((u8*)&fst[0].filelen);
    u32 stringoffset;
    if (index < count)
    {
        //stringoffset = *(u32 *)&(fst[index]) % (256*256*256);
        stringoffset = _be32((u8*)&(fst[index])) % (256*256*256);
        return (char *)((u32)fst + count*12 + stringoffset);
    } else
    {
        return NULL;
    }
}

void fst_list(void *afst)
{
    //FST_ENTRY *fst = (FST_ENTRY *)*(u32 *)0x80000038;
    FST_ENTRY *fst = (FST_ENTRY *)afst;
    u32 count = _be32((u8*)&fst[0].filelen);
    u32 i;
    printf("fst files: %d\n", count);
    for (i=1;i<count;i++) {        
        //printf("%d %p %p\n", i, fst, fstfilename2(fst, i));
        printf("%d %s\n", i, fstfilename2(fst, i));
        fflush(stdout);
    }
}


int wbfs_applet_ls_file(wbfs_t *p,char*argv)
{
    wbfs_disc_t *d;
    int size = 0;
    void *fst;
    d = wbfs_open_disc(p,(u8*)argv);
    if (!d) {
        printf("%s not in disc..\n",argv);
        return -1;
    }
    size = wbfs_extract_file(d, "", &fst);
    wbfs_close_disc(d);
    if (!fst || size <= 0) {
        printf("%s not in disc..\n", argv);
        return -1;
    }
    printf("fst found: %d\n", size);
    fst_list(fst);
    free(fst);
    return 0;
}


int wbfs_applet_create(char *dest_name, char*argv)
{
    u8 hdr[0x100];
    split_info_t *sp = NULL;
    wbfs_t *p = wbfs_split_create_partition(&sp, dest_name, 1);
    int ret = -1;
    if (!p) return -1;

    // estimate size
    FILE *f = fopen_dev(argv, "rb");
    if(!f) {
        wbfs_error("unable to open disc file");
        return -1;
    }
    off64_t size = estimate_size(p, read_wii_file, f);
    fclose(f);
    // preallocate
    ret = split_truncate(sp, size);
    if (ret) {
        printf("Error pre-allocating!\n");
        return -1;
    }

    memset(hdr, 0, sizeof(hdr));
    ret = wbfs_applet_addiso_gethdr(p,argv, hdr);
    if (ret == 0 && hdr[0] != 0) {
        // success
        mk_title_txt(dest_name, hdr);
    }
    wbfs_trim(p);
    split_truncate(sp, p->n_hd_sec*512ULL);
    wbfs_close(p);
    return ret;
}


int conv_to_wbfs(char *filename, char *dest_dir)
{
    printf("Converting %s to .wbfs\n", filename);
    char newname[1024];
    u8 hdr[0x100];

    FILE *f = fopen_dev(filename,"rb");
    if(!f) {
        printf("unable to open iso file '%s'", filename);
        return -1;
    }
    fread(hdr, sizeof(hdr), 1,f);
    fclose(f);
    if (get_dest_name(newname, filename, hdr, dest_dir, ".wbfs", OPT_layout)) {
        return -1;
    }
    printf("Writing: %s\n", newname);
    wbfs_applet_create(newname, filename);
    return 0;
}

int conv_to_iso(char *filename, char *dest_dir)
{
    char discid[8];

    printf("Converting %s to ISO\n", filename);

    wbfs_t *p = wbfs_auto_open_partition(filename, 0);
    if(!p) {
        return -1;
    }
    if (get_first_disc_id(p, discid))
    {
        printf("error finding ID in %s\n", filename);
        return -1;
    }
    char path[1024], *c;
    if (*dest_dir == 0) {
        strcpy(path, filename);
        c = strrchr(path, '/');
        if (!c) c = strrchr(path, '\\');
        if (c) c++; else c = path;
        *c = 0;
        dest_dir = path;
    }
    wbfs_applet_extract_iso(p, discid, dest_dir);
    return 0;
}

int convert(char *filename, char *dest_dir)
{
    if (is_device(filename)) {
        if (conv_to_wbfs(filename, dest_dir)) return -1;
        return 0;
    }
    // only filename specified
    char *dot;
    dot = strrchr(filename, '.');
    if (!dot) return -2;
    if (strcasecmp(dot, ".iso") == 0) {
        if (conv_to_wbfs(filename, dest_dir)) return -1;
    } else if (strcasecmp(dot, ".wbfs") == 0) {
        if (conv_to_iso(filename, dest_dir)) return -1;
    } else {
        return -2;
    }
    return 0;
}

int get_iso_info(
        char *src,
        off64_t *p_size,
        off64_t *p_trim_size,
        u32 *p_num_blk,
        u32 *p_used_blk,
        u32 *p_trim_blk,
        FILE **pf,
        u8 *hdr,
        u8 *used
        )
{
    char *dot;
    int ret;
    int i;
    off64_t size = 0;
    off64_t trim_size = 0;
    u32 num_blk = 0;
    u32 used_blk = 0;
    u32 trim_blk = 0; // = last used + 1

    if (is_device(src)) {
        u32 sec_size = 0, sec_count = 0;
        get_capacity(src, &sec_size, &sec_count);
        if (sec_size != DVD_SECT_SIZE) {
            printf("ERROR: not a DVD\n");
            return -1;
        }
        size = (off64_t)sec_size * sec_count;
    } else {
        dot = strrchr(src, '.');
        if (!dot || strcasecmp(dot, ".iso") != 0) {
            printf("ERROR: Specify an .iso file! (%s)\n", src);
            return -1;
        }
    }
    FILE *f = fopen_dev(src, "rb");
    if(!f) {
        printf("unable to open iso file '%s'", src);
        return -1;
    }
    if (!is_device(src)) {
        ret = fseeko(f, 0, SEEK_END);
        if (ret == -1) {
            perror("fseeko");
            return -1;
        }
        size = ftello(f);
    }

    fseeko(f, 0, SEEK_SET);
    fread(hdr, 0x100, 1, f);
    // get usage table
    wiidisc_t *d = 0;
    if (!OPT_copy_1_1) {
        memset(used, 0, WII_MAX_SECTORS);
        d = wd_open_disc(read_wii_file, f);
        if (!d) {
            printf("unable to open wii disc");
            return -1;
        }
        int part = OPT_part_all ? ALL_PARTITIONS : ONLY_GAME_PARTITION;
        wd_build_disc_usage(d, part, used);
        wd_close_disc(d);
        d = 0;
    }
    // debug: dump usage table
    /*
       for (i=0; i<WII_MAX_SECTORS; i++) {
       if ( (i%64) == 0 ) {
       printf("\n%05x : ", i);
       }
       printf("%d", block_used(used,i,1));
       }
       printf("\n");
       exit(0);
       */
    // calculate sizes
    num_blk = (size + WII_SECTOR_SIZE - 1) / WII_SECTOR_SIZE;
    if (size % WII_SECTOR_SIZE) {
        printf("WARNING: size not wii sector aligned!\n");
    }
    if (OPT_copy_1_1) {
        trim_size = size;
        used_blk = num_blk;
        trim_blk = num_blk;
    } else {
        for (i=0; i<WII_MAX_SECTORS; i++) {
            if (used[i]) {
                trim_blk = i + 1;
                used_blk++;
            }
        }
        if (num_blk < trim_blk) {
            printf("ERROR: invalid block count: %d < %d\n", num_blk, trim_blk);
            return -1;
        }
        if (OPT_trim) {
            trim_size = (off64_t)trim_blk * WII_SECTOR_SIZE;
        } else {
            trim_size = size;
        }
    }
    *p_size      = size;
    *p_trim_size = trim_size;
    *p_num_blk   = num_blk;
    *p_used_blk  = used_blk;
    *p_trim_blk  = trim_blk;
    *pf          = f;
    return 0;
}


int scrub(char *src, char *dest)
{
    char destname[1024];
    u8 hdr[0x100];
    int ret;
    int i, j;
    off64_t size = 0;
    off64_t trim_size = 0;
    u32 num_blk = 0;
    u32 used_blk = 0;
    u32 trim_blk = 0; // = last used + 1
    FILE *f = NULL;
    u8 *used = 0;

    printf("Scrubbing %s\n", src);

    used = calloc(1, WII_MAX_SECTORS);
    if (!used) {
        printf("unable to alloc memory");
        return -1;
    }

    ret = get_iso_info(src,
            &size,
            &trim_size,
            &num_blk,
            &used_blk,
            &trim_blk,
            &f,
            hdr,
            used);
    if (ret) return -1;	

    // create dest file
    if (get_dest_name(destname, src, hdr, dest, ".iso", OPT_layout)) {
        return -1;
    }
    printf("Writing %s\n", destname);
    struct stat st;
    if (stat(destname, &st) == 0) {
        if (OPT_overwrite) {
            printf("\nNote: file already exists: %s (overwriting)\n", destname);
        } else {
            printf("ERROR: already exists: %s\n", destname);
            return -1;
        }
    }
    FILE *f_dest = fopen(destname, "wb");
    if(!f_dest) {
        printf("unable to open iso file '%s'", destname);
        return -1;
    }
    ret = file_truncate(fileno(f_dest), trim_size);
    if (ret) {
        printf("ERROR: TRUNCATE %s "FMT_lld"\n", destname, trim_size);
        goto error;
    }
    // copy
    u32 blk, off, blk_size;
    u32 cnt_blk; // for spinner
    u32 write_blk = 0;
    off64_t offset;
    off64_t sparse_off = 0, sparse_len = 0;
    void *zbuf = calloc(1, WII_SECTOR_SIZE);
    void *buf = malloc(WII_SECTOR_SIZE);

    if (!buf) {
        printf("unable to alloc memory");
        goto error;
    }
    cnt_blk = 0;
    spinner(0, used_blk);
    for (i=0; i<(int)(num_blk+OPT_scrub_size-1)/OPT_scrub_size; i++) {
        if (!OPT_copy_1_1) {
            if (!block_used(used, i, OPT_scrub_size)) {
                if (sparse_len == 0) {
                    sparse_off = (off64_t)i * OPT_scrub_size * WII_SECTOR_SIZE;
                }
                sparse_len += (off64_t)OPT_scrub_size * WII_SECTOR_SIZE;
                continue;
            }
        }
        for (j=0; j<OPT_scrub_size; j++) {
            blk = i * OPT_scrub_size + j;
            if (blk >= num_blk) break;
            off = blk * (WII_SECTOR_SIZE >> 2);
            offset = (off64_t)blk * WII_SECTOR_SIZE;
            if (offset >= size) break;
            if (size - offset < WII_SECTOR_SIZE) {
                blk_size = size - offset;
            } else {
                blk_size = WII_SECTOR_SIZE;
            }
            spinner(cnt_blk, used_blk);
            cnt_blk++;
            ret = read_wii_file(f, off, blk_size, buf);
            if (ret) {
                printf("ERROR: read!\n");
                goto error;
            }
            if (OPT_zero_sparse) {
                // skip writing zero filled blocks
                if (memcmp(buf, zbuf, blk_size) == 0) {
                    if (sparse_len == 0) {
                        sparse_off = offset;
                    }
                    sparse_len += blk_size;
                    continue;
                }
            }
            if (sparse_len) {
                fflush(f_dest);
                file_zero_data(fileno(f_dest), sparse_off, sparse_len);
                sparse_off = sparse_len = 0;
            }
            ret = write_wii_file(f_dest, off, blk_size, buf);
            if (ret) {
                printf("ERROR: write!\n");
                goto error;
            }
            write_blk++;
        }
    }
    spinner(used_blk, used_blk);
    //printf("Blocks written: %d bytes: "FMT_lld"\n",
    //        write_blk, (off64_t)write_blk * WII_SECTOR_SIZE);

    if (sparse_len) {
        //printf("Trailing sparse block: "FMT_lld" "FMT_lld"\n", sparse_off, sparse_len);
        fflush(f_dest);
        file_zero_data(fileno(f_dest), sparse_off, sparse_len);
        sparse_off = sparse_len = 0;
    }

    fclose(f_dest);
    fclose(f);
    return 0;

error:
    if (f) fclose(f);
    if (f_dest) fclose(f_dest);
    remove(destname);
    return -1;
}

int iso_info(char *src)
{
    u8 hdr[0x100];
    int ret;
    off64_t size = 0;
    off64_t trim_size;
    u32 num_blk = 0;
    u32 used_blk = 0;
    u32 trim_blk = 0; // = last used + 1
    FILE *f = NULL;
    u8 *used = 0;

    printf("ISO INFO %s\n", src);

    used = calloc(1, WII_MAX_SECTORS);
    if (!used) {
        printf("unable to alloc memory");
        return -1;
    }

    ret = get_iso_info(src,
            &size,
            &trim_size,
            &num_blk,
            &used_blk,
            &trim_blk,
            &f,
            hdr,
            used);
    if (ret) return -1;	

    printf("id:         %.6s\n", hdr);
    printf("title:      '%.64s'\n", hdr+0x20);
    printf("size:       "FMT_lld"\n", size);
    printf("trim size:  "FMT_lld"\n", (u64)trim_blk * WII_SECTOR_SIZE);
    printf("trim sect:  %u\n",        trim_blk);
    printf("trim gb:    %.2f\n", (float)trim_blk * WII_SECTOR_SIZE / GB);
    printf("scrub size: "FMT_lld"\n", (u64)used_blk * WII_SECTOR_SIZE);
    printf("scrub sect: %u\n",       used_blk);
    printf("scrub gb:   %.2f\n", (float)used_blk * WII_SECTOR_SIZE / GB);
    fclose(f);
    return 0;
}


int wbfs_applet_debug_info(wbfs_t *p)
{
#define PRINT_X(X) printf("%-20s: %-7d 0x%x\n", #X, (u32)p->X, (u32)p->X)
    //wbfs_head_t *head;
    //PRINT_X(head->magic);
    // parameters copied in the partition for easy dumping, and bug reports
    PRINT_X(head->n_hd_sec);           // total number of hd_sec in this partition
    PRINT_X(head->hd_sec_sz_s);       // sector size in this partition
    PRINT_X(head->wbfs_sec_sz_s);     // size of a wbfs sec

    /* hdsectors, the size of the sector provided by the hosting hard drive */
    PRINT_X(hd_sec_sz);
    PRINT_X(hd_sec_sz_s); // the power of two of the last number
    PRINT_X(n_hd_sec);     // the number of hd sector in the wbfs partition

    /* standard wii sector (0x8000 bytes) */
    PRINT_X(wii_sec_sz); 
    PRINT_X(wii_sec_sz_s);
    PRINT_X(n_wii_sec);
    PRINT_X(n_wii_sec_per_disc);

    /* The size of a wbfs sector */
    PRINT_X(wbfs_sec_sz);
    PRINT_X(wbfs_sec_sz_s); 
    PRINT_X(n_wbfs_sec);   // this must fit in 16 bit!
    PRINT_X(n_wbfs_sec_per_disc);   // size of the lookup table

    PRINT_X(part_lba);
    PRINT_X(max_disc);
    PRINT_X(freeblks_lba);
    //u32 *freeblks;
    PRINT_X(disc_info_sz);

    PRINT_X(n_disc_open);
    return 0;
}


struct wbfs_applets{
    char *opt;
    int (*func)(wbfs_t *p);
    int (*func_arg)(wbfs_t *p, char *argv);
    int (*func_arg2)(wbfs_t *p, char *arg1, char *arg2);
    char *arg_name;
    int dest; // is first arg a src or dest
} wbfs_applets[] = {
#define APPLET_0(d,x)   { #x,wbfs_applet_##x,NULL,NULL, "", d}
#define APPLET_1(d,x,A) { #x,NULL,wbfs_applet_##x,NULL, A, d}
#define APPLET_2(d,x,A) { #x,NULL,NULL,wbfs_applet_##x, A, d}
    APPLET_0(0, ls),
    APPLET_0(0, df),
    APPLET_0(0, make_info),
    APPLET_0(0, id_title),
    APPLET_0(1, init),
    APPLET_1(1, add_iso,          "<SRC:drive or file.iso>"),
    APPLET_1(1, add_wbfs,         "<SRC:filename.wbfs>"),
    APPLET_1(1, rm,               "<GAMEID>"),
    APPLET_2(0, extract_iso,      "<GAMEID> <DST:dir or file.iso>"),
    APPLET_2(0, extract_wbfs,     "<GAMEID> <DST:dir or file.wbfs>"),
    APPLET_1(0, extract_wbfs_all, "<DST:dir>"),
    APPLET_2(0, wbfs_copy,        "<GAMEID> <DST:drive or file.wbfs>"),
    APPLET_1(0, ls_file,          "<GAMEID>"),
    APPLET_2(0, extract_file,     "<GAMEID> <file> [<DST:file>]"),
    //APPLET_0(0, mkhbc),
    APPLET_0(0, debug_info),
};
static int num_applets = sizeof(wbfs_applets)/sizeof(wbfs_applets[0]);

void usage_basic(char **argv)
{
    char *tool = strrchr(argv[0], '/');
    if (!tool) tool = strrchr(argv[0], '\\');
    if (tool) tool++; else tool = argv[0];
    //printf("Usage: %s [-d disk|-p partition]\n", argv[0]);
    printf("%s %s by oggzee, based on wbfs by kwiirk\n\n", tool, tool_version);
    printf("Usage: %s [OPTIONS] <DRIVE or FILENAME> [COMMAND [ARGS]]:\n", tool);
    printf("\n");
    printf("  Given just a filename it will convert from iso to wbfs or vice versa:\n");
    printf("\n");
    printf("    %s filename.iso\n", tool);
    printf("    Will convert filename.iso to GAMEID.wbfs\n");
    printf("    And create an info file GAMEID_TITLE.txt\n");
    printf("\n");
    printf("    %s filename.wbfs\n", tool);
    printf("    Will convert filename.wbfs to GAMEID_TITLE.iso\n");
    printf("\n");
}

void usage(char **argv)
{
    int i;
    usage_basic(argv);
    printf("  COMMANDS:\n");
    printf("    <drive or file.iso>  convert  <DST:dir or file.wbfs>\n");
    printf("        <filename.wbfs>  convert  <DST:dir or file.iso>\n");
    printf("    <drive or file.iso>  scrub    <DST:dir or file.iso>\n");
    printf("    <DST:filename.wbfs>  create   <SRC:drive or file.iso>\n");
    for (i=0;i<num_applets;i++) {
        printf("    %sdrive or file>  %-16s %s\n",
                wbfs_applets[i].dest ? "<DST:" : "    <",
                wbfs_applets[i].opt,
                wbfs_applets[i].arg_name);
    }
    printf("        <drive or file>  iso_info\n");
    printf("\n");
    printf("  OPTIONS: (it's recommended to just use the defaults)\n");
    printf("    -s SIZE  :  Set split size ["FMT_lld"] ", DEF_SPLIT_SIZE);
    printf("(%d sectors)\n", (u32)(DEF_SPLIT_SIZE/512));
    printf("                Must be a multiple of 512 (sector size)\n");
    printf("    -2       :  Use split size: 2GB-32kb ("FMT_lld")\n", SPLIT_SIZE_2);
    printf("    -4       :  Use split size: 4GB-32kb ("FMT_lld")\n", SPLIT_SIZE_4);
    printf("    -0       :  Don't split (split size: "FMT_lld")\n", SPLIT_SIZE_0);
    printf("    -u SIZE  :  Set scrub block size [32768] (1 wii sector)\n");
    printf("                Must be a multiple of 32768 (wii sector size)\n");
    printf("                Special values: 1=1 wii sector, 2=2mb (.wbfs block)\n");
    printf("    -z       :  make zero filled blocks as sparse when scrubbing\n");
    printf("    -a       :  Copy ALL partitions from ISO [default]\n");
    printf("    -g       :  Copy only game partition from ISO\n");
    printf("    -1       :  Copy 1:1 from ISO\n");
    printf("    -f       :  Force wbfs mode even if the wbfs file or partition\n");
    printf("                integrity check is invalid (non matching number of\n");
    printf("                sectors or other parameters)\n");
    printf("    -t       :  trim extracted iso size\n");
    printf("    -x 0|1   :  disable|enable .txt file creation [default:0]\n");
    printf("    -l X     :  Layout of the destination filename:\n");
    printf("                -l f0 = file: ID.ext             (same as -b)\n");
    printf("                -l f1 = file: ID_TITLE.ext\n");
    printf("                -l f2 = file: TITLE [ID].ext\n");
    printf("                -l d0 = dir:  ID/ID.ext\n");
    printf("                -l d1 = dir:  ID_TITLE/ID.ext    (same as -d)\n");
    printf("                -l d2 = dir:  TITLE [ID]/ID.ext  [default]\n");
    printf("    -b       :  Same as -l f0\n");
    printf("    -d       :  Same as -l d1\n");
    //printf("    -w       :  Overwrite\n");
    printf("    -h       :  Help\n");
    exit(EXIT_FAILURE);
}

int _main(int argc, char *argv[])
{
    int opt;
    int i;
    int ret = -1;
    //char *partition=0,*disc =0;
    char *filename = 0;
    char *dest_name = "";

    // disable stdout buffering
    setvbuf(stdout, NULL, _IONBF, 0); 
    setvbuf(stdin, NULL, _IONBF, 0); 

    if (argc == 1) {
        usage_basic(argv);
        printf("  Use -h for help on commands and options\n");
        exit(EXIT_FAILURE);
    }
    for (i=1; i<argc;i++) {
        if (strcmp(argv[i], "id_title") == 0) {
            OPT_verbose = 0;
        }
    }

    while ((opt = getopt(argc, argv, "s:u:x:l:hag0124dbftwz")) != -1) {
        switch (opt) {
            /*case 'p':
              partition = optarg;
              break;
              case 'd':
              disc = optarg;
              break;*/
            case 's':
                {
                    long long size;
                    if (sscanf(optarg, ""FMT_lld"", &size) != 1) {
                        printf("Invalid split size value!\n");
                        goto err;
                    }
                    if (size <= 0 || size % 512) {
                        printf("Invalid split size!\n");
                        goto err;
                    }
                    if (size % (32*1024)) {
                        printf("WARNING: split size not 32kb aligned!\n");
                    }
                    OPT_split_size = size;
                    printf("Split size: "FMT_lld" (%d sectors)\n",
                            OPT_split_size, (u32)(OPT_split_size/512));
                }
                break;
            case 'u':
                {
                    int size;
                    if (sscanf(optarg, "%d", &size) != 1) {
                        printf("Invalid scrub size value!\n");
                        goto err;
                    }
                    if (size == 1) {
                        // 32k
                        OPT_scrub_size = 1;
                    } else if (size == 2) {
                        // 2MB
                        OPT_scrub_size = (2*1024*1024) / WII_SECTOR_SIZE;
                    } else if (size <= 0 || size % WII_SECTOR_SIZE) {
                        printf("Invalid scrub size! (%d)\n", size);
                        goto err;
                    } else {
                        OPT_scrub_size = size / WII_SECTOR_SIZE;
                    }
                    printf("Scrub block size: %d (%d wii sectors)\n",
                            OPT_scrub_size * WII_SECTOR_SIZE, OPT_scrub_size);
                }
                break;
            case 'x':
                {
                    int n;
                    if (sscanf(optarg, "%d", &n) != 1) {
                        printf("Invalid -x value! (%s)\n", optarg);
                        goto err;
                    }
                    if (n != 0 && n != 1) {
                        printf("Invalid -x value! (%s)\n", optarg);
                        goto err;
                    }
                    OPT_title_txt = n;
                    printf("Using OPTION: -x : %s id_title.txt creation\n",
                            OPT_title_txt ? "enable" : "disable");
                }
                break;
            case 'l':
                {
                    int i;
                    for (i=0; i<LAY_NUM; i++) {
                        if (strcmp(optarg, layout_desc[i].opt) == 0) {
                            break;
                        }
                    }
                    if (i >= LAY_NUM) {
                        printf("Invalid -l value! (%s)\n", optarg);
                        goto err;
                    }
                    OPT_layout = i;
                    OPT_layout_spec = 1;
                    if (OPT_verbose) {
                        printf("Using OPTION: -l : %s (%s)\n", optarg, layout_desc[i].desc);
                    }
                }
                break;
            case 'd':
                printf("Using OPTION -d : Create a GAMEID_TITLE directory\n");
                OPT_layout = LAY_DIR_ID_TITLE;
                OPT_layout_spec = 1;
                break;
            case 'b':
                printf("Using OPTION -b : Create files in base directory\n");
                OPT_layout = LAY_FILE_ID;
                OPT_layout_spec = 1;
                break;
            case 'a':
                printf("Using OPTION -a : install all partitions\n");
                OPT_part_all = 1;
                break;
            case 'g':
                printf("Using OPTION -g : install only game partitions\n");
                OPT_part_all = 0;
                break;
            case '1':
                printf("Using OPTION -1 : make a 1:1 copy\n");
                OPT_copy_1_1 = 1;
                OPT_part_all = 1;
                break;
            case '0':
                OPT_split_size = SPLIT_SIZE_0;
                printf("Using OPTION -0 : no splits.\n");
                printf("Split size: "FMT_lld" (%d sectors)\n",
                        OPT_split_size, (u32)(OPT_split_size/512));
                break;
            case '2':
                OPT_split_size = SPLIT_SIZE_2;
                printf("Using OPTION -2 : ");
                printf("Split size: "FMT_lld" (%d sectors)\n",
                        OPT_split_size, (u32)(OPT_split_size/512));
                break;
            case '4':
                OPT_split_size = SPLIT_SIZE_4;
                printf("Using OPTION -4 : ");
                printf("Split size: "FMT_lld" (%d sectors)\n",
                        OPT_split_size, (u32)(OPT_split_size/512));
                break;
            case 'f':
                printf("Using OPTION -f : force wbfs even if wbfs integrity is invalid\n");
                wbfs_set_force_mode(1);
                OPT_force = 1;
                break;
            case 't':
                printf("Using OPTION -t : trim extracted iso size\n");
                OPT_trim = 1;
                break;
            case 'w':
                printf("Using OPTION -w : overwrite target iso\n");
                OPT_overwrite = 1;
                break;
            case 'z':
                printf("Using OPTION -z : sparse zero filled blocks\n");
                OPT_zero_sparse = 1;
                break;
            case 'h':
            default: /* '?' */
                usage(argv);
        }
    }
    if (optind >= argc) {
        usage(argv);
        exit(EXIT_FAILURE);
    }

    OPT_filename = filename = argv[optind];
    optind++;

    if (optind == argc)
    {
        // only filename specified
        dest_name = "";
        goto L_convert;
    }

    if (optind >= argc) {
        goto usage;
    }


    if (strcmp(argv[optind], "create")==0)
    {
        if(optind + 1 >= argc) goto usage;
        dest_name = argv[optind+1];
        return wbfs_applet_create(filename, dest_name);
    }

    if (strcmp(argv[optind], "convert")==0)
    {
        if(optind + 1 >= argc) goto usage;
        dest_name = argv[optind+1];
L_convert:
        ret = convert(filename, dest_name);
        if (ret == -2) goto usage;
        if (ret == -1) goto err;
        goto exit;
    }

    if (strcmp(argv[optind], "scrub")==0)
    {
        if(optind + 1 >= argc) goto usage;
        dest_name = argv[optind+1];
        ret = scrub(filename, dest_name);
        if (ret) goto err;
        goto exit;
    }

    if (strcmp(argv[optind], "id_title")==0)
    {
        char *p;
        p = strrchr(filename, '.');
        if (p && (strcasecmp(p, ".iso") == 0)) {
            ret = iso_id_title(filename);
            if (ret) goto err;
            goto exit;
        }
    }

    if (strcmp(argv[optind], "init")==0)
    {
        if (!OPT_force) {
            printf("init disabled, use -f to force wbfs formatting!\n");
            goto err;
        }
    }

    if (strcmp(argv[optind], "iso_info")==0)
    {
        ret = iso_info(filename);
        if (ret) goto err;
        goto exit;
    }

    for (i=0;i<num_applets;i++)
    {
        struct wbfs_applets *ap = &wbfs_applets[i];
        if (strcmp(argv[optind],ap->opt)==0)
        {
            //wbfs_t *p = wbfs_try_open_partition(filename,
            //                          ap->func== wbfs_applet_init);
            wbfs_t *p = wbfs_auto_open_partition(filename,
                    ap->func== wbfs_applet_init);
            if(!p) {
                return 1;
            }
            if(ap->func)
            {
                ret = ap->func(p);
            }
            else if(ap->func_arg)
            {
                if(optind + 1 >= argc)
                    usage(argv);
                else
                    ret = ap->func_arg(p, argv[optind+1]);
            }
            else if(ap->func_arg2)
            {
                if(optind + 2 >= argc)
                    usage(argv);
                else {
                    if (optind + 3 < argc)
                        OPT_arg3 = argv[optind+3];
                    else
                        OPT_arg3 = NULL;
                    ret = ap->func_arg2(p, argv[optind+1], argv[optind+2]);
                }
            }
            wbfs_close(p);
            break;
        }
    }
    if (i==num_applets) {
        printf("Error: unknown command: %s\n\n", argv[optind]);
        goto usage;
    }
    if (ret) goto err;

exit:
    exit(EXIT_SUCCESS);
usage:
    usage(argv);
err:
    exit(EXIT_FAILURE);
}

