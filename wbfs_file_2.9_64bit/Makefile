PROGS = negentig scrub wbfs 
#COMMON = tools.o bn.o ec.o disc_usage_table.o libwbfs.o wiidisc.o rijndael.o splits.o
COMMON = tools.o bn.o ec.o libwbfs.o wiidisc.o rijndael.o splits.o
DEFINES = -DLARGE_FILES -D_FILE_OFFSET_BITS=64

CC = gcc
LIBS = -lcrypto 

ifdef CYGWIN
MSYSTEM = MINGW32
CC = /c/MinGW/bin/gcc
endif

ifeq ($(MSYSTEM),MINGW32)

	# http://sourceforge.net/projects/mingw/files/
	LIBS = -lcrypto -lwsock32 -lm
	ADD_CF = -I/mingw/include
	ADD_LF = -static
	COMMON += libwbfs_win32.o 

else ifeq ($(shell uname),Darwin)

	COMMON += libwbfs_osx.o 
	LIBS = -lcrypto -ldl -lm
	ADD_CF = -arch i386 -arch ppc
	ADD_LF = -arch i386 -arch ppc

else
	COMMON += libwbfs_linux.o 
	ADD_LF = -static
	LIBS = -lcrypto -ldl -lm
endif

#CFLAGS = -Wall -W -Os -Ilibwbfs -I.
#CFLAGS = -Wall -W  -g -ggdb $(ADD_CF) -Ilibwbfs -I.
CFLAGS = -Wall -W -O $(ADD_CF) -Ilibwbfs -I.
#LDFLAGS = -m32 -static
LDFLAGS = $(ADD_LF) 

VPATH+=libwbfs
OBJS = $(patsubst %,%.o,$(PROGS)) $(COMMON)

wbfs_file: wbfs.o $(COMMON) Makefile
	$(CC) $(CFLAGS) $(LDFLAGS) $< $(COMMON) $(LIBS) -o $@

all: $(PROGS)

$(PROGS): %: %.o $(COMMON) Makefile
	$(CC) $(CFLAGS) $(LDFLAGS) $< $(COMMON) $(LIBS) -o $@

# $(OBJS): %.o: %.c tools.h Makefile
$(OBJS): %.o: %.c tools.h
	$(CC) $(CFLAGS) $(DEFINES) -c $< -o $@ 

clean:
	-rm -f $(OBJS) $(PROGS)
