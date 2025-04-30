#![doc = include_str!("../README.md")]
#![feature(impl_trait_in_assoc_type)]

/// The implementation of basic concurrency limiter as [volo::Service].
///
/// For the informations and notices, see the [documentation page of this crate](crate).
#[derive(Clone)]
pub struct ConcurrencyLimiterService<S> {
    inner: S,
    status: std::sync::Arc<ConcurrencyLimiterServiceSharedStatus>,
}

struct ConcurrencyLimiterServiceSharedStatus {
    limit: u64,
    current: std::sync::atomic::AtomicU64,
}

/// NOTE:
///
/// The [volo_thrift] using [volo_thrift::AnyhowError] as error type and it has a default way of converting from `std::error::Error`
/// so it's not necessary (and will cause conflict) for [ConcurrencyLimitError] to implement `Into<volo_thrift::AnyhowError>`.
///
/// This also leads to a potential problem, when the request rejected by this limiter, the error type is always "ApplicationError" and
/// error kind is always "Unknown", making it difficult for clients and monitor components to identify the failure caused by the limiter.
#[derive(Debug)]
struct ConcurrencyLimitError {
    limit: u64,
    current: u64,
}

impl std::fmt::Display for ConcurrencyLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "concurrency limited ({}/{})", self.current, self.limit)
    }
}

impl std::error::Error for ConcurrencyLimitError {}

#[cfg(feature = "volo-grpc")]
impl Into<volo_grpc::Status> for ConcurrencyLimitError {
    fn into(self) -> volo_grpc::Status {
        volo_grpc::Status::resource_exhausted(self.to_string())
    }
}

#[volo::service]
impl<Cx, Req, S> volo::Service<Cx, Req> for ConcurrencyLimiterService<S>
where
    Req: std::fmt::Debug + Send + 'static,
    S: Send + 'static + volo::Service<Cx, Req> + Sync,
    Cx: Send + 'static,
    ConcurrencyLimitError: Into<S::Error>,
{
    async fn call<'cx, 's>(&'s self, cx: &'cx mut Cx, req: Req) -> Result<S::Response, S::Error>
    where
        's: 'cx,
    {
        loop {
            let curr = self
                .status
                .current
                .load(std::sync::atomic::Ordering::Relaxed);

            if curr >= self.status.limit {
                return Err(ConcurrencyLimitError {
                    limit: self.status.limit,
                    current: curr,
                }
                .into());
            }

            if self
                .status
                .current
                .compare_exchange(
                    curr,
                    curr + 1,
                    std::sync::atomic::Ordering::Relaxed,
                    std::sync::atomic::Ordering::Relaxed,
                )
                .is_ok()
            {
                break;
            }
        }

        let res = self.inner.call(cx, req).await;

        self.status
            .current
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);

        res
    }
}

/// The [volo::Layer] for [ConcurrencyLimiterService].
pub struct ConcurrencyLimiterServiceLayer {
    limit: u64,
}

impl ConcurrencyLimiterServiceLayer {
    pub fn new(limit: u64) -> Self {
        Self { limit }
    }
}

impl<S> volo::Layer<S> for ConcurrencyLimiterServiceLayer {
    type Service = ConcurrencyLimiterService<S>;

    fn layer(self, inner: S) -> Self::Service {
        ConcurrencyLimiterService {
            inner,
            status: std::sync::Arc::new(ConcurrencyLimiterServiceSharedStatus {
                limit: self.limit,
                current: std::sync::atomic::AtomicU64::new(0),
            }),
        }
    }
}
