use fastly::experimental::RequestUpgradeWebsocket;
use fastly::http::StatusCode;
use fastly::{Error, Request, Response};

fn handle_root(_req: Request) -> Response {
    Response::from_status(StatusCode::OK)
        .with_header("Content-Type", "text/plain")
        .with_body("Hello from the Fanout test handler!\n")
}

fn handle_long_poll(_req: Request) -> Response {
    Response::from_status(StatusCode::OK)
        .with_header("Content-Type", "text/plain")
        .with_header("Grip-Hold", "response")
        .with_header("Grip-Channel", "test")
        .with_body("no data\n")
}

fn handle_stream(_req: Request) -> Response {
    Response::from_status(StatusCode::OK)
        .with_header("Content-Type", "text/plain")
        .with_header("Grip-Hold", "stream")
        .with_header("Grip-Channel", "test")
        .with_body("stream open\n")
}

fn handle_sse(_req: Request) -> Response {
    Response::from_status(StatusCode::OK)
        .with_header("Content-Type", "text/event-stream")
        .with_header("Grip-Hold", "stream")
        .with_header("Grip-Channel", "test")
        .with_body("event: message\ndata: stream open\n\n")
}

fn handle_ws(mut req: Request) -> Response {
    let content_type = match req.get_header_str("Content-Type") {
        Some(s) => s,
        None => {
            return Response::from_status(StatusCode::BAD_REQUEST).with_body("Need Content-Type.\n")
        }
    };

    if content_type != "application/websocket-events" {
        return Response::from_status(StatusCode::BAD_REQUEST)
            .with_body("Not a WebSocket-over-HTTP request.\n");
    }

    let body = req.take_body().into_bytes();

    let mut sub = false;

    if body.len() >= 6 && &body[..6] == b"OPEN\r\n" {
        sub = true;
    }

    // echo
    let mut out_body = body.clone();

    if sub {
        let msg = "c:{\"type\":\"subscribe\",\"channel\":\"test\"}";
        out_body.extend(format!("TEXT {:02x}\r\n{}\r\n", msg.len(), msg).as_bytes());
    }

    let mut resp = Response::from_status(StatusCode::OK)
        .with_header("Content-Type", "application/websocket-events");

    if sub {
        resp.set_header("Sec-WebSocket-Extensions", "grip; message-prefix=\"\"");
    }

    resp.set_body(out_body);

    resp
}

fn handle(req: Request) -> Result<Response, Error> {
    let path = req.get_url().path();

    match path {
        "/" => Ok(handle_root(req)),
        "/response" => Ok(handle_long_poll(req)),
        "/stream" => Ok(handle_stream(req)),
        "/sse" => Ok(handle_sse(req)),
        "/ws" => Ok(handle_ws(req)),
        _ => Ok(Response::from_status(StatusCode::NOT_FOUND).with_body("no such test resource\n")),
    }
}

fn main() -> Result<(), Error> {
    let mut req = Request::from_client();

    if !(req.get_header_str("Grip-Sig").is_some()) {
        // if the request didn't come through Fanout,
        // tell the subsystem that the request should be routed
        // through Fanout first

        if req.get_body_mut().read_chunks(1).next().is_some() {
            return Ok(Response::from_status(StatusCode::BAD_REQUEST)
                .with_body("Request body must be empty.\n")
                .send_to_client());
        }

        // despite the name, this function works for both http requests and
        // websockets. we plan to update the SDK to make this more intuitive
        Ok(req.upgrade_websocket("self")?)
    } else {
        // if the request came from Fanout, process it with handle()
        Ok(handle(req)?.send_to_client())
    }
}
