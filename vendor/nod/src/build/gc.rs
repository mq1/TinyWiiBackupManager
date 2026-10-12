#![allow(missing_docs, unused)] // TODO
use std::{
    io,
    io::{Read, Seek, Write},
    sync::Arc,
};

use tracing::debug;
use zerocopy::{FromZeros, IntoBytes};

use crate::{
    Error, Result, ResultContext,
    disc::{
        BB2_OFFSET, BI2_SIZE, BOOT_SIZE, BootHeader, DiscHeader, GCN_MAGIC, MINI_DVD_SIZE,
        SECTOR_SIZE, WII_MAGIC,
        fst::{Fst, FstBuilder},
    },
    read::{CloneableStream, DiscStream, NonCloneableStream},
    util::{Align, array_ref, array_ref_mut, lfg::LaggedFibonacci},
};

pub trait FileCallback: Send {
    fn read_file(&mut self, out: &mut [u8], name: &str, offset: u64) -> io::Result<()>;
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub offset: Option<u64>,
    pub alignment: Option<u32>,
}

pub struct GCPartitionBuilder {
    disc_header: Box<DiscHeader>,
    boot_header: Box<BootHeader>,
    user_files: Vec<FileInfo>,
    overrides: PartitionOverrides,
    junk_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteKind {
    File(String),
    Static(Arc<[u8]>, &'static str),
    Junk,
}

impl WriteKind {
    fn name(&self) -> &str {
        match self {
            WriteKind::File(name) => name,
            WriteKind::Static(_, name) => name,
            WriteKind::Junk => "[junk data]",
        }
    }
}

#[derive(Debug, Clone)]
pub struct WriteInfo {
    pub kind: WriteKind,
    pub size: u64,
    pub offset: u64,
}

pub struct GCPartitionWriter {
    write_info: Vec<WriteInfo>,
    disc_size: u64,
    disc_id: [u8; 4],
    disc_num: u8,
}

const BI2_OFFSET: u64 = BOOT_SIZE as u64;
const APPLOADER_OFFSET: u64 = BI2_OFFSET + BI2_SIZE as u64;

#[derive(Debug, Clone, Default)]
pub struct PartitionOverrides {
    pub game_id: Option<[u8; 6]>,
    pub game_title: Option<String>,
    pub disc_num: Option<u8>,
    pub disc_version: Option<u8>,
    pub audio_streaming: Option<bool>,
    pub audio_stream_buf_size: Option<u8>,
    pub junk_id: Option<[u8; 4]>,
    pub region: Option<u8>,
}

impl GCPartitionBuilder {
    pub fn new(is_wii: bool, overrides: PartitionOverrides) -> Self {
        let mut disc_header = DiscHeader::new_box_zeroed().unwrap();
        if is_wii {
            disc_header.gcn_magic = [0u8; 4];
            disc_header.wii_magic = WII_MAGIC;
        } else {
            disc_header.gcn_magic = GCN_MAGIC;
            disc_header.wii_magic = [0u8; 4];
        }
        Self {
            disc_header,
            boot_header: BootHeader::new_box_zeroed().unwrap(),
            user_files: Vec::new(),
            overrides,
            junk_files: Vec::new(),
        }
    }

    #[inline]
    pub fn set_disc_header(&mut self, disc_header: Box<DiscHeader>) {
        self.disc_header = disc_header;
    }

    #[inline]
    pub fn set_boot_header(&mut self, boot_header: Box<BootHeader>) {
        self.boot_header = boot_header;
    }

    pub fn add_file(&mut self, info: FileInfo) -> Result<()> {
        if let (Some(offset), Some(alignment)) = (info.offset, info.alignment) {
            if offset % alignment as u64 != 0 {
                return Err(Error::Other(format!(
                    "File {} offset {:#X} is not aligned to {}",
                    info.name, offset, alignment
                )));
            }
        }
        self.user_files.push(info);
        Ok(())
    }

    /// A junk file exists in the FST, but is excluded from the disc layout, so junk data will be
    /// written in its place.
    pub fn add_junk_file(&mut self, name: String) { self.junk_files.push(name); }

    pub fn build(
        &self,
        sys_file_callback: impl FnMut(&mut dyn Write, &str) -> io::Result<()>,
    ) -> Result<GCPartitionWriter> {
        let mut layout = GCPartitionLayout::new(self);
        layout.locate_sys_files(sys_file_callback)?;
        layout.apply_overrides(&self.overrides)?;
        let write_info = layout.layout_files()?;
        let disc_size =
            layout.boot_header.user_offset.get() as u64 + layout.boot_header.user_size.get() as u64;
        let junk_id = layout.junk_id();
        Ok(GCPartitionWriter::new(write_info, disc_size, junk_id, self.disc_header.disc_num))
    }
}

struct GCPartitionLayout {
    disc_header: Box<DiscHeader>,
    boot_header: Box<BootHeader>,
    user_files: Vec<FileInfo>,
    apploader_file: Option<FileInfo>,
    dol_file: Option<FileInfo>,
    raw_fst: Option<Box<[u8]>>,
    raw_bi2: Option<Box<[u8]>>,
    junk_id: Option<[u8; 4]>,
    junk_files: Vec<String>,
}

impl GCPartitionLayout {
    fn new(builder: &GCPartitionBuilder) -> Self {
        GCPartitionLayout {
            disc_header: builder.disc_header.clone(),
            boot_header: builder.boot_header.clone(),
            user_files: builder.user_files.clone(),
            apploader_file: None,
            dol_file: None,
            raw_fst: None,
            raw_bi2: None,
            junk_id: builder.overrides.junk_id,
            junk_files: builder.junk_files.clone(),
        }
    }

    fn locate_sys_files(
        &mut self,
        mut file_callback: impl FnMut(&mut dyn Write, &str) -> io::Result<()>,
    ) -> Result<()> {
        let mut handled = vec![false; self.user_files.len()];

        // Locate fixed offset system files
        for (info, handled) in self.user_files.iter().zip(handled.iter_mut()) {
            if info.offset == Some(0) || info.name == "sys/boot.bin" {
                let mut data = Vec::with_capacity(BOOT_SIZE);
                file_callback(&mut data, &info.name)
                    .with_context(|| format!("Failed to read file {}", info.name))?;
                if data.len() != BOOT_SIZE {
                    return Err(Error::Other(format!(
                        "Boot file {} is {} bytes, expected {}",
                        info.name,
                        data.len(),
                        BOOT_SIZE
                    )));
                }
                self.disc_header.as_mut_bytes().copy_from_slice(&data[..size_of::<DiscHeader>()]);
                self.boot_header
                    .as_mut_bytes()
                    .copy_from_slice(&data[BB2_OFFSET..BB2_OFFSET + size_of::<BootHeader>()]);

                *handled = true;
                continue;
            }

            if info.offset == Some(BI2_OFFSET) || info.name == "sys/bi2.bin" {
                let mut data = Vec::with_capacity(BI2_SIZE);
                file_callback(&mut data, &info.name)
                    .with_context(|| format!("Failed to read file {}", info.name))?;
                if data.len() != BI2_SIZE {
                    return Err(Error::Other(format!(
                        "BI2 file {} is {} bytes, expected {}",
                        info.name,
                        data.len(),
                        BI2_SIZE
                    )));
                }
                self.raw_bi2 = Some(data.into_boxed_slice());
                *handled = true;
                continue;
            }

            if info.offset == Some(APPLOADER_OFFSET) || info.name == "sys/apploader.img" {
                self.apploader_file = Some(info.clone());
                *handled = true;
                continue;
            }
        }

        // Locate other system files
        let is_wii = self.disc_header.is_wii();
        for (info, handled) in self.user_files.iter().zip(handled.iter_mut()) {
            let dol_offset = self.boot_header.dol_offset(is_wii);
            if (dol_offset != 0 && info.offset == Some(dol_offset)) || info.name == "sys/main.dol" {
                let mut info = info.clone();
                if info.alignment.is_none() {
                    info.alignment = Some(128);
                }
                self.dol_file = Some(info);
                *handled = true; // TODO DOL in user data
                continue;
            }

            let fst_offset = self.boot_header.fst_offset(is_wii);
            if (fst_offset != 0 && info.offset == Some(fst_offset)) || info.name == "sys/fst.bin" {
                let mut data = Vec::with_capacity(info.size as usize);
                file_callback(&mut data, &info.name)
                    .with_context(|| format!("Failed to read file {}", info.name))?;
                if data.len() != info.size as usize {
                    return Err(Error::Other(format!(
                        "FST file {} is {} bytes, expected {}",
                        info.name,
                        data.len(),
                        info.size
                    )));
                }
                self.raw_fst = Some(data.into_boxed_slice());
                *handled = true;
                continue;
            }
        }

        // Remove handled files
        let mut iter = handled.iter();
        self.user_files.retain(|_| !iter.next().unwrap());
        Ok(())
    }

    fn apply_overrides(&mut self, overrides: &PartitionOverrides) -> Result<()> {
        if let Some(game_id) = overrides.game_id {
            self.disc_header.game_id.copy_from_slice(&game_id);
        }
        if let Some(game_title) = overrides.game_title.as_ref() {
            let max_size = self.disc_header.game_title.len() - 1; // nul terminator
            if game_title.len() > max_size {
                return Err(Error::Other(format!(
                    "Game title \"{}\" is too long ({} > {})",
                    game_title,
                    game_title.len(),
                    max_size
                )));
            }
            let len = game_title.len().min(max_size);
            self.disc_header.game_title[..len].copy_from_slice(&game_title.as_bytes()[..len]);
            self.disc_header.game_title[len..].fill(0);
        }
        if let Some(disc_num) = overrides.disc_num {
            self.disc_header.disc_num = disc_num;
        }
        if let Some(disc_version) = overrides.disc_version {
            self.disc_header.disc_version = disc_version;
        }
        if let Some(audio_streaming) = overrides.audio_streaming {
            self.disc_header.audio_streaming = audio_streaming as u8;
        }
        if let Some(audio_stream_buf_size) = overrides.audio_stream_buf_size {
            self.disc_header.audio_stream_buf_size = audio_stream_buf_size;
        }
        let set_bi2 = self.raw_bi2.is_none() || overrides.region.is_some();
        let raw_bi2 = self.raw_bi2.get_or_insert_with(|| {
            <[u8]>::new_box_zeroed_with_elems(BI2_SIZE).expect("Failed to allocate BI2")
        });
        if set_bi2 {
            let region = overrides.region.unwrap_or(0xFF) as u32;
            *array_ref_mut![raw_bi2, 0x18, 4] = region.to_be_bytes();
        }
        Ok(())
    }

    fn can_use_orig_fst(&self) -> bool {
        if let Some(existing) = self.raw_fst.as_deref() {
            let Ok(existing_fst) = Fst::new(existing) else {
                return false;
            };
            for (_, node, path) in existing_fst.iter() {
                if node.is_dir() {
                    continue;
                }
                if !self.user_files.iter().any(|info| info.name == path)
                    && !self.junk_files.contains(&path)
                {
                    println!("FST file {} not found", path);
                    return false;
                }
            }
            println!("Using existing FST");
            return true;
        }
        false
    }

    fn calculate_fst_size(&self) -> Result<u64> {
        if self.can_use_orig_fst() {
            return Ok(self.raw_fst.as_deref().unwrap().len() as u64);
        }

        let mut file_names = Vec::with_capacity(self.user_files.len());
        for info in &self.user_files {
            file_names.push(info.name.as_str());
        }
        // file_names.sort_unstable();
        let is_wii = self.disc_header.is_wii();
        let mut builder = if let Some(existing) = self.raw_fst.as_deref() {
            let existing_fst = Fst::new(existing)?;
            FstBuilder::new_with_string_table(is_wii, Vec::from(existing_fst.string_table))?
        } else {
            FstBuilder::new(is_wii)
        };
        for name in file_names {
            builder.add_file(name, 0, 0);
        }
        let size = builder.byte_size() as u64;
        // if size != self.partition_header.fst_size(is_wii) {
        //     return Err(Error::Other(format!(
        //         "FST size {} != {}",
        //         size,
        //         self.partition_header.fst_size(is_wii)
        //     )));
        // }
        Ok(size)
    }

    fn generate_fst(&mut self, write_info: &[WriteInfo]) -> Result<Arc<[u8]>> {
        if self.can_use_orig_fst() {
            let fst_data = self.raw_fst.as_ref().unwrap().clone();
            // TODO update offsets and sizes
            // let node_count = Fst::new(fst_data.as_ref())?.nodes.len();
            // let string_base = node_count * size_of::<Node>();
            // let (node_buf, string_table) = fst_data.split_at_mut(string_base);
            // let nodes = <[Node]>::mut_from_bytes(node_buf).unwrap();
            return Ok(Arc::from(fst_data));
        }

        let files = write_info.to_vec();
        // files.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        let is_wii = self.disc_header.is_wii();
        let mut builder = if let Some(existing) = self.raw_fst.as_deref() {
            let existing_fst = Fst::new(existing)?;
            FstBuilder::new_with_string_table(is_wii, Vec::from(existing_fst.string_table))?
        } else {
            FstBuilder::new(is_wii)
        };
        for info in files {
            if let WriteKind::File(name) = info.kind {
                builder.add_file(&name, info.offset, info.size as u32);
            }
        }
        let raw_fst = builder.finalize();
        if raw_fst.len() != self.boot_header.fst_size(is_wii) as usize {
            return Err(Error::Other(format!(
                "FST size mismatch: {} != {}",
                raw_fst.len(),
                self.boot_header.fst_size(is_wii)
            )));
        }
        Ok(Arc::from(raw_fst))
    }

    fn layout_system_data(&mut self, write_info: &mut Vec<WriteInfo>) -> Result<u64> {
        let mut last_offset = 0;

        let Some(apploader_file) = self.apploader_file.as_ref() else {
            return Err(Error::Other("Apploader not set".to_string()));
        };
        let Some(dol_file) = self.dol_file.as_ref() else {
            return Err(Error::Other("DOL not set".to_string()));
        };
        let Some(raw_bi2) = self.raw_bi2.as_ref() else {
            return Err(Error::Other("BI2 not set".to_string()));
        };
        // let Some(raw_fst) = self.raw_fst.as_ref() else {
        //     return Err(Error::Other("FST not set".to_string()));
        // };

        // Reserve space in write_info for the boot block — we fill it in after all header
        // mutations below so the snapshot reflects the final offsets/sizes.
        let boot_write_info_idx = write_info.len();
        write_info.push(WriteInfo { kind: WriteKind::Junk, size: BOOT_SIZE as u64, offset: 0 }); // placeholder
        last_offset += BOOT_SIZE as u64;
        write_info.push(WriteInfo {
            kind: WriteKind::Static(Arc::from(raw_bi2.as_ref()), "[BI2]"),
            size: BI2_SIZE as u64,
            offset: last_offset,
        });
        last_offset += BI2_SIZE as u64;
        write_info.push(WriteInfo {
            kind: WriteKind::File(apploader_file.name.clone()),
            size: apploader_file.size,
            offset: last_offset,
        });
        last_offset += apploader_file.size;

        // Update DOL and FST offsets if not set
        let is_wii = self.disc_header.is_wii();
        let mut dol_offset = self.boot_header.dol_offset(is_wii);
        if dol_offset == 0 {
            dol_offset = last_offset.align_up(dol_file.alignment.unwrap() as u64);
            self.boot_header.set_dol_offset(dol_offset, is_wii);
        }
        let mut fst_offset = self.boot_header.fst_offset(is_wii);
        if fst_offset == 0 {
            // TODO handle DOL in user data
            fst_offset = (dol_offset + dol_file.size).align_up(128);
            self.boot_header.set_fst_offset(fst_offset, is_wii);
        }
        let fst_size = self.calculate_fst_size()?;
        self.boot_header.set_fst_size(fst_size, is_wii);
        if self.boot_header.fst_max_size(is_wii) < fst_size {
            self.boot_header.set_fst_max_size(fst_size, is_wii);
        }

        // Now snapshot the fully-updated boot block.
        let mut boot = <[u8]>::new_box_zeroed_with_elems(BOOT_SIZE)?;
        boot[..size_of::<DiscHeader>()].copy_from_slice(self.disc_header.as_bytes());
        boot[BB2_OFFSET..BB2_OFFSET + size_of::<BootHeader>()]
            .copy_from_slice(self.boot_header.as_bytes());
        write_info[boot_write_info_idx] = WriteInfo {
            kind: WriteKind::Static(Arc::from(boot), "[BOOT]"),
            size: BOOT_SIZE as u64,
            offset: 0,
        };

        if dol_offset < fst_offset {
            write_info.push(WriteInfo {
                kind: WriteKind::File(dol_file.name.clone()),
                size: dol_file.size,
                offset: dol_offset,
            });
        } else {
            // DOL in user data
        }
        // write_info.push(WriteInfo {
        //     kind: WriteKind::Static(Arc::from(raw_fst.as_ref()), "[FST]"),
        //     size: fst_size,
        //     offset: fst_offset,
        // });

        Ok(fst_offset + fst_size)
    }

    fn layout_files(&mut self) -> Result<Vec<WriteInfo>> {
        let mut system_write_info = Vec::new();
        let mut write_info = Vec::with_capacity(self.user_files.len());
        let mut last_offset = self.layout_system_data(&mut system_write_info)?;

        // Layout user data
        let mut user_offset = self.boot_header.user_offset.get() as u64;
        if user_offset == 0 {
            user_offset = last_offset.align_up(SECTOR_SIZE as u64);
            self.boot_header.user_offset.set(user_offset as u32);
        } else if user_offset < last_offset {
            return Err(Error::Other(format!(
                "User offset {:#X} is before FST {:#X}",
                user_offset, last_offset
            )));
        }
        last_offset = user_offset;
        for info in &self.user_files {
            let offset = info
                .offset
                .unwrap_or_else(|| last_offset.align_up(info.alignment.unwrap_or(32) as u64));
            write_info.push(WriteInfo {
                kind: WriteKind::File(info.name.clone()),
                offset,
                size: info.size,
            });
            last_offset = offset + info.size;
        }

        // Generate FST from only user files
        let is_wii = self.disc_header.is_wii();
        let fst_data = self.generate_fst(&write_info)?;
        let fst_size = fst_data.len() as u64;
        write_info.push(WriteInfo {
            kind: WriteKind::Static(fst_data, "[FST]"),
            size: fst_size,
            offset: self.boot_header.fst_offset(is_wii),
        });
        // Add system files to write info
        write_info.extend(system_write_info);
        // Sort files by offset
        sort_files(&mut write_info)?;

        // Update user size if not set
        if self.boot_header.user_size.get() == 0 {
            let user_end = if self.disc_header.is_wii() {
                last_offset.align_up(SECTOR_SIZE as u64)
            } else {
                MINI_DVD_SIZE
            };
            self.boot_header.user_size.set((user_end - user_offset) as u32);
        }

        // Insert junk data
        let write_info = insert_junk_data(write_info, &self.boot_header, self.disc_header.is_wii());

        Ok(write_info)
    }

    fn junk_id(&self) -> [u8; 4] {
        self.junk_id.unwrap_or_else(|| *array_ref![self.disc_header.game_id, 0, 4])
    }
}

pub(crate) fn insert_junk_data(
    write_info: Vec<WriteInfo>,
    boot_header: &BootHeader,
    is_wii: bool,
) -> Vec<WriteInfo> {
    let mut new_write_info = Vec::with_capacity(write_info.len());

    let fst_end = boot_header.fst_offset(is_wii) + boot_header.fst_size(is_wii);
    let file_gap = find_file_gap(&write_info, fst_end);
    let mut last_file_end = 0;
    for info in write_info {
        if let WriteKind::File(..) | WriteKind::Static(..) = &info.kind {
            let aligned_end = gcm_align(last_file_end);
            if info.offset > aligned_end && last_file_end >= fst_end {
                // Junk data is aligned to 4 bytes with a 28 byte padding (aka `(n + 31) & !3`)
                // but a few cases don't have the 28 byte padding. Namely, the junk data after the
                // FST, and the junk data in between the inner and outer rim files. This attempts to
                // determine the correct alignment, but is not 100% accurate.
                let junk_start = if file_gap == Some(last_file_end) {
                    last_file_end.align_up(4)
                } else {
                    aligned_end
                };
                new_write_info.push(WriteInfo {
                    kind: WriteKind::Junk,
                    size: info.offset - junk_start,
                    offset: junk_start,
                });
            }
            last_file_end = info.offset + info.size;
        }
        new_write_info.push(info);
    }
    let aligned_end = gcm_align(last_file_end);
    let user_end = boot_header.user_offset.get() as u64 + boot_header.user_size.get() as u64;
    if aligned_end < user_end && aligned_end >= fst_end {
        new_write_info.push(WriteInfo {
            kind: WriteKind::Junk,
            size: user_end - aligned_end,
            offset: aligned_end,
        });
    }

    new_write_info
}

impl GCPartitionWriter {
    fn new(write_info: Vec<WriteInfo>, disc_size: u64, disc_id: [u8; 4], disc_num: u8) -> Self {
        Self { write_info, disc_size, disc_id, disc_num }
    }

    pub(crate) fn into_gc_stream<Cb>(self, file_callback: Cb) -> GCPartitionStream<Cb> {
        let Self { write_info, disc_size, disc_id, disc_num } = self;
        GCPartitionStream::new(file_callback, Arc::from(write_info), disc_size, disc_id, disc_num)
    }

    pub fn write_to<W>(
        &self,
        out: &mut W,
        mut file_callback: impl FnMut(&mut dyn Write, &str) -> io::Result<()>,
    ) -> Result<()>
    where
        W: Write + ?Sized,
    {
        let mut out = WriteCursor { inner: out, position: 0 };
        let mut lfg = LaggedFibonacci::default();
        for info in &self.write_info {
            out.write_zeroes_until(info.offset).context("Writing padding")?;
            match &info.kind {
                WriteKind::File(name) => file_callback(&mut out, name)
                    .with_context(|| format!("Writing file {}", name))?,
                WriteKind::Static(data, name) => out.write_all(data).with_context(|| {
                    format!("Writing static data {} ({} bytes)", name, data.len())
                })?,
                WriteKind::Junk => {
                    lfg.write_sector_chunked(
                        &mut out,
                        info.size,
                        self.disc_id,
                        self.disc_num,
                        info.offset,
                    )
                    .with_context(|| {
                        format!(
                            "Writing junk data at {:X} -> {:X}",
                            info.offset,
                            info.offset + info.size
                        )
                    })?;
                }
            };
            if out.position != info.offset + info.size {
                return Err(Error::Other(format!(
                    "File {}: Wrote {} bytes, expected {}",
                    info.kind.name(),
                    out.position - info.offset,
                    info.size
                )));
            }
        }
        out.write_zeroes_until(self.disc_size).context("Writing end of file")?;
        out.flush().context("Flushing output")?;
        Ok(())
    }

    pub fn into_cloneable_stream<Cb>(self, file_callback: Cb) -> Result<Box<dyn DiscStream>>
    where Cb: FileCallback + Clone + 'static {
        Ok(Box::new(CloneableStream::new(GCPartitionStream::new(
            file_callback,
            Arc::from(self.write_info),
            self.disc_size,
            self.disc_id,
            self.disc_num,
        ))))
    }

    pub fn into_non_cloneable_stream<Cb>(self, file_callback: Cb) -> Result<Box<dyn DiscStream>>
    where Cb: FileCallback + 'static {
        Ok(Box::new(NonCloneableStream::new(GCPartitionStream::new(
            file_callback,
            Arc::from(self.write_info),
            self.disc_size,
            self.disc_id,
            self.disc_num,
        ))))
    }
}

