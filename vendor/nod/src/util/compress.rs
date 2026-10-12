use std::{ffi::CStr, io};

use tracing::instrument;

use crate::{
    Error, Result,
    common::Compression,
    io::wia::{WIACompression, WIADisc},
};

#[derive(Debug, Clone)]
pub enum DecompressionKind {
    None,
    #[cfg(feature = "compress-zlib")]
    Deflate,
    #[cfg(feature = "compress-bzip2")]
    Bzip2,
    #[cfg(feature = "compress-lzma")]
    Lzma(Box<[u8]>),
    #[cfg(feature = "compress-lzma")]
    Lzma2(Box<[u8]>),
    #[cfg(feature = "compress-zstd")]
    Zstandard,
}

impl DecompressionKind {
    pub fn from_wia(disc: &WIADisc) -> Result<Self> {
        let _data = &disc.compr_data[..disc.compr_data_len as usize];
        match disc.compression() {
            WIACompression::None => Ok(Self::None),
            #[cfg(feature = "compress-bzip2")]
            WIACompression::Bzip2 => Ok(Self::Bzip2),
            #[cfg(feature = "compress-lzma")]
            WIACompression::Lzma => Ok(Self::Lzma(Box::from(_data))),
            #[cfg(feature = "compress-lzma")]
            WIACompression::Lzma2 => Ok(Self::Lzma2(Box::from(_data))),
            #[cfg(feature = "compress-zstd")]
            WIACompression::Zstandard => Ok(Self::Zstandard),
            comp => Err(Error::DiscFormat(format!("Unsupported WIA/RVZ compression: {:?}", comp))),
        }
    }

    #[instrument(name = "DecompressionKind::decompress", skip_all)]
    pub fn decompress(&self, buf: &[u8], out: &mut [u8]) -> io::Result<usize> {
        match self {
            DecompressionKind::None => {
                if buf.len() > out.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Decompressed data too large: {} > {}", buf.len(), out.len()),
                    ));
                }
                out[..buf.len()].copy_from_slice(buf);
                Ok(buf.len())
            }
            #[cfg(feature = "compress-zlib")]
            DecompressionKind::Deflate => zlib_api::decompress(buf, out),
            #[cfg(feature = "compress-bzip2")]
            DecompressionKind::Bzip2 => bzip2_api::decompress(buf, out),
            #[cfg(feature = "compress-lzma")]
            DecompressionKind::Lzma(data) => lzma_api::decompress_lzma(data, buf, out),
            #[cfg(feature = "compress-lzma")]
            DecompressionKind::Lzma2(data) => lzma_api::decompress_lzma2(data, buf, out),
            #[cfg(feature = "compress-zstd")]
            DecompressionKind::Zstandard => zstd_api::decompress(buf, out),
        }
    }

    pub fn get_content_size(&self, buf: &[u8]) -> io::Result<Option<usize>> {
        match self {
            DecompressionKind::None => Ok(Some(buf.len())),
            #[cfg(feature = "compress-zstd")]
            DecompressionKind::Zstandard => zstd_api::get_content_size(buf),
            #[allow(unreachable_patterns)] // if compression features are disabled
            _ => Ok(None),
        }
    }
}

pub struct Compressor {
    pub kind: Compression,
    pub buffer: Vec<u8>,
}

impl Clone for Compressor {
    fn clone(&self) -> Self {
        Self { kind: self.kind, buffer: Vec::with_capacity(self.buffer.capacity()) }
    }
}

impl Compressor {
    pub fn new(kind: Compression, buffer_size: usize) -> Self {
        Self { kind, buffer: Vec::with_capacity(buffer_size) }
    }

