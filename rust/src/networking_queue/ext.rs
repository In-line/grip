
pub trait RequestExt {
    fn extend_headers(self, headers: hyper::HeaderMap) -> Self;
}

impl<T> RequestExt for hyper::Request<T> {
    fn extend_headers(mut self, headers: hyper::HeaderMap) -> Self {
        self.headers_mut().extend(headers);

        self
    }
}
