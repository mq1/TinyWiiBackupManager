use bytes::Bytes;
use digest::Digest;
use tracing::instrument;

use crate::{
    common::HashBytes,
    io::nkit::NKitHeader,
    write::{DiscFinalization, ProcessOptions},
};

/// Hashes a byte slice with SHA-1.
#[instrument(skip_all)]
pub fn sha1_hash(buf: &[u8]) -> HashBytes {
    #[cfg(feature = "openssl")]
    {
        // The one-shot openssl::sha::sha1 ends up being much slower
        let mut hasher = openssl::sha::Sha1::new();
        hasher.update(buf);
        hasher.finish()
    }
    #[cfg(not(feature = "openssl"))]
    {
        use sha1::Digest;
        HashBytes::from(sha1::Sha1::digest(buf))
    }
}

/// Hashes a byte slice with XXH64.
#[allow(unused_braces)] // https://github.com/rust-lang/rust/issues/116347
#[instrument(skip_all)]
pub fn xxh64_hash(buf: &[u8]) -> u64 { xxhash_rust::xxh64::xxh64(buf, 0) }

#[cfg(feature = "threading")]
mod multi_threaded {
    use std::{thread, thread::JoinHandle};

    use crossbeam_channel::Sender;

    use super::*;

    type DigestThread = (Sender<Bytes>, JoinHandle<DigestResult>);