    /// Compresses the given buffer into `out`. `out`'s capacity will not be extended. Instead, if
    /// the compressed data is larger than `out`, this function will bail and return `false`.
    #[instrument(name = "Compressor::compress", skip_all)]
    pub fn compress(&mut self, buf: &[u8]) -> io::Result<bool> {
        self.buffer.clear();
        match self.kind {
            Compression::None => {
                if self.buffer.capacity() >= buf.len() {
                    self.buffer.extend_from_slice(buf);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            #[cfg(feature = "compress-zlib")]
            Compression::Deflate(level) => zlib_api::compress(buf, level, &mut self.buffer),
            #[cfg(feature = "compress-bzip2")]
            Compression::Bzip2(level) => bzip2_api::compress(buf, level, &mut self.buffer),
            #[cfg(feature = "compress-lzma")]
            Compression::Lzma(level) => lzma_api::compress_lzma(level, buf, &mut self.buffer),
            #[cfg(feature = "compress-lzma")]
            Compression::Lzma2(level) => lzma_api::compress_lzma2(level, buf, &mut self.buffer),
            #[cfg(feature = "compress-zstd")]
            Compression::Zstandard(level) => zstd_api::compress(buf, level, &mut self.buffer),
            #[allow(unreachable_patterns)] // if compression is disabled
            _ => Err(io::Error::other(format!("Unsupported compression: {:?}", self.kind))),
        }
    }
}

#[cfg(feature = "compress-zlib-vendored")]
use libz_sys as zlib_raw;

#[cfg(all(feature = "compress-zlib", not(feature = "compress-zlib-vendored")))]
mod zlib_raw {
    use core::ffi::{c_int, c_ulong};

    #[allow(non_camel_case_types)]
    pub type uLong = c_ulong;
    #[allow(non_camel_case_types)]
    pub type uLongf = c_ulong;

    pub const Z_OK: c_int = 0;
    pub const Z_BUF_ERROR: c_int = -5;

    #[cfg_attr(not(target_env = "msvc"), link(name = "z"))]
    #[cfg_attr(target_env = "msvc", link(name = "zlib", kind = "static"))]
    unsafe extern "C" {
        pub fn uncompress(
            dest: *mut u8,
            destLen: *mut uLongf,
            source: *const u8,
            sourceLen: uLong,
        ) -> c_int;

        pub fn compress2(
            dest: *mut u8,
            destLen: *mut uLongf,
            source: *const u8,
            sourceLen: uLong,
            level: c_int,
        ) -> c_int;
    }
}

#[cfg(feature = "compress-zlib")]
mod zlib_api {
    use std::{ffi::c_int, io};

    use super::zlib_raw;

    pub fn decompress(buf: &[u8], out: &mut [u8]) -> io::Result<usize> {
        let mut out_len = zlib_raw::uLongf::try_from(out.len())
            .map_err(|_| io::Error::other("Output buffer length exceeds zlib limits"))?;
        let in_len = zlib_raw::uLong::try_from(buf.len())
            .map_err(|_| io::Error::other("Input buffer length exceeds zlib limits"))?;
        let code =
            unsafe { zlib_raw::uncompress(out.as_mut_ptr(), &mut out_len, buf.as_ptr(), in_len) };
        match code {
            zlib_raw::Z_OK => Ok(out_len as usize),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("zlib decompression failed with code {code}"),
            )),
        }
    }

    pub fn compress(buf: &[u8], level: u8, out: &mut Vec<u8>) -> io::Result<bool> {
        let in_len = zlib_raw::uLong::try_from(buf.len())
            .map_err(|_| io::Error::other("Input buffer length exceeds zlib limits"))?;
        let capacity = zlib_raw::uLongf::try_from(out.capacity())
            .map_err(|_| io::Error::other("Output buffer capacity exceeds zlib limits"))?;
        out.resize(out.capacity(), 0);
        let mut out_len = capacity;
        let code = unsafe {
            zlib_raw::compress2(
                out.as_mut_ptr(),
                &mut out_len,
                buf.as_ptr(),
                in_len,
                level as c_int,
            )
        };
        match code {
            zlib_raw::Z_OK => {
                out.truncate(out_len as usize);
                Ok(true)
            }
            zlib_raw::Z_BUF_ERROR => {
                out.clear();
                Ok(false)
            }
            _ => Err(io::Error::other(format!("zlib compression failed with code {code}"))),
        }
    }
}

#[cfg(feature = "compress-bzip2-vendored")]
use bzip2_sys as bzip2_raw;

#[cfg(all(feature = "compress-bzip2", not(feature = "compress-bzip2-vendored")))]
mod bzip2_raw {
    use core::ffi::{c_char, c_int, c_uint, c_void};

    pub const BZ_FINISH: c_int = 2;

    pub const BZ_OK: c_int = 0;
    pub const BZ_RUN_OK: c_int = 1;
    pub const BZ_FINISH_OK: c_int = 3;
    pub const BZ_STREAM_END: c_int = 4;
    pub const BZ_OUTBUFF_FULL: c_int = -8;

