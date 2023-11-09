use anyhow::Result;
use std::collections::HashMap;

pub struct HttpRequest<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub query: HashMap<&'a str, &'a str>,
    pub headers: HashMap<&'a str, &'a str>,
    pub body: Option<&'a [u8]>,
    pub body_len: usize,
}

impl<'request> HttpRequest<'request> {
    pub fn parse_request(
        request: &'request [u8],
    ) -> Result<(&'request [u8], Option<HttpRequest<'request>>)> {
        let mut method: Option<&str> = None;
        let mut path: Option<&str> = None;
        let mut headers: HashMap<&str, &str> = HashMap::new();

        let mut remaining: &[u8] = request;
        loop {
            let mut i = 0;
            while i < remaining.len() && remaining[i] != b'\r' {
                i += 1;
            }
            if i + 1 >= remaining.len() {
                // Incomplete http request
                return Ok((request, None));
            }
            assert_eq!(remaining[i + 1], b'\n');
            i += 1;
            // 0../r/n
            let line = std::str::from_utf8(&remaining[..i - 1])?;
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
                    assert_eq!(v, "HTTP/1.1");
                } else {
                    // http headers
                    let (key, value) = line.split_once(": ").unwrap();
                    headers.insert(key, value);
                }
                remaining = &remaining[i + 1..];
            }
        }
        let content_length = headers
            .get("Content-Length")
            .map(|s| s.parse::<usize>().expect("invalid content-length header"))
            .unwrap_or(0);

        Ok((
            remaining,
            Some(HttpRequest {
                method: method.unwrap(),
                path: path.unwrap(),
                headers,
                body: None,
                query: Default::default(),
                body_len: content_length,
            }),
        ))
    }

    pub fn set_body(&mut self, body: &'request [u8]) {
        assert_eq!(body.len(), self.body_len);
        self.body = Some(body);
    }
}
#[cfg(test)]
mod tests {
    use super::HttpRequest;

    #[test]
    fn parse_no_headers_no_body() {
        let input = b"GET /index.html HTTP/1.1\r\n\r\n";
        let (rest, request) = HttpRequest::parse_request(input).unwrap();

        assert!(rest.is_empty());

        let request = request.unwrap();
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/index.html");
        assert!(request.headers.is_empty());
        assert_eq!(request.body_len, 0);
        assert!(request.body.is_none());
    }

    #[test]
    fn parse_headers_no_body() {
        let input =
            b"GET /index.html HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/7.64.1\r\n\r\n";
        let (rest, request) = HttpRequest::parse_request(input).unwrap();

        assert!(rest.is_empty());

        let request = request.unwrap();
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/index.html");
        assert_eq!(request.headers.len(), 2);
        assert_eq!(request.headers.get("Host"), Some(&"localhost:4221"));
        assert_eq!(request.headers.get("User-Agent"), Some(&"curl/7.64.1"));
        assert_eq!(request.body_len, 0);
        assert!(request.body.is_none());
    }

    #[test]
    fn parse_headers_body() {
        let input =
            b"GET /index.html HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/7.64.1\r\nContent-Length: 3\r\n\r\nfoo";
        let (rest, request) = HttpRequest::parse_request(input).unwrap();

        assert_eq!(rest, b"foo");

        let mut request = request.unwrap();
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/index.html");
        assert_eq!(request.headers.len(), 3);
        assert_eq!(request.headers.get("Host"), Some(&"localhost:4221"));
        assert_eq!(request.headers.get("User-Agent"), Some(&"curl/7.64.1"));
        assert_eq!(request.headers.get("Content-Length"), Some(&"3"));
        assert_eq!(request.body_len, 3);
        assert!(request.body.is_none());

        request.set_body(rest);
        assert_eq!(request.body.unwrap(), b"foo");
    }

    #[test]
    fn parse_incomplete_header() {
        {
            let input = b"GET /index.html HTTP/1.1\r\nHost: localhos";
            let (rest, request) = HttpRequest::parse_request(input).unwrap();
            assert_eq!(rest, input);
            assert!(request.is_none());
        }

        {
            let input = b"GET /index.html HTTP/1.1\r\nHost: localhost:4221\r\n";
            let (rest, request) = HttpRequest::parse_request(input).unwrap();
            assert_eq!(rest, input);
            assert!(request.is_none());
        }
    }
}
