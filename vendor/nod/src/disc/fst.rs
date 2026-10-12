//! File system table (FST) types.

use std::{borrow::Cow, ffi::CStr, mem::size_of};

use encoding_rs::SHIFT_JIS;
use itertools::Itertools;
use zerocopy::{FromBytes, FromZeros, Immutable, IntoBytes, KnownLayout, big_endian::*};

use crate::{
    Error, Result,
    util::{array_ref, static_assert},
};

/// File system node kind.
#[derive(Clone, Debug, PartialEq)]
pub enum NodeKind {
    /// Node is a file.
    File,
    /// Node is a directory.
    Directory,
    /// Invalid node kind. (Should not normally occur)
    Invalid,
}

/// An individual file system node.
#[derive(Copy, Clone, Debug, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
pub struct Node {
    kind: u8,
    // u24 big-endian
    name_offset: [u8; 3],
    offset: U32,
    length: U32,
}

static_assert!(size_of::<Node>() == 12);

impl Node {
    /// Create a new node.
    #[inline]
    pub fn new(kind: NodeKind, name_offset: u32, offset: u64, length: u32, is_wii: bool) -> Self {
        let name_offset_bytes = name_offset.to_be_bytes();
        Self {
            kind: match kind {
                NodeKind::File => 0,
                NodeKind::Directory => 1,
                NodeKind::Invalid => u8::MAX,
            },
            name_offset: *array_ref![name_offset_bytes, 1, 3],
            offset: U32::new(match kind {
                NodeKind::File if is_wii => (offset / 4) as u32,
                _ => offset as u32,
            }),
            length: U32::new(length),
        }
    }

    /// File system node kind.
    #[inline]
    pub fn kind(&self) -> NodeKind {
        match self.kind {
            0 => NodeKind::File,
            1 => NodeKind::Directory,
            _ => NodeKind::Invalid,
        }
    }

    /// Set the node kind.
    #[inline]
    pub fn set_kind(&mut self, kind: NodeKind) {
        self.kind = match kind {
            NodeKind::File => 0,
            NodeKind::Directory => 1,
            NodeKind::Invalid => u8::MAX,
        };
    }

    /// Whether the node is a file.
    #[inline]
    pub fn is_file(&self) -> bool { self.kind == 0 }

    /// Whether the node is a directory.
    #[inline]
    pub fn is_dir(&self) -> bool { self.kind == 1 }

    /// Offset in the string table to the filename.
    #[inline]
    pub fn name_offset(&self) -> u32 {
        u32::from_be_bytes([0, self.name_offset[0], self.name_offset[1], self.name_offset[2]])
    }

    /// Set the name offset of the node.
    #[inline]
    pub fn set_name_offset(&mut self, name_offset: u32) {
        let name_offset_bytes = name_offset.to_be_bytes();
        self.name_offset = *array_ref![name_offset_bytes, 1, 3];
    }

    /// For files, this is the partition offset of the file data. (Wii: >> 2)
    ///
    /// For directories, this is the parent node index in the FST.
    #[inline]
    pub fn offset(&self, is_wii: bool) -> u64 {
        if is_wii && self.is_file() {
            self.offset.get() as u64 * 4
        } else {
            self.offset.get() as u64
        }
    }

    /// Set the offset of the node. See [`Node::offset`] for details.
    #[inline]
    pub fn set_offset(&mut self, offset: u64, is_wii: bool) {
        self.offset.set(if is_wii && self.is_file() { (offset / 4) as u32 } else { offset as u32 });
    }

    /// For files, this is the byte size of the file.
    ///
    /// For directories, this is the child end index in the FST.
    ///
    /// Number of child files and directories recursively is `length - offset`.
    #[inline]
    pub fn length(&self) -> u32 { self.length.get() }

    /// Set the length of the node. See [`Node::length`] for details.
    #[inline]
    pub fn set_length(&mut self, length: u32) { self.length.set(length); }
}

/// A view into the file system table (FST).
#[derive(Clone)]
pub struct Fst<'a> {
    /// The nodes in the FST.
    pub nodes: &'a [Node],
    /// The string table containing all file and directory names.
    pub string_table: &'a [u8],
}

