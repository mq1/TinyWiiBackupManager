#[cfg(feature = "threading")]
use std::{collections::HashMap, thread::JoinHandle, time::Instant};
use std::{
    fmt::{Display, Formatter},
    io,
    num::NonZeroUsize,
    sync::{Arc, Mutex},
    time::Duration,
};

use bytes::{Bytes, BytesMut};
#[cfg(feature = "threading")]
use crossbeam_channel::{Receiver, Sender};
#[cfg(feature = "threading")]
use crossbeam_utils::sync::WaitGroup;
use lru::LruCache;
use polonius_the_crab::{polonius, polonius_return};
#[cfg(feature = "threading")]
use simple_moving_average::{SMA, SingleSumSMA};
#[cfg(feature = "threading")]
use tracing::{Level, span};
use tracing::{debug, error, instrument};
use zerocopy::FromZeros;

use crate::{
    IoResultContext,
    common::PartitionInfo,
    disc::{
        DiscHeader, SECTOR_GROUP_SIZE, SECTOR_SIZE,
        hashes::{GroupHashes, hash_sector_group},
        wii::HASHES_SIZE,
    },
    io::{
        block::{Block, BlockKind, BlockReader},
        wia::WIAException,
    },
    read::PartitionEncryption,
    util::{
        aes::{decrypt_sector, encrypt_sector},
        array_ref_mut,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectorGroupRequest {
    pub group_idx: u32,
    pub partition_idx: Option<u8>,
    pub mode: PartitionEncryption,
    pub force_rehash: bool,
}

impl Display for SectorGroupRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.partition_idx {
            Some(idx) => write!(f, "Partition {} group {}", idx, self.group_idx),
            None => write!(f, "Group {}", self.group_idx),
        }
    }
}

#[derive(Clone)]
pub struct SectorGroup {
    pub request: SectorGroupRequest,
    pub start_sector: u32,
    pub data: Bytes,
    pub sector_bitmap: u64,
    #[allow(unused)]
    pub io_duration: Option<Duration>,
    #[allow(unused)] // TODO WIA hash exceptions
    pub group_hashes: Option<Arc<GroupHashes>>,
}

impl SectorGroup {
    /// Calculate the number of consecutive sectors starting from `start`.
    #[inline]
    pub fn consecutive_sectors(&self, start: u32) -> u32 {
        (self.sector_bitmap >> start).trailing_ones()
    }
}

pub type SectorGroupResult = io::Result<SectorGroup>;

#[allow(unused)]
pub struct Preloader {
    #[cfg(feature = "threading")]
    request_tx: Sender<SectorGroupRequest>,
    #[cfg(feature = "threading")]
    request_rx: Receiver<SectorGroupRequest>,
    #[cfg(feature = "threading")]
    stat_tx: Sender<PreloaderThreadStats>,
    #[cfg(feature = "threading")]
    stat_rx: Receiver<PreloaderThreadStats>,
    #[cfg(feature = "threading")]
    threads: Mutex<PreloaderThreads>,
    cache: Arc<Mutex<PreloaderCache>>,
    // Fallback single-threaded loader
    loader: Mutex<SectorGroupLoader>,
}

#[allow(unused)]
#[cfg(feature = "threading")]
struct PreloaderThreads {
    join_handles: Vec<JoinHandle<()>>,
    last_adjust: Instant,
    num_samples: usize,
    wait_time_avg: SingleSumSMA<Duration, u32, 100>,
    req_time_avg: SingleSumSMA<Duration, u32, 100>,
    io_time_avg: SingleSumSMA<Duration, u32, 100>,
}

#[cfg(feature = "threading")]
impl PreloaderThreads {
    fn new(join_handles: Vec<JoinHandle<()>>) -> Self {
        Self {
            join_handles,
            last_adjust: Instant::now(),
            num_samples: 0,
            wait_time_avg: SingleSumSMA::<_, _, 100>::from_zero(Duration::default()),
            req_time_avg: SingleSumSMA::<_, _, 100>::from_zero(Duration::default()),
            io_time_avg: SingleSumSMA::<_, _, 100>::from_zero(Duration::default()),
        }
    }

