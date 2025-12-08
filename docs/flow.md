## RPC flow and Protocol Support

Griffin is designed to handle both gRPC-Web and standard gRPC traffic
concurrently, enabling seamless interoperability between
browser-based clients and backend gRPC services.

### 1. gRPC-Web Interoperability

Griffin acts as a translation layer between gRPC-Web clients and standard gRPC servers:

```
gRPC-Web Client ⇄ Griffin (Web ↔ gRPC Translation) ⇄ gRPC Server
```

This allows browser environments—where native gRPC over HTTP/2 is not available—to
communicate with existing gRPC services without requiring Envoy or other heavy intermediaries.

### 2. Lightweight gRPC Reverse Proxy

Griffin can also operate as a minimal reverse proxy for native gRPC traffic:

```
gRPC Client ⇄ Griffin ⇄ gRPC Server
```

This makes it suitable for routing, connection management, or hot-reload configurations without introducing unnecessary overhead.

### 3. Supported RPC Types

Griffin fully supports the core RPC patterns for both protocols:

gRPC-Web

- Unary
- Server-streaming

Standard gRPC

- Unary
- Server-streaming
- Client-streaming
- Bidirectional streaming