    #[repr(C)]
    pub struct bz_stream {
        pub next_in: *mut c_char,
        pub avail_in: c_uint,
        pub total_in_lo32: c_uint,
        pub total_in_hi32: c_uint,

        pub next_out: *mut c_char,
        pub avail_out: c_uint,
        pub total_out_lo32: c_uint,
        pub total_out_hi32: c_uint,

        pub state: *mut c_void,

        pub bzalloc: Option<extern "C" fn(*mut c_void, c_int, c_int) -> *mut c_void>,
        pub bzfree: Option<extern "C" fn(*mut c_void, *mut c_void)>,
        pub opaque: *mut c_void,
    }

    macro_rules! abi_compat {
        ($(pub fn $name:ident($($arg:ident: $t:ty),*) -> $ret:ty,)*) => {
            #[cfg(all(windows, target_env = "msvc"))]
            #[link(name = "bz2", kind = "static")]
            unsafe extern "system" {
                $(pub fn $name($($arg: $t),*) -> $ret;)*
            }
            #[cfg(all(windows, not(target_env = "msvc")))]
            #[link(name = "bz2")]
            unsafe extern "system" {
                $(pub fn $name($($arg: $t),*) -> $ret;)*
            }
            #[cfg(not(windows))]
            #[link(name = "bz2")]
            unsafe extern "C" {
                $(pub fn $name($($arg: $t),*) -> $ret;)*
            }
        }
    }

    abi_compat! {
        pub fn BZ2_bzCompressInit(stream: *mut bz_stream,
                                  blockSize100k: c_int,
                                  verbosity: c_int,
                                  workFactor: c_int) -> c_int,
        pub fn BZ2_bzCompress(stream: *mut bz_stream, action: c_int) -> c_int,
        pub fn BZ2_bzCompressEnd(stream: *mut bz_stream) -> c_int,
        pub fn BZ2_bzDecompressInit(stream: *mut bz_stream,
                                    verbosity: c_int,
                                    small: c_int) -> c_int,
        pub fn BZ2_bzDecompress(stream: *mut bz_stream) -> c_int,
        pub fn BZ2_bzDecompressEnd(stream: *mut bz_stream) -> c_int,
    }
}

#[cfg(feature = "compress-bzip2")]
mod bzip2_api {
    use std::{
        ffi::{c_char, c_int, c_uint},
        io,
    };

    use super::bzip2_raw;

    fn total_out(stream: &bzip2_raw::bz_stream) -> usize {
        (((stream.total_out_hi32 as u64) << 32) | stream.total_out_lo32 as u64) as usize
    }

