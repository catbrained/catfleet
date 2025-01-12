use hyper::{Request, Uri};

#[derive(Debug, Clone)]
pub struct BaseUrlLayer {
    base_url: Uri,
}

impl BaseUrlLayer {
    pub fn new(base_url: Uri) -> Self {
        Self { base_url }
    }
}

impl<S> tower_layer::Layer<S> for BaseUrlLayer {
    type Service = BaseUrl<S>;

    fn layer(&self, inner: S) -> Self::Service {
        BaseUrl {
            base_url: self.base_url.clone(),
            inner,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BaseUrl<S> {
    base_url: Uri,
    inner: S,
}

impl<S, B> tower_service::Service<Request<B>> for BaseUrl<S>
where
    S: tower_service::Service<Request<B>>,
{
    type Error = S::Error;
    type Future = S::Future;
    type Response = S::Response;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let (mut parts, body) = req.into_parts();
        parts.uri = overwrite_base_url(&self.base_url, parts.uri);
        self.inner.call(Request::from_parts(parts, body))
    }
}

fn overwrite_base_url(base_url: &Uri, input_url: Uri) -> Uri {
    let input_path_and_query = input_url.path_and_query();
    let mut url_builder = Uri::builder();

    match input_url.scheme() {
        Some(scheme) => url_builder = url_builder.scheme(scheme.as_str()),
        None => {
            if let Some(scheme) = base_url.scheme() {
                url_builder = url_builder.scheme(scheme.as_str());
            }
        }
    }

    match input_url.authority() {
        Some(authority) => url_builder = url_builder.authority(authority.as_str()),
        None => {
            if let Some(authority) = base_url.authority() {
                url_builder = url_builder.authority(authority.as_str());
            }
        }
    }

    if let Some(path_and_query) = base_url.path_and_query() {
        url_builder = if let Some(input_path_and_query) = input_path_and_query {
            let base_path = path_and_query.path().trim_end_matches('/');
            if !input_path_and_query.as_str().starts_with(base_path) {
                url_builder.path_and_query(format!("{base_path}{input_path_and_query}"))
            } else {
                url_builder.path_and_query(input_path_and_query.as_str())
            }
        } else {
            url_builder.path_and_query(path_and_query.as_str())
        };
    } else if let Some(input_path_and_query) = input_path_and_query {
        url_builder = url_builder.path_and_query(input_path_and_query.as_str());
    }

    url_builder
        .build()
        .expect("Joining valid Uris should result in a valid Uri")
}

#[cfg(test)]
mod tests {
    use hyper::Uri;

    use super::*;

    #[test]
    fn joining_host_with_relative_path() {
        let base_url = Uri::from_static("https://api.spacetraders.io/");
        let input_url = Uri::from_static("/v2/my/ships?limit=20&page=2");

        assert_eq!(
            overwrite_base_url(&base_url, input_url),
            "https://api.spacetraders.io/v2/my/ships?limit=20&page=2"
        );
    }

    #[test]
    fn joining_host_and_path_with_relative_path_and_query() {
        let base_url = Uri::from_static("https://api.spacetraders.io/v2/");
        let input_url = Uri::from_static("/my/ships?limit=20&page=2");

        assert_eq!(
            overwrite_base_url(&base_url, input_url),
            "https://api.spacetraders.io/v2/my/ships?limit=20&page=2"
        );
    }

    #[test]
    fn joining_host_no_trailing_slash_with_relative_path_and_query() {
        let base_url = Uri::from_static("https://api.spacetraders.io");
        let input_url = Uri::from_static("/v2/my/ships?limit=20&page=2");

        assert_eq!(
            overwrite_base_url(&base_url, input_url),
            "https://api.spacetraders.io/v2/my/ships?limit=20&page=2"
        );
    }

    #[test]
    fn joining_ip_and_port_with_relative_path() {
        let base_url = Uri::from_static("https://127.0.0.1:3000/");
        let input_url = Uri::from_static("/v2/my/ships?limit=20&page=2");

        assert_eq!(
            overwrite_base_url(&base_url, input_url),
            "https://127.0.0.1:3000/v2/my/ships?limit=20&page=2"
        );
    }
}
