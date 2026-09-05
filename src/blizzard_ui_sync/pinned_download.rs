use cascette_crypto::ContentKey;
use cascette_formats::{CascFormat, blte::BlteFile};
use reqwest::{StatusCode, blocking::Client, header};
use serde::Deserialize;
use std::io::Read;
use std::time::{Duration, SystemTime};

#[derive(Debug, Deserialize)]
pub(super) struct ContentEntry {
    pub path: String,
    pub content_key: String,
    pub encoding_key: String,
    pub content_size: usize,
    pub archive: String,
    pub offset: u64,
    pub size: usize,
}

pub(super) fn decode_entry(entry: &ContentEntry, encoded: &[u8]) -> crate::Result<Vec<u8>> {
    if encoded.len() != entry.size || encoded.len() < 8 || &encoded[..4] != b"BLTE" {
        return Err(error(entry, "invalid BLTE length or signature"));
    }
    let header_size = u32::from_be_bytes(encoded[4..8].try_into().unwrap()) as usize;
    let hash_bytes = if header_size == 0 {
        encoded
    } else {
        encoded
            .get(..header_size)
            .ok_or_else(|| error(entry, "truncated BLTE header"))?
    };
    if ContentKey::from_data(hash_bytes).to_hex() != entry.encoding_key {
        return Err(error(entry, "encoding key mismatch"));
    }
    let blte = BlteFile::parse(encoded).map_err(|cause| error(entry, cause))?;
    let content = blte.decompress().map_err(|cause| error(entry, cause))?;
    if !content_matches(entry, &content) {
        return Err(error(entry, "decoded content size or key mismatch"));
    }
    Ok(content)
}

pub(super) fn content_matches(entry: &ContentEntry, content: &[u8]) -> bool {
    content.len() == entry.content_size
        && ContentKey::from_data(content).to_hex() == entry.content_key
}

fn error(entry: &ContentEntry, cause: impl std::fmt::Display) -> crate::Error {
    crate::Error::Other(format!("pinned Blizzard CDN {}: {cause}", entry.path))
}

pub(super) fn download_entry(client: &Client, entry: &ContentEntry) -> crate::Result<Vec<u8>> {
    let url = format!(
        "https://us.cdn.blizzard.com/tpr/wow/data/{}/{}/{}",
        &entry.archive[..2],
        &entry.archive[2..4],
        entry.archive,
    );
    download_from_url(client, entry, &url)
}

fn download_from_url(client: &Client, entry: &ContentEntry, url: &str) -> crate::Result<Vec<u8>> {
    let end = entry
        .offset
        .checked_add(entry.size as u64)
        .and_then(|end| end.checked_sub(1))
        .ok_or_else(|| error(entry, "invalid archive range"))?;
    let range = format!("bytes={}-{}", entry.offset, end);
    for attempt in 0..3 {
        let response = client.get(url).header(header::RANGE, &range).send();
        let response = match response {
            Ok(response) => response,
            Err(cause) if attempt < 2 && (cause.is_timeout() || cause.is_connect()) => {
                eprintln!("Retrying {}: {cause}", entry.path);
                std::thread::sleep(retry_delay(None, attempt)?);
                continue;
            }
            Err(cause) => return Err(error(entry, cause)),
        };
        let status = response.status();
        if status == StatusCode::PARTIAL_CONTENT {
            return read_range(entry, response, end);
        }
        wait_before_response_retry(entry, &response, attempt)?;
    }
    unreachable!("each final HTTP attempt returns")
}

fn wait_before_response_retry(
    entry: &ContentEntry,
    response: &reqwest::blocking::Response,
    attempt: u32,
) -> crate::Result<()> {
    let status = response.status();
    let transient = status == StatusCode::REQUEST_TIMEOUT
        || status == StatusCode::TOO_MANY_REQUESTS
        || status.is_server_error();
    if !transient || attempt == 2 {
        return Err(error(entry, format!("HTTP {status}; expected 206")));
    }
    let retry_after = response
        .headers()
        .get(header::RETRY_AFTER)
        .map(|value| value.to_str())
        .transpose()
        .map_err(|cause| error(entry, cause))?;
    let delay = retry_delay(retry_after, attempt)?;
    eprintln!("Retrying {} after HTTP {status} in {delay:?}", entry.path);
    std::thread::sleep(delay);
    Ok(())
}

