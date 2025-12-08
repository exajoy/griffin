<img src="misc/logo/logo.png" width="200" />

# Griffin - A lightweight gRPC-Web and gRPC proxy

In my previous role, we struggled with slow Pod cold starts.
Whenever a Pod spun up, incoming requests would pile up
and wait until it was finally ready. Most of that delay
came from running Envoy just to handle the gRPC-Web → gRPC
translation layer. It felt like using a huge,
complex system for a very small piece of functionality.

That experience pushed me to build Griffin — a lightweight
proxy designed specifically to remove that bottleneck.
Griffin is built on top of hyper.rs and focuses on doing
one thing well: translating gRPC-Web requests into standard gRPC calls.

The result is a proxy that’s incredibly small and fast.
Griffin’s binary is only 1 MB, which is:
• 100× smaller than a full Envoy build (~140 MB+)
• 15× smaller than grpcwebproxy (~15.3 MB)
• no garbage collector

## Features

- [x] Telemetry support (Prometheus)
- [x] Health check support
- [x] Hot configuration reload (explain in [here](/docs/hot_config_reload.md))
- [ ] CORS support
- [ ] TLS support

## How to use

```ssh
griffin -c config.yaml
```

You can find example here [default_config.yaml](/griffin/default_config.yaml).

### Requirements

rustc 1.91.0

cargo 1.81.0

### Run from source

```ssh
git clone https://github.com/exajoy/griffin
cd griffin
cargo build --release
```

### Run tests

```ssh
cargo netest --feature test
```

## FAQs

You can see more FAQs in [here](/docs/faqs.md).

## Contribution

Please feel free to open issues or submit pull requests.
Make sure to follow the existing code style and include
tests for any new features or bug fixes.
