// Copyright 2007,2008  Segher Boessenkool  <segher@kernel.crashing.org>
// Licensed under the terms of the GNU GPL, version 2
// http://www.gnu.org/licenses/old-licenses/gpl-2.0.txt

#include "tools.h"

#include <stddef.h>	// to accommodate certain broken versions of openssl
#include <openssl/md5.h>
#include <openssl/aes.h>
#include <openssl/sha.h>
#include <stdarg.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <time.h>
#include <sys/time.h>
#include <math.h>
//
// basic data types
//

u16 be16(const u8 *p)
{
	return (p[0] << 8) | p[1];
}

u32 be32(const u8 *p)
{
	return (p[0] << 24) | (p[1] << 16) | (p[2] << 8) | p[3];
}

u64 be64(const u8 *p)
{
	return ((u64)be32(p) << 32) | be32(p + 4);
}

u64 be34(const u8 *p)
{
	return 4 * (u64)be32(p);
}

void wbe16(u8 *p, u16 x)
{
	p[0] = x >> 8;
	p[1] = x;
}

void wbe32(u8 *p, u32 x)
{
	wbe16(p, x >> 16);
	wbe16(p + 2, x);
}

void wbe64(u8 *p, u64 x)
{
	wbe32(p, x >> 32);
	wbe32(p + 4, x);
}

//
// crypto
//

void md5(u8 *data, u32 len, u8 *hash)
{
	MD5(data, len, hash);
}

void sha(u8 *data, u32 len, u8 *hash)
{
	SHA1(data, len, hash);
}

void get_key(const char *name, u8 *key, u32 len)
{
	char path[256];
	char *home;
	FILE *fp;

	home = getenv("HOME");
	if (home == 0)
		fatal("cannot find HOME");
	snprintf(path, sizeof path, "%s/.wii/%s", home, name);

	fp = fopen(path, "rb");
	if (fp == 0)
		fatal("cannot open %s", name);
	if (fread(key, len, 1, fp) != 1)
		fatal("error reading %s", name);
	fclose(fp);
}

void aes_cbc_dec(u8 *key, u8 *iv, u8 *in, u32 len, u8 *out)
{
	AES_KEY aes_key;

	AES_set_decrypt_key(key, 128, &aes_key);
	AES_cbc_encrypt(in, out, len, &aes_key, iv, AES_DECRYPT);
}

void aes_cbc_enc(u8 *key, u8 *iv, u8 *in, u32 len, u8 *out)
{
	AES_KEY aes_key;

	AES_set_encrypt_key(key, 128, &aes_key);
	AES_cbc_encrypt(in, out, len, &aes_key, iv, AES_ENCRYPT);
}

void decrypt_title_key(u8 *tik, u8 *title_key)
{
	u8 common_key[16];
	u8 iv[16];

	get_key("common-key", common_key, 16);

	memset(iv, 0, sizeof iv);
	memcpy(iv, tik + 0x01dc, 8);
	aes_cbc_dec(common_key, iv, tik + 0x01bf, 16, title_key);
        printf("title key: %02x %02x %02x\n",title_key[0],title_key[1],title_key[2]);
}

static u8 root_key[0x204];
static u8 *get_root_key(void)
{
	//get_key("root-key", root_key, sizeof root_key);
	return root_key;
}

static u32 get_sig_len(u8 *sig)
{
	u32 type;

	type = be32(sig);
	switch (type - 0x10000) {
	case 0:
		return 0x240;

	case 1:
		return 0x140;

	case 2:
		return 0x80;
	}

	printf("get_sig_len(): unhandled sig type %08x\n", type);
	return 0;
}

static u32 get_sub_len(u8 *sub)
{
	u32 type;

	type = be32(sub + 0x40);
	switch (type) {
	case 0:
		return 0x2c0;

	case 1:
		return 0x1c0;

	case 2:
		return 0x100;
	}

	printf("get_sub_len(): unhandled sub type %08x\n", type);
	return 0;
}

int check_ec(u8 *ng, u8 *ap, u8 *sig, u8 *sig_hash)
{
	u8 ap_hash[20];
	u8 *ng_Q, *ap_R, *ap_S;
	u8 *ap_Q, *sig_R, *sig_S;

	ng_Q = ng + 0x0108;
	ap_R = ap + 0x04;
	ap_S = ap + 0x22;

	SHA1(ap + 0x80, 0x100, ap_hash);

	ap_Q = ap + 0x0108;
	sig_R = sig;
	sig_S = sig + 30;

	return check_ecdsa(ng_Q, ap_R, ap_S, ap_hash)
	       && check_ecdsa(ap_Q, sig_R, sig_S, sig_hash);
}

