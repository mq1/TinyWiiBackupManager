// SPDX-FileCopyrightText: 2026 Yanik Fuchs <yanik.fuchs@me.com>
// SPDX-License-Identifier: GPL-3.0-only

//! The consoles running a companion app: found in the network and remembered

use crate::link::{
    discovery::{self, Found},
    protocol::{Console, Info, PORT, n3ds},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use smol::fs;
use std::{
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    time::Duration,
};

const FILENAME: &str = "consoles.json";
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(2);

/// The vcinject Link version this app expects; older ones get an update offer
pub const VCINJECT_LINK_VERSION: u32 = 13;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleEntry {
    pub addr: SocketAddr,
    pub info: Info,
    #[serde(skip)]
    pub online: bool,
    #[serde(skip)]
    pub label: String,
}

impl ConsoleEntry {
    fn new(addr: SocketAddr, info: Info, online: bool) -> Self {
        let mut entry = Self {
            addr,
            info,
            online,
            label: String::new(),
        };
        entry.update_label();
        entry
    }

    /// Name and model, e.g. "Petter (3DS XL)": names alone are often alike
    fn update_label(&mut self) {
        let model = match self.info.console() {
            Some(Console::N3ds) => n3ds::model_name(self.info.get("model").unwrap_or_default()),
            Some(Console::Wii) => "Wii",
            Some(Console::WiiU) => "Wii U",
            None => "Console",
        };

        self.label = match self.info.name() {
            Some(name) => format!("{name} ({model})"),
            None => format!("{model} at {}", self.addr.ip()),
        };
    }

    fn is(&self, found: &Found) -> bool {
        match (self.info.device_id(), found.info.device_id()) {
            (Some(a), Some(b)) => a == b,
            _ => self.addr.ip() == found.addr.ip(),
        }
    }

    pub fn outdated(&self) -> bool {
        self.info.app() == "vcinject-link" && self.info.version() < VCINJECT_LINK_VERSION
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConsolesState {
    pub searching: bool,
    pub searched_once: bool,
    pub entries: Vec<ConsoleEntry>,
}

impl ConsolesState {
    pub fn known_addresses(&self) -> Vec<IpAddr> {
        self.entries.iter().map(|entry| entry.addr.ip()).collect()
    }

    pub fn set_known(&mut self, entries: Vec<ConsoleEntry>) {
        self.entries = entries
            .into_iter()
            .map(|entry| ConsoleEntry::new(entry.addr, entry.info, false))
            .collect();
    }

    /// Takes the result of a search: answering consoles are online, the others offline
    pub fn merge(&mut self, found: Vec<Found>) {
        for entry in &mut self.entries {
            entry.online = false;
        }

        for found in found {
            match self.entries.iter_mut().find(|entry| entry.is(&found)) {
                Some(entry) => *entry = ConsoleEntry::new(found.addr, found.info, true),
                None => self
                    .entries
                    .push(ConsoleEntry::new(found.addr, found.info, true)),
            }
        }

        self.entries
            .sort_by(|a, b| b.online.cmp(&a.online).then(a.label.cmp(&b.label)));
    }
}

fn path(data_dir: &Path) -> PathBuf {
    data_dir.join(FILENAME)
}

pub async fn load(data_dir: &Path) -> Vec<ConsoleEntry> {
    let Ok(contents) = fs::read_to_string(path(data_dir)).await else {
        return Vec::new();
    };

    serde_json::from_str(&contents).unwrap_or_default()
}

pub async fn save(data_dir: &Path, entries: Vec<ConsoleEntry>) -> Result<()> {
    let contents = serde_json::to_string_pretty(&entries)?;
    fs::write(path(data_dir), contents).await?;
    Ok(())
}

/// Asks the network; when nobody answers, tries every address of the /24
pub async fn search(known: Vec<IpAddr>) -> Result<Vec<Found>, String> {
    smol::unblock(move || {
        let found = discovery::discover(DISCOVERY_TIMEOUT, &known, PORT)
            .map_err(|e| format!("Searching for consoles failed: {e}"))?;

        Ok(if found.is_empty() {
            discovery::scan(PORT)
        } else {
            found
        })
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(ip: &str, keys: &str) -> Found {
        Found {
            addr: SocketAddr::new(ip.parse().unwrap(), PORT),
            info: Info::parse(keys),
        }
    }

    #[test]
    fn merge_recognises_a_console_under_a_new_address() {
        let mut state = ConsolesState::default();
        state.merge(vec![
            found("10.0.0.2", "id=AB\nname=Petter\nmodel=1"),
            found("10.0.0.3", "console=wii"),
        ]);
        assert_eq!(state.entries[0].label, "Petter (3DS XL)");

        state.merge(vec![found("10.0.0.9", "id=AB\nname=Petter\nmodel=1")]);

        assert_eq!(state.entries.len(), 2);
        assert!(state.entries[0].online);
        assert_eq!(state.entries[0].addr.ip().to_string(), "10.0.0.9");
        assert!(!state.entries[1].online);
        assert_eq!(state.entries[1].label, "Wii at 10.0.0.3");
    }

    #[test]
    fn outdated_vcinject_link() {
        let old = ConsoleEntry::new(
            found("10.0.0.2", "").addr,
            Info::parse("app=vcinject-link\nversion=12"),
            true,
        );
        assert!(old.outdated());
    }
}
