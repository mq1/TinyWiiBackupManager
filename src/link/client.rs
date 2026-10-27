// SPDX-FileCopyrightText: 2026 Yanik Fuchs <yanik.fuchs@me.com>
// SPDX-License-Identifier: GPL-3.0-only

//! A blocking connection to a companion app; run it with smol::unblock

use crate::link::protocol::{
    AnswerError, Archive, DirEntry, Info, Path, RemoteError, Title, command, parse_metadata_batch,
    read_answer, write_request,
};
use anyhow::{Context, Result, anyhow};
use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const IO_TIMEOUT: Duration = Duration::from_secs(15);
// finishing a large install takes a while
const INSTALL_TIMEOUT: Duration = Duration::from_secs(120);
const STREAM_CHUNK: usize = 1024 * 1024;

pub struct Link {
    stream: TcpStream,
}

impl Link {
    pub fn connect(addr: SocketAddr) -> Result<Self> {
        let stream = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT).with_context(|| {
            format!("Could not connect to {addr}. Is the companion app open on the console?")
        })?;
        stream.set_read_timeout(Some(IO_TIMEOUT))?;
        stream.set_write_timeout(Some(IO_TIMEOUT))?;
        stream.set_nodelay(true)?;

        Ok(Self { stream })
    }

    fn request(&mut self, command: u32, payload: &[u8], what: &str) -> Result<Vec<u8>> {
        write_request(&mut self.stream, command, payload)
            .map_err(AnswerError::Io)
            .and_then(|()| read_answer(&mut self.stream))
            .map_err(|e| answer_error(e, what))
    }

    pub fn info(&mut self) -> Result<Info> {
        let text = self.request(command::INFO, &[], "Reading the console's info")?;
        Ok(Info::parse(&String::from_utf8_lossy(&text)))
    }

    pub fn titles(&mut self, location: u8, product_codes: bool) -> Result<Vec<Title>> {
        let data = self.request(
            command::TITLES,
            &[location, u8::from(product_codes)],
            "Reading the installed titles",
        )?;
        Ok(Title::parse_list(&data))
    }

    pub fn metadata(&mut self, title: &Title) -> Result<Vec<u8>> {
        self.request(
            command::METADATA,
            &title.id_payload(),
            "Reading a title's icon",
        )
    }

    /// One entry per title, None where a title has no metadata
    pub fn metadata_batch(&mut self, titles: &[Title]) -> Result<Vec<Option<Vec<u8>>>> {
        let payload: Vec<u8> = titles.iter().flat_map(Title::id_payload).collect();
        let data = self.request(command::METADATA_BATCH, &payload, "Reading the icons")?;
        Ok(parse_metadata_batch(&data))
    }

    pub fn read_file(&mut self, archive: &Archive, path: &Path) -> Result<Vec<u8>> {
        self.request(command::READ_FILE, &archive.encode(path), "Reading a file")
    }

    pub fn write_file(
        &mut self,
        archive: &Archive,
        path: &Path,
        flags: u32,
        data: &[u8],
    ) -> Result<()> {
        let mut payload = archive.encode(path);
        payload.extend_from_slice(&flags.to_le_bytes());
        payload.extend_from_slice(data);
        self.request(command::WRITE_FILE, &payload, "Writing a file")?;
        Ok(())
    }

    pub fn list_dir(&mut self, archive: &Archive, path: &Path) -> Result<Vec<DirEntry>> {
        let data = self.request(command::LIST_DIR, &archive.encode(path), "Listing a folder")?;
        Ok(DirEntry::parse_list(&data))
    }

    /// Installs a CIA with vcinject Link's install command, which predates streams
    pub fn install_legacy(
        &mut self,
        location: u8,
        package: impl Read,
        size: u64,
        on_progress: impl FnMut(u64),
    ) -> Result<()> {
        let length = u32::try_from(size + 1).context("The package is larger than 4 GiB")?;
        let mut header = command::INSTALL_LEGACY.to_le_bytes().to_vec();
        header.extend_from_slice(&length.to_le_bytes());
        header.push(location);

        self.send_stream(&header, package, size, on_progress, "Installing")
    }

    /// Installs a package (a CIA on the 3DS, a WAD on the Wii) as a stream
    pub fn install_stream(
        &mut self,
        location: u8,
        package: impl Read,
        size: u64,
        on_progress: impl FnMut(u64),
    ) -> Result<()> {
        let mut params = vec![location];
        params.extend_from_slice(&size.to_le_bytes());

        let mut header = command::INSTALL_STREAM.to_le_bytes().to_vec();
        header.extend_from_slice(&(params.len() as u32).to_le_bytes());
        header.extend_from_slice(&params);

        self.send_stream(&header, package, size, on_progress, "Installing")
    }

    fn send_stream(
        &mut self,
        header: &[u8],
        mut data: impl Read,
        size: u64,
        mut on_progress: impl FnMut(u64),
        what: &str,
    ) -> Result<()> {
        let result = (|| {
            self.stream.write_all(header)?;

            let mut buf = vec![0; STREAM_CHUNK];
            let mut sent = 0;
            while sent < size {
                let n = data.read(&mut buf[..STREAM_CHUNK.min((size - sent) as usize)])?;
                if n == 0 {
                    return Err(io::Error::from(io::ErrorKind::UnexpectedEof).into());
                }
                self.stream.write_all(&buf[..n])?;
                sent += n as u64;
                on_progress(sent);
            }

            self.stream.set_read_timeout(Some(INSTALL_TIMEOUT))?;
            let answer = read_answer(&mut self.stream);
            self.stream.set_read_timeout(Some(IO_TIMEOUT))?;
            answer
        })();

        result.map(drop).map_err(|e| answer_error(e, what))
    }

    pub fn delete(&mut self, title: &Title) -> Result<()> {
        self.request(command::DELETE, &title.id_payload(), "Deleting a title")?;
        Ok(())
    }

    pub fn launch(&mut self, title: &Title) -> Result<()> {
        self.request(command::LAUNCH, &title.id_payload(), "Starting a title")?;
        Ok(())
    }

    pub fn note(&mut self, text: &str) -> Result<()> {
        self.request(command::NOTE, text.as_bytes(), "Showing a note")?;
        Ok(())
    }

    pub fn report(&mut self, text: &str) -> Result<()> {
        self.request(command::REPORT, text.as_bytes(), "Showing the report")?;
        Ok(())
    }

    /// total 0 hides the bar
    pub fn progress(&mut self, done: u32, total: u32, text: &str) -> Result<()> {
        let mut payload = done.to_le_bytes().to_vec();
        payload.extend_from_slice(&total.to_le_bytes());
        payload.extend_from_slice(text.as_bytes());
        self.request(command::PROGRESS, &payload, "Showing progress")?;
        Ok(())
    }

    pub fn restart_app(mut self) -> Result<()> {
        self.request(command::RESTART, &[], "Restarting the companion app")?;
        Ok(())
    }

    pub fn reboot(mut self) -> Result<()> {
        self.request(command::REBOOT, &[], "Rebooting the console")?;
        Ok(())
    }
}

