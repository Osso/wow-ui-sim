//! Advertised CDN transport roots; content identity remains pinned independently.
use reqwest::{Url, blocking::Client};
use std::collections::HashSet;
use std::io::Read;

/// Blizzard serves the same BPSV record over HTTPS and plain HTTP on port 1119.
/// HTTPS goes first so discovery works through HTTPS-only egress proxies.
fn discovery_urls(product: &str) -> [String; 2] {
    [
        format!("https://us.version.battle.net/v2/products/{product}/cdns"),
        format!("http://us.patch.battle.net:1119/{product}/cdns"),
    ]
}

pub(super) fn fetch_roots(client: &Client, product: &str) -> crate::Result<Vec<String>> {
    fetch_roots_from(client, &discovery_urls(product))
}

fn fetch_roots_from(client: &Client, urls: &[String]) -> crate::Result<Vec<String>> {
    let mut failures = Vec::new();
    for url in urls {
        match fetch_record(client, url).and_then(|record| parse_roots(&record)) {
            Ok(roots) => return Ok(roots),
            Err(cause) => {
                eprintln!("CDN discovery via {url} failed: {cause}");
                failures.push(format!("{url}: {cause}"));
            }
        }
    }
    Err(error(failures.join("; ")))
}

fn fetch_record(client: &Client, url: &str) -> crate::Result<String> {
    let response = super::pinned_download::request_with_retry(client, url, None)?;
    const MAX_RECORD_SIZE: u64 = 64 * 1024;
    let mut record = String::new();
    response
        .take(MAX_RECORD_SIZE + 1)
        .read_to_string(&mut record)
        .map_err(|cause| error(format!("read {url}: {cause}")))?;
    if record.len() as u64 > MAX_RECORD_SIZE {
        return Err(error("CDN record exceeds 64 KiB"));
    }
    Ok(record)
}

fn error(cause: impl std::fmt::Display) -> crate::Error {
    crate::Error::Other(format!("Blizzard CDN discovery: {cause}"))
}

fn parse_roots(record: &str) -> crate::Result<Vec<String>> {
    let document = cascette_formats::bpsv::parse(record).map_err(error)?;
    let row_roots = document
        .rows()
        .iter()
        .map(|row| parse_row_roots(row, document.schema()))
        .collect::<crate::Result<Vec<_>>>()?;
    let mut seen = HashSet::new();
    let mut roots: Vec<_> = row_roots
        .into_iter()
        .flatten()
        .filter(|root| seen.insert(root.clone()))
        .collect();
    if roots.is_empty() {
        return Err(error("CDN record advertises no endpoints"));
    }
    roots.sort_by_key(|root| !root.starts_with("https://"));
    Ok(roots)
}

fn parse_row_roots(
    row: &cascette_formats::bpsv::BpsvRow,
    schema: &cascette_formats::bpsv::BpsvSchema,
) -> crate::Result<Vec<String>> {
    let field = |name| {
        row.get_raw_by_name(name, schema)
            .ok_or_else(|| error(format!("missing {name} field")))
    };
    let path = field("Path")?;
    let mut endpoints = field("Servers")?
        .split_whitespace()
        .map(parse_endpoint)
        .collect::<crate::Result<Vec<_>>>()?;
    for host in field("Hosts")?.split_whitespace() {
        let advertised = endpoints.iter().any(|url| url.host_str() == Some(host));
        if !advertised {
            endpoints.push(parse_endpoint(&format!("https://{host}"))?);
        }
    }
    Ok(endpoints
        .into_iter()
        .map(|mut endpoint| {
            let prefix = endpoint.path().trim_end_matches('/');
            endpoint.set_path(&format!("{prefix}/{}/", path.trim_matches('/')));
            endpoint.set_query(None);
            endpoint.set_fragment(None);
            endpoint.to_string()
        })
        .collect())
}

