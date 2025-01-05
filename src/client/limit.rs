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
