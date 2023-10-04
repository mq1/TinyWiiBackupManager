
#ifdef __MINGW32__
#define fseeko fseeko64
#define ftello ftello64
#define mkdir(N,M) mkdir(N)
#else
#define off64_t off_t
#endif

#if defined(_MSC_VER) || defined(__MINGW32__) || defined(__MINGW64__)
#define FMT_llu "%I64u"
#define FMT_lld "%I64d"
#else
#define FMT_llu "%llu"
#define FMT_lld "%lld"
#endif

#ifdef WIN32
int file_truncate(int fd, off64_t length);
int file_zero_data(int fd, off64_t offset, off64_t length);
char *get_dev_name(char *name);
#else
#define file_truncate ftruncate
#define file_zero_data(fd,offset,length)
#define get_dev_name(NAME) NAME
#define Sleep(MS) usleep(MS*1000)
#endif

int get_capacity(char *file,u32 *sector_size,u32 *n_sector);
int is_device(char *fname);
FILE *fopen_dev(const char *filename, const char *mode);

