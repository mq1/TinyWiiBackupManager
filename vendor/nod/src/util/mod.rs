//! Utility functions and types.

use std::{
    io,
    io::{Read, Seek, SeekFrom},
    ops::{Div, Rem},
};

use io::{BufRead, Write};

pub(crate) mod aes;
pub(crate) mod compress;
pub(crate) mod digest;
pub mod lfg;
pub(crate) mod read;

/// Copies from a [`BufRead`] to a [`Write`] without allocating a buffer.
pub fn buf_copy<R, W>(reader: &mut R, writer: &mut W) -> io::Result<u64>
where
    R: BufRead + ?Sized,
    W: Write + ?Sized,
{
    let mut copied = 0;
    loop {
        let buf = reader.fill_buf()?;
        let len = buf.len();
        if len == 0 {
            break;
        }
        writer.write_all(buf)?;
        reader.consume(len);
        copied += len as u64;
    }
    Ok(copied)
}

/// A reader with a fixed window.
#[derive(Clone)]
pub struct WindowedReader<T>
where T: BufRead + Seek
{
    base: T,
    pos: u64,
    begin: u64,
    end: u64,
}

impl<T> WindowedReader<T>
where T: BufRead + Seek
{
    /// Creates a new windowed stream with offset and size.
    ///
    /// Seeks underlying stream immediately.
    #[inline]
    pub fn new(mut base: T, offset: u64, size: u64) -> io::Result<Self> {
        base.seek(SeekFrom::Start(offset))?;
        Ok(Self { base, pos: offset, begin: offset, end: offset + size })
    }

    /// Returns the length of the window.
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> u64 { self.end - self.begin }
}

impl<T> Read for WindowedReader<T>
where T: BufRead + Seek
{
    #[inline]
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let buf = self.fill_buf()?;
        let len = buf.len().min(out.len());
        out[..len].copy_from_slice(&buf[..len]);
        self.consume(len);
        Ok(len)
    }
}

impl<T> BufRead for WindowedReader<T>
where T: BufRead + Seek
{
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let limit = self.end.saturating_sub(self.pos);
        if limit == 0 {
            return Ok(&[]);
        }
        let buf = self.base.fill_buf()?;
        let max = (buf.len() as u64).min(limit) as usize;
        Ok(&buf[..max])
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        self.base.consume(amt);
        self.pos += amt as u64;
    }
}

impl<T> Seek for WindowedReader<T>
where T: BufRead + Seek
{
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let mut pos = match pos {
            SeekFrom::Start(p) => self.begin + p,
            SeekFrom::End(p) => self.end.saturating_add_signed(p),
            SeekFrom::Current(p) => self.pos.saturating_add_signed(p),
        };
        if pos < self.begin {
            pos = self.begin;
        } else if pos > self.end {
            pos = self.end;
        }
        let result = self.base.seek(SeekFrom::Start(pos))?;
        self.pos = result;
        Ok(result - self.begin)
    }

    #[inline]
    fn stream_position(&mut self) -> io::Result<u64> { Ok(self.pos) }
}

#[inline(always)]
pub(crate) fn div_rem<T>(x: T, y: T) -> (T, T)
where T: Div<Output = T> + Rem<Output = T> + Copy {
    (x / y, x % y)
}

pub(crate) trait Align {
    fn align_up(self, align: Self) -> Self;

    fn align_down(self, align: Self) -> Self;
}

macro_rules! impl_align {
    ($ty:ident) => {
        impl Align for $ty {
            #[inline(always)]
            fn align_up(self, align: Self) -> Self { (self + (align - 1)) & !(align - 1) }

            #[inline(always)]
            fn align_down(self, align: Self) -> Self { self & !(align - 1) }
        }
    };
}

impl_align!(u8);
impl_align!(u16);
impl_align!(u32);
impl_align!(u64);
impl_align!(usize);

/// Creates a fixed-size array reference from a slice.
macro_rules! array_ref {
    ($slice:expr, $offset:expr, $size:expr) => {{
        #[inline(always)]
        fn to_array<T>(slice: &[T]) -> &[T; $size] {
            unsafe { &*(slice as *const [T] as *const [T; $size]) }
        }
        to_array(&$slice[$offset..$offset + $size])
    }};
}
pub(crate) use array_ref;

/// Creates a mutable fixed-size array reference from a slice.
macro_rules! array_ref_mut {
    ($slice:expr, $offset:expr, $size:expr) => {{
        #[inline(always)]
        fn to_array<T>(slice: &mut [T]) -> &mut [T; $size] {
            unsafe { &mut *(slice as *mut [T] as *mut [T; $size]) }
        }
        to_array(&mut $slice[$offset..$offset + $size])
    }};
}
pub(crate) use array_ref_mut;

/// Compile-time assertion.
macro_rules! static_assert {
    ($condition:expr) => {
        const _: () = core::assert!($condition);
    };
}
pub(crate) use static_assert;

macro_rules! impl_read_for_bufread {
    ($ty:ident) => {
        impl std::io::Read for $ty {
            fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
                use std::io::BufRead;
                let buf = self.fill_buf()?;
                let len = buf.len().min(out.len());
                out[..len].copy_from_slice(&buf[..len]);
                self.consume(len);
                Ok(len)
            }
        }
    };
}
pub(crate) use impl_read_for_bufread;
