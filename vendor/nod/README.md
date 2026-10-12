# nod [![Build Status]][actions] [![Latest Version]][crates.io] [![Api Rustdoc]][rustdoc] ![Rust Version]

[Build Status]: https://github.com/encounter/nod/actions/workflows/build.yaml/badge.svg
[actions]: https://github.com/encounter/nod/actions
[Latest Version]: https://img.shields.io/crates/v/nod.svg
[crates.io]: https://crates.io/crates/nod
[Api Rustdoc]: https://img.shields.io/badge/api-rustdoc-blue.svg
[rustdoc]: https://docs.rs/nod
[Rust Version]: https://img.shields.io/badge/rust-1.85+-blue.svg?maxAge=3600

Library for reading and writing Nintendo Optical Disc (GameCube and Wii) images.

Primarily a Rust crate with a [C API](#c-api) for integration with C and C++ projects.

Originally based on the C++ library [nod](https://github.com/AxioDL/nod),
but with extended format support and many additional features.

Currently supported file formats:

- ISO (GCM)
- WIA / RVZ
- WBFS (+ NKit 2 lossless)
- CISO (+ NKit 2 lossless)
- NFS (Wii U VC, read-only)
- GCZ
- TGC

## CLI tool

This crate includes a command-line tool called `nodtool`.

Download the latest release from the [releases page](https://github.com/encounter/nod-rs/releases),
or install it using Cargo:

```shell
cargo install --locked nodtool
```

### info

Displays information about a disc image.

```shell
nodtool info /path/to/game.iso
```

### extract

Extracts the contents of a disc image to a directory.

```shell
nodtool extract /path/to/game.iso [outdir]
```

For Wii U VC titles, use `content/hif_000000.nfs`:

```shell
nodtool extract /path/to/game/content/hif_000000.nfs [outdir]
```

### convert

Converts a disc image to any supported format.

See `nodtool convert --help` for more information.

```shell
nodtool convert /path/to/game.iso /path/to/game.rvz
```

### verify

Verifies a disc image against an internal Redump database.

```shell
nodtool verify /path/to/game.iso
```

## Library example

Opening a disc image and reading a file:

```rust
use std::io::Read;

use nod::{
    common::PartitionKind,
    read::{DiscOptions, DiscReader, PartitionOptions},
};

// Open a disc image and the first data partition.
let disc =
    DiscReader::new("path/to/file.iso", &DiscOptions::default()).expect("Failed to open disc");
let mut partition = disc
    .open_partition_kind(PartitionKind::Data, &PartitionOptions::default())
    .expect("Failed to open data partition");

// Read partition metadata and the file system table.
let meta = partition.meta().expect("Failed to read partition metadata");
let fst = meta.fst().expect("File system table is invalid");

// Find a file by path and read it into a string.
if let Some((_, node)) = fst.find("/MP3/Worlds.txt") {
    let mut s = String::new();
    partition
        .open_file(node)
        .expect("Failed to open file stream")
        .read_to_string(&mut s)
        .expect("Failed to read file");
    println!("{}", s);
}
```

Converting a disc image to raw ISO:

```rust
use nod::read::{DiscOptions, DiscReader, PartitionEncryption};

let options = DiscOptions {
    partition_encryption: PartitionEncryption::Original,
    // Use 4 threads to preload data as the disc is read. This can speed up sequential reads,
    // especially when the disc image format uses compression.
    preloader_threads: 4,
};
// Open a disc image.
let mut disc = DiscReader::new("path/to/file.rvz", &options).expect("Failed to open disc");

// Create a new output file.
let mut out = std::fs::File::create("output.iso").expect("Failed to create output file");
// Read directly from the DiscReader and write to the output file.
// NOTE: Any copy method that accepts `Read` and `Write` can be used here,
// such as `std::io::copy`. This example utilizes `BufRead` for efficiency,
// since `DiscReader` has its own internal buffer.
nod::util::buf_copy(&mut disc, &mut out).expect("Failed to write data");
```

Converting a disc image to RVZ:

```rust
use std::fs::File;
use std::io::{Seek, Write};
use nod::common::{Compression, Format};
use nod::read::{DiscOptions, DiscReader, PartitionEncryption};
use nod::write::{DiscWriter, DiscWriterWeight, FormatOptions, ProcessOptions};

let open_options = DiscOptions {
    partition_encryption: PartitionEncryption::Original,
    // Use 4 threads to preload data as the disc is read. This can speed up sequential reads,
    // especially when the disc image format uses compression.
    preloader_threads: 4,
};
// Open a disc image.
let disc = DiscReader::new("path/to/file.iso", &open_options)
    .expect("Failed to open disc");
// Create a new output file.
let mut output_file = File::create("output.rvz")
    .expect("Failed to create output file");

let options = FormatOptions {
    format: Format::Rvz,
    compression: Compression::Zstandard(19),
    block_size: Format::Rvz.default_block_size(),
};
// Create a disc writer with the desired output format.
let mut writer = DiscWriter::new(disc, &options)
    .expect("Failed to create writer");

// Ideally we'd base this on the actual number of CPUs available.
// This is just an example.
let num_threads = match writer.weight() {
    DiscWriterWeight::Light => 0,
    DiscWriterWeight::Medium => 4,
    DiscWriterWeight::Heavy => 12,
};
let process_options = ProcessOptions {
    processor_threads: num_threads,
    // Enable checksum calculation for the _original_ disc data.
    // Digests will be stored in the output file for verification, if supported.
    // They will also be returned in the finalization result.
    digest_crc32: true,
    digest_md5: false, // MD5 is slow, skip it
    digest_sha1: true,
    digest_xxh64: true,
};
// Start processing the disc image.
let finalization = writer.process(
    |data, _progress, _total| {
        output_file.write_all(data.as_ref())?;
        // One could display progress here, if desired.
        Ok(())
    },
    &process_options
)
.expect("Failed to process disc image");

// Some disc writers calculate data during processing.
// If the finalization returns header data, seek to the beginning of the file and write it.
if !finalization.header.is_empty() {
    output_file.rewind()
        .expect("Failed to seek");
    output_file.write_all(finalization.header.as_ref())
        .expect("Failed to write header");
}
output_file.flush().expect("Failed to flush output file");

// Display the calculated digests.
println!("CRC32: {:08X}", finalization.crc32.unwrap());
// ...
```

## C API

This repository also provides a [C API](nod-ffi/include/nod.h) for interfacing with other languages.

For a full end-to-end example of using the C API, see [SDL3 IOStream Demo](nod-ffi/examples/sdl3-stream-demo/README.md).

### Integration with CMake

The top-level `CMakeLists.txt` builds the Rust library via [Corrosion](https://github.com/corrosion-rs/corrosion) and exports the target `nod::nod`.

Features can be toggled with CMake options:

- `NOD_COMPRESS_BZIP2` (default `ON`)
- `NOD_COMPRESS_LZMA` (default `ON`)
- `NOD_COMPRESS_ZLIB` (default `ON`)
- `NOD_COMPRESS_ZSTD` (default `ON`)
- `NOD_THREADING` (default `ON`)

Example:

```cmake
cmake_minimum_required(VERSION 3.23)
project(my_app C CXX)

include(FetchContent)

FetchContent_Declare(
  nod
  GIT_REPOSITORY https://github.com/encounter/nod.git
  GIT_TAG [tag]
)

# Optional feature toggles
set(NOD_COMPRESS_BZIP2 ON CACHE INTERNAL "Enable BZIP2 support")
set(NOD_COMPRESS_LZMA ON CACHE INTERNAL "Enable LZMA/LZMA2 support")
set(NOD_COMPRESS_ZLIB ON CACHE INTERNAL "Enable zlib/deflate support")
set(NOD_COMPRESS_ZSTD ON CACHE INTERNAL "Enable Zstandard support")
set(NOD_THREADING ON CACHE INTERNAL "Enable threaded processing support")

FetchContent_MakeAvailable(nod)

add_executable(my_app main.cpp)
target_link_libraries(my_app PRIVATE nod::nod)
```

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
