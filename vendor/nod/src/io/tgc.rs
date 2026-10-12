use std::{
    io,
    io::{BufRead, Read, Seek, SeekFrom},
    sync::Arc,
};

use bytes::{BufMut, Bytes, BytesMut};
use zerocopy::{FromBytes, FromZeros, Immutable, IntoBytes, KnownLayout, big_endian::U32};

use crate::{
    Error, Result, ResultContext,
    build::gc::{FileCallback, GCPartitionStream, WriteInfo, WriteKind, insert_junk_data},
    common::{Compression, Format, MagicBytes, PartitionKind},
    disc::{
        BB2_OFFSET, BootHeader, DiscHeader, SECTOR_SIZE,
        fst::Fst,
        gcn::{read_dol, read_fst},
        reader::DiscReader,
        writer::DiscWriter,
    },
    io::block::{Block, BlockKind, BlockReader, TGC_MAGIC},
    read::{DiscMeta, DiscStream, PartitionOptions, PartitionReader},
    util::{
        Align, array_ref,
        read::{read_arc_at, read_arc_slice_at, read_at, read_with_zero_fill},
        static_assert,
    },
    write::{DataCallback, DiscFinalization, DiscWriterWeight, FormatOptions, ProcessOptions},
};

/// TGC header (big endian)
#[derive(Clone, Debug, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
struct TGCHeader {
    /// Magic bytes
    magic: MagicBytes,
    /// TGC version
    version: U32,
    /// Offset to the start of the GCM header
    header_offset: U32,
    /// Size of the GCM header
    header_size: U32,
    /// Offset to the FST
    fst_offset: U32,
    /// Size of the FST
    fst_size: U32,
    /// Maximum size of the FST across discs
    fst_max_size: U32,
    /// Offset to the DOL
    dol_offset: U32,
    /// Size of the DOL
    dol_size: U32,
    /// Offset to user data
    user_offset: U32,
    /// Size of user data
    user_size: U32,
    /// Offset to the banner
    banner_offset: U32,
    /// Size of the banner
    banner_size: U32,
    /// Start of user files in the original GCM
    gcm_files_start: U32,
}

static_assert!(size_of::<TGCHeader>() == 0x38);

const GCM_HEADER_SIZE: usize = 0x100000;

#[derive(Clone)]
pub struct BlockReaderTGC {
    inner: GCPartitionStream<FileCallbackTGC>,
}

impl BlockReaderTGC {
    pub fn new(mut inner: Box<dyn DiscStream>) -> Result<Box<dyn BlockReader>> {
        // Read header
        let header: TGCHeader = read_at(inner.as_mut(), 0).context("Reading TGC header")?;
        if header.magic != TGC_MAGIC {
            return Err(Error::DiscFormat("Invalid TGC magic".to_string()));
        }
        let disc_size = (header.gcm_files_start.get() + header.user_size.get()) as u64;

        // Read GCM header
        let raw_header = read_arc_at::<[u8; GCM_HEADER_SIZE], _>(
            inner.as_mut(),
            header.header_offset.get() as u64,
        )
        .context("Reading GCM header")?;

        let disc_header =
            DiscHeader::ref_from_bytes(array_ref![raw_header, 0, size_of::<DiscHeader>()])
                .expect("Invalid disc header alignment");
        let disc_header = disc_header.clone();
        let boot_header =
            BootHeader::ref_from_bytes(array_ref![raw_header, BB2_OFFSET, size_of::<BootHeader>()])
                .expect("Invalid boot header alignment");
        let boot_header = boot_header.clone();

        // Read DOL
        let raw_dol = read_arc_slice_at::<u8, _>(
            inner.as_mut(),
            header.dol_size.get() as usize,
            header.dol_offset.get() as u64,
        )
        .context("Reading DOL")?;

        // Read FST
        let raw_fst = read_arc_slice_at::<u8, _>(
            inner.as_mut(),
            header.fst_size.get() as usize,
            header.fst_offset.get() as u64,
        )
        .context("Reading FST")?;
        let fst = Fst::new(&raw_fst)?;

        let mut write_info = Vec::with_capacity(5 + fst.num_files());
        write_info.push(WriteInfo {
            kind: WriteKind::Static(raw_header, "sys/header.bin"),
            size: GCM_HEADER_SIZE as u64,
            offset: 0,
        });
        write_info.push(WriteInfo {
            kind: WriteKind::Static(raw_dol, "sys/main.dol"),
            size: header.dol_size.get() as u64,
            offset: boot_header.dol_offset(false),
        });
        write_info.push(WriteInfo {
            kind: WriteKind::Static(raw_fst.clone(), "sys/fst.bin"),
            size: header.fst_size.get() as u64,
            offset: boot_header.fst_offset(false),
        });

        // Collect files
        for (_, node, path) in fst.iter() {
            if node.is_dir() {
                continue;
            }
            write_info.push(WriteInfo {
                kind: WriteKind::File(path),
                size: node.length() as u64,
                offset: node.offset(false),
            });
        }
        write_info.sort_unstable_by(|a, b| a.offset.cmp(&b.offset).then(a.size.cmp(&b.size)));
        let write_info = insert_junk_data(write_info, &boot_header, false);

        let file_callback = FileCallbackTGC::new(inner, raw_fst, header);
        let disc_id = *array_ref![disc_header.game_id, 0, 4];
        let disc_num = disc_header.disc_num;
        Ok(Box::new(Self {
            inner: GCPartitionStream::new(
                file_callback,
                Arc::from(write_info),
                disc_size,
                disc_id,
                disc_num,
            ),
        }))
    }
}

