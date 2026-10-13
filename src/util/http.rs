// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result, bail};
use serde::de::DeserializeOwned;
use smol::{
    fs::{self, File},
    io::AsyncSeekExt,
    net::TcpStream,
};
use std::{
    ffi::OsStr,
    io::{Seek, SeekFrom},
    path::Path,
};
use tempfile::tempfile;
use wiiload::WIILOAD_PORT;
use zip::ZipArchive;

use crate::util;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
// minreq otherwise waits indefinitely. These deadlines cover the whole request,
// including redirects and streamed response bodies, rather than each read.
const REQUEST_TIMEOUT_SECONDS: u64 = 60;
const ARCHIVE_TIMEOUT_SECONDS: u64 = 300;

fn request(request: minreq::Request, timeout_seconds: u64) -> minreq::Request {
    request
        .with_header("User-Agent", USER_AGENT)
        .with_timeout(timeout_seconds)
}

/// Downloads a file, creating the parent directory if needed
/// Skips if the file already exists
pub async fn download_file(uri: &str, dest: &Path) -> Result<()> {
    download_file_with_timeout(uri, dest, REQUEST_TIMEOUT_SECONDS).await
}

async fn download_file_with_timeout(uri: &str, dest: &Path, timeout_seconds: u64) -> Result<()> {
    let dest_filename = dest
        .file_name()
        .and_then(OsStr::to_str)
        .context("invalid filename")?;

    if fs::symlink_metadata(&dest).await.is_ok() {
        println!("INFO: {} already exists, skipping", dest.display());
        return Ok(());
    }

    let dest_parent = dest.parent().context("invalid path")?;
    fs::create_dir_all(dest_parent).await?;

    smol::unblock({
        let uri = uri.to_string();
        let dest = dest.to_path_buf();
        let dest_filename = dest_filename.to_string();
        let dest_parent = dest_parent.to_path_buf();

        move || {
            let mut out = tempfile::Builder::new()
                .prefix(&dest_filename)
                .suffix(".part")
                .rand_bytes(0)
                .tempfile_in(dest_parent)?;

            let mut resp = request(minreq::get(&uri), timeout_seconds).send_lazy()?;
            if resp.status_code >= 400 {
                bail!("error downloading {}: {}", uri, resp.status_code);
            }

            std::io::copy(&mut resp, &mut out)?;

            out.persist(&dest)?;

            Ok(())
        }
    })
    .await
}

pub async fn download_file_with_fallback(uri: &str, dest: &Path, fallback: &str) -> Result<()> {
    if download_file(uri, dest).await.is_err() {
        download_file(fallback, dest).await
    } else {
        Ok(())
    }
}

pub async fn download_and_extract_zip(uri: &str, dest: &Path) -> Result<()> {
    smol::unblock({
        let uri = uri.to_string();
        let dest = dest.to_path_buf();

        move || {
            let mut tmp = tempfile()?;

            let mut resp = request(minreq::get(&uri), ARCHIVE_TIMEOUT_SECONDS).send_lazy()?;
            if resp.status_code >= 400 {
                bail!("error downloading {}: {}", uri, resp.status_code);
            }

            std::io::copy(&mut resp, &mut tmp)?;

            tmp.rewind()?;
            let mut archive = ZipArchive::new(&mut tmp)?;
            std::fs::create_dir_all(&dest)?;
            archive.extract(dest)?;

            Ok(())
        }
    })
    .await
}

pub async fn download_and_send_via_wiiload(uri: &str, wii_ip: &str) -> Result<Vec<String>> {
    let filename = uri
        .split('/')
        .next_back()
        .and_then(|s| s.strip_suffix(".zip"))
        .context("invalid filename")?;

    let (file, excluded_files) = smol::unblock({
        let uri = uri.to_string();

        move || {
            let mut og_app = tempfile()?;

            let mut resp = request(minreq::get(&uri), ARCHIVE_TIMEOUT_SECONDS).send_lazy()?;
            if resp.status_code >= 400 {
                bail!("error downloading {}: {}", uri, resp.status_code);
            }

            std::io::copy(&mut resp, &mut og_app)?;

            util::wiiload::rebuild_zip(og_app)
        }
    })
    .await?;

    // make it async
    let mut file = File::from(file);
    file.seek(SeekFrom::Start(0)).await?;

    let size = file.metadata().await?.len() as usize;
    let mut conn = TcpStream::connect((wii_ip, WIILOAD_PORT)).await?;
    wiiload::send_async(&mut conn, filename, &mut file, size).await?;

    Ok(excluded_files)
}