impl<'a> Fst<'a> {
    /// Create a new FST view from a buffer.
    pub fn new(buf: &'a [u8]) -> Result<Self, &'static str> {
        let Ok((root_node, _)) = Node::ref_from_prefix(buf) else {
            return Err("FST root node not found");
        };
        // String table starts after the last node
        let string_base = root_node.length() * size_of::<Node>() as u32;
        if string_base > buf.len() as u32 {
            return Err("FST string table out of bounds");
        }
        let (node_buf, string_table) = buf.split_at(string_base as usize);
        let nodes = <[Node]>::ref_from_bytes(node_buf).unwrap();
        Ok(Self { nodes, string_table })
    }

    /// Iterate over the nodes in the FST.
    #[inline]
    pub fn iter(&self) -> FstIter<'_> { FstIter { fst: self.clone(), idx: 1, segments: vec![] } }

    /// Get the name of a node.
    pub fn get_name(&self, node: Node) -> Result<Cow<'a, str>, String> {
        let name_buf = self.string_table.get(node.name_offset() as usize..).ok_or_else(|| {
            format!(
                "FST: name offset {} out of bounds (string table size: {})",
                node.name_offset(),
                self.string_table.len()
            )
        })?;
        let c_string = CStr::from_bytes_until_nul(name_buf).map_err(|_| {
            format!("FST: name at offset {} not null-terminated", node.name_offset())
        })?;
        let (decoded, _, _) = SHIFT_JIS.decode(c_string.to_bytes());
        // Ignore decoding errors, we can't do anything about them. Consumers may check for
        // U+FFFD (REPLACEMENT CHARACTER), or fetch the raw bytes from the string table.
        Ok(decoded)
    }

    /// Finds a particular file or directory by path.
    pub fn find(&self, path: &str) -> Option<(usize, Node)> {
        let mut split = path.trim_matches('/').split('/');
        let mut current = next_non_empty(&mut split);
        if current.is_empty() {
            return Some((0, self.nodes[0]));
        }
        let mut idx = 1;
        let mut stop_at = None;
        while let Some(node) = self.nodes.get(idx).copied() {
            if self.get_name(node).as_ref().is_ok_and(|name| name.eq_ignore_ascii_case(current)) {
                current = next_non_empty(&mut split);
                if current.is_empty() {
                    return Some((idx, node));
                }
                // Descend into directory
                idx += 1;
                stop_at = Some(node.length() as usize + idx);
            } else if node.is_dir() {
                // Skip directory
                idx = node.length() as usize;
            } else {
                // Skip file
                idx += 1;
            }
            if let Some(stop) = stop_at {
                if idx >= stop {
                    break;
                }
            }
        }
        None
    }

    /// Count the number of files in the FST.
    pub fn num_files(&self) -> usize { self.nodes.iter().filter(|n| n.is_file()).count() }
}

/// Iterator over the nodes in an FST.
///
/// For each node, the iterator yields the node index, the node itself,
/// and the full path to the node (separated by `/`).
pub struct FstIter<'a> {
    fst: Fst<'a>,
    idx: usize,
    segments: Vec<(Cow<'a, str>, usize)>,
}

impl Iterator for FstIter<'_> {
    type Item = (usize, Node, String);

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx;
        let node = self.fst.nodes.get(idx).copied()?;
        let name = self.fst.get_name(node).unwrap_or("<invalid>".into());
        self.idx += 1;

        // Remove ended path segments
        let mut new_size = 0;
        for (_, end) in self.segments.iter() {
            if *end == idx {
                break;
            }
            new_size += 1;
        }
        self.segments.truncate(new_size);

        // Add the new path segment
        let length = node.length() as u64;
        let end = if node.is_dir() { length as usize } else { idx + 1 };
        self.segments.push((name, end));
        let path = self.segments.iter().map(|(name, _)| name.as_ref()).join("/");
        Some((idx, node, path))
    }
}

#[inline]
fn next_non_empty<'a>(iter: &mut impl Iterator<Item = &'a str>) -> &'a str {
    loop {
        match iter.next() {
            Some("") => continue,
            Some(next) => break next,
            None => break "",
        }
    }
}

/// A builder for creating a file system table (FST).
pub struct FstBuilder {
    nodes: Vec<Node>,
    string_table: Vec<u8>,
    stack: Vec<(String, u32)>,
    is_wii: bool,
}

