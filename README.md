![Volo](https://github.com/volo-rs/.github/raw/main/.github/assets/logo.png?sanitize=true)

This crate implements a concurrency limiter layer for Volo-based service.

## Notice

This limiter in this crate may not limit if the service calling process does not contain any async operations. This may happen with some of pure-computing servers and caching servers.

The reason here is that, without any async operations, the handling process of a requests (service's `call` method) becomes "atomic" that each worker will not begin to handle a new request until the current request is finished. Base on this situation, the possibly maximum concurrency is the number of the workers (usually equals to the number of logical CPU threads), which may never reach the configured limitation.

## Quick Start

### Volo gRPC Server

Add the required dependencies to the `Cargo.toml` file:
```toml
[dependencies]
# -- snip --
volo-concurrency-limiter = { version = "*", features = ["volo-grpc"] }
# -- snip --
```

Add the middleware layer to the server, which looks like this:
```rust{.ignore}
use std::net::SocketAddr;

use volo_grpc::server::{Server, ServiceBuilder};

use volo_example::S;
use volo_concurrency_limiter::ConcurrencyLimiterServiceLayer;

#[volo::main]
async fn main() {
    let addr: SocketAddr = "[::]:8080".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    Server::new()
        .add_service(ServiceBuilder::new(volo_gen::volo::example::ItemServiceServer::new(S)).build())
        .layer_front(ConcurrencyLimiterServiceLayer::new(100)) // which configures the max concurrency to be 100
        .run(addr)
        .await
        .unwrap();
}
```

### Volo Thrift Server

Add the required dependencies to the `Cargo.toml` file:
```toml
[dependencies]
# -- snip --
volo-concurrency-limiter = { version = "*", features = ["volo-thrift"] }
# -- snip --
```

Add the middleware layer to the server, which looks like this:
```rust{.ignore}
use std::net::SocketAddr;

use volo_example::{S};
use volo_concurrency_limiter::ConcurrencyLimiterServiceLayer;

#[volo::main]
async fn main() {
    let addr: SocketAddr = "[::]:8080".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    volo_gen::volo::example::ItemServiceServer::new(S)
        .layer_front(ConcurrencyLimiterServiceLayer::new(10))
        .run(addr)
        .await
        .unwrap();
}
```

## License

Volo is dual-licensed under the MIT license and the Apache License (Version 2.0).

See [LICENSE-MIT](https://github.com/volo-rs/.github/blob/main/LICENSE-MIT) and [LICENSE-APACHE](https://github.com/volo-rs/.github/blob/main/LICENSE-APACHE) for details.

## Community

- Email: [volo@cloudwego.io](mailto:volo@cloudwego.io)
- How to become a member: [COMMUNITY MEMBERSHIP](https://github.com/cloudwego/community/blob/main/COMMUNITY_MEMBERSHIP.md)
- Issues: [Issues](https://github.com/volo-rs/.github/issues)
- Feishu: Scan the QR code below with [Feishu](https://www.feishu.cn/) or [click this link](https://applink.feishu.cn/client/chat/chatter/add_by_link?link_token=7f0oe1a4-930f-41f9-808a-03b89a681020) to join our CloudWeGo Volo user group.

  <img src="https://github.com/cloudwego/volo/raw/main/.github/assets/volo-feishu-user-group.png" alt="Volo user group" width="50%" height="50%" />