struct WriteCursor<W> {
    inner: W,
    position: u64,
}

impl<W> WriteCursor<W>
where W: Write
{
    fn write_zeroes_until(&mut self, until: u64) -> io::Result<()> {
        static ZEROES: [u8; 0x1000] = [0u8; 0x1000];
        let mut remaining = until.saturating_sub(self.position);
        while remaining > 0 {
            let write_len = remaining.min(ZEROES.len() as u64) as usize;
            let written = self.write(&ZEROES[..write_len])?;
            remaining -= written as u64;
        }
        Ok(())
    }
}

impl<W> Write for WriteCursor<W>
where W: Write
{
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = self.inner.write(buf)?;
        self.position += len as u64;
        Ok(len)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> { self.inner.flush() }
}

#[derive(Clone)]
pub(crate) struct GCPartitionStream<Cb> {
    file_callback: Cb,
    pos: u64,
    write_info: Arc<[WriteInfo]>,
    size: u64,
    disc_id: [u8; 4],
    disc_num: u8,
}

impl<Cb> GCPartitionStream<Cb> {
    pub fn new(
        file_callback: Cb,
        write_info: Arc<[WriteInfo]>,
        size: u64,
        disc_id: [u8; 4],
        disc_num: u8,
    ) -> Self {
        Self { file_callback, pos: 0, write_info, size, disc_id, disc_num }
    }