    fn digest_thread<H>() -> DigestThread
    where H: Hasher + Send + 'static {
        let (tx, rx) = crossbeam_channel::bounded::<Bytes>(1);
        let handle = thread::Builder::new()
            .name(format!("Digest {}", H::NAME))
            .spawn(move || {
                let mut hasher = H::new();
                while let Ok(data) = rx.recv() {
                    hasher.update(data.as_ref());
                }
                hasher.finalize()
            })
            .expect("Failed to spawn digest thread");
        (tx, handle)
    }

    pub struct DigestManager {
        threads: Vec<DigestThread>,
    }

    impl DigestManager {
        pub fn new(options: &ProcessOptions) -> Self {
            let mut threads = Vec::new();
            if options.digest_crc32 {
                threads.push(digest_thread::<crc32fast::Hasher>());
            }
            if options.digest_md5 {
                #[cfg(feature = "openssl")]
                threads.push(digest_thread::<openssl_util::HasherMD5>());
                #[cfg(not(feature = "openssl"))]
                threads.push(digest_thread::<md5::Md5>());
            }
            if options.digest_sha1 {
                #[cfg(feature = "openssl")]
                threads.push(digest_thread::<openssl_util::HasherSHA1>());
                #[cfg(not(feature = "openssl"))]
                threads.push(digest_thread::<sha1::Sha1>());
            }
            if options.digest_xxh64 {
                threads.push(digest_thread::<xxhash_rust::xxh64::Xxh64>());
            }
            DigestManager { threads }
        }

        #[instrument(name = "DigestManager::send", skip_all)]
        pub fn send(&self, data: Bytes) {
            let mut sent = 0usize;
            // Non-blocking send to all threads
            for (idx, (tx, _)) in self.threads.iter().enumerate() {
                if tx.try_send(data.clone()).is_ok() {
                    sent |= 1 << idx;
                }
            }
            // Blocking send to any remaining threads
            for (idx, (tx, _)) in self.threads.iter().enumerate() {
                if sent & (1 << idx) == 0 {
                    tx.send(data.clone()).expect("Failed to send data to digest thread");
                }
            }
        }

        #[instrument(name = "DigestManager::finish", skip_all)]
        pub fn finish(mut self) -> DigestResults {
            let mut results = DigestResults { crc32: None, md5: None, sha1: None, xxh64: None };
            for result in self.join_threads() {
                match result.unwrap() {
                    DigestResult::Crc32(v) => results.crc32 = Some(v),
                    DigestResult::Md5(v) => results.md5 = Some(v),
                    DigestResult::Sha1(v) => results.sha1 = Some(v),
                    DigestResult::Xxh64(v) => results.xxh64 = Some(v),
                }
            }
            results
        }

        fn join_threads(&mut self) -> Vec<thread::Result<DigestResult>> {
            let handles: Vec<_> = self
                .threads
                .drain(..)
                .map(|(tx, handle)| {
                    drop(tx);
                    handle
                })
                .collect();
            // Close ALL senders before waiting for any worker. Collect every
            // join result before propagating a panic from normal finish().
            handles.into_iter().map(|handle| handle.join()).collect()
        }
    }

    impl Drop for DigestManager {
        fn drop(&mut self) {
            // On writer errors/cancellation finish() is skipped. Do not detach
            // digest workers or leave input blocks alive after cleanup returns.
            let _ = self.join_threads();
        }
    }

    #[cfg(test)]
    mod shutdown_tests {
        use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
        use std::time::Duration;

        use super::*;

        fn coordinated_manager() -> (DigestManager, Arc<AtomicUsize>) {
            let (first_tx, first_rx) = crossbeam_channel::bounded::<Bytes>(1);
            let (second_tx, second_rx) = crossbeam_channel::bounded::<Bytes>(1);
            let (second_closed_tx, second_closed_rx) = crossbeam_channel::bounded(1);
            let finished = Arc::new(AtomicUsize::new(0));
            let first_finished = finished.clone();
            let first = thread::spawn(move || {
                assert!(first_rx.recv().is_err());
                // This cannot complete if cleanup closes/joins workers one by
                // one while the second sender remains alive.
                second_closed_rx.recv_timeout(Duration::from_secs(2)).unwrap();
                first_finished.fetch_add(1, Ordering::SeqCst);
                DigestResult::Crc32(7)
            });
            let second_finished = finished.clone();
            let second = thread::spawn(move || {
                assert!(second_rx.recv().is_err());
                second_closed_tx.send(()).unwrap();
                second_finished.fetch_add(1, Ordering::SeqCst);
                DigestResult::Sha1([8; 20])
            });
            (DigestManager { threads: vec![(first_tx, first), (second_tx, second)] }, finished)
        }

        #[test]
        fn drop_closes_all_inputs_and_joins_every_digest_worker() {
            let (manager, finished) = coordinated_manager();
            drop(manager);
            assert_eq!(finished.load(Ordering::SeqCst), 2);
        }

        #[test]
        fn finish_closes_all_inputs_and_keeps_digest_results() {
            let (manager, finished) = coordinated_manager();
            let results = manager.finish();
            assert_eq!(finished.load(Ordering::SeqCst), 2);
            assert_eq!(results.crc32, Some(7));
            assert_eq!(results.sha1, Some([8; 20]));
        }

        #[test]
        fn normal_finish_hashes_all_queued_input() {
            let manager = DigestManager::new(&ProcessOptions {
                digest_crc32: true,
                digest_sha1: true,
                ..Default::default()
            });
            manager.send(Bytes::from_static(b"a"));
            manager.send(Bytes::from_static(b"bc"));
            let results = manager.finish();
            assert_eq!(results.crc32, Some(0x352441c2));
            assert_eq!(results.sha1, Some([
                0xa9, 0x99, 0x3e, 0x36, 0x47, 0x06, 0x81, 0x6a, 0xba, 0x3e,
                0x25, 0x71, 0x78, 0x50, 0xc2, 0x6c, 0x9c, 0xd0, 0xd8, 0x9d,
            ]));
        }
    }
}

#[cfg(not(feature = "threading"))]
mod single_threaded {
    use std::cell::RefCell;

    use super::*;

    pub struct DigestManager {
        hashers: Vec<RefCell<Box<dyn Hasher>>>,
    }

