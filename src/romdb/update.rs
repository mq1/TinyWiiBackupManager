// SPDX-FileCopyrightText: 2026 Yanik Fuchs <yanik.fuchs@me.com>
// SPDX-License-Identifier: GPL-3.0-only

//! Newer lists at run time: once a day the app asks libretro-database for the No-Intro DATs that
//! changed since it last asked (by ETag), compacts them as `scripts/update-databases.py` does and
//! keeps a list in the data folder when it is newer than the one in use. Without a network, or
//! when libretro-database doesn't answer, the lists in use stay: the shipped ones, or the ones
//! downloaded earlier.

use super::{List, downloaded_dir, header_version, is_newer, reload};
use crate::util::http::{self, Changed};
use anyhow::{Result, bail};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

const BASE: &str =
    "https://raw.githubusercontent.com/libretro/libretro-database/master/metadat/no-intro/";

/// What the app remembers between checks, in the folder of the downloaded lists
const STATE_FILE: &str = "state.json";

/// Seconds between two checks
const CHECK_EVERY: u64 = 24 * 60 * 60;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct State {
    /// when every list was last asked for, in seconds since 1970
    last_check: u64,
    /// the ETag of each list's DAT as it was last downloaded, by file name
    etags: BTreeMap<String, String>,
}

/// A list that was replaced by a newer one
#[derive(Debug, Clone)]
pub struct Updated {
    /// e.g. "Nintendo - Game Boy Advance"
    pub list: &'static str,
    /// e.g. "2026.11.01"
    pub version: String,
}

/// Asks libretro-database for newer lists, at most once a day, and uses the ones it gets from
/// now on. Returns the lists that got newer. `UGBM_DISABLE_UPDATES=1` turns it off, as it does
/// the check for a new version of the app.
pub async fn check_for_updates(data_dir: &'static Path) -> Result<Vec<Updated>> {
    if std::env::var("UGBM_DISABLE_UPDATES").is_ok_and(|v| v == "1") {
        return Ok(Vec::new());
    }

    smol::unblock(move || {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let updated = update_lists(&downloaded_dir(data_dir), now, http::get_text_if_changed)?;
        if !updated.is_empty() {
            reload();
        }
        Ok(updated)
    })
    .await
}

/// Downloads the lists whose DAT changed into `dir` and returns the ones that are newer than the
/// ones in use; blocking. `fetch` downloads a URL unless it still has the given ETag.
fn update_lists(
    dir: &Path,
    now: u64,
    fetch: impl Fn(&str, Option<&str>) -> Result<Option<Changed>>,
) -> Result<Vec<Updated>> {
    let mut state: State = fs::read_to_string(dir.join(STATE_FILE))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default();
    // a clock set back doesn't stop the checks
    if now
        .checked_sub(state.last_check)
        .is_some_and(|since| since < CHECK_EVERY)
    {
        return Ok(Vec::new());
    }
    fs::create_dir_all(dir)?;

    let mut updated = Vec::new();
    let mut failed = None;
    for list in List::ALL {
        let url = format!("{BASE}{}.dat", list.dat_name().replace(' ', "%20"));
        let etag = state.etags.get(list.file_name()).map(String::as_str);
        let result = fetch(&url, etag).and_then(|changed| match changed {
            Some(changed) => {
                let version = save_if_newer(dir, list, &changed.text)?;
                // remembered also when the list isn't newer: no need to download it again
                if let Some(etag) = changed.etag {
                    state.etags.insert(list.file_name().to_string(), etag);
                }
                Ok(version)
            }
            None => Ok(None),
        });
        match result {
            Ok(Some(version)) => updated.push(Updated {
                list: list.dat_name(),
                version,
            }),
            Ok(None) => {}
            Err(e) => {
                eprintln!("WARNING: game database {}: {e:#}", list.dat_name());
                failed.get_or_insert(e);
            }
        }
    }

    // a list that failed is asked for again at the next start
    if failed.is_none() {
        state.last_check = now;
    }
    fs::write(dir.join(STATE_FILE), serde_json::to_string_pretty(&state)?)?;

    match failed {
        Some(e) if updated.is_empty() => Err(e),
        _ => Ok(updated),
    }
}