    fn map_code(context: &str, code: c_int) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, format!("{context} failed with code {code}"))
    }

    pub fn decompress(buf: &[u8], out: &mut [u8]) -> io::Result<usize> {
        let in_len = c_uint::try_from(buf.len())
            .map_err(|_| io::Error::other("Input buffer length exceeds bzip2 limits"))?;
        let out_len = c_uint::try_from(out.len())
            .map_err(|_| io::Error::other("Output buffer length exceeds bzip2 limits"))?;
        let mut stream: bzip2_raw::bz_stream = unsafe { std::mem::zeroed() };
        stream.next_in = buf.as_ptr() as *mut c_char;
        stream.avail_in = in_len;
        stream.next_out = out.as_mut_ptr() as *mut c_char;
        stream.avail_out = out_len;

        let init = unsafe { bzip2_raw::BZ2_bzDecompressInit(&mut stream, 0, 0) };
        if init != bzip2_raw::BZ_OK {
            return Err(map_code("bzip2 decompressor init", init));
        }

        let mut ret = bzip2_raw::BZ_OK;
        while stream.avail_out > 0 {
            ret = unsafe { bzip2_raw::BZ2_bzDecompress(&mut stream) };
            match ret {
                bzip2_raw::BZ_OK => {
                    if stream.avail_in == 0 {
                        break;
                    }
                }
                bzip2_raw::BZ_STREAM_END => break,
                _ => break,
            }
        }

        let _ = unsafe { bzip2_raw::BZ2_bzDecompressEnd(&mut stream) };

        match ret {
            bzip2_raw::BZ_STREAM_END => Ok(total_out(&stream)),
            bzip2_raw::BZ_OK if stream.avail_out == 0 => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "bzip2 decompression output buffer too small",
            )),
            _ => Err(map_code("bzip2 decompression", ret)),
        }
    }

    pub fn compress(buf: &[u8], level: u8, out: &mut Vec<u8>) -> io::Result<bool> {
        let in_len = c_uint::try_from(buf.len())
            .map_err(|_| io::Error::other("Input buffer length exceeds bzip2 limits"))?;
        let out_len = c_uint::try_from(out.capacity())
            .map_err(|_| io::Error::other("Output buffer capacity exceeds bzip2 limits"))?;

        out.resize(out.capacity(), 0);

        let mut stream: bzip2_raw::bz_stream = unsafe { std::mem::zeroed() };
        stream.next_in = buf.as_ptr() as *mut c_char;
        stream.avail_in = in_len;
        stream.next_out = out.as_mut_ptr() as *mut c_char;
        stream.avail_out = out_len;

        let init = unsafe { bzip2_raw::BZ2_bzCompressInit(&mut stream, level as c_int, 0, 30) };
        if init != bzip2_raw::BZ_OK {
            return Err(map_code("bzip2 compressor init", init));
        }

        let mut ret = bzip2_raw::BZ_RUN_OK;
        while ret == bzip2_raw::BZ_RUN_OK || ret == bzip2_raw::BZ_FINISH_OK {
            ret = unsafe { bzip2_raw::BZ2_bzCompress(&mut stream, bzip2_raw::BZ_FINISH) };
        }

        let _ = unsafe { bzip2_raw::BZ2_bzCompressEnd(&mut stream) };

        match ret {
            bzip2_raw::BZ_STREAM_END => {
                out.truncate(total_out(&stream));
                Ok(true)
            }
            bzip2_raw::BZ_FINISH_OK if stream.avail_out == 0 => {
                out.clear();
                Ok(false)
            }
            bzip2_raw::BZ_OUTBUFF_FULL => {
                out.clear();
                Ok(false)
            }
            _ => Err(map_code("bzip2 compression", ret)),
        }
    }
}

#[cfg(feature = "compress-zstd-vendored")]
use zstd_sys as zstd_raw;

#[cfg(all(feature = "compress-zstd", not(feature = "compress-zstd-vendored")))]
mod zstd_raw {
    use core::ffi::{c_char, c_int, c_uint, c_ulonglong, c_void};

    pub const ZSTD_CONTENTSIZE_UNKNOWN: i32 = -1;
    pub const ZSTD_CONTENTSIZE_ERROR: i32 = -2;

    #[cfg_attr(not(target_env = "msvc"), link(name = "zstd"))]
    #[cfg_attr(target_env = "msvc", link(name = "zstd", kind = "static"))]
    unsafe extern "C" {
        pub fn ZSTD_compress(
            dst: *mut c_void,
            dstCapacity: usize,
            src: *const c_void,
            srcSize: usize,
            compressionLevel: c_int,
        ) -> usize;

        pub fn ZSTD_decompress(
            dst: *mut c_void,
            dstCapacity: usize,
            src: *const c_void,
            srcSize: usize,
        ) -> usize;

        pub fn ZSTD_getFrameContentSize(src: *const c_void, srcSize: usize) -> c_ulonglong;
        pub fn ZSTD_compressBound(srcSize: usize) -> usize;
        pub fn ZSTD_isError(result: usize) -> c_uint;
        pub fn ZSTD_getErrorName(result: usize) -> *const c_char;
    }
}

#[cfg(feature = "compress-zstd")]
pub(crate) mod zstd_api {
    use std::{ffi::c_void, io};

    use super::{CStr, zstd_raw};

    const ZSTD_ERROR_DST_SIZE_TOO_SMALL: usize = 70usize.wrapping_neg();

    pub fn compress_bound(size: usize) -> usize { unsafe { zstd_raw::ZSTD_compressBound(size) } }

    fn map_error_code(code: usize) -> io::Error {
        let msg = unsafe { CStr::from_ptr(zstd_raw::ZSTD_getErrorName(code)) }
            .to_string_lossy()
            .into_owned();
        io::Error::other(msg)
    }

    pub fn decompress(buf: &[u8], out: &mut [u8]) -> io::Result<usize> {
        let code = unsafe {
            zstd_raw::ZSTD_decompress(
                out.as_mut_ptr().cast::<c_void>(),
                out.len(),
                buf.as_ptr().cast::<c_void>(),
                buf.len(),
            )
        };
        if unsafe { zstd_raw::ZSTD_isError(code) } != 0 {
            return Err(map_error_code(code));
        }
        Ok(code)
    }