    impl DigestManager {
        pub fn new(options: &ProcessOptions) -> Self {
            let mut hashers = Vec::<RefCell<Box<dyn Hasher>>>::new();
            if options.digest_crc32 {
                hashers.push(RefCell::new(Box::new(crc32fast::Hasher::new())));
            }
            if options.digest_md5 {
                #[cfg(feature = "openssl")]
                hashers.push(RefCell::new(Box::new(openssl_util::HasherMD5::new())));
                #[cfg(not(feature = "openssl"))]
                hashers.push(RefCell::new(Box::new(md5::Md5::new())));
            }
            if options.digest_sha1 {
                #[cfg(feature = "openssl")]
                hashers.push(RefCell::new(Box::new(openssl_util::HasherSHA1::new())));
                #[cfg(not(feature = "openssl"))]
                hashers.push(RefCell::new(Box::new(sha1::Sha1::new())));
            }
            if options.digest_xxh64 {
                hashers.push(RefCell::new(Box::new(xxhash_rust::xxh64::Xxh64::new(0))));
            }
            Self { hashers }
        }

        #[instrument(name = "DigestManager::send", skip_all)]
        pub fn send(&self, data: Bytes) {
            for hasher in &self.hashers {
                hasher.borrow_mut().update(&data);
            }
        }

        #[instrument(name = "DigestManager::finish", skip_all)]
        pub fn finish(self) -> DigestResults {
            let mut results = DigestResults { crc32: None, md5: None, sha1: None, xxh64: None };
            for hasher in self.hashers {
                match hasher.borrow_mut().finalize() {
                    DigestResult::Crc32(v) => results.crc32 = Some(v),
                    DigestResult::Md5(v) => results.md5 = Some(v),
                    DigestResult::Sha1(v) => results.sha1 = Some(v),
                    DigestResult::Xxh64(v) => results.xxh64 = Some(v),
                }
            }
            results
        }
    }
}

#[cfg(feature = "threading")]
pub use multi_threaded::DigestManager;
#[cfg(not(feature = "threading"))]
pub use single_threaded::DigestManager;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DigestResult {
    Crc32(u32),
    Md5([u8; 16]),
    Sha1([u8; 20]),
    Xxh64(u64),
}

pub trait Hasher {
    #[cfg(feature = "threading")]
    const NAME: &'static str;

    #[cfg(feature = "threading")]
    fn new() -> Self;
    fn finalize(&mut self) -> DigestResult;
    fn update(&mut self, data: &[u8]);
}

impl Hasher for md5::Md5 {
    #[cfg(feature = "threading")]
    const NAME: &'static str = "MD5";

    #[cfg(feature = "threading")]
    fn new() -> Self { Digest::new() }

    fn finalize(&mut self) -> DigestResult {
        DigestResult::Md5(Digest::finalize_reset(self).into())
    }

    #[allow(unused_braces)] // https://github.com/rust-lang/rust/issues/116347
    #[instrument(name = "md5::Md5::update", skip_all)]
    fn update(&mut self, data: &[u8]) { Digest::update(self, data) }
}

impl Hasher for sha1::Sha1 {
    #[cfg(feature = "threading")]
    const NAME: &'static str = "SHA-1";

    #[cfg(feature = "threading")]
    fn new() -> Self { Digest::new() }

    fn finalize(&mut self) -> DigestResult {
        DigestResult::Sha1(Digest::finalize_reset(self).into())
    }

    #[allow(unused_braces)] // https://github.com/rust-lang/rust/issues/116347
    #[instrument(name = "sha1::Sha1::update", skip_all)]
    fn update(&mut self, data: &[u8]) { Digest::update(self, data) }
}

impl Hasher for crc32fast::Hasher {
    #[cfg(feature = "threading")]
    const NAME: &'static str = "CRC32";

    #[cfg(feature = "threading")]
    fn new() -> Self { crc32fast::Hasher::new() }

    fn finalize(&mut self) -> DigestResult {
        DigestResult::Crc32(crc32fast::Hasher::finalize(self.clone()))
    }

    #[allow(unused_braces)] // https://github.com/rust-lang/rust/issues/116347
    #[instrument(name = "crc32fast::Hasher::update", skip_all)]
    fn update(&mut self, data: &[u8]) { crc32fast::Hasher::update(self, data) }
}

