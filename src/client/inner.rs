use std::{error::Error, future::Future, pin::Pin, sync::Arc, task::Poll};

use anyhow::Context;
use hyper::{
    body::{Body, Incoming},
    header, Request, Response, Uri,
};
use hyper_util::rt::{TokioExecutor, TokioIo};
use tokio::net::TcpStream;
use tokio_rustls::{
    rustls::{pki_types::ServerName, ClientConfig, RootCertStore},
    TlsConnector,
};
use tracing::{event, field, instrument, span, Instrument, Level, Span};

#[derive(Debug)]
pub struct InnerClient<B> {
    sender: hyper::client::conn::http2::SendRequest<B>,
    base_url: Arc<Uri>,
}

impl<B> InnerClient<B>
where
    B: Body + Send + Unpin + 'static,
    B::Data: Send,
    B::Error: Into<Box<dyn Error + Send + Sync>>,
{
    #[instrument(level = Level::DEBUG)]
    pub async fn new(base_url: &str) -> Result<Self, anyhow::Error> {
        let url = base_url.parse::<Uri>()?;
        let host = url.host().context("URI has no host")?;
        let port = url.port_u16().unwrap_or(443);
        let address = format!("{}:{}", host, port);

        // Configure root certs.
        let mut root_cert_store = RootCertStore::empty();
        root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        // Configure TLS client.
        let mut config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();
        // We want to talk HTTP/2.
        config.alpn_protocols = vec![b"h2".to_vec()];

        // Set up network IO.
        let connector = TlsConnector::from(Arc::new(config));
        // 1. Establish TCP connection.
        let tcp_stream = TcpStream::connect(address).await?;
        // 2. Establish TLS connection.
        let stream = connector
            .connect(
                ServerName::try_from(host.to_string()).expect("domain should be valid"),
                tcp_stream,
            )
            .await?;
        event!(
            Level::TRACE,
            "TLS connection established; performing http handshake"
        );
        // 3. Wrap stream in hyper/tokio compatibility layer.
        let io = TokioIo::new(stream);

        // 4. Perform HTTP handshake.
        let (mut sender, conn) = hyper::client::conn::http2::Builder::new(TokioExecutor::new())
            .handshake(io)
            .instrument({
                let span = span!(parent: None, Level::TRACE, "http_conn");
                span.follows_from(Span::current());
                span
            })
            .await?;
        event!(
            Level::TRACE,
            "http2 handshake complete; spawning background dispatcher"
        );

        tokio::task::spawn(
            async move {
                if let Err(err) = conn.await {
                    event!(Level::ERROR, "Client connection error: {:?}", err);
                }
            }
            .instrument({
                let span = span!(parent: None, Level::TRACE, "http_dispatcher");
                span.follows_from(Span::current());
                span
            }),
        );

        // Wait for connection to become ready.
        sender.ready().await?;

        Ok(Self {
            sender,
            base_url: Arc::new(url),
        })
    }

    #[instrument(level = Level::DEBUG, skip(self, req), fields(req.url))]
    pub async fn send_request(
        mut self,
        mut req: Request<B>,
    ) -> Result<Response<Incoming>, anyhow::Error> {
        let pq = match req.uri().path_and_query() {
            Some(pq) if pq != "/" => &format!(
                "{}{}",
                self.base_url
                    .path()
                    .strip_suffix('/')
                    .expect("base url should end with slash `/`"),
                pq
            ),
            _ => self.base_url.path(),
        };
        let url = Uri::builder()
            .scheme("https")
            .authority(
                self.base_url
                    .authority()
                    .expect("base url should have authority")
                    .as_str(),
            )
            .path_and_query(pq)
            .build()?;

        Span::current().record("req.url", field::display(url.clone()));
        *req.uri_mut() = url;

        let headers = req.headers_mut();
        if let Some(ua) = headers.insert(
            header::USER_AGENT,
            "catfleet/0.1.0"
                .try_into()
                .expect("user agent should be valid"),
        ) {
            event!(
                Level::WARN,
                ?ua,
                "USER_AGENT header should only be set in one place"
            );
        }
        *req.version_mut() = hyper::Version::HTTP_2;

        event!(Level::TRACE, "Sending request");

        self.sender
            .send_request(req)
            .await
            .map_err(anyhow::Error::new)
    }

    #[instrument(level = Level::TRACE, skip(self, req))]
    pub fn request(&self, req: Request<B>) -> ResponseFuture {
        ResponseFuture::new(self.clone().send_request(req))
    }
}

impl<B> Clone for InnerClient<B> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            base_url: self.base_url.clone(),
        }
    }
}

pub struct ResponseFuture {
    inner: Pin<Box<dyn Future<Output = Result<Response<Incoming>, anyhow::Error>> + Send>>,
}

impl ResponseFuture {
    fn new<F>(value: F) -> Self
    where
        F: Future<Output = Result<Response<Incoming>, anyhow::Error>> + Send + 'static,
    {
        Self {
            inner: Box::pin(value),
        }
    }
}

impl Future for ResponseFuture {
    type Output = Result<Response<Incoming>, anyhow::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        self.inner.as_mut().poll(cx)
    }
}

impl<B> tower_service::Service<Request<B>> for InnerClient<B>
where
    B: Body + Send + Unpin + 'static,
    B::Data: Send,
    B::Error: Into<Box<dyn Error + Send + Sync>>,
{
    type Response = Response<Incoming>;
    type Error = anyhow::Error;
    type Future = ResponseFuture;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        self.request(req)
    }
}
