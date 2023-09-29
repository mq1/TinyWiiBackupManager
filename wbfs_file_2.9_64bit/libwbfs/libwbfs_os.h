#ifndef LIBWBFS_OS_H
#define LIBWBFS_OS_H

// this file abstract the os integration
// libwbfs_glue.h for segher tools env.

// standard u8, u32 and co types, + fatal
#include "tools.h"
#include <stdio.h>

#define wbfs_fatal(x) fatal(x)
#define wbfs_error(x) fatal(x)

#include <stdlib.h>
//#define wbfs_malloc(x) malloc(x)
// some allocations are not cleared in libwbfs, so,
// we're using calloc to get always the same reproducible output
#define wbfs_malloc(x) calloc(x,1)
#define wbfs_free(x) free(x)
// alloc memory space suitable for disk io
//#define wbfs_ioalloc(x) malloc(x)
#define wbfs_ioalloc(x) calloc(x,1)
#define wbfs_iofree(x) free(x)
/*
inline static void*wbfs_mallocx(size_t s, int line, const char *fun)
{
    printf("m(%x) %s:%d\n", s, fun, line);
    return malloc(s);
}
#define wbfs_malloc(x) wbfs_mallocx(x,__LINE__,__FUNCTION__)

// alloc memory space suitable for disk io
inline static void*wbfs_ioallocx(size_t s, int line, const char *fun)
{
    printf("io(%x) %s:%d\n", s, fun, line);
    return malloc(s);
}
#define wbfs_ioalloc(x) wbfs_ioallocx(x,__LINE__,__FUNCTION__)
*/

#ifdef unix
#include <arpa/inet.h>
#elif defined(WIN32)
#include <winsock.h>
#endif

// endianess tools
#define wbfs_ntohl(x) ntohl(x)
#define wbfs_ntohs(x) ntohs(x)
#define wbfs_htonl(x) htonl(x)
#define wbfs_htons(x) htons(x)

#include <string.h>
#define wbfs_memcmp(x,y,z) memcmp(x,y,z)
#define wbfs_memcpy(x,y,z) memcpy(x,y,z)
#define wbfs_memset(x,y,z) memset(x,y,z)

#endif