impl Hasher for xxhash_rust::xxh64::Xxh64 {
    #[cfg(feature = "threading")]
    const NAME: &'static str = "XXH64";

    #[cfg(feature = "threading")]
    fn new() -> Self { xxhash_rust::xxh64::Xxh64::new(0) }

    fn finalize(&mut self) -> DigestResult {
        DigestResult::Xxh64(xxhash_rust::xxh64::Xxh64::digest(self))
    }

    #[allow(unused_braces)] // https://github.com/rust-lang/rust/issues/116347
    #[instrument(name = "xxhash_rust::xxh64::Xxh64::update", skip_all)]
    fn update(&mut self, data: &[u8]) { xxhash_rust::xxh64::Xxh64::update(self, data) }
}

#[cfg(feature = "openssl")]
mod openssl_util {
    use tracing::instrument;

    use super::{DigestResult, Hasher};

    pub type HasherMD5 = HashWrapper<MessageDigestMD5>;
    pub type HasherSHA1 = HashWrapper<MessageDigestSHA1>;

    pub struct HashWrapper<T>
    where T: MessageDigest
    {
        hasher: openssl::hash::Hasher,
        _marker: std::marker::PhantomData<T>,
    }

    impl<T> HashWrapper<T>
    where T: MessageDigest
    {
        pub(super) fn new() -> Self {
            Self {
                hasher: openssl::hash::Hasher::new(T::new()).unwrap(),
                _marker: Default::default(),
            }
        }
    }

    pub trait MessageDigest {
        fn new() -> openssl::hash::MessageDigest;
    }

    pub struct MessageDigestMD5;

    impl MessageDigest for MessageDigestMD5 {
        fn new() -> openssl::hash::MessageDigest { openssl::hash::MessageDigest::md5() }
    }

    pub struct MessageDigestSHA1;

    impl MessageDigest for MessageDigestSHA1 {
        fn new() -> openssl::hash::MessageDigest { openssl::hash::MessageDigest::sha1() }
    }

    impl Hasher for HasherMD5 {
        #[cfg(feature = "threading")]
        const NAME: &'static str = "MD5";

        #[cfg(feature = "threading")]
        fn new() -> Self { Self::new() }

        fn finalize(&mut self) -> DigestResult {
            DigestResult::Md5((*self.hasher.finish().unwrap()).try_into().unwrap())
        }

        #[allow(unused_braces)] // https://github.com/rust-lang/rust/issues/116347
        #[instrument(name = "openssl_util::HasherMD5::update", skip_all)]
        fn update(&mut self, data: &[u8]) { self.hasher.update(data).unwrap() }
    }

    impl Hasher for HasherSHA1 {
        #[cfg(feature = "threading")]
        const NAME: &'static str = "SHA-1";

        #[cfg(feature = "threading")]
        fn new() -> Self { Self::new() }

        fn finalize(&mut self) -> DigestResult {
            DigestResult::Sha1((*self.hasher.finish().unwrap()).try_into().unwrap())
        }

        #[allow(unused_braces)] // https://github.com/rust-lang/rust/issues/116347
        #[instrument(name = "openssl_util::HasherSHA1::update", skip_all)]
        fn update(&mut self, data: &[u8]) { self.hasher.update(data).unwrap() }
    }
}

pub struct DigestResults {
    pub crc32: Option<u32>,
    pub md5: Option<[u8; 16]>,
    pub sha1: Option<[u8; 20]>,
    pub xxh64: Option<u64>,
}

impl DiscFinalization {
    pub(crate) fn apply_digests(&mut self, results: &DigestResults) {
        self.crc32 = results.crc32;
        self.md5 = results.md5;
        self.sha1 = results.sha1;
        self.xxh64 = results.xxh64;
    }
}

impl NKitHeader {
    pub(crate) fn apply_digests(&mut self, results: &DigestResults) {
        self.crc32 = results.crc32;
        self.md5 = results.md5;
        self.sha1 = results.sha1;
        self.xxh64 = results.xxh64;
    }
}