impl BlockReader for BlockReaderTGC {
    fn read_block(&mut self, out: &mut [u8], sector: u32) -> io::Result<Block> {
        let count = (out.len() / SECTOR_SIZE) as u32;
        self.inner.set_position(sector as u64 * SECTOR_SIZE as u64);
        let read = read_with_zero_fill(&mut self.inner, out)?;
        Ok(Block::sectors(sector, count, if read == 0 { BlockKind::None } else { BlockKind::Raw }))
    }

    fn block_size(&self) -> u32 { SECTOR_SIZE as u32 }

    fn meta(&self) -> DiscMeta {
        DiscMeta { format: Format::Tgc, disc_size: Some(self.inner.len()), ..Default::default() }
    }
}

#[derive(Clone)]
struct FileCallbackTGC {
    inner: Box<dyn DiscStream>,
    fst: Arc<[u8]>,
    header: TGCHeader,
}

impl FileCallbackTGC {
    fn new(inner: Box<dyn DiscStream>, fst: Arc<[u8]>, header: TGCHeader) -> Self {
        Self { inner, fst, header }
    }
}

impl FileCallback for FileCallbackTGC {
    fn read_file(&mut self, out: &mut [u8], name: &str, offset: u64) -> io::Result<()> {
        let fst = Fst::new(&self.fst).map_err(io::Error::other)?;
        let (_, node) = fst.find(name).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, format!("File not found in FST: {}", name))
        })?;
        if !node.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Path is a directory: {}", name),
            ));
        }
        // Calculate file offset in TGC
        let file_start = (node.offset(false) as u32 - self.header.gcm_files_start.get())
            + self.header.user_offset.get();
        self.inner.read_exact_at(out, file_start as u64 + offset)?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct DiscWriterTGC {
    inner: Box<dyn PartitionReader>,
    header: TGCHeader,
    header_data: Bytes,
    output_size: u64,
}

