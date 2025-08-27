use crate::jobs::{JobContext, update_status_with_bytes};
use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::sync::mpsc::Receiver;
use ureq::Body;
use ureq::http::Response;

/// Reads from an HTTP response body in chunks, writing to the provided output writer,
/// while updating the job context with download progress.
#[allow(clippy::too_many_arguments)]
pub fn download_with_progress<W: Write>(
    response: Response<Body>,
    out: &mut W,
    status_message: String,
    item_name: Option<String>,
    current_item: u32,
    total_items: u32,
    context: &JobContext,
    cancel: &Receiver<()>,
) -> Result<()> {
    let (_, body) = response.into_parts();
    let content_length = body.content_length();
    if let Some(len) = content_length {
        update_status_with_bytes(
            context,
            status_message.clone(),
            current_item,
            total_items,
            0,
            len,
            item_name.clone(),
            cancel,
        )?;
    }
    // Read the response body in chunks, updating progress as we go.
    let mut reader = body.into_reader();
    let mut written = 0u64;
    loop {
        let mut chunk = [0u8; 8192];
        let n = reader
            .read(&mut chunk)
            .with_context(|| "Failed to read data from response")?;
        if n == 0 {
            // EOF
            break;
        }
        out.write_all(&chunk[..n])?;
        written += n as u64;
        if let Some(len) = content_length {
            update_status_with_bytes(
                context,
                status_message.clone(),
                current_item,
                total_items,
                written,
                len,
                item_name.clone(),
                cancel,
            )?;
        }
    }
    out.flush()?;
    Ok(())
}

/// Checks if the given error is an HTTP 404 Not Found error.
pub fn is_404_error(e: &anyhow::Error) -> bool {
    e.downcast_ref::<ureq::Error>()
        .is_some_and(|e| matches!(e, ureq::Error::StatusCode(404)))
}
