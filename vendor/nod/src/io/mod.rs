//! Disc file format related logic (CISO, NFS, WBFS, WIA, etc.)

pub(crate) mod block;
pub(crate) mod ciso;
#[cfg(feature = "compress-zlib")]
pub(crate) mod gcz;
pub(crate) mod iso;
pub(crate) mod nfs;
pub(crate) mod nkit;
pub(crate) mod split;
pub(crate) mod tgc;
pub(crate) mod wbfs;
pub(crate) mod wia;
