use std::{
    future::Future,
    pin::Pin,
    task::{ready, Poll},
    time::Duration,
};

use tokio::time::{Instant, Sleep};
use tracing::{event, Level};

#[derive(Debug, Clone)]
pub struct RateLimitWithBurstLayer {
    rate_default: Rate,
    rate_burst: Rate,
}

impl RateLimitWithBurstLayer {
    pub const fn new(
        num_default: u64,
        per_default: Duration,
        num_burst: u64,
        per_burst: Duration,
    ) -> Self {
        let rate_default = Rate::new(num_default, per_default);
        let rate_burst = Rate::new(num_burst, per_burst);

        Self {
            rate_default,
            rate_burst,
        }
    }
}

impl<S> tower_layer::Layer<S> for RateLimitWithBurstLayer {
    type Service = RateLimitWithBurst<S>;

    fn layer(&self, service: S) -> Self::Service {
        RateLimitWithBurst::new(service, self.rate_default, self.rate_burst)
    }
}

/// Enforces a rate limit on the underlying service.
/// It has a default limit and a burst limit, which are
/// refreshed at separate rates.
#[derive(Debug)]
pub struct RateLimitWithBurst<T> {
    inner: T,
    rate_default: Rate,
    rate_burst: Rate,
    state: State,
    until_default: Instant,
    until_burst: Instant,
    rem: u64,
    sleep: Pin<Box<Sleep>>,
}

#[derive(Debug, Copy, Clone)]
enum State {
    Limited,
    Ready,
}

impl<T> RateLimitWithBurst<T> {
    #[expect(private_interfaces)]
    pub fn new(inner: T, rate_default: Rate, rate_burst: Rate) -> Self {
        let until = Instant::now();
        let state = State::Ready;

        Self {
            inner,
            rate_default,
            rate_burst,
            state,
            until_default: until,
            until_burst: until,
            // The total amount of available requests is the default bucket + burst bucket.
            rem: rate_default.num() + rate_burst.num(),
            sleep: Box::pin(tokio::time::sleep_until(until)),
        }
    }
}

/// A rate of requests per time period.
#[derive(Debug, Copy, Clone)]
struct Rate {
    num: u64,
    per: Duration,
}

impl Rate {
    /// Create a new rate.
    ///
    /// # Panics
    ///
    /// This function panics if `num` or `per` is 0.
    const fn new(num: u64, per: Duration) -> Self {
        assert!(num > 0);
        assert!(per.as_nanos() > 0);

        Rate { num, per }
    }

    fn num(&self) -> u64 {
        self.num
    }

    fn per(&self) -> Duration {
        self.per
    }
}

