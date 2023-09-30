// Copyright 2023  Manuel Quarneti  <manuel.quarneti@proton.me>
// Copyright 2007,2008  Segher Boessenkool  <segher@kernel.crashing.org>
// Licensed under the terms of the GNU GPL, version 2
// http://www.gnu.org/licenses/old-licenses/gpl-2.0.txt

#ifndef _TOOLS_H
#define _TOOLS_H

#include <stdint.h>

// basic data types
typedef unsigned char u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;

u16 be16(const u8 *p);
u32 be32(const u8 *p);
u64 be64(const u8 *p);
u64 be34(const u8 *p);

void wbe16(u8 *p, u16 x);
void wbe32(u8 *p, u32 x);
void wbe64(u8 *p, u64 x);

//#define round_down(x,n) ((x) & -(n))
#define round_up(x,n) (-(-(x) & -(n)))

// bignum
int bn_compare(u8 *a, u8 *b, u32 n);
void bn_sub_modulus(u8 *a, u8 *N, u32 n);
void bn_add(u8 *d, u8 *a, u8 *b, u8 *N, u32 n);
void bn_mul(u8 *d, u8 *a, u8 *b, u8 *N, u32 n);
void bn_inv(u8 *d, u8 *a, u8 *N, u32 n);	// only for prime N
void bn_exp(u8 *d, u8 *a, u8 *N, u32 n, u8 *e, u32 en);

// crypto
void get_key(const char *name, u8 *key, u32 len);
void generate_ecdsa(u8 *R, u8 *S, u8 *k, u8 *hash);
int check_ecdsa(u8 *Q, u8 *R, u8 *S, u8 *hash);
void ec_priv_to_pub(u8 *k, u8 *Q);

// compression
void do_yaz0(u8 *in, u32 in_size, u8 *out, u32 out_size);

// error handling
void fatal(const char *s, ...);

// output formatting
void print_bytes(u8 *x, u32 n);
void hexdump(u8 *x, u32 n);
void dump_tmd(u8 *tmd);
void spinner(u64 x, u64 max);

#endif