fn retry_delay(retry_after: Option<&str>, attempt: u32) -> crate::Result<Duration> {
    let now = SystemTime::now();
    let jitter = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_millis() as u64
        % 250;
    let backoff = Duration::from_millis((1u64 << attempt) * 250 + jitter);
    let Some(value) = retry_after else {
        return Ok(backoff);
    };
    let requested = if let Ok(seconds) = value.parse::<u64>() {
        Duration::from_secs(seconds)
    } else {
        httpdate::parse_http_date(value)
            .map_err(|cause| crate::Error::Other(format!("invalid CDN Retry-After: {cause}")))?
            .duration_since(now)
            .unwrap_or_default()
    };
    Ok(backoff.max(requested))
}

fn read_range(
    entry: &ContentEntry,
    response: reqwest::blocking::Response,
    end: u64,
) -> crate::Result<Vec<u8>> {
    let content_range = response
        .headers()
        .get(header::CONTENT_RANGE)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| error(entry, "missing Content-Range"))?;
    let expected = format!("bytes {}-{end}/", entry.offset);
    let total = content_range
        .strip_prefix(&expected)
        .and_then(|total| total.parse::<u64>().ok());
    if !total.is_some_and(|total| total > end) {
        return Err(error(
            entry,
            format!("wrong Content-Range: {content_range}"),
        ));
    }
    let mut encoded = Vec::with_capacity(entry.size);
    response
        .take(entry.size as u64 + 1)
        .read_to_end(&mut encoded)
        .map_err(|cause| error(entry, cause))?;
    decode_entry(entry, &encoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cascette_crypto::ContentKey;

    fn fixture() -> (ContentEntry, Vec<u8>) {
        let content = b"PTR source\r\n";
        let mut encoded = b"BLTE\0\0\0\0N".to_vec();
        encoded.extend_from_slice(content);
        let entry = ContentEntry {
            path: "Blizzard_Test/Test.lua".into(),
            content_key: ContentKey::from_data(content).to_hex(),
            encoding_key: ContentKey::from_data(&encoded).to_hex(),
            content_size: content.len(),
            archive: "0017a402f556fbece46c38dc431a2c9b".into(),
            offset: 42,
            size: encoded.len(),
        };
        (entry, encoded)
    }

    #[test]
    fn pinned_cdn_decodes_exact_source_bytes() {
        let (entry, encoded) = fixture();
        assert_eq!(decode_entry(&entry, &encoded).unwrap(), b"PTR source\r\n");
    }

    fn serve(responses: Vec<Vec<u8>>) -> (String, std::thread::JoinHandle<()>) {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/archive", listener.local_addr().unwrap());
        let handle = std::thread::spawn(move || {
            for response in responses {
                let (mut socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut request = Vec::new();
                let mut byte = [0];
                while !request.ends_with(b"\r\n\r\n") {
                    socket.read_exact(&mut byte).unwrap();
                    request.push(byte[0]);
                }
                assert!(
                    String::from_utf8(request)
                        .unwrap()
                        .to_ascii_lowercase()
                        .contains("range: bytes=42-")
                );
                socket.write_all(&response).unwrap();
            }
        });
        (url, handle)
    }

    fn response(status: &str, range: &str, body: &[u8]) -> Vec<u8> {
        let mut response = format!(
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\n{range}Connection: close\r\n\r\n",
            body.len()
        )
        .into_bytes();
        response.extend_from_slice(body);
        response
    }

    #[test]
    fn pinned_cdn_fetches_exact_range_after_transient_response() {
        let (entry, encoded) = fixture();
        let end = entry.offset + entry.size as u64 - 1;
        let range = format!("Content-Range: bytes 42-{end}/1000\r\n");
        let (url, server) = serve(vec![
            response("503 Unavailable", "Retry-After: 0\r\n", b""),
            response("206 Partial Content", &range, &encoded),
        ]);
        let client = Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap();
        assert_eq!(
            download_from_url(&client, &entry, &url).unwrap(),
            b"PTR source\r\n"
        );
        server.join().unwrap();
    }

    #[test]
    fn pinned_cdn_rejects_full_archive_and_wrong_ranges() {
        let (entry, encoded) = fixture();
        for (status, range) in [
            ("200 OK", ""),
            ("206 Partial Content", "Content-Range: bytes 0-20/1000\r\n"),
        ] {
            let (url, server) = serve(vec![response(status, range, &encoded)]);
            let client = Client::builder()
                .timeout(Duration::from_secs(3))
                .build()
                .unwrap();
            assert!(download_from_url(&client, &entry, &url).is_err());
            server.join().unwrap();
        }
    }

    #[test]
    fn pinned_cdn_rejects_corrupt_encoding_and_content() {
        let (mut entry, mut encoded) = fixture();
        *encoded.last_mut().unwrap() ^= 1;
        assert!(decode_entry(&entry, &encoded).is_err());
        entry.encoding_key = ContentKey::from_data(&encoded).to_hex();
        assert!(decode_entry(&entry, &encoded).is_err());
    }
}