/// Compacts a downloaded DAT and keeps it when it is newer than the list in use; returns its
/// version then
fn save_if_newer(dir: &Path, list: List, dat: &str) -> Result<Option<String>> {
    let (version, lines) = compact(dat, list);
    if lines.is_empty() {
        bail!("no dumps in the downloaded list");
    }

    let path = dir.join(list.file_name());
    let downloaded = fs::read_to_string(&path).ok();
    let mut in_use = header_version(list.shipped()).unwrap_or_default();
    if let Some(v) = downloaded.as_deref().and_then(header_version)
        && is_newer(v, in_use)
    {
        in_use = v;
    }
    if !is_newer(&version, in_use) {
        return Ok(None);
    }

    let mut file = tempfile::NamedTempFile::new_in(dir)?;
    file.write_all(to_text(list, &version, &lines).as_bytes())?;
    file.persist(&path)?;
    Ok(Some(version))
}

/// A list's text as `scripts/update-databases.py` writes it
fn to_text(list: List, version: &str, lines: &[String]) -> String {
    let mut text = format!(
        "# {} (No-Intro via libretro-database, {version}); CC BY-SA 4.0\n",
        list.dat_name()
    );
    for line in lines {
        text.push_str(line);
        text.push('\n');
    }
    text
}

/// What `compact()` in `scripts/update-databases.py` does, line for line: the DAT's version and
/// one `CRC32 \t SHA-1 \t serial \t name` line per dump, sorted by name
fn compact(dat: &str, list: List) -> (String, Vec<String>) {
    let mut version = None;
    let mut lines = HashSet::new();
    // the name of the game being read, and whether one is open
    let mut game: Option<Option<&str>> = None;

    for line in dat.lines().map(|l| l.strip_suffix('\r').unwrap_or(l)) {
        if version.is_none()
            && let Some(rest) = line.strip_prefix("\tversion \"")
        {
            version = rest.split_once('"').map(|(v, _)| v);
        }

        if line == "game (" {
            game = Some(None);
        } else if line.starts_with(')') {
            game = None;
        } else if let Some(name) = &mut game {
            if name.is_none()
                && let Some(rest) = line.strip_prefix("\tname \"")
            {
                *name = quoted(rest).map(|(value, _)| value);
            } else if let Some(rest) = line.strip_prefix("\trom ( name \"")
                && let Some(name) = *name
                && let Some(entry) = rom_line(rest, name, list)
            {
                lines.insert(entry);
            }
        }
    }

    let mut lines: Vec<String> = lines.into_iter().collect();
    lines.sort_by_cached_key(|line| {
        let name = line.splitn(4, '\t').nth(3).unwrap_or_default();
        (name.to_lowercase(), line.clone())
    });
    (version.unwrap_or("unknown").to_string(), lines)
}

/// One `rom ( name "…" crc … sha1 … serial "…" )` line of game `name`, after `rom ( name "`
fn rom_line(rest: &str, name: &str, list: List) -> Option<String> {
    let (file, rest) = quoted(rest)?;
    let fields = rest.strip_prefix(' ')?.strip_suffix(" )")?;
    // the NES list has every dump twice: with the iNES header (.nes) and without (.unh); the app
    // strips the header and looks up the headerless one
    if list == List::Nes && !file.ends_with(".unh") {
        return None;
    }

    let fields = fields_of(fields);
    let (crc, sha1) = (fields.get("crc")?, fields.get("sha1")?);
    let serial = fields.get("serial").copied().unwrap_or_default();
    if [name, serial].iter().any(|t| t.contains(['\t', '\n'])) {
        return None;
    }
    Some(format!(
        "{}\t{}\t{serial}\t{name}",
        crc.to_uppercase(),
        sha1.to_lowercase()
    ))
}

