use RequestMethod::*;

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

impl From<RequestMethod> for String {
    fn from(method: RequestMethod) -> Self {
        format!("{:?}", method).to_uppercase()
    }
}

impl From<std::string::String> for RequestMethod {
    fn from(s: String) -> Self {
        match s.as_str() {
            "GET" => Get,
            "HEAD" => Head,
            "POST" => Post,
            "PUT" => Put,
            "DELETE" => Delete,
            "CONNECT" => Connect,
            "OPTIONS" => Options,
            "TRACE" => Trace,
            "PATCH" => Patch,
            &_ => Get,
        }
    }
}

pub struct HttpRequest {
    pub method: RequestMethod,
    pub route: String,
    pub version: String,
    pub headers: Option<String>,
    pub body: Option<String>,
}

impl Default for HttpRequest {
    fn default() -> Self {
        HttpRequest {
            method: RequestMethod::Get,
            route: "/".to_owned(),
            version: HTTP_VERSION.to_owned(),
            headers: None,
            body: None,
        }
    }
}

impl HttpRequest {
    pub fn format(&self) -> String {
        let method = format!("{:?}", self.method).to_uppercase();
        let mut formatted = format!("{} {} {}\r\n", method, self.route, self.version);
        if let Some(headers) = &self.headers {
            formatted.push_str(&headers);
        }
        formatted.push_str("\r\n");
        if let Some(body) = &self.body {
            formatted.push_str(&body);
        }
        formatted
    }
}

pub struct HttpResponse<'a> {
    pub statuscode: u16,
    pub phrase: String,
    pub headers: Option<&'a str>,
    pub body: Option<&'a str>,
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
    pub fn format(&self) -> String {
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
