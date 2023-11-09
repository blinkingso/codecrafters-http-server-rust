use anyhow::Result;
use std::collections::HashMap;

pub fn parse_request(request: &[u8]) -> Result<Option<HttpRequest<'_>>> {
    let mut method: Option<&str> = None;
    let mut path: Option<&str> = None;
    let mut version: Option<&str> = None;
    let mut body: Option<&[u8]> = None;
    let mut headers: HashMap<&str, &str> = HashMap::new();
    let mut body_len = 0;

    let mut remaining: &[u8] = request;
    loop {
        let mut i = 0;
        let b = &request[i];
        while i < remaining.len() && *b != b'\r' {
            i += 1;
        }
        if i + 1 > remaining.len() {
            // Incomplete http request
            return Ok(None);
        }
        assert_eq!(remaining[i + 1], b'\n');
        i += 1;
        // 0../r/n
        let buf = &request[..i - 1];
        let line = std::str::from_utf8(buf)?;
        if line.is_empty() {
            // /r/n/r/n
            remaining = &remaining[i + 1..];
            break;
        } else {
            if method.is_none() {
                let (m, r) = line.split_once(' ').unwrap();
                let (p, v) = r.split_once(' ').unwrap();
                method = Some(m);
                path = Some(p);
                version = Some(v);
                assert_eq!(v, "HTTP/1.1");
            } else {
                // http headers
                let (key, value) = line.split_once(":").unwrap();
                headers.insert(key, value.trim());
            }
            remaining = &remaining[i + 1..];
        }

        let content_length = headers
            .get("Content-Length")
            .map(|s| s.parse::<usize>().expect("invalid content-length header"))
            .unwrap_or(0);

        if content_length > 0 && content_length <= remaining.len() {
            body_len = content_length;
            body = Some(&remaining[..content_length]);
        } else if content_length > 0 {
            return Ok(None);
        }
    }
    Ok(Some(HttpRequest {
        method: method.unwrap(),
        path: path.unwrap(),
        headers,
        body,
        query: Default::default(),
        body_len,
    }))
}

pub struct HttpRequest<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub query: HashMap<&'a str, &'a str>,
    pub headers: HashMap<&'a str, &'a str>,
    pub body: Option<&'a [u8]>,
    pub body_len: usize,
}
