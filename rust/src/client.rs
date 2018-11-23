use std::ops::Deref;

type HyperClient =
    hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>, hyper::Body>;

#[derive(Into, From, Constructor)]
pub struct Client(HyperClient);

impl Deref for Client {
    type Target = HyperClient;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