    fn push_stats(&mut self, stat: PreloaderThreadStats, _outer: &Preloader) {
        self.wait_time_avg.add_sample(stat.wait_time);
        self.req_time_avg.add_sample(stat.req_time);
        self.io_time_avg.add_sample(stat.io_time);
        self.num_samples += 1;
        if self.num_samples % 100 == 0 {
            let avg_wait = self.wait_time_avg.get_average();
            let avg_req = self.req_time_avg.get_average();
            let avg_io = self.io_time_avg.get_average();
            let utilization =
                avg_req.as_secs_f64() / (avg_req.as_secs_f64() + avg_wait.as_secs_f64());
            let io_time = avg_io.as_secs_f64() / avg_req.as_secs_f64();
            debug!(
                "Preloader stats: count {}, wait: {:?}, req: {:?}, util: {:.2}%, io: {:.2}%",
                self.num_samples,
                avg_wait,
                avg_req,
                utilization * 100.0,
                io_time * 100.0
            );
            // if self.last_adjust.elapsed() > Duration::from_secs(2) {
            //     if utilization > 0.9 && io_time < 0.1 {
            //         println!("Preloader is CPU-bound, increasing thread count");
            //         let id = self.join_handles.len();
            //         self.join_handles.push(preloader_thread(
            //             id,
            //             outer.request_rx.clone(),
            //             outer.cache.clone(),
            //             outer.loader.lock().unwrap().clone(),
            //             outer.stat_tx.clone(),
            //         ));
            //         self.last_adjust = Instant::now();
            //     } /*else if io_time > 0.9 {
            //         println!("Preloader is I/O-bound, decreasing thread count");
            //         if self.join_handles.len() > 1 {
            //             let handle = self.join_handles.pop().unwrap();
            //
            //         }
            //     }*/
            // }
        }
    }
}

struct PreloaderCache {
    #[cfg(feature = "threading")]
    inflight: HashMap<SectorGroupRequest, WaitGroup>,
    lru_cache: LruCache<SectorGroupRequest, SectorGroup>,
}

impl Default for PreloaderCache {
    fn default() -> Self {
        Self {
            #[cfg(feature = "threading")]
            inflight: Default::default(),
            lru_cache: LruCache::new(NonZeroUsize::new(64).unwrap()),
        }
    }
}

impl PreloaderCache {
    fn push(&mut self, request: SectorGroupRequest, group: SectorGroup) {
        self.lru_cache.push(request, group);
        #[cfg(feature = "threading")]
        self.inflight.remove(&request);
    }

    #[cfg(feature = "threading")]
    fn remove(&mut self, request: &SectorGroupRequest) { self.inflight.remove(request); }

    #[cfg(feature = "threading")]
    fn contains(&self, request: &SectorGroupRequest) -> bool {
        self.lru_cache.contains(request) || self.inflight.contains_key(request)
    }
}

#[allow(unused)]
#[cfg(feature = "threading")]
struct PreloaderThreadStats {
    thread_id: usize,
    wait_time: Duration,
    req_time: Duration,
    io_time: Duration,
}

#[cfg(feature = "threading")]
fn preloader_thread(
    thread_id: usize,
    request_rx: Receiver<SectorGroupRequest>,
    cache: Arc<Mutex<PreloaderCache>>,
    mut loader: SectorGroupLoader,
    stat_tx: Sender<PreloaderThreadStats>,
) -> JoinHandle<()> {
    std::thread::Builder::new()
        .name(format!("Preloader {thread_id}"))
        .spawn(move || {
            let mut last_request_end: Option<Instant> = None;
            while let Ok(request) = request_rx.recv() {
                let wait_time = if let Some(last_request) = last_request_end {
                    last_request.elapsed()
                } else {
                    Duration::default()
                };
                let start = Instant::now();
                let mut io_time = Duration::default();
                match loader.load(request) {
                    Ok(group) => {
                        let Ok(mut cache_guard) = cache.lock() else {
                            break;
                        };
                        io_time = group.io_duration.unwrap_or_default();
                        cache_guard.push(request, group);
                    }
                    Err(_) => {
                        let Ok(mut cache_guard) = cache.lock() else {
                            break;
                        };
                        // Just drop the request if it failed
                        cache_guard.remove(&request);
                    }
                }
                let end = Instant::now();
                last_request_end = Some(end);
                let req_time = end - start;
                if stat_tx
                    .send(PreloaderThreadStats { thread_id, wait_time, req_time, io_time })
                    .is_err()
                {
                    break;
                }
            }
        })
        .expect("Failed to spawn preloader thread")
}

