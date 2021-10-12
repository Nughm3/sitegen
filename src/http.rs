const HTTP_VERSION: &str = "HTTP/1.1";

pub struct HttpResponse<'a> {
    statuscode: u16,
    phrase: String,
    headers: Option<&'a str>,
    body: Option<&'a str>,
}

impl Default for HttpResponse<'_> {
    fn default() -> Self {
        HttpResponse {
            statuscode: 200,
            phrase: "OK".to_owned(),
            headers: None,
            body: None,
        }
    }
}

impl HttpResponse<'_> {
    fn format(&self) -> String {
        let mut formatted = format!("{} {} {}", HTTP_VERSION, self.statuscode, self.phrase);
        formatted.push_str("\r\n");
        if let Some(headers) = self.headers {
            formatted.push_str(headers);
        }
        formatted.push_str("\r\n");
        if let Some(body) = self.body {
            formatted.push_str(body);
        }
        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_headers_nor_body() {
        let method = HttpResponse {
            ..Default::default()
        };
        assert_eq!(method.format(), "HTTP/1.1 200 OK\r\n\r\n");
    }
    #[test]
    fn headers_no_body() {
        let method = HttpResponse {
            headers: Some("Content-Length = 500"),
            ..Default::default()
        };
        assert_eq!(
            method.format(),
            "HTTP/1.1 200 OK\r\nContent-Length = 500\r\n"
        );
    }
    #[test]
    fn body_no_headers() {
        let method = HttpResponse {
            body: Some("<h1>Hello from Rust!</h1>"),
            ..Default::default()
        };
        assert_eq!(
            method.format(),
            "HTTP/1.1 200 OK\r\n\r\n<h1>Hello from Rust!</h1>"
        );
    }
}