static int check_rsa(u8 *h, u8 *sig, u8 *key, u32 n)
{
	u8 correct[0x200];
	u8 x[0x200];
	static const u8 ber[16] = "\x00\x30\x21\x30\x09\x06\x05\x2b"
	                          "\x0e\x03\x02\x1a\x05\x00\x04\x14";

//printf("n = %x\n", n);
//printf("key:\n");
//hexdump(key, n);
//printf("sig:\n");
//hexdump(sig, n);

	correct[0] = 0;
	correct[1] = 1;
	memset(correct + 2, 0xff, n - 38);
	memcpy(correct + n - 36, ber, 16);
	memcpy(correct + n - 20, h, 20);
//printf("correct is:\n");
//hexdump(correct, n);

	bn_exp(x, sig, key, n, key + n, 4);
//printf("x is:\n");
//hexdump(x, n);

	if (memcmp(correct, x, n) == 0)
		return 0;

	return -5;
}

static int check_hash(u8 *h, u8 *sig, u8 *key)
{
	u32 type;

	type = be32(sig) - 0x10000;
	if (type != be32(key + 0x40))
		return -6;

	switch (type) {
	case 1:
		return check_rsa(h, sig + 4, key + 0x88, 0x100);
	}

	return -7;
}

static u8 *find_cert_in_chain(u8 *sub, u8 *cert, u32 cert_len)
{
	char parent[64];
	char *child;
	u32 sig_len, sub_len;
	u8 *p;
	u8 *issuer;

	strncpy(parent, (char*)sub, sizeof parent);
	parent[sizeof parent - 1] = 0;
	child = strrchr(parent, '-');
	if (child)
		*child++ = 0;
	else {
		*parent = 0;
		child = (char*)sub;
	}

	for (p = cert; p < cert + cert_len; p += sig_len + sub_len) {
		sig_len = get_sig_len(p);
		if (sig_len == 0)
			return 0;
		issuer = p + sig_len;
		sub_len = get_sub_len(issuer);
		if (sub_len == 0)
			return 0;

		if (strcmp(parent, (char*)issuer) == 0
		    && strcmp(child, (char*)issuer + 0x44) == 0)
			return p;
	}

	return 0;
}

int check_cert_chain(u8 *data, u32 data_len, u8 *cert, u32 cert_len)
{
	u8 *sig;
	u8 *sub;
	u32 sig_len;
	u32 sub_len;
	u8 h[20];
	u8 *key_cert;
	u8 *key;
	int ret;
	sig = data;
	sig_len = get_sig_len(sig);
	if (sig_len == 0)
		return -1;
	sub = data + sig_len;
	sub_len = data_len - sig_len;
	if (sub_len == 0)
		return -2;

	for (;;) {
                printf(">>>>>> checking sig by %s...\n", sub);
                if (strcmp((char*)sub, "Root") == 0) {
			key = get_root_key();
			sha(sub, sub_len, h);
			if (be32(sig) != 0x10000)
				return -8;
			return check_rsa(h, sig + 4, key, 0x200);
		}

		key_cert = find_cert_in_chain(sub, cert, cert_len);
		if (key_cert == 0)
			return -3;

		key = key_cert + get_sig_len(key_cert);

		sha(sub, sub_len, h);
		ret = check_hash(h, sig, key);
		if (ret)
			return ret;

		sig = key_cert;
		sig_len = get_sig_len(sig);
		if (sig_len == 0)
			return -4;
		sub = sig + sig_len;
		sub_len = get_sub_len(sub);
		if (sub_len == 0)
			return -5;
	}
}

//
// compression
//

void do_yaz0(u8 *in, u32 in_size, u8 *out, u32 out_size)
{
	u32 nout;
	u8 bits;
	u32 nbits;
	u32 n, d, i;
        in_size++;
	bits = 0;
	nbits = 0;
	in += 0x10;
	for (nout = 0; nout < out_size; ) {
		if (nbits == 0) {
			bits = *in++;
			nbits = 8;
		}

		if ((bits & 0x80) != 0) {
			*out++ = *in++;
			nout++;
		} else {
			n = *in++;
			d = *in++;
			d |= (n << 8) & 0xf00;
			n >>= 4;
			if (n == 0)
				n = 0x10 + *in++;
			n += 2;
			d++;

			for (i = 0; i < n; i++) {
				*out = *(out - d);
				out++;
			}
			nout += n;
		}

		nbits--;
		bits <<= 1;
	};
}