fn answer_error(e: impl Into<AnswerError>, what: &str) -> anyhow::Error {
    match e.into() {
        AnswerError::Io(e) => anyhow!("{what}: the connection to the console failed ({e})"),
        AnswerError::Remote(RemoteError(code)) => anyhow!("{what}: {}", RemoteError(code)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::link::protocol::{error, n3ds};
    use std::{net::TcpListener, thread};

    /// Serves one connection like vcinject Link 13 would, with a few fixed answers
    fn fake_companion() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            loop {
                let mut header = [0; 8];
                if stream.read_exact(&mut header).is_err() {
                    return;
                }
                let command = u32::from_le_bytes(header[..4].try_into().unwrap());
                let size = u32::from_le_bytes(header[4..].try_into().unwrap());
                let mut payload = vec![0; size as usize];
                stream.read_exact(&mut payload).unwrap();

                let (result, answer): (u32, Vec<u8>) = match command {
                    command::INFO => (0, b"app=vcinject-link\nversion=13\n".to_vec()),
                    command::TITLES => {
                        let mut record = vec![0; 40];
                        record[..8].copy_from_slice(&7_u64.to_le_bytes());
                        record[18] = payload[0];
                        (0, record)
                    }
                    // answers with the byte count and the last byte of the CIA
                    command::INSTALL_LEGACY => {
                        (0, vec![payload.len() as u8, payload[size as usize - 1]])
                    }
                    command::NOTE => (0, Vec::new()),
                    _ => (error::UNKNOWN_COMMAND, Vec::new()),
                };

                let mut frame = result.to_le_bytes().to_vec();
                frame.extend_from_slice(&(answer.len() as u32).to_le_bytes());
                frame.extend_from_slice(&answer);
                stream.write_all(&frame).unwrap();
            }
        });

        addr
    }

    #[test]
    fn talks_to_vcinject_link() {
        let mut link = Link::connect(fake_companion()).unwrap();

        assert_eq!(link.info().unwrap().version(), 13);

        let titles = link.titles(n3ds::LOCATION_SD, true).unwrap();
        assert_eq!((titles[0].id, titles[0].location), (7, n3ds::LOCATION_SD));

        link.note("hello").unwrap();
        let cia = [1, 2, 3];
        let mut progress = Vec::new();
        link.install_legacy(n3ds::LOCATION_SD, &cia[..], 3, |sent| progress.push(sent))
            .unwrap();
        assert_eq!(progress, vec![3]);

        let error = link.progress(1, 2, "x").unwrap_err().to_string();
        assert!(error.contains("too old"), "{error}");

        // the connection is still usable after an error
        link.note("still there").unwrap();
    }
}