impl FstBuilder {
    /// Create a new FST builder.
    pub fn new(is_wii: bool) -> Self {
        let mut builder = Self { nodes: vec![], string_table: vec![], stack: vec![], is_wii };
        builder.add_node(NodeKind::Directory, "<root>", 0, 0);
        builder
    }

    /// Create a new FST builder with an existing string table. This allows matching the string
    /// ordering of an existing FST.
    pub fn new_with_string_table(is_wii: bool, string_table: Vec<u8>) -> Result<Self> {
        if matches!(string_table.last(), Some(n) if *n != 0) {
            return Err(Error::DiscFormat("String table must be null-terminated".to_string()));
        }
        let root_name = CStr::from_bytes_until_nul(&string_table)
            .map_err(|_| {
                Error::DiscFormat("String table root name not null-terminated".to_string())
            })?
            .to_str()
            .unwrap_or("<root>")
            .to_string();
        let mut builder = Self { nodes: vec![], string_table, stack: vec![], is_wii };
        builder.add_node(NodeKind::Directory, &root_name, 0, 0);
        Ok(builder)
    }

    /// Add a file to the FST. All paths within a directory must be added sequentially,
    /// otherwise the output FST will be invalid.
    pub fn add_file(&mut self, path: &str, offset: u64, size: u32) {
        let components = path.split('/').collect::<Vec<_>>();
        for i in 0..components.len() - 1 {
            if matches!(self.stack.get(i), Some((name, _)) if name != components[i]) {
                // Pop directories
                while self.stack.len() > i {
                    let (_, idx) = self.stack.pop().unwrap();
                    let length = self.nodes.len() as u32;
                    self.nodes[idx as usize].set_length(length);
                }
            }
            while i >= self.stack.len() {
                // Push a new directory node
                let component_idx = self.stack.len();
                let parent = if component_idx == 0 { 0 } else { self.stack[component_idx - 1].1 };
                let node_idx =
                    self.add_node(NodeKind::Directory, components[component_idx], parent as u64, 0);
                self.stack.push((components[i].to_string(), node_idx));
            }
        }
        if components.len() == 1 {
            // Pop all directories
            while let Some((_, idx)) = self.stack.pop() {
                let length = self.nodes.len() as u32;
                self.nodes[idx as usize].set_length(length);
            }
        }
        // Add file node
        self.add_node(NodeKind::File, components.last().unwrap(), offset, size);
    }

    /// Get the byte size of the FST.
    pub fn byte_size(&self) -> usize {
        size_of_val(self.nodes.as_slice()) + self.string_table.len()
    }

    /// Finalize the FST and return the serialized data.
    pub fn finalize(mut self) -> Box<[u8]> {
        // Finalize directory lengths
        let node_count = self.nodes.len() as u32;
        while let Some((_, idx)) = self.stack.pop() {
            self.nodes[idx as usize].set_length(node_count);
        }
        self.nodes[0].set_length(node_count);

        // Serialize nodes and string table
        let nodes_data = self.nodes.as_bytes();
        let string_table_data = self.string_table.as_bytes();
        let mut data =
            <[u8]>::new_box_zeroed_with_elems(nodes_data.len() + string_table_data.len()).unwrap();
        data[..nodes_data.len()].copy_from_slice(self.nodes.as_bytes());
        data[nodes_data.len()..].copy_from_slice(self.string_table.as_bytes());
        data
    }

    fn add_node(&mut self, kind: NodeKind, name: &str, offset: u64, length: u32) -> u32 {
        let (bytes, _, _) = SHIFT_JIS.encode(name);
        // Check if the name already exists in the string table
        let mut name_offset = 0;
        while name_offset < self.string_table.len() {
            let string_buf = &self.string_table[name_offset..];
            let existing = CStr::from_bytes_until_nul(string_buf).unwrap();
            if existing.to_bytes() == bytes.as_ref() {
                break;
            }
            name_offset += existing.to_bytes_with_nul().len();
        }
        // Otherwise, add the name to the string table
        if name_offset == self.string_table.len() {
            self.string_table.extend_from_slice(bytes.as_ref());
            self.string_table.push(0);
        }
        let idx = self.nodes.len() as u32;
        self.nodes.push(Node::new(kind, name_offset as u32, offset, length, self.is_wii));
        idx
    }
}