    pub fn compress(buf: &[u8], level: i8, out: &mut Vec<u8>) -> io::Result<bool> {
        out.resize(out.capacity(), 0);
        let code = unsafe {
            zstd_raw::ZSTD_compress(
                out.as_mut_ptr().cast::<c_void>(),
                out.len(),
                buf.as_ptr().cast::<c_void>(),
                buf.len(),
                level as i32,
            )
        };
        if unsafe { zstd_raw::ZSTD_isError(code) } != 0 {
            // dstSize_tooSmall means compressed data doesn't fit; signal caller to store uncompressed
            if code == ZSTD_ERROR_DST_SIZE_TOO_SMALL {
                out.clear();
                return Ok(false);
            }
            return Err(map_error_code(code));
        }
        out.truncate(code);
        Ok(true)
    }

    pub fn get_content_size(buf: &[u8]) -> io::Result<Option<usize>> {
        let size =
            unsafe { zstd_raw::ZSTD_getFrameContentSize(buf.as_ptr().cast::<c_void>(), buf.len()) };
        if size == zstd_raw::ZSTD_CONTENTSIZE_UNKNOWN as u64 {
            return Ok(None);
        } else if size == zstd_raw::ZSTD_CONTENTSIZE_ERROR as u64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Zstandard frame header",
            ));
        }
        usize::try_from(size)
            .map(Some)
            .map_err(|_| io::Error::other("Zstandard frame size exceeds usize"))
    }
}

#[cfg(feature = "compress-lzma-vendored")]
use liblzma_sys as lzma_raw;

#[cfg(all(feature = "compress-lzma", not(feature = "compress-lzma-vendored")))]
mod lzma_raw {
    #![allow(non_camel_case_types)]

    use core::ffi::{c_uchar, c_uint, c_void};

    #[cfg(target_env = "msvc")]
    pub type __enum_ty = core::ffi::c_int;
    #[cfg(not(target_env = "msvc"))]
    pub type __enum_ty = core::ffi::c_uint;

    pub type lzma_bool = c_uchar;
    pub type lzma_ret = __enum_ty;
    pub type lzma_vli = u64;
    pub type lzma_mode = __enum_ty;
    pub type lzma_match_finder = __enum_ty;

    pub const LZMA_OK: lzma_ret = 0;
    pub const LZMA_OPTIONS_ERROR: lzma_ret = 8;
    pub const LZMA_DATA_ERROR: lzma_ret = 9;
    pub const LZMA_BUF_ERROR: lzma_ret = 10;
    pub const LZMA_PROG_ERROR: lzma_ret = 11;

    pub const LZMA_PRESET_DEFAULT: u32 = 6;

    pub const LZMA_DICT_SIZE_MIN: u32 = 4096;

    pub const LZMA_VLI_UNKNOWN: lzma_vli = u64::MAX;

    pub const LZMA_FILTER_LZMA1: lzma_vli = 0x4000000000000001;
    pub const LZMA_FILTER_LZMA2: lzma_vli = 0x21;

    #[repr(C)]
    pub struct lzma_filter {
        pub id: lzma_vli,
        pub options: *mut c_void,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct lzma_options_lzma {
        pub dict_size: u32,
        pub preset_dict: *const u8,
        pub preset_dict_size: u32,
        pub lc: u32,
        pub lp: u32,
        pub pb: u32,
        pub mode: lzma_mode,
        pub nice_len: u32,
        pub mf: lzma_match_finder,
        pub depth: u32,

        reserved_int1: u32,
        reserved_int2: u32,
        reserved_int3: u32,
        reserved_int4: u32,
        reserved_int5: u32,
        reserved_int6: u32,
        reserved_int7: u32,
        reserved_int8: u32,
        reserved_enum1: __enum_ty,
        reserved_enum2: __enum_ty,
        reserved_enum3: __enum_ty,
        reserved_enum4: __enum_ty,
        reserved_ptr1: *mut c_void,
        reserved_ptr2: *mut c_void,
    }

    pub type lzma_allocator = c_void;