impl<S, Request> tower_service::Service<Request> for RateLimitWithBurst<S>
where
    S: tower_service::Service<Request>,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = S::Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        match self.state {
            State::Ready => return Poll::Ready(ready!(self.inner.poll_ready(cx))),
            State::Limited => {
                // Check if the next refill is due.
                if Pin::new(&mut self.sleep).poll(cx).is_pending() {
                    event!(Level::TRACE, "rate limit exceeded; sleeping.");
                    return Poll::Pending;
                }
            }
        }

        // At least one bucket is due for a refill.
        let now = Instant::now();

        // If the default rate limit period has elapsed,
        // reset it and refill the bucket.
        if now >= self.until_default {
            self.until_default = now + self.rate_default.per();
            // Only refill up to the maximum of default bucket + burst bucket.
            self.rem = (self.rem + self.rate_default.num())
                .min(self.rate_default.num() + self.rate_burst.num());
        }

        // If the burst rate limit period has elapsed,
        // reset it and refill the bucket.
        if now >= self.until_burst {
            self.until_burst = now + self.rate_burst.per();
            // Only refill up to the maximum of default bucket + burst bucket.
            self.rem = (self.rem + self.rate_burst.num())
                .min(self.rate_default.num() + self.rate_burst.num());
        }

        // Go back to Ready state.
        self.state = State::Ready;

        Poll::Ready(ready!(self.inner.poll_ready(cx)))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        match self.state {
            State::Ready => {
                let now = Instant::now();

                // If the default rate limit period has elapsed,
                // reset it and refill the bucket.
                if now >= self.until_default {
                    self.until_default = now + self.rate_default.per();
                    // Only refill up to the maximum of default bucket + burst bucket.
                    self.rem = (self.rem + self.rate_default.num())
                        .min(self.rate_default.num() + self.rate_burst.num());
                }

                // If the burst rate limit period has elapsed,
                // reset it and refill the bucket.
                if now >= self.until_burst {
                    self.until_burst = now + self.rate_burst.per();
                    // Only refill up to the maximum of default bucket + burst bucket.
                    self.rem = (self.rem + self.rate_burst.num())
                        .min(self.rate_default.num() + self.rate_burst.num());
                }

                // If we have more than one token remaining,
                // we will stay in the Ready state.
                if self.rem > 1 {
                    self.rem -= 1;
                    self.state = State::Ready;
                } else {
                    // We're spending the last token in the bucket.
                    // Reset the sleep until either the default refill
                    // or burst refill, whichever is shorter.
                    let until = if self.until_default <= self.until_burst {
                        self.until_default
                    } else {
                        self.until_burst
                    };

                    self.sleep.as_mut().reset(until);
                    // Can't service any more requests until next refill.
                    self.state = State::Limited;
                }

                self.inner.call(req)
            }
            State::Limited => panic!("service not ready; poll_ready must be called first"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time;
    use tokio_test::{assert_pending, assert_ready_ok};
    use tower_test::{assert_request_eq, mock};

    fn trace_init() -> tracing::subscriber::DefaultGuard {
        let subscriber = tracing_subscriber::fmt()
            .with_test_writer()
            .with_max_level(tracing::Level::TRACE)
            .with_thread_names(true)
            .finish();
        tracing::subscriber::set_default(subscriber)
    }

    #[tokio::test]
    async fn reaches_limit_and_refills_buckets_independently() {
        let _t = trace_init();
        time::pause();

        let rate_limit = RateLimitWithBurstLayer::new(
            1,
            Duration::from_millis(100),
            2,
            Duration::from_millis(400),
        );
        let (mut service, mut handle) = mock::spawn_layer(rate_limit);

        // Should start out ready.
        assert_ready_ok!(service.poll_ready());

        let response = service.call("hello 1");

        assert_request_eq!(handle, "hello 1").send_response("world 1");
        assert_eq!(response.await.unwrap(), "world 1");

        // Should still be ready after default bucket is empty,
        // because we still have two in the burst bucket.
        assert_ready_ok!(service.poll_ready());

        let response = service.call("hello 2");

        assert_request_eq!(handle, "hello 2").send_response("world 2");
        assert_eq!(response.await.unwrap(), "world 2");

        // Should still be ready after default bucket is empty,
        // because we still have one in the burst bucket.
        assert_ready_ok!(service.poll_ready());

        let response = service.call("hello 3");

        assert_request_eq!(handle, "hello 3").send_response("world 3");
        assert_eq!(response.await.unwrap(), "world 3");

        // Should be pending, because we have exhausted both the default and the burst bucket.
        assert_pending!(service.poll_ready());
        assert_pending!(handle.poll_request());

        // Advance time past the interval of the default bucket.
        time::advance(Duration::from_millis(101)).await;

        // Should be ready again because the default bucket should have been refilled.
        assert_ready_ok!(service.poll_ready());

        let response = service.call("ping 1");

        assert_request_eq!(handle, "ping 1").send_response("pong 1");
        assert_eq!(response.await.unwrap(), "pong 1");

        // Should be pending, because we have exhausted the default bucket again,
        // and the burst bucket has not refilled yet. (Elapsed time: 101 ms)
        assert_pending!(service.poll_ready());
        assert_pending!(handle.poll_request());

        // Advance time until the burst bucket refills.
        time::advance(Duration::from_millis(301)).await;

        // Should be ready again, because both buckets should have been refilled.
        // (Elapsed time: 402 ms)
        assert_ready_ok!(service.poll_ready());

        let response = service.call("check 1");

        assert_request_eq!(handle, "check 1").send_response("ok 1");
        assert_eq!(response.await.unwrap(), "ok 1");

        // Should still be ready because we still have the burst bucket.
        assert_ready_ok!(service.poll_ready());

        let response = service.call("check 2");

        assert_request_eq!(handle, "check 2").send_response("ok 2");
        assert_eq!(response.await.unwrap(), "ok 2");

        // Should still be ready because we still have the burst bucket.
        assert_ready_ok!(service.poll_ready());

        let response = service.call("check 3");

        assert_request_eq!(handle, "check 3").send_response("ok 3");
        assert_eq!(response.await.unwrap(), "ok 3");

        // Should be pending, because we have exhausted both the default and the burst bucket.
        assert_pending!(service.poll_ready());
        assert_pending!(handle.poll_request());
    }
}