impl Preloader {
    #[cfg(feature = "threading")]
    pub fn new(loader: SectorGroupLoader, num_threads: usize) -> Arc<Self> {
        debug!("Creating preloader with {} threads", num_threads);

        let (request_tx, request_rx) = crossbeam_channel::unbounded();
        let (stat_tx, stat_rx) = crossbeam_channel::unbounded();
        let cache = Arc::new(Mutex::new(PreloaderCache::default()));
        let mut join_handles = Vec::with_capacity(num_threads);
        for i in 0..num_threads {
            join_handles.push(preloader_thread(
                i,
                request_rx.clone(),
                cache.clone(),
                loader.clone(),
                stat_tx.clone(),
            ));
        }
        let threads = Mutex::new(PreloaderThreads::new(join_handles));
        let loader = Mutex::new(loader);
        Arc::new(Self { request_tx, request_rx, stat_tx, stat_rx, threads, cache, loader })
    }

    #[cfg(not(feature = "threading"))]
    pub fn new(loader: SectorGroupLoader) -> Arc<Self> {
        debug!("Creating single-threaded preloader");
        let cache = Arc::new(Mutex::new(PreloaderCache::default()));
        let loader = Mutex::new(loader);
        Arc::new(Self { cache, loader })
    }

    #[allow(unused)]
    pub fn shutdown(self) {
        drop(self);
    }

