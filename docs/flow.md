## Request flow

Griffin supports both gRPC-web and gRPC traffics at the same time:

- Interoeprability between grpc-web clients and grpc servers

```
grpc-web client <--> griffin (grpc-web to grpc proxy) <--> grpc server
```

- Minimal gRPC reverse proxy

```
grpc client <--> griffin <--> grpc server
```

- Support 2 types of grpc-web requests (unary and server streaming)
- Support 4 types standard grpc requests (unary request, server streaming, client streaming, bidi streaming)
