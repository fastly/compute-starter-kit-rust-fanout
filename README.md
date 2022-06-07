# Compute@Edge Fanout starter kit for Rust

This is a Compute@Edge app that implements basic Fanout handlers.

## Security issues

Please see [SECURITY.md](SECURITY.md) for guidance on reporting security-related issues.

## Setup

The app expects a configured backend named "self" that points back to app itself. For example, if the service has a domain `foo.edgecompute.app`, then you'll need to create a backend on the service named "self" with the destination host/port set to `foo.edgecompute.app` and port 443. Also set Override Host to the same host value.

## Endpoints

The app exposes the following endpoints to clients:

* `/ws`: bi-directional WebSocket
* `/stream`: HTTP streaming of `text/plain`
* `/sse`: SSE (streaming of `text/event-stream`)
* `/response`: Long-polling

Connecting to any endpoint will subscribe the connection to channel "test". The WebSocket endpoint echos back any messages it receives from the client.

Data can be sent to the connections via the GRIP publish endpoint at `https://fanout.fastly.com/{service-id}/publish/`. For example, here's a curl command to send a WebSocket message:

```sh
curl \
  --user {service-id}:{secret} \
  -d '{"items":[{"channel":"test","formats":{"ws-message":{"content":"hello"}}}]}' \
  https://fanout.fastly.com/{service-id}/publish/
```

## How it works

The app is invoked twice:

1. Initially, a client request arrives at the app without having been routed through the Fanout proxy yet. The app checks for this via the lack of a `Grip-Sig` header. If that header is not present, the app calls `upgrade_websocket("self")` and exits. This tells the subsystem that the connection should be routed through Fanout. Despite the name of this function, it works for both HTTP and WebSocket requests.

2. Fanout then makes one or more HTTP requests to the specified backend server. In this case the backend server is the same app. These requests have the `Grip-Sig` header set, so the app knows they have been routed through Fanout, and the app handles them accordingly.