    #[inline]
    pub fn set_position(&mut self, pos: u64) { self.pos = pos; }

    #[inline]
    pub fn len(&self) -> u64 { self.size }
}

impl<Cb> Read for GCPartitionStream<Cb>
where Cb: FileCallback
{
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.pos >= self.size {
            // Out of bounds
            return Ok(0);
        }

        let end = (self.size - self.pos).min(out.len() as u64) as usize;
        let mut buf = &mut out[..end];
        let mut curr = self
            .write_info
            .binary_search_by_key(&self.pos, |i| i.offset)
            .unwrap_or_else(|idx| idx.saturating_sub(1));
        let mut pos = self.pos;
        let mut total = 0;
        while !buf.is_empty() {
            let Some(info) = self.write_info.get(curr) else {
                buf.fill(0);
                total += buf.len();
                break;
            };
            if pos > info.offset + info.size {
                curr += 1;
                continue;
            }
            let read = if pos < info.offset {
                let read = buf.len().min((info.offset - pos) as usize);
                buf[..read].fill(0);
                read
            } else {
                let read = buf.len().min((info.offset + info.size - pos) as usize);
                match &info.kind {
                    WriteKind::File(name) => {
                        self.file_callback.read_file(&mut buf[..read], name, pos - info.offset)?;
                    }
                    WriteKind::Static(data, _) => {
                        let offset = (pos - info.offset) as usize;
                        buf[..read].copy_from_slice(&data[offset..offset + read]);
                    }
                    WriteKind::Junk => {
                        let mut lfg = LaggedFibonacci::default();
                        lfg.fill_sector_chunked(&mut buf[..read], self.disc_id, self.disc_num, pos);
                    }
                }
                curr += 1;
                read
            };
            buf = &mut buf[read..];
            pos += read as u64;
            total += read;
        }

        self.pos = pos;
        Ok(total)
    }
}

