<img src="misc/logo/logo.png" width="200" />

# Griffin

A lightweight proxy built on top of hyper.rs to handle gRPC-web, translating gRPC-web requests to standard gRPC requests.

```
grpc-web client <--> griffin (grpc-web to grpc proxy) <--> grpc server
```

```
grpc client <--> griffin <--> grpc server
```

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

- [x] Handle both grpc-web and grpc requests
- [] Add telemetry and health check
- [] Add CORS
- [] Add TLS support

## Contribution

Please feel free to open issues or submit pull requests.

### Run tests (init test and integration test)

```ssh
cargo test --feature test
```
