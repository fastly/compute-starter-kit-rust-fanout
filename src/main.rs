use fastly::experimental::RequestUpgradeWebsocket;
use fastly::{Error, Request, Response};

fn main() -> Result<(), Error> {
    let req = Request::from_client();

    if req.get_url().path() == "/ws" {
        return Ok(req.upgrade_websocket("self")?);
    }

    let resp = match req.send("fp") {
        Ok(resp) => {
            println!("got response: code={}", resp.get_status());
            resp
        }
        Err(e) => Response::from_body(format!("send error: {:?}", e)),
    };

    Ok(resp.send_to_client())
}