    #[cfg_attr(not(target_env = "msvc"), link(name = "lzma"))]
    #[cfg_attr(target_env = "msvc", link(name = "lzma", kind = "static"))]
    unsafe extern "C" {
        pub fn lzma_raw_buffer_encode(
            filters: *const lzma_filter,
            allocator: *const lzma_allocator,
            input: *const u8,
            in_size: usize,
            out: *mut u8,
            out_pos: *mut usize,
            out_size: usize,
        ) -> lzma_ret;

        pub fn lzma_raw_buffer_decode(
            filters: *const lzma_filter,
            allocator: *const lzma_allocator,
            input: *const u8,
            in_pos: *mut usize,
            in_size: usize,
            out: *mut u8,
            out_pos: *mut usize,
            out_size: usize,
        ) -> lzma_ret;

        pub fn lzma_lzma_preset(options: *mut lzma_options_lzma, preset: c_uint) -> lzma_bool;
    }
}

#[cfg(feature = "compress-lzma")]
pub(crate) mod lzma_api {
    use std::{
        cmp::Ordering,
        ffi::c_void,
        io::{self, ErrorKind},
    };

    use super::lzma_raw;

    fn map_error_code(code: lzma_raw::lzma_ret, context: &str) -> io::Error {
        let reason = match code {
            x if x == lzma_raw::LZMA_OPTIONS_ERROR => "options error",
            x if x == lzma_raw::LZMA_DATA_ERROR => "data error",
            x if x == lzma_raw::LZMA_BUF_ERROR => "output buffer too small",
            x if x == lzma_raw::LZMA_PROG_ERROR => "program error",
            _ => "unknown error",
        };
        io::Error::new(ErrorKind::InvalidData, format!("{context}: {reason} ({code})"))
    }

