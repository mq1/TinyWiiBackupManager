# Link protocol

How Ultimate Game Backup Manager talks to the small *companion apps* that run on the consoles:
vcinject Link on the 3DS today, and later a Wii app (libogc) and a Wii U app (Aroma). There is
one protocol for all of them. It is vcinject Link's protocol, which already works on real 3DS
consoles, with a few additions so that it can describe more than one console.

- **Protocol version:** 1
- **Status:** draft. It becomes binding once the 3DS client (phase 1) is done.

## Overview

```text
┌──────────────────────────────┐   UDP 34013  "ugbm?"     ┌───────────────────────────┐
│ Ultimate Game Backup Manager │ ───────────────────────► │ companion app             │
│ (the client)                 │ ◄─────────────────────── │ (3DS / Wii / Wii U)       │
│                              │   "ugbm-link" + info     │                           │
│                              │                          │ serves one connection     │
│                              │   TCP 34013  frames      │ at a time and shows on    │
│                              │ ◄──────────────────────► │ its screen what happens   │
└──────────────────────────────┘                          └───────────────────────────┘
```

The companion only moves raw data: title lists, files, icons, install streams. Everything is parsed
on the computer, so companion apps stay small and rarely need an update.

## Conventions

- Integers are **little endian** on the wire, on every console. The Wii and the Wii U are big
  endian and swap bytes; the 3DS doesn't have to.
- Text is UTF-8. A *text payload* is `key=value` lines separated by `\n`; values contain no `\n`.
  Unknown keys are ignored. Lines without `=` (such as the first line of a discovery reply) are
  not key/value pairs.
- Title IDs are `u64`, as on all three consoles.

## Discovery

The client sends the UDP datagram `ugbm?` to port **34013**: to the broadcast address, to the /24
broadcast address of every network interface and to every console it has seen before. It repeats
this every 0.4 s for about 2 s, because a 3DS in power saving misses broadcasts.

