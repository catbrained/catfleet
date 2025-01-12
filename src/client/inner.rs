use std::{error::Error, future::Future, pin::Pin, sync::Arc, task::Poll};

use anyhow::{anyhow, Context};
use hyper::{
    body::{Body, Incoming},
    Request, Response, Uri,
};
use hyper_util::rt::{TokioExecutor, TokioIo};
use tokio::net::TcpStream;
use tokio_rustls::{
    rustls::{pki_types::ServerName, ClientConfig, RootCertStore},
    TlsConnector,
};
use tracing::{event, instrument, span, Instrument, Level, Span};

#[derive(Debug)]
pub struct InnerClient<B> {
    sender: hyper::client::conn::http2::SendRequest<B>,
    connector: Arc<Connector>,
}

impl<B> InnerClient<B>
where
    B: Body + Send + Unpin + Clone + 'static,
    B::Data: Send,
    B::Error: Into<Box<dyn Error + Send + Sync>>,
{
    #[instrument(level = Level::DEBUG)]
    pub async fn new(base_url: Uri) -> Result<Self, anyhow::Error> {
        let connector = Connector::new(base_url);
        let sender = connector.connect().await?;

        Ok(Self {
            sender,
            connector: Arc::new(connector),
        })
    }

    #[instrument(level = Level::DEBUG, skip(self, req), fields(req.url =% req.uri()))]
    pub async fn send_request(
        mut self,
        mut req: Request<B>,
    ) -> Result<Response<Incoming>, anyhow::Error> {
        *req.version_mut() = hyper::Version::HTTP_2;

        event!(Level::TRACE, "Sending request");

        // XXX: Is this really the correct way of doing this?
        loop {
            match self.sender.ready().await {
                Ok(_) => {}
                Err(e) if e.is_closed() => {
                    event!(Level::TRACE, "Connection closed. Reconnecting...");
                    self.sender = self.connector.connect().await?;
                    continue;
                }
                Err(e) => return Err(anyhow!(e)),
            }

            match self.sender.send_request(req.clone()).await {
                Ok(res) => return Ok(res),
                Err(e) if e.is_canceled() => {
                    event!(Level::WARN, "Request was cancelled. Retrying...");
                    continue;
                }
                Err(e) if e.is_closed() => {
                    event!(Level::TRACE, "Connection closed. Reconnecting...");
                    self.sender = self.connector.connect().await?;
                    continue;
                }
                Err(e) => return Err(anyhow!(e)),
            }
        }
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
            connector: self.connector.clone(),
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
    B: Body + Send + Unpin + Clone + 'static,
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

struct Connector {
    inner: TlsConnector,
    url: Uri,
}

impl Connector {
    #[instrument(level = Level::TRACE)]
    fn new(url: Uri) -> Self {
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

        Self {
            inner: connector,
            url,
        }
    }

    #[instrument(level = Level::TRACE, skip(self))]
    async fn connect<B>(&self) -> Result<hyper::client::conn::http2::SendRequest<B>, anyhow::Error>
    where
        B: Body + Send + Unpin + Clone + 'static,
        B::Data: Send,
        B::Error: Into<Box<dyn Error + Send + Sync>>,
    {
        let host = self.url.host().context("URI has no host")?;
        let port = self.url.port_u16().unwrap_or(443);
        let address = format!("{}:{}", host, port);

        // 1. Establish TCP connection.
        let tcp_stream = TcpStream::connect(address).await?;
        // 2. Establish TLS connection.
        let stream = self
            .inner
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

        Ok(sender)
    }
}

impl std::fmt::Debug for Connector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Connector {{ url: {}}}", self.url)
    }
}