pub async fn post_then_download_file(uri: &str, data: String, dest: &Path) -> Result<()> {
    let dest_filename = dest
        .file_name()
        .and_then(OsStr::to_str)
        .context("invalid filename")?;

    if fs::symlink_metadata(&dest).await.is_ok() {
        println!("INFO: {} already exists, skipping", dest.display());
        return Ok(());
    }

    let dest_parent = dest.parent().context("invalid path")?;
    fs::create_dir_all(dest_parent).await?;

    smol::unblock({
        let uri = uri.to_string();
        let dest = dest.to_path_buf();
        let dest_filename = dest_filename.to_string();
        let dest_parent = dest_parent.to_path_buf();

        move || {
            let mut out = tempfile::Builder::new()
                .prefix(&dest_filename)
                .suffix(".part")
                .rand_bytes(0)
                .tempfile_in(dest_parent)?;

            let mut resp = request(minreq::post(&uri), REQUEST_TIMEOUT_SECONDS)
                .with_body(data)
                .send_lazy()?;
            if resp.status_code >= 400 {
                bail!("error downloading {}: {}", uri, resp.status_code);
            }

            std::io::copy(&mut resp, &mut out)?;

            out.persist(&dest)?;

            Ok(())
        }
    })
    .await
}

pub async fn get_json<T: DeserializeOwned + Send + 'static>(uri: &'static str) -> Result<T> {
    smol::unblock(move || {
        let resp = request(minreq::get(uri), REQUEST_TIMEOUT_SECONDS)
            .send()?
            .json()?;

        Ok(resp)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::mpsc,
        thread,
        time::{Duration, Instant},
    };

    fn stalled_download(send_partial_body: bool) {
        let destination = tempfile::tempdir().unwrap();
        let target = destination.path().join("download.bin");
        let existing = destination.path().join("existing.bin");
        let original = b"existing complete download";
        std::fs::write(&existing, original).unwrap();

        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let url = format!("http://{}/download", listener.local_addr().unwrap());
        listener.set_nonblocking(true).unwrap();
        let (stop, stopped) = mpsc::channel();
        let server = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(5);
            let mut socket = loop {
                match listener.accept() {
                    Ok((socket, _)) => break socket,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "client did not connect");
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("accept failed: {error}"),
                }
            };
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            socket
                .set_write_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut headers = Vec::new();
            let mut buffer = [0; 1024];
            while !headers.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                let count = socket.read(&mut buffer).unwrap();
                assert_ne!(count, 0, "client closed before sending headers");
                headers.extend_from_slice(&buffer[..count]);
                assert!(headers.len() < 8192, "unexpectedly large test request");
            }
            if send_partial_body {
                socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 32\r\nConnection: close\r\n\r\npartial").unwrap();
                socket.flush().unwrap();
            }
            // A channel lets the test close this socket immediately after the
            // client returns, including if the assertions below would fail.
            let _ = stopped.recv_timeout(Duration::from_secs(5));
        });

        let started = Instant::now();
        let result = smol::block_on(download_file_with_timeout(&url, &target, 1));
        let elapsed = started.elapsed();
        let _ = stop.send(());
        server.join().unwrap();

        let error = result.expect_err("stalled request must time out");
        assert!(
            error.chain().any(|error| error
                .downcast_ref::<std::io::Error>()
                .is_some_and(|error| error.kind() == std::io::ErrorKind::TimedOut)),
            "unexpected error: {error:#}"
        );
        assert!(
            elapsed < Duration::from_secs(4),
            "download did not return on its deadline: {elapsed:?}"
        );
        assert!(!target.exists(), "incomplete download was published");
        assert_eq!(std::fs::read(&existing).unwrap(), original);
        assert_eq!(
            std::fs::read_dir(destination.path()).unwrap().count(),
            1,
            "temporary download survived the timeout"
        );

        // Existing files still skip the network, even with the same stalled URL.
        smol::block_on(download_file_with_timeout(&url, &existing, 1)).unwrap();
        assert_eq!(std::fs::read(existing).unwrap(), original);
    }

    #[test]
    fn stalled_headers_time_out_without_publishing_or_leaving_temporary_files() {
        stalled_download(false);
    }

    #[test]
    fn stalled_body_times_out_without_publishing_or_leaving_temporary_files() {
        stalled_download(true);
    }
}