//
// error handling
//

void fatal(const char *s, ...)
{
	char message[256];
	va_list ap;

	va_start(ap, s);
	vsnprintf(message, sizeof message, s, ap);

	perror(message);

	exit(1);
}

//
// output formatting
//

void print_bytes(u8 *x, u32 n)
{
	u32 i;

	for (i = 0; i < n; i++)
		printf("%02x", x[i]);
}

void hexdump(u8 *x, u32 n)
{
	u32 i, j;

	for (i = 0; i < n; i += 16) {
		printf("%04x:", i);
		for (j = 0; j < 16 && i + j < n; j++) {
			if ((j & 3) == 0)
				printf(" ");
			printf("%02x", *x++);
		}
		printf("\n");
	}
}

void dump_tmd(u8 *tmd)
{
	u32 i, n;
	u8 *p;

	printf("       issuer: %s\n", tmd + 0x140);
	printf("  sys_version: %016llx\n", be64(tmd + 0x0184));
	printf("     title_id: %016llx\n", be64(tmd + 0x018c));
	printf("   title_type: %08x\n", be32(tmd + 0x0194));
	printf("     group_id: %04x\n", be16(tmd + 0x0198));
	printf("title_version: %04x\n", be16(tmd + 0x01dc));
	printf(" num_contents: %04x\n", be16(tmd + 0x01de));
	printf("   boot_index: %04x\n", be16(tmd + 0x01e0));

	n = be16(tmd + 0x01de);
	p = tmd + 0x01e4;
	for (i = 0; i < n; i++) {
		printf("cid %08x  index %04x  type %04x  size %08llx\n",
		       be32(p), be16(p + 4), be16(p + 6), be64(p + 8));
		p += 0x24;
	}
}

double spinner_bs = 32 * 1024;

long long tv_2_ms(struct timeval *tv)
{
	return (long long)tv->tv_sec * 1000 + (long long)tv->tv_usec / 1000;
}

void spinner(u64 x, u64 max)
{
	const double MB = 1024. * 1024.;

	static long long start_ms;
	static long long update_ms;
	static double expected_total;
	static int updates;

	double d, delta;
	double percent;
	double copied, size;
	u32 h, m, s;
	struct timeval tv;
	long long ms;
	long dl;

	if (x == 0) {
		fflush(stderr); // mingw...
		//start_time = time(0);
		gettimeofday(&tv, NULL);
		start_ms = tv_2_ms(&tv);
		update_ms = start_ms;
		updates = 0;
		expected_total = 300;
	}

	//d = time(0) - start_time;
	gettimeofday(&tv, NULL);
	ms = tv_2_ms(&tv);
	delta = d = (double)(ms - start_ms) / 1000.;

	if (x && d != 0)
		expected_total = (3 * expected_total + d * max / x) / 4;

	if (expected_total > d)
		d = expected_total - d;
	else
		d = 1;
	if (x == max) d = 0;

	dl = lround(d);
	h = dl / 3600;
	m = (dl / 60) % 60;
	s = dl % 60;
	percent = 100.0 * x / max;
	copied = spinner_bs * (double)x;
	size = spinner_bs * (double)max;

	// 50ms = 1000/50 = 20 fps
	if (ms - update_ms >= 50 || x == 0 || x == max) {
		printf("%5.2f%% (%c) ETA: %d:%02d:%02d ",
				percent, "/|\\-"[updates % 4], h, m, s);
		printf("(%5.2fMB of %5.2fMB ~ %5.2fMB/s) time: %.2fs ",
				copied / MB, size / MB,
				copied / (delta?delta:1) / MB,
				delta);
		//printf("%d", (int)(ms - update_ms));
		printf("  \r");
		fflush(stdout);
		update_ms = ms;
		updates++;
	}
	if (x == max) {
		dl = lround(delta);
		h = dl / 3600;
		m = (dl / 60) % 60;
		s = dl % 60;
		printf("\nDone in  %d:%02d:%02d %60s\n", h, m, s, " ");
		fflush(stdout);
		return;
	}
}

