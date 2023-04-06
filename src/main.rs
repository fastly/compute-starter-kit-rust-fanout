use fastly::experimental::RequestUpgradeWebsocket;
use fastly::http::StatusCode;
use fastly::{Error, Request, Response};

mod fanout_util;

fn handle_fanout_ws(mut req: Request, chan: &str) -> Response {
    if req.get_header_str("Content-Type") != Some("application/websocket-events") {
        return Response::from_status(StatusCode::BAD_REQUEST)
            .with_body("Not a WebSocket-over-HTTP request.\n");
    }

    let req_body = req.take_body().into_bytes();
    let mut resp_body: Vec<u8> = [].to_vec();

    let mut resp = Response::from_status(StatusCode::OK)
        .with_header("Content-Type", "application/websocket-events");

    if req_body.starts_with(b"OPEN\r\n") {
        resp.set_header("Sec-WebSocket-Extensions", "grip; message-prefix=\"\"");
        resp_body.extend("OPEN\r\n".as_bytes());
        resp_body.extend(fanout_util::ws_sub(chan));
    } else if req_body.starts_with(b"TEXT ") {
        resp_body.extend(fanout_util::ws_text(
            format!("You said: {}", std::str::from_utf8(&req_body).unwrap_or("")).as_str(),
        ));
    }

    resp.set_body(resp_body);
    resp
}

fn handle_fanout(req: Request, chan: &str) -> Response {
    match req.get_url().path() {
        "/stream/long-poll" => fanout_util::grip_response("text/plain", "response", chan),
        "/stream/plain" => fanout_util::grip_response("text/plain", "stream", chan),
        "/stream/sse" => fanout_util::grip_response("text/event-stream", "stream", chan),
        "/stream/websocket" => handle_fanout_ws(req, chan),
        _ => Response::from_status(StatusCode::BAD_REQUEST).with_body("Invalid Fanout request\n"),
    }
}

fn main() -> Result<(), Error> {
    // Log service version
    println!(
        "FASTLY_SERVICE_VERSION: {}",
        std::env::var("FASTLY_SERVICE_VERSION").unwrap_or_else(|_| String::new())
    );
    let req = Request::from_client();

    // Request is a stream request - from client, or from fanout
    if req.get_path().starts_with("/stream/") {
        return Ok(if req.get_header_str("Grip-Sig").is_some() {
            // Request is from Fanout
            handle_fanout(req, "test").send_to_client()
        } else {
            // Not from fanout, hand it off to Fanout to manage
            req.handoff_fanout("self")?
        });
    }

    // Forward all non-stream requests to the primary backend
    let be_resp = req.send("primary_backend");

    Ok(be_resp?.send_to_client())
}
