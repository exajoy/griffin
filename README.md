<img src="misc/logo/logo.png" width="200" />

# Griffin - A lightweight gRPC-Web to gRPC proxy

<!-- In the world of package transmission, gRPC-Web and gRPC are becoming increasingly polular. -->
<!-- However, the current solutions of converting gRPRC-Web to gRPC, such as Envoy, -->
<!-- are often too large and complex for lightweight applications. -->

In my previous role, Pods experienced slow cold starts, causing requests to queue until the Pod was ready.
The primary overhead came from using Envoy solely for gRPC-web → gRPC translation.
This inspired me to build a lightweight proxy to remove that bottleneck.

This motivated me to build Griffin, a lightweight, purpose-built gRPC-web → gRPC proxy to remove that bottleneck and reduce cold-start latency.

Griffin is built on top of hyper.rs that translate gRPC-web to standard gRPC requests.
Griffin's binary is only [1MB](https://github.com/exajoy/griffin/releases), **100x smaller** than
full Envoy's binary [(140MB+)](https://hub.docker.com/r/envoyproxy/envoy/tags?name=dev) and **15x smaller**
than grpcwebproxy [(15.3MB)](https://github.com/improbable-eng/grpc-web/releases) **without garbage collection**.

## Features

- [x] Telemetry support (Prometheus)
- [x] Health check support
- [x] Hot swapping configuration (explain in here)
- [ ] CORS support
- [ ] TLS support

## How to use

```ssh
griffin \
--proxy-host=127.0.0.1 \
--proxy-port=8080 \
--forward-host=127.0.0.1 \
--forward-port=3000
```

## Inspirations

[Grpc Web](https://github.com/improbable-eng/grpc-web)

[Tonic](https://github.com/hyperium/tonic)

## Implementation Documents

<https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md>

<https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-WEB.md>

<https://datatracker.ietf.org/doc/rfc7540/>

## Installation

### Build from source

#### Requirements

rustc 1.91.0

cargo 1.81.0

#### Commands

```ssh
git clone https://github.com/exajoy/griffin
cd griffin
cargo build --release
```

## TODO

## Contribution

Please feel free to open issues or submit pull requests.

### Run tests (unit tests and integration tests)

```ssh
cargo test --feature test
```

## FAQs

### 1. Why this proxy is called Griffin?

Griffin is a hybrid mythical creature with the body of a lion and the head and wings of an eagle.
This proxy is a hybrid proxy that combines gRPC-web and gRPC functionalities, just like a Griffin.
