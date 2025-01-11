use std::sync::Arc;

use hyper::{
    header::{HeaderName, HeaderValue},
    Request,
};

type ExtraHeadersList = Arc<Vec<(HeaderName, HeaderValue)>>;

#[derive(Debug, Clone)]
pub struct ExtraHeadersLayer {
    headers: ExtraHeadersList,
}

impl ExtraHeadersLayer {
    pub fn new(headers: ExtraHeadersList) -> Self {
        ExtraHeadersLayer { headers }
    }
}

impl<S> tower_layer::Layer<S> for ExtraHeadersLayer {
    type Service = ExtraHeaders<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ExtraHeaders {
            inner,
            headers: self.headers.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExtraHeaders<S> {
    inner: S,
    headers: ExtraHeadersList,
}

impl<S, B> tower_service::Service<Request<B>> for ExtraHeaders<S>
where
    S: tower_service::Service<Request<B>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        req.headers_mut().extend(self.headers.iter().cloned());
        self.inner.call(req)
    }
}