fn parse_endpoint(raw: &str) -> crate::Result<Url> {
    let url = Url::parse(raw).map_err(error)?;
    let valid_scheme = matches!(url.scheme(), "http" | "https");
    let has_credentials = !url.username().is_empty() || url.password().is_some();
    if !valid_scheme || url.host_str().is_none() || has_credentials {
        return Err(error(format!("invalid advertised endpoint {raw}")));
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advertised_roots_include_all_regions_and_https_aliases() {
        let record = concat!(
            "Name!STRING:0|Path!STRING:0|Hosts!STRING:0|Servers!STRING:0|ConfigPath!STRING:0\n",
            "## seqn = 4019523\n",
            "us|tpr/wow|level3.blizzard.com us.cdn.blizzard.com|http://level3.blizzard.com/?maxhosts=8 https://level3.ssl.blizzard.com/?fallback=1 https://us.cdn.blizzard.com/?fallback=1|tpr/configs/data\n",
            "kr|tpr/wow|kr.cdn.blizzard.com|https://kr.cdn.blizzard.com/?maxhosts=4|tpr/configs/data\n",
            "cn|tpr/wow|blzdist-wow.necdn.leihuo.netease.com||tpr/configs/data\n",
        );
        let roots = parse_roots(record).unwrap();
        assert_eq!(
            roots,
            [
                "https://level3.ssl.blizzard.com/tpr/wow/",
                "https://us.cdn.blizzard.com/tpr/wow/",
                "https://kr.cdn.blizzard.com/tpr/wow/",
                "https://blzdist-wow.necdn.leihuo.netease.com/tpr/wow/",
                "http://level3.blizzard.com/tpr/wow/",
            ]
        );
    }

    #[test]
    fn advertised_roots_reject_missing_or_non_http_endpoints() {
        let header =
            "Name!STRING:0|Path!STRING:0|Hosts!STRING:0|Servers!STRING:0|ConfigPath!STRING:0\n";
        assert!(parse_roots(header).is_err());
        assert!(
            parse_roots(&format!(
                "{header}us|tpr/wow||file:///tmp/test|tpr/configs/data\n"
            ))
            .is_err()
        );
        assert!(parse_roots("Name!STRING:0\nus\n").is_err());
    }
    #[test]
    fn discovery_prefers_https_and_keeps_port_1119_fallback() {
        let [first, second] = discovery_urls("wowf");
        assert_eq!(first, "https://us.version.battle.net/v2/products/wowf/cdns");
        assert_eq!(second, "http://us.patch.battle.net:1119/wowf/cdns");
    }

    #[test]
    fn discovery_falls_back_when_an_earlier_source_fails() {
        use std::io::{Read, Write};
        let closed = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let unreachable = format!("http://{}/cdns", closed.local_addr().unwrap());
        drop(closed);
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let reachable = format!("http://{}/cdns", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut byte = [0];
            while !request.ends_with(b"\r\n\r\n") {
                socket.read_exact(&mut byte).unwrap();
                request.push(byte[0]);
            }
            let body = "Name!STRING:0|Path!STRING:0|Hosts!STRING:0|Servers!STRING:0|ConfigPath!STRING:0\nus|tpr/wow|cdn.example||tpr/configs/data\n";
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            socket.write_all(head.as_bytes()).unwrap();
            socket.write_all(body.as_bytes()).unwrap();
        });
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build()
            .unwrap();
        assert_eq!(
            fetch_roots_from(&client, &[unreachable, reachable]).unwrap(),
            ["https://cdn.example/tpr/wow/"]
        );
        server.join().unwrap();
    }

    fn serve_record_once(body: &'static str) -> (String, std::thread::JoinHandle<()>) {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/cdns", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut byte = [0];
            while !request.ends_with(b"\r\n\r\n") {
                socket.read_exact(&mut byte).unwrap();
                request.push(byte[0]);
            }
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            socket.write_all(head.as_bytes()).unwrap();
            socket.write_all(body.as_bytes()).unwrap();
        });
        (url, server)
    }

    #[test]
    fn discovery_error_names_every_failed_source() {
        let empty = "Name!STRING:0|Path!STRING:0|Hosts!STRING:0|Servers!STRING:0|ConfigPath!STRING:0\nus|tpr/wow|||tpr/configs/data\n";
        let (first, first_server) = serve_record_once(empty);
        let (second, second_server) = serve_record_once(empty);
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build()
            .unwrap();
        let message = fetch_roots_from(&client, &[first.clone(), second.clone()])
            .unwrap_err()
            .to_string();
        assert!(message.contains(&format!("{first}: ")), "{message}");
        assert!(message.contains(&format!("{second}: ")), "{message}");
        first_server.join().unwrap();
        second_server.join().unwrap();
    }

    #[test]
    fn advertised_roots_preserve_server_prefix_scheme_and_deduplicate() {
        let record = concat!(
            "Name!STRING:0|Path!STRING:0|Hosts!STRING:0|Servers!STRING:0|ConfigPath!STRING:0\n",
            "us|tpr/wow|cdn.example|http://cdn.example/base/?maxhosts=8 https://secure.example/?fallback=1|tpr/configs/data\n",
            "eu|tpr/wow|cdn.example|http://cdn.example/base/|tpr/configs/data\n",
        );
        assert_eq!(
            parse_roots(record).unwrap(),
            [
                "https://secure.example/tpr/wow/",
                "http://cdn.example/base/tpr/wow/"
            ]
        );
    }
}
