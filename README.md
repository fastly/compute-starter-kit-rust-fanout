# Fanout Starter Kit for Rust

[![Deploy to Fastly](https://deploy.edgecompute.app/button)](https://deploy.edgecompute.app/deploy)

Learn about Fastly Compute@Edge with Fanout using a basic starter that demonstrates basic Fanout handlers.

**For more details about this and other starter kits for Compute@Edge, see the [Fastly Developer Hub](https://developer.fastly.com/solutions/starters/)**.

## Setup

The app expects a configured backend named "self" that points back to app itself. For example, if the service has a domain `foo.edgecompute.app`, then you'll need to create a backend on the service named "self" with the destination host set to `foo.edgecompute.app` and port 443. Also set 'Override Host' to the same host value.

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

For each call, the app is actually invoked twice. 

1. Initially, a client request arrives at the app without having been routed through the Fanout proxy yet. The app checks for this via the presence of a `Grip-Sig` header. If that header is not present, the app calls `req.handoff_fanout("self")` and exits. This tells the subsystem that the connection should be routed through Fanout, and is used for HTTP requests controlled by GRIP.

2. Since `self` refers to the same app, a second request is made to the same app, this time coming through Fanout. The app checks for this, and then handles the request accordingly (in `handle()`).

## Note

This app is not currently supported in Fastly's [local development server](https://developer.fastly.com/learning/compute/testing/#running-a-local-testing-server), as the development server does not support Fanout features. To experiment with Fanout, you will need to publish this project to your Fastly Compute@Edge service. using the `fastly compute publish` command.

## Security issues

Please see [SECURITY.md](SECURITY.md) for guidance on reporting security-related issues.
