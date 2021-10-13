const HTTP_VERSION: &str = "HTTP/1.1";

#[derive(Debug)]
pub enum RequestMethod {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch,
}

pub struct HttpRequest<'a> {
    method: RequestMethod,
    route: &'a str,
    version: &'a str,
    headers: Option<&'a str>,
    body: Option<&'a str>,
}

impl Default for HttpRequest<'_> {
    fn default() -> Self {
        HttpRequest {
            method: RequestMethod::Get,
            route: "/",
            version: HTTP_VERSION,
            headers: None,
            body: None,
        }
    }
}

impl HttpRequest<'_> {
    fn format(&self) -> String {
        let method = format!("{:?}", self.method).to_uppercase();
        let mut formatted = format!("{} {} {}\r\n", method, self.route, self.version);
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
        let mut formatted = format!("{} {} {}\r\n", HTTP_VERSION, self.statuscode, self.phrase);
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
    use super::RequestMethod::*;

    // Request formatting tests
    #[test]
    fn blank_request() {
        let request = HttpRequest {
            ..Default::default()
        };
        assert_eq!(request.format(), "GET / HTTP/1.1\r\n\r\n");
    }
    #[test]
    fn route_request() {
        let request = HttpRequest {
            route: "/index",
            ..Default::default()
        };
        assert_eq!(request.format(), "GET /index HTTP/1.1\r\n\r\n");
    }
    #[test]
    fn method_request() {
        let request = HttpRequest {
            method: Post,
            ..Default::default()
        };
        assert_eq!(request.format(), "POST / HTTP/1.1\r\n\r\n");
    }
    #[test]
    fn both_request() {
        let request = HttpRequest {
            method: Put,
            route: "/home",
            ..Default::default()
        };
        assert_eq!(request.format(), "PUT /home HTTP/1.1\r\n\r\n");
    }
    #[test]
    fn headers_request() {
        let request = HttpRequest {
            headers: Some("Connection: keep-alive"),
            ..Default::default()
        };
        assert_eq!(request.format(), "GET / HTTP/1.1\r\nConnection: keep-alive\r\n")
    }
    #[test]
    fn body_request() {
        let request = HttpRequest {
            body: Some("<p>Hi</p>"),
            ..Default::default()
        };
        assert_eq!(request.format(), "GET / HTTP/1.1\r\n\r\n<p>Hi</p>")
    }
    #[test]
    fn headers_body_request() {
        let request = HttpRequest {
            headers: Some("Connection: keep-alive"),
            body: Some("<p>Hi</p>"),
            ..Default::default()
        };
        assert_eq!(request.format(), "GET / HTTP/1.1\r\nConnection: keep-alive\r\n<p>Hi</p>")
    }

    // Response formatting tests
    #[test]
    fn blank_response() {
        let response = HttpResponse {
            ..Default::default()
        };
        assert_eq!(response.format(), "HTTP/1.1 200 OK\r\n\r\n");
    }
    #[test]
    fn headers_response() {
        let response = HttpResponse {
            headers: Some("Content-Length: 500"),
            ..Default::default()
        };
        assert_eq!(
            response.format(),
            "HTTP/1.1 200 OK\r\nContent-Length: 500\r\n"
        );
    }
    #[test]
    fn body_response() {
        let response = HttpResponse {
            body: Some("<h1>Hello from Rust!</h1>"),
            ..Default::default()
        };
        assert_eq!(
            response.format(),
            "HTTP/1.1 200 OK\r\n\r\n<h1>Hello from Rust!</h1>"
        );
    }
    #[test]
    fn both_response() {
        let response = HttpResponse {
            headers: Some("Content-Length: 500"),
            body: Some("<h1>Hello from Rust!</h1>"),
            ..Default::default()
        };
        assert_eq!(
            response.format(),
            "HTTP/1.1 200 OK\r\nContent-Length: 500\r\n<h1>Hello from Rust!</h1>"
        )
    }
}