impl<Cb> Seek for GCPartitionStream<Cb> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.pos = match pos {
            io::SeekFrom::Start(pos) => pos,
            io::SeekFrom::End(v) => self.size.saturating_add_signed(v),
            io::SeekFrom::Current(v) => self.pos.saturating_add_signed(v),
        };
        Ok(self.pos)
    }
}

#[inline(always)]
fn gcm_align(n: u64) -> u64 { (n + 31) & !3 }

fn sort_files(files: &mut [WriteInfo]) -> Result<()> {
    files.sort_unstable_by_key(|info| (info.offset, info.size));
    for i in 1..files.len() {
        let prev = &files[i - 1];
        let cur = &files[i];
        if cur.offset < prev.offset + prev.size {
            let name = match &cur.kind {
                WriteKind::File(name) => name.as_str(),
                WriteKind::Static(_, name) => name,
                WriteKind::Junk => "[junk data]",
            };
            let prev_name = match &prev.kind {
                WriteKind::File(name) => name.as_str(),
                WriteKind::Static(_, name) => name,
                WriteKind::Junk => "[junk data]",
            };
            return Err(Error::Other(format!(
                "File {} ({:#X}-{:#X}) overlaps with {} ({:#X}-{:#X})",
                name,
                cur.offset,
                cur.offset + cur.size,
                prev_name,
                prev.offset,
                prev.offset + prev.size
            )));
        }
    }
    Ok(())
}

/// Files can be located on the inner rim of the disc (closer to the center) or the outer rim
/// (closer to the edge). The inner rim is slower to read, so developers often configured certain
/// files to be located on the outer rim. This function attempts to find a gap in the file offsets
/// between the inner and outer rim, which we need to recreate junk data properly.
fn find_file_gap(file_infos: &[WriteInfo], fst_end: u64) -> Option<u64> {
    let mut last_offset = 0;
    for info in file_infos {
        if let WriteKind::File(..) | WriteKind::Static(..) = &info.kind {
            if last_offset > fst_end && info.offset > last_offset + SECTOR_SIZE as u64 {
                debug!("Found file gap at {:X} -> {:X}", last_offset, info.offset);
                return Some(last_offset);
            }
            last_offset = info.offset + info.size;
        }
    }
    None
}