    #[instrument(name = "Preloader::fetch", skip_all)]
    pub fn fetch(&self, request: SectorGroupRequest, max_groups: u32) -> SectorGroupResult {
        #[cfg(feature = "threading")]
        {
            let num_threads = {
                let mut threads_guard = self.threads.lock().map_err(map_poisoned)?;
                while let Ok(stat) = self.stat_rx.try_recv() {
                    threads_guard.push_stats(stat, self);
                }
                threads_guard.join_handles.len()
            };
            let mut cache_guard = self.cache.lock().map_err(map_poisoned)?;
            // Preload n groups ahead
            for i in 0..num_threads as u32 {
                let group_idx = request.group_idx + i;
                if group_idx >= max_groups {
                    break;
                }
                let request = SectorGroupRequest { group_idx, ..request };
                if cache_guard.contains(&request) {
                    continue;
                }
                if self.request_tx.send(request).is_ok() {
                    cache_guard.inflight.insert(request, WaitGroup::new());
                }
            }
            if let Some(cached) = cache_guard.lru_cache.get(&request) {
                return Ok(cached.clone());
            }
            if let Some(wg) = cache_guard.inflight.get(&request) {
                // Wait for inflight request to finish
                let wg = wg.clone();
                drop(cache_guard);
                {
                    let _span = span!(Level::TRACE, "wg.wait").entered();
                    wg.wait();
                }
                let mut cache_guard = self.cache.lock().map_err(map_poisoned)?;
                if let Some(cached) = cache_guard.lru_cache.get(&request) {
                    return Ok(cached.clone());
                }
            } else {
                drop(cache_guard);
            }
        }
        #[cfg(not(feature = "threading"))]
        let _ = max_groups;

        // No threads are running, fallback to single-threaded loader
        let result = {
            let mut loader = self.loader.lock().map_err(map_poisoned)?;
            loader.load(request)
        };
        match result {
            Ok(group) => {
                let mut cache_guard = self.cache.lock().map_err(map_poisoned)?;
                cache_guard.push(request, group.clone());
                Ok(group)
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(feature = "threading")]
impl Drop for Preloader {
    fn drop(&mut self) {
        // Workers own loaders and the cache, not an Arc<Preloader>. Disconnect
        // every original channel before joining: idle workers leave recv(), and
        // active workers stop after the current load instead of draining queued
        // prefetch requests. In particular, never join with request_tx alive.
        let (stat_tx, stat_rx) = crossbeam_channel::unbounded();
        drop(std::mem::replace(&mut self.stat_tx, stat_tx));
        drop(std::mem::replace(&mut self.stat_rx, stat_rx));
        let (request_tx, request_rx) = crossbeam_channel::unbounded();
        drop(std::mem::replace(&mut self.request_tx, request_tx));
        drop(std::mem::replace(&mut self.request_rx, request_rx));

        let threads = self.threads.get_mut().unwrap_or_else(|error| error.into_inner());
        for handle in threads.join_handles.drain(..) {
            // Cleanup may run while another error is unwinding. Join every
            // worker without turning a worker panic into a second panic.
            let _ = handle.join();
        }
    }
}

#[inline]
fn map_poisoned<T>(_: std::sync::PoisonError<T>) -> io::Error { io::Error::other("Mutex poisoned") }

pub struct SectorGroupLoader {
    io: Box<dyn BlockReader>,
    disc_header: Arc<DiscHeader>,
    partitions: Arc<[PartitionInfo]>,
    block: Block,
    block_buf: Box<[u8]>,
}

impl Clone for SectorGroupLoader {
    fn clone(&self) -> Self {
        let block_size = self.io.block_size() as usize;
        Self {
            io: self.io.clone(),
            disc_header: self.disc_header.clone(),
            partitions: self.partitions.clone(),
            block: Block::default(),
            block_buf: <[u8]>::new_box_zeroed_with_elems(block_size).unwrap(),
        }
    }
}

#[derive(Default)]
struct LoadedSectorGroup {
    /// Start sector of the group
    start_sector: u32,
    /// Bitmap of sectors that were read
    sector_bitmap: u64,
    /// Total duration of I/O operations
    io_duration: Option<Duration>,
    /// Calculated sector group hashes
    group_hashes: Option<Arc<GroupHashes>>,
}

impl SectorGroupLoader {
    pub fn new(
        io: Box<dyn BlockReader>,
        disc_header: Arc<DiscHeader>,
        partitions: Arc<[PartitionInfo]>,
    ) -> Self {
        let block_buf = <[u8]>::new_box_zeroed_with_elems(io.block_size() as usize).unwrap();
        Self { io, disc_header, partitions, block: Block::default(), block_buf }
    }

    #[instrument(name = "SectorGroupLoader::load", skip_all)]
    pub fn load(&mut self, request: SectorGroupRequest) -> SectorGroupResult {
        let mut sector_group_buf = BytesMut::zeroed(SECTOR_GROUP_SIZE);

        let out = array_ref_mut![sector_group_buf, 0, SECTOR_GROUP_SIZE];
        let LoadedSectorGroup { start_sector, sector_bitmap, io_duration, group_hashes } =
            if request.partition_idx.is_some() {
                self.load_partition_group(request, out)?
            } else {
                self.load_raw_group(request, out)?
            };

        Ok(SectorGroup {
            request,
            start_sector,
            data: sector_group_buf.freeze(),
            sector_bitmap,
            io_duration,
            group_hashes,
        })
    }

    /// Load a sector group from a partition.
    ///
    /// This will handle encryption, decryption, and hash recovery as needed.
    fn load_partition_group(
        &mut self,
        request: SectorGroupRequest,
        sector_group_buf: &mut [u8; SECTOR_GROUP_SIZE],
    ) -> io::Result<LoadedSectorGroup> {
        let Some(partition) =
            request.partition_idx.and_then(|idx| self.partitions.get(idx as usize))
        else {
            return Ok(LoadedSectorGroup::default());
        };

        let abs_group_sector = partition.data_start_sector + request.group_idx * 64;
        if abs_group_sector >= partition.data_end_sector {
            return Ok(LoadedSectorGroup::default());
        }

        // Bitmap of sectors that were read
        let mut sector_bitmap = 0u64;
        // Bitmap of sectors that are decrypted
        let mut decrypted_sectors = 0u64;
        // Bitmap of sectors that need hash recovery
        let mut hash_recovery_sectors = 0u64;
        // Hash exceptions
        let mut hash_exceptions = Vec::<WIAException>::new();
        // Total duration of I/O operations
        let mut io_duration = None;
        // Calculated sector group hashes
        let mut group_hashes = None;

        // Read sector group
        for sector in 0..64 {
            let sector_data =
                array_ref_mut![sector_group_buf, sector as usize * SECTOR_SIZE, SECTOR_SIZE];
            let abs_sector = abs_group_sector + sector;
            if abs_sector >= partition.data_end_sector {
                // Already zeroed
                decrypted_sectors |= 1 << sector;
                hash_recovery_sectors |= 1 << sector;
                continue;
            }

            // Read new block
            if !self.block.contains(abs_sector) {
                self.block = self
                    .io
                    .read_block(self.block_buf.as_mut(), abs_sector)
                    .io_with_context(|| format!("Reading block for sector {abs_sector}"))?;
                if let Some(duration) = self.block.io_duration {
                    *io_duration.get_or_insert_with(Duration::default) += duration;
                }
                if self.block.kind == BlockKind::None {
                    error!("Failed to read block for sector {}", abs_sector);
                    break;
                }
            }

            // Add hash exceptions
            self.block
                .append_hash_exceptions(abs_sector, sector, &mut hash_exceptions)
                .io_with_context(|| format!("Appending hash exceptions for sector {abs_sector}"))?;

            // Read new sector into buffer
            let (encrypted, has_hashes) = self
                .block
                .copy_sector(
                    sector_data,
                    self.block_buf.as_mut(),
                    abs_sector,
                    partition.disc_header(),
                    Some(partition),
                )
                .io_with_context(|| format!("Copying sector {abs_sector} from block"))?;
            if !encrypted {
                decrypted_sectors |= 1 << sector;
            }
            if !has_hashes && partition.has_hashes {
                hash_recovery_sectors |= 1 << sector;
            }
            sector_bitmap |= 1 << sector;
        }

        // Recover hashes
        if request.force_rehash
            || (request.mode != PartitionEncryption::ForceDecryptedNoHashes
                && hash_recovery_sectors != 0)
        {
            // Decrypt any encrypted sectors
            if decrypted_sectors != u64::MAX {
                for sector in 0..64 {
                    let sector_data =
                        array_ref_mut![sector_group_buf, sector * SECTOR_SIZE, SECTOR_SIZE];
                    if (decrypted_sectors >> sector) & 1 == 0 {
                        decrypt_sector(sector_data, &partition.key);
                    }
                }
                decrypted_sectors = u64::MAX;
            }

            // Recover hashes
            let hashes = hash_sector_group(sector_group_buf, request.force_rehash);

            // Apply hashes
            for sector in 0..64 {
                let sector_data =
                    array_ref_mut![sector_group_buf, sector * SECTOR_SIZE, SECTOR_SIZE];
                if (hash_recovery_sectors >> sector) & 1 == 1 {
                    hashes.apply(sector_data, sector);
                }
            }

            // Persist hashes
            group_hashes = Some(Arc::from(hashes));
        }

        // Apply hash exceptions
        if request.mode != PartitionEncryption::ForceDecryptedNoHashes
            && !hash_exceptions.is_empty()
        {
            for exception in hash_exceptions {
                let offset = exception.offset.get();
                let sector = offset / HASHES_SIZE as u16;

                // Decrypt sector if needed
                let sector_data =
                    array_ref_mut![sector_group_buf, sector as usize * SECTOR_SIZE, SECTOR_SIZE];
                if (decrypted_sectors >> sector) & 1 == 0 {
                    decrypt_sector(sector_data, &partition.key);
                    decrypted_sectors |= 1 << sector;
                }

                let sector_offset = (offset - (sector * HASHES_SIZE as u16)) as usize;
                *array_ref_mut![sector_data, sector_offset, 20] = exception.hash;
            }
        }

        // Encrypt/decrypt sectors
        if match request.mode {
            PartitionEncryption::Original => partition.has_encryption,
            PartitionEncryption::ForceEncrypted => true,
            PartitionEncryption::ForceDecrypted | PartitionEncryption::ForceDecryptedNoHashes => {
                false
            }
        } {
            // Encrypt any decrypted sectors
            if decrypted_sectors != 0 {
                for sector in 0..64 {
                    let sector_data = array_ref_mut![
                        sector_group_buf,
                        sector as usize * SECTOR_SIZE,
                        SECTOR_SIZE
                    ];
                    if (decrypted_sectors >> sector) & 1 == 1 {
                        encrypt_sector(sector_data, &partition.key);
                    }
                }
            }
        } else if decrypted_sectors != u64::MAX {
            // Decrypt any encrypted sectors
            for sector in 0..64 {
                let sector_data =
                    array_ref_mut![sector_group_buf, sector as usize * SECTOR_SIZE, SECTOR_SIZE];
                if (decrypted_sectors >> sector) & 1 == 0 {
                    decrypt_sector(sector_data, &partition.key);
                }
            }
        }

        Ok(LoadedSectorGroup {
            start_sector: abs_group_sector,
            sector_bitmap,
            io_duration,
            group_hashes,
        })
    }

    /// Loads a non-partition sector group.
    fn load_raw_group(
        &mut self,
        request: SectorGroupRequest,
        sector_group_buf: &mut [u8; SECTOR_GROUP_SIZE],
    ) -> io::Result<LoadedSectorGroup> {
        let abs_group_sector = request.group_idx * 64;

        // Bitmap of sectors that were read
        let mut sector_bitmap = 0u64;
        // Total duration of I/O operations
        let mut io_duration = None;

        for sector in 0..64 {
            let sector_data =
                array_ref_mut![sector_group_buf, sector as usize * SECTOR_SIZE, SECTOR_SIZE];
            let abs_sector = abs_group_sector + sector;
            if self.partitions.iter().any(|p| p.data_contains_sector(abs_sector)) {
                continue;
            }

            // Read new block
            if !self.block.contains(abs_sector) {
                self.block = self
                    .io
                    .read_block(self.block_buf.as_mut(), abs_sector)
                    .io_with_context(|| format!("Reading block for sector {abs_sector}"))?;
                if let Some(duration) = self.block.io_duration {
                    *io_duration.get_or_insert_with(Duration::default) += duration;
                }
                if self.block.kind == BlockKind::None {
                    break;
                }
            }

            // Read new sector into buffer
            self.block
                .copy_sector(
                    sector_data,
                    self.block_buf.as_mut(),
                    abs_sector,
                    self.disc_header.as_ref(),
                    None,
                )
                .io_with_context(|| format!("Copying sector {abs_sector} from block"))?;
            sector_bitmap |= 1 << sector;
        }

        Ok(LoadedSectorGroup {
            start_sector: abs_group_sector,
            sector_bitmap,
            io_duration,
            group_hashes: None,
        })
    }
}

/// Fetch a sector group from the cache or from the preloader.
/// Returns a boolean indicating if the group was updated.
pub fn fetch_sector_group<'a>(
    request: SectorGroupRequest,
    max_groups: u32,
    mut cached: &'a mut Option<SectorGroup>,
    preloader: &Preloader,
) -> io::Result<(&'a SectorGroup, bool)> {
    polonius!(|cached| -> io::Result<(&'polonius SectorGroup, bool)> {
        if let Some(sector_group) = cached {
            if sector_group.request == request {
                polonius_return!(Ok((sector_group, false)));
            }
        }
    });
    let sector_group = preloader.fetch(request, max_groups)?;
    Ok((cached.insert(sector_group), true))
}

#[cfg(all(test, feature = "threading"))]
mod shutdown_tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::read::DiscMeta;

    #[derive(Clone)]
    struct UnusedReader;

    impl BlockReader for UnusedReader {
        fn read_block(&mut self, _: &mut [u8], _: u32) -> io::Result<Block> {
            panic!("shutdown test must not read disc data")
        }

        fn block_size(&self) -> u32 { SECTOR_SIZE as u32 }

        fn meta(&self) -> DiscMeta { DiscMeta::default() }
    }

    #[test]
    fn shutdown_disconnects_request_and_stat_channels_before_joining_workers() {
        let loader = SectorGroupLoader::new(
            Box::new(UnusedReader),
            Arc::new(DiscHeader::new_zeroed()),
            Arc::from([]),
        );
        let mut preloader = match Arc::try_unwrap(Preloader::new(loader, 0)) {
            Ok(preloader) => preloader,
            Err(_) => panic!("unexpected shared preloader"),
        };
        let finished = Arc::new(AtomicUsize::new(0));
        for thread_id in 0..2 {
            let requests = preloader.request_rx.clone();
            let stats = preloader.stat_tx.clone();
            let finished = finished.clone();
            let handle = std::thread::spawn(move || {
                assert!(matches!(
                    requests.recv_timeout(Duration::from_secs(2)),
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected)
                ));
                assert!(stats
                    .send(PreloaderThreadStats {
                        thread_id,
                        wait_time: Duration::ZERO,
                        req_time: Duration::ZERO,
                        io_time: Duration::ZERO,
                    })
                    .is_err());
                finished.fetch_add(1, Ordering::SeqCst);
            });
            preloader.threads.get_mut().unwrap().join_handles.push(handle);
        }
        preloader.shutdown();
        assert_eq!(finished.load(Ordering::SeqCst), 2);
    }
}