/// The text up to the closing quote (backslash escapes kept as they are), and what follows it
fn quoted(text: &str) -> Option<(&str, &str)> {
    let mut escaped = false;
    for (i, c) in text.char_indices() {
        match c {
            _ if escaped => escaped = false,
            '\\' => escaped = true,
            '"' => return Some((&text[..i], &text[i + 1..])),
            _ => {}
        }
    }
    None
}

/// `key value` pairs, a value quoted or up to the next space; the quotes are left out
fn fields_of(text: &str) -> HashMap<&str, &str> {
    let mut fields = HashMap::new();
    let mut rest = text.trim_start();
    while let Some((key, after)) = rest.split_once(' ') {
        let value;
        (value, rest) = match after.strip_prefix('"') {
            Some(after) => match quoted(after) {
                Some(found) => found,
                None => break,
            },
            None => after.split_once(' ').unwrap_or((after, "")),
        };
        fields.insert(key, value);
        rest = rest.trim_start();
    }
    fields
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        library::Platform,
        romdb::{Databases, Match, RomHashes, sha1_from_hex},
    };
    use std::cell::RefCell;

    const NEW_SHA1: &str = "0123456789abcdef0123456789abcdef01234567";

    /// A GBA DAT dated after the shipped list, with a game the shipped list doesn't have
    fn gba_dat(version: &str) -> String {
        format!(
            "clrmamepro (\n\tname \"Nintendo - Game Boy Advance\"\n\tversion \"{version}\"\n)\n\n\
             game (\n\tname \"Zeta Test \\\"Quest\\\" (USA)\"\n\tregion \"USA\"\n\tserial \"ZZZE\"\n\
             \trom ( name \"Zeta Test.gba\" size 4 crc 89abcdef md5 00 sha1 {} serial \"ZZZE\" )\n)\n\
             game (\n\tname \"Alpha Test (Europe)\"\n\
             \trom ( name \"Alpha Test (Europe).gba\" size 4 crc 01234567 sha1 {} )\n)\n",
            NEW_SHA1.to_uppercase(),
            "1".repeat(40),
        )
    }

    #[test]
    fn a_dat_is_compacted_like_the_script_does() {
        let (version, lines) = compact(&gba_dat("2099.01.02"), List::GameBoyAdvance);
        assert_eq!(version, "2099.01.02");
        assert_eq!(
            lines,
            [
                format!("01234567\t{}\t\tAlpha Test (Europe)", "1".repeat(40)),
                format!("89ABCDEF\t{NEW_SHA1}\tZZZE\tZeta Test \\\"Quest\\\" (USA)"),
            ]
        );

        // of a NES game only the dump without the iNES header
        let nes = "game (\n\tname \"N (USA)\"\n\
                   \trom ( name \"N (USA).nes\" size 16 crc AAAAAAAA sha1 aa )\n\
                   \trom ( name \"N (USA).unh\" size 0 crc BBBBBBBB sha1 bb )\n)\n";
        assert_eq!(
            compact(nes, List::Nes),
            (
                "unknown".to_string(),
                vec!["BBBBBBBB\tbb\t\tN (USA)".to_string()]
            )
        );
    }

    /// The Rust version gives the shipped lists from the DATs they were made of. Needs the six
    /// DATs of the shipped version in a folder:
    /// `UGBM_DAT_DIR=… cargo test -- --ignored compacts_the_real_dats`
    #[test]
    #[ignore = "needs the DATs from libretro-database"]
    fn compacts_the_real_dats() {
        let dir = std::env::var("UGBM_DAT_DIR").expect("UGBM_DAT_DIR");
        for list in List::ALL {
            let dat = fs::read_to_string(Path::new(&dir).join(format!("{}.dat", list.dat_name())))
                .unwrap();
            let (version, lines) = compact(&dat, list);
            assert_eq!(to_text(list, &version, &lines), list.shipped(), "{list:?}");
        }
    }

    #[test]
    fn a_newer_list_is_downloaded_and_used() {
        let dir = tempfile::tempdir().unwrap();
        let asked = RefCell::new(Vec::new());
        let fetch = |url: &str, etag: Option<&str>| {
            asked
                .borrow_mut()
                .push((url.to_string(), etag.map(str::to_string)));
            Ok(url.contains("Advance").then(|| Changed {
                text: gba_dat("2099.01.02"),
                etag: Some("\"v1\"".into()),
            }))
        };

        let updated = update_lists(dir.path(), 1_000_000, fetch).unwrap();
        assert_eq!(updated.len(), 1);
        assert_eq!(updated[0].list, "Nintendo - Game Boy Advance");
        assert_eq!(updated[0].version, "2099.01.02");
        assert_eq!(asked.borrow().len(), List::ALL.len());
        assert!(asked.borrow().iter().any(|(url, _)| {
            url.ends_with("/no-intro/Nintendo%20-%20Super%20Nintendo%20Entertainment%20System.dat")
        }));

        // the downloaded list is used in place of the shipped one, the others stay
        let databases = Databases::load(Some(&dir.path().to_path_buf()));
        let rom = RomHashes {
            crc32: 0x89AB_CDEF,
            sha1: sha1_from_hex(NEW_SHA1).unwrap(),
            serial: None,
        };
        let found = databases.identify(Platform::GameBoyAdvance, &rom).unwrap();
        assert_eq!(found.how, Match::Verified);
        assert_eq!(found.release, "Zeta Test \\\"Quest\\\" (USA)");
        assert!(databases.get(List::Nes).dumps.len() > 1000);

        // within a day nothing is asked
        asked.borrow_mut().clear();
        update_lists(dir.path(), 1_000_000 + 3600, fetch).unwrap();
        assert!(asked.borrow().is_empty());

        // a day later the ETag goes along; an unchanged list isn't downloaded again
        let updated = update_lists(dir.path(), 1_000_000 + CHECK_EVERY, |url, etag| {
            if url.contains("Advance") {
                assert_eq!(etag, Some("\"v1\""));
            }
            Ok(None)
        })
        .unwrap();
        assert!(updated.is_empty());
    }

    #[test]
    fn an_older_list_or_a_failed_download_changes_nothing() {
        let dir = tempfile::tempdir().unwrap();

        // older than the shipped list: not kept
        let updated = update_lists(dir.path(), 1_000_000, |url, _| {
            Ok(url.contains("Advance").then(|| Changed {
                text: gba_dat("2020.01.01"),
                etag: None,
            }))
        })
        .unwrap();
        assert!(updated.is_empty());
        assert!(!dir.path().join("gba.tsv").exists());

        // libretro-database doesn't answer: an error, and asked again at the next start
        let failed = update_lists(dir.path(), 2_000_000, |_, _| bail!("offline"));
        assert!(failed.is_err());
        let asked = std::cell::Cell::new(0);
        update_lists(dir.path(), 2_000_001, |_, _| {
            asked.set(asked.get() + 1);
            Ok(None)
        })
        .unwrap();
        assert_eq!(asked.get(), List::ALL.len());

        // a broken download isn't kept either
        let dir = tempfile::tempdir().unwrap();
        let result = update_lists(dir.path(), 1_000_000, |url, _| {
            Ok(url.contains("Advance").then(|| Changed {
                text: "<html>rate limited</html>".into(),
                etag: Some("x".into()),
            }))
        });
        assert!(result.is_err());
        assert!(!dir.path().join("gba.tsv").exists());

        // a downloaded list older than the shipped one (after an update of the app) isn't used
        fs::write(
            dir.path().join("gba.tsv"),
            to_text(
                List::GameBoyAdvance,
                "2020.01.01",
                &[format!("89ABCDEF\t{NEW_SHA1}\t\tOld")],
            ),
        )
        .unwrap();
        let databases = Databases::load(Some(&dir.path().to_path_buf()));
        assert_eq!(
            databases.get(List::GameBoyAdvance).version,
            header_version(List::GameBoyAdvance.shipped()).unwrap()
        );
    }
}