A companion answers with a text payload whose first line is `ugbm-link`, followed by `port=34013`
and the same keys as the [info](#1--info) command.

If nobody answers (some routers drop broadcasts), the client tries a TCP connection to port 34013
on every address of the /24 network, with a short timeout.

A companion that is busy with a connection doesn't answer discovery. The client therefore keeps a
console *online* for 35 s after its last answer.

**Legacy:** vcinject Link up to version 13 only knows the query `vcinject?` and answers
`vcinject-link`. The client sends both queries. Companion apps for the 3DS keep answering
`vcinject?` so that vcinject still finds them.

## Connection

A TCP connection to the port from discovery. The companion serves **one connection at a time**;
requests on it are answered one after the other, in order.

### Request

| Field | Type | |
| --- | --- | --- |
| command | `u32` | see [Commands](#commands) |
| length | `u32` | bytes of payload that follow |
| payload | `length` bytes | |

### Answer

| Field | Type | |
| --- | --- | --- |
| result | `u32` | 0 is success, everything else an [error](#errors) |
| length | `u32` | bytes of payload that follow |
| payload | `length` bytes | |

Payloads are at most **8 MiB**. Larger data is sent as a [stream](#streams).

### Streams

Some commands carry more data than fits in a payload: game installs, large files. For these, the
request payload holds only the parameters, ending in a `u64 size`, and exactly `size` raw bytes
follow the request on the connection. The companion reads them all, even when it fails part way
(so the connection stays usable), and answers once at the end.

*Legacy:* the 3DS command [7 install](#7--install-3ds-legacy) predates streams: its `length`
covers the whole CIA.

While a stream is running, the client may not send other requests. To show progress, the companion
draws its own bar from the bytes received; the client may additionally show its own.

## Versions and capabilities

A companion tells the client what it is and what it can do in [info](#1--info):

| Key | Meaning |
| --- | --- |
| `app` | the companion's name, e.g. `vcinject-link`, `ugbm-link-wii` |
| `version` | the companion's own version, an integer that only grows. The client offers an update when it ships a newer one. |
| `proto` | the protocol version it speaks. Missing means 0: vcinject Link before this document. |
| `console` | `3ds`, `wii` or `wiiu`. Missing means `3ds`. |
| `commands` | the command numbers it implements, comma separated, e.g. `1,2,3,4,5,6,8,9`. Missing: guess from `proto` and `version`. |

The client checks `commands` before using a command. A companion answers a command it doesn't know
with `0xE0000004`; the client then falls back to something older or asks for an update. Commands
are never renumbered and their payloads are only ever *extended at the end*, so an old client can
ignore what it doesn't know.

The protocol version only changes when framing or discovery change in a way an old client can't
handle. New commands don't change it.

## Commands

Numbers 1–99 are **common**: their meaning is the same on every console, though some values in
them (such as `location`) are per console. Numbers from 0x100 on are **console-specific**:
0x100–0x1FF 3DS, 0x200–0x2FF Wii, 0x300–0x3FF Wii U.

| # | Name | Request payload | Answer payload | 3DS | Wii | Wii U |
| --- | --- | --- | --- | :-: | :-: | :-: |
| 1 | info | – | text, see below | ✅ | 🔜 | 🔜 |
| 2 | titles | `u8 location[, u8 product codes]` | 40-byte title records | ✅ | 🔜 | 🔜 |
| 3 | metadata | `u8 location, u64 id` | the title's native metadata | ✅ | 🔜 | 🔜 |
| 4 | read file | archive, path | the file | ✅ | 🔜 | 🔜 |
| 5 | write file | archive, path, `u32 flags`, bytes | – | ✅ | 🔜 | 🔜 |
| 6 | list directory | archive, path | directory entries | ✅ | 🔜 | 🔜 |
| 7 | install (3DS legacy) | `u8 location`, then the CIA | – | ✅ | – | – |
| 8 | delete title | `u8 location, u64 id` | – | ✅ | 🔜 | 🔜 |
| 9 | reboot | – | – | ✅ | 🔜 | 🔜 |
| 10 | report | text | – | ✅ | 🔜 | 🔜 |
| 11 | note | text | – | ✅ | 🔜 | 🔜 |
| 12 | metadata batch | `u8 location, u64 id` per title | per title `u32 size` + metadata | ✅ | 🔜 | 🔜 |
| 13 | progress | `u32 done, u32 total`, text | – | ✅ | 🔜 | 🔜 |
| 14 | restart | – | – | ✅ | 🔜 | 🔜 |
| 15–17 | *reserved* | experiments of vcinject Link, never reuse | | | | |
| 18 | launch | `u8 location, u64 id` | – | ✅ | 🔜 | 🔜 |
| 19 | install stream | `u8 location, u64 size`, then the package | – | 🔜 | 🔜 | – |
| 20 | write stream | archive, path, `u64 size`, then the bytes | – | 🔜 | 🔜 | 🔜 |
| 21 | read stream | archive, path | `u64 size`, then the bytes | 🔜 | 🔜 | 🔜 |
| 0x300 | install from folder | `u8 location`, archive, path of a folder with `title.tmd`, `title.tik`, `title.cert`, `*.app`, `*.h3` | – | – | – | 🔜 |

✅ exists today, 🔜 planned, – doesn't apply.

### 1 · info

A text payload with the keys from [Versions and capabilities](#versions-and-capabilities) and:

| Key | | 3DS | Wii | Wii U |
| --- | --- | :-: | :-: | :-: |
| `name` | the console's user name | ✅ | – | ✅ |
| `id` | a stable ID of the device, hex | device ID | console ID | serial |
| `region` | console region (console's own numbering) | ✅ | ✅ | ✅ |
| `model` | console model (console's own numbering) | ✅ | ✅ | ✅ |
| `system` | system version, e.g. `11.17.0-50E` | ✅ | ✅ | ✅ |
| `battery`, `charging` | battery level 0–5, 0/1 | ✅ | – | – |
| `free.<location>`, `total.<location>` | free and total bytes of a storage location | `sd_free`, `sd_total` (legacy names) | 🔜 | 🔜 |

Console-specific keys may be added, e.g. `luma` (Luma3DS version) and `new3ds` on the 3DS, or the
Aroma version on the Wii U.

### 2 · titles

One 40-byte record per title, the same on every console:

| Offset | Type | |
| --- | --- | --- |
| 0 | `u64` | title ID |
| 8 | `u64` | size in bytes |
| 16 | `u16` | title version |
| 18 | `u8` | location |
| 19 | 5 bytes | padding, zero |
| 24 | `char[16]` | product code (e.g. `CTR-P-AXXE`, `RSBE`, `WUP-P-AXXE`), NUL padded; empty when not asked for |

Reading product codes is slow on the 3DS, so it's optional: the second request byte, 1 by default.

### 3 / 12 · metadata

The title's metadata **in the console's own format**, parsed by the client:

| Console | Metadata |
| --- | --- |
| 3DS | the SMDH (icon, names in every language, ratings, flags) |
| Wii | the channel's `opening.bnr` IMET header (names) |
| Wii U | `meta.xml`, followed by `iconTex.tga` (see the Wii U section) |

Command 12 asks for many titles in one round trip. A title without metadata answers size 0.

### 4 / 5 / 6 / 20 / 21 · files

An *archive* says where a path lives: `u32 archive` followed by an *archive path*
(`u32 type, u32 length, bytes`). A *path* has the same form: `u32 type, u32 length, bytes`.

| Path type | |
| --- | --- |
| 1 | empty |
| 2 | binary (e.g. IDs) |
| 3 | ASCII |
| 4 | UTF-16 (3DS) |
| 5 | UTF-8 |

ASCII and UTF-16 paths end in a NUL character, which `length` includes (the 3DS's FS service
expects it); UTF-8 paths don't.

The archive values are per console, see the console sections. The 3DS uses its FS archive IDs
directly, the Wii and the Wii U small numbers for their mounts.

Write file flag 1 commits save data (3DS); other bits are reserved and zero.

A directory listing is, per entry: `u8 is_dir, 3 × padding, u64 size, u32 name length`, the name
in UTF-8.

### 10 / 11 / 13 · messages on the console's screen

So that the person in front of the console sees what the computer is doing:

- **report**: a status report with `from\t<computer>` and `level\ttitle\tdetail` lines (levels as
  in the client). It replaces the last one.
- **note**: one line for the activity log, e.g. "Reading the HOME Menu".
- **progress**: a bar; `total` 0 hides it.

### 9 / 14 / 18 · power

**reboot** answers first, then reboots. **restart** quits the companion and starts it again (after
an update). **launch** quits and starts a title. In all three cases the connection ends after the
answer.

## Errors

The result is either a code of the console itself (a 3DS `Result`, a negative IOS error on the Wii
as `u32`, a Wii U `MCPError` …) or one of the protocol's own:

| Code | |
| --- | --- |
| `0xE0000001` | bad request (payload too short or malformed) |
| `0xE0000002` | out of memory |
| `0xE0000003` | too large (payload over 8 MiB, or not enough space) |
| `0xE0000004` | unknown command |
| `0xE0000005` | busy (another operation is running) |
| `0xE0000006` | not supported on this console or location |
| `0xE0000007` | cancelled on the console |

After `0xE0000001`, `0xE0000003` on a payload or a broken stream, the companion may close the
connection.

## Consoles

### 3DS: vcinject Link

| Location | |
| --- | --- |
| 0 | NAND |
| 1 | SD card |
| 2 | game card |

| Archive | ID | Archive path | For |
| --- | --- | --- | --- |
| SD card | `0x9` | empty | files, Luma3DS `boot.firm` |
| extdata | `0x6` | `media 1, extdata ID, 0` | the HOME Menu's `SaveData.dat` |
| system save data | `0x8` | `media 0, save ID` | `Launcher.dat` |
| NAND (CTR) | `0x567890AB` | empty | the HOME Menu's save container, read only |
| title content | `0x2345678A` | `ID low, ID high, media, 0`; file `{0, 0, 2, name}` | ExeFS `icon`, `banner`, `logo`, `.code` |
| user save data | `0x567890B2` | `media, ID low, ID high` | save backups |

vcinject Link 13 speaks protocol 0. To speak protocol 1 it needs to: answer `ugbm?` with
`ugbm-link`, add `proto`, `console` and `commands` to info, and implement commands 19–21.

### Wii (planned)

A libogc homebrew app, started from the Homebrew Channel. Wiiload keeps working next to it.

| Location | |
| --- | --- |
| 0 | NAND |

| Archive | Archive path | |
| --- | --- | --- |
| 1 | empty | SD card (`sd:/`) |
| 2 | empty | USB storage (`usb:/`) |
| 3 | empty | NAND file system through IOS (`/title`, `/shared2` …), needs AHB access |

Install stream (19) takes a WAD. Saves are files in `/title/<high>/<low>/data` on the NAND.

### Wii U (planned)

An Aroma app (`.wuhb`) on the HOME Menu, which installs like WUP Installer GX2: the client uploads
a title's files to a folder with [write stream](#4--5--6--20--21--files) and then asks for
[0x300 install from folder](#commands).

| Location | |
| --- | --- |
| 0 | system memory (MLC) |
| 1 | USB storage |

| Archive | Archive path | |
| --- | --- | --- |
| 1 | empty | SD card (`/vol/external01`) |
| 2 | empty | system memory (`/vol/storage_mlc01`) |
| 3 | empty | USB storage (`/vol/storage_usb01`) |

Metadata (3 / 12) is `u32 size` + `meta.xml`, then `u32 size` + `iconTex.tga`.

## Security

There is no authentication: anybody in the same network can talk to a running companion app. It
runs only while it's open on the console, shows every connection and what it does on the console's
screen, and is meant for home networks. Pairing with a code shown on the console is planned for a
later protocol version.