    fn preset_options(level: u32) -> io::Result<lzma_raw::lzma_options_lzma> {
        let mut options: lzma_raw::lzma_options_lzma = unsafe { std::mem::zeroed() };
        if unsafe { lzma_raw::lzma_lzma_preset(&mut options, level) } != 0 {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                format!("Invalid LZMA preset level {level}"),
            ));
        }
        Ok(options)
    }

    fn lzma_lclppb_decode(options: &mut lzma_raw::lzma_options_lzma, byte: u8) -> io::Result<()> {
        let mut d = byte as u32;
        if d >= (9 * 5 * 5) {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Invalid LZMA props byte: {d}"),
            ));
        }
        options.lc = d % 9;
        d /= 9;
        options.pb = d / 5;
        options.lp = d % 5;
        Ok(())
    }

    fn lzma_props_decode(props: &[u8]) -> io::Result<lzma_raw::lzma_options_lzma> {
        if props.len() != 5 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Invalid LZMA props length: {}", props.len()),
            ));
        }
        let mut options = preset_options(lzma_raw::LZMA_PRESET_DEFAULT)?;
        lzma_lclppb_decode(&mut options, props[0])?;
        options.dict_size = u32::from_le_bytes([props[1], props[2], props[3], props[4]]);
        Ok(options)
    }

    fn lzma2_props_decode(props: &[u8]) -> io::Result<lzma_raw::lzma_options_lzma> {
        if props.len() != 1 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Invalid LZMA2 props length: {}", props.len()),
            ));
        }
        let d = props[0] as u32;
        let mut options = preset_options(lzma_raw::LZMA_PRESET_DEFAULT)?;
        options.dict_size = match d.cmp(&40) {
            Ordering::Greater => {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("Invalid LZMA2 props byte: {d}"),
                ));
            }
            Ordering::Equal => u32::MAX,
            Ordering::Less => (2 | (d & 1)) << (d / 2 + 11),
        };
        Ok(options)
    }

    fn lzma_lclppb_encode(options: &lzma_raw::lzma_options_lzma) -> io::Result<u8> {
        let byte = (options.pb * 5 + options.lp) * 9 + options.lc;
        if byte >= (9 * 5 * 5) {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Invalid LZMA props byte: {byte}"),
            ));
        }
        Ok(byte as u8)
    }

    fn lzma_props_encode(options: &lzma_raw::lzma_options_lzma) -> io::Result<[u8; 5]> {
        let mut props = [0u8; 5];
        props[0] = lzma_lclppb_encode(options)?;
        props[1..].copy_from_slice(&options.dict_size.to_le_bytes());
        Ok(props)
    }

    fn get_dist_slot(dist: u32) -> u32 {
        if dist <= 4 {
            dist
        } else {
            let i = dist.leading_zeros() ^ 31;
            (i + i) + ((dist >> (i - 1)) & 1)
        }
    }

    fn lzma2_props_encode(options: &lzma_raw::lzma_options_lzma) -> [u8; 1] {
        let mut d = options.dict_size.max(lzma_raw::LZMA_DICT_SIZE_MIN);
        d -= 1;
        d |= d >> 2;
        d |= d >> 3;
        d |= d >> 4;
        d |= d >> 8;
        d |= d >> 16;
        if d == u32::MAX { [40] } else { [(get_dist_slot(d + 1) - 24) as u8] }
    }

    fn make_filters(
        filter_id: lzma_raw::lzma_vli,
        options: &mut lzma_raw::lzma_options_lzma,
    ) -> [lzma_raw::lzma_filter; 2] {
        [
            lzma_raw::lzma_filter {
                id: filter_id,
                options: options as *mut lzma_raw::lzma_options_lzma as *mut c_void,
            },
            lzma_raw::lzma_filter { id: lzma_raw::LZMA_VLI_UNKNOWN, options: std::ptr::null_mut() },
        ]
    }

    fn compress_raw(
        filter_id: lzma_raw::lzma_vli,
        level: u8,
        buf: &[u8],
        out: &mut Vec<u8>,
    ) -> io::Result<bool> {
        let mut options = preset_options(level as u32)?;
        let filters = make_filters(filter_id, &mut options);
        out.resize(out.capacity(), 0);
        let mut out_pos = 0usize;
        let ret = unsafe {
            lzma_raw::lzma_raw_buffer_encode(
                filters.as_ptr(),
                std::ptr::null(),
                buf.as_ptr(),
                buf.len(),
                out.as_mut_ptr(),
                &mut out_pos,
                out.len(),
            )
        };
        match ret {
            x if x == lzma_raw::LZMA_OK => {
                out.truncate(out_pos);
                Ok(true)
            }
            x if x == lzma_raw::LZMA_BUF_ERROR => {
                out.clear();
                Ok(false)
            }
            _ => Err(map_error_code(ret, "LZMA compression failed")),
        }
    }

    fn decompress_raw(
        filter_id: lzma_raw::lzma_vli,
        props: &[u8],
        buf: &[u8],
        out: &mut [u8],
    ) -> io::Result<usize> {
        let mut options = if filter_id == lzma_raw::LZMA_FILTER_LZMA1 {
            lzma_props_decode(props)?
        } else {
            lzma2_props_decode(props)?
        };
        let filters = make_filters(filter_id, &mut options);
        let mut in_pos = 0usize;
        let mut out_pos = 0usize;
        let ret = unsafe {
            lzma_raw::lzma_raw_buffer_decode(
                filters.as_ptr(),
                std::ptr::null(),
                buf.as_ptr(),
                &mut in_pos,
                buf.len(),
                out.as_mut_ptr(),
                &mut out_pos,
                out.len(),
            )
        };
        if ret != lzma_raw::LZMA_OK {
            return Err(map_error_code(ret, "LZMA decompression failed"));
        }
        if in_pos != buf.len() {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("LZMA decompression consumed {} of {} bytes", in_pos, buf.len()),
            ));
        }
        Ok(out_pos)
    }

    pub fn compress_lzma(level: u8, buf: &[u8], out: &mut Vec<u8>) -> io::Result<bool> {
        compress_raw(lzma_raw::LZMA_FILTER_LZMA1, level, buf, out)
    }

    pub fn compress_lzma2(level: u8, buf: &[u8], out: &mut Vec<u8>) -> io::Result<bool> {
        compress_raw(lzma_raw::LZMA_FILTER_LZMA2, level, buf, out)
    }

    pub fn decompress_lzma(props: &[u8], buf: &[u8], out: &mut [u8]) -> io::Result<usize> {
        decompress_raw(lzma_raw::LZMA_FILTER_LZMA1, props, buf, out)
    }

    pub fn decompress_lzma2(props: &[u8], buf: &[u8], out: &mut [u8]) -> io::Result<usize> {
        decompress_raw(lzma_raw::LZMA_FILTER_LZMA2, props, buf, out)
    }

    pub fn lzma_props_encode_preset(level: u32) -> io::Result<[u8; 5]> {
        let options = preset_options(level)?;
        lzma_props_encode(&options)
    }

    pub fn lzma2_props_encode_preset(level: u32) -> io::Result<[u8; 1]> {
        let options = preset_options(level)?;
        Ok(lzma2_props_encode(&options))
    }
}