impl DiscWriterTGC {
    pub fn new(reader: DiscReader, options: &FormatOptions) -> Result<Box<dyn DiscWriter>> {
        if options.format != Format::Tgc {
            return Err(Error::DiscFormat("Invalid format for TGC writer".to_string()));
        }
        if options.compression != Compression::None {
            return Err(Error::DiscFormat("TGC does not support compression".to_string()));
        }

        let mut inner =
            reader.open_partition_kind(PartitionKind::Data, &PartitionOptions::default())?;

        // Read GCM header
        let mut raw_header = <[u8; GCM_HEADER_SIZE]>::new_box_zeroed()?;
        inner.read_exact(raw_header.as_mut()).context("Reading GCM header")?;
        let boot_header =
            BootHeader::ref_from_bytes(array_ref![raw_header, BB2_OFFSET, size_of::<BootHeader>()])
                .expect("Invalid boot header alignment");

        // Read DOL
        let raw_dol = read_dol(inner.as_mut(), boot_header, false)?;
        let raw_fst = read_fst(inner.as_mut(), boot_header, false)?;

        // Parse FST
        let fst = Fst::new(&raw_fst)?;
        let mut gcm_files_start = u32::MAX;
        for (_, node, _) in fst.iter() {
            if node.is_file() {
                let start = node.offset(false) as u32;
                if start < gcm_files_start {
                    gcm_files_start = start;
                }
            }
        }

        // Layout system files
        let gcm_header_offset = SECTOR_SIZE as u32;
        let fst_offset = gcm_header_offset + GCM_HEADER_SIZE as u32;
        let dol_offset = (fst_offset + boot_header.fst_size.get()).align_up(32);
        let user_size =
            boot_header.user_offset.get() + boot_header.user_size.get() - gcm_files_start;
        let user_end = (dol_offset + raw_dol.len() as u32 + user_size).align_up(SECTOR_SIZE as u32);
        let user_offset = user_end - user_size;

        let header = TGCHeader {
            magic: TGC_MAGIC,
            version: 0.into(),
            header_offset: gcm_header_offset.into(),
            header_size: (GCM_HEADER_SIZE as u32).into(),
            fst_offset: fst_offset.into(),
            fst_size: boot_header.fst_size,
            fst_max_size: boot_header.fst_max_size,
            dol_offset: dol_offset.into(),
            dol_size: (raw_dol.len() as u32).into(),
            user_offset: user_offset.into(),
            user_size: user_size.into(),
            banner_offset: 0.into(),
            banner_size: 0.into(),
            gcm_files_start: gcm_files_start.into(),
        };
        let mut buffer = BytesMut::with_capacity(user_offset as usize);
        buffer.put_slice(header.as_bytes());
        buffer.put_bytes(0, gcm_header_offset as usize - buffer.len());

        // Write GCM header
        buffer.put_slice(raw_header.as_ref());
        buffer.put_bytes(0, fst_offset as usize - buffer.len());

        // Write FST
        buffer.put_slice(raw_fst.as_ref());
        buffer.put_bytes(0, dol_offset as usize - buffer.len());

        // Write DOL
        buffer.put_slice(raw_dol.as_ref());
        buffer.put_bytes(0, user_offset as usize - buffer.len());

        let header_data = buffer.freeze();
        Ok(Box::new(Self { inner, header, header_data, output_size: user_end as u64 }))
    }
}

impl DiscWriter for DiscWriterTGC {
    fn process(
        &self,
        data_callback: &mut DataCallback,
        _options: &ProcessOptions,
    ) -> Result<DiscFinalization> {
        let mut data_position = self.header.user_offset.get() as u64;
        data_callback(self.header_data.clone(), data_position, self.output_size)
            .context("Failed to write TGC header")?;

        // Write user data serially
        let mut inner = self.inner.clone();
        inner
            .seek(SeekFrom::Start(self.header.gcm_files_start.get() as u64))
            .context("Seeking to GCM files start")?;
        loop {
            // TODO use DiscReader::fill_buf_internal
            let buf = inner
                .fill_buf()
                .with_context(|| format!("Reading disc data at offset {data_position}"))?;
            let len = buf.len();
            if len == 0 {
                break;
            }
            data_position += len as u64;
            data_callback(Bytes::copy_from_slice(buf), data_position, self.output_size)
                .context("Failed to write disc data")?;
            inner.consume(len);
        }

        Ok(DiscFinalization::default())
    }

    fn progress_bound(&self) -> u64 { self.output_size }

    fn weight(&self) -> DiscWriterWeight { DiscWriterWeight::Light }
}
