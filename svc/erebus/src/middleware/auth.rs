use hyper::{Body, Request, Response, StatusCode};
use std::str;

pub async fn dev_wallet_auth<S>(
    req: Request<Body>,
    next: S,
) -> Result<Response<Body>, hyper::Error>
where
    S: hyper::service::Service<Request<Body>, Response = Response<Body>, Error = hyper::Error>,
{
    let auth_header = match req.headers().get("Authorization") {
        Some(val) => val,
        None => return unauthorized_response(),
    };

    let auth_str = match auth_header.to_str() {
        Ok(s) => s,
        Err(_) => return unauthorized_response(),
    };

    if !auth_str.starts_with("Basic ") {
        return unauthorized_response();
    }

    let decoded = match base64::decode(&auth_str[6..]) {
        Ok(decoded) => decoded,
        Err(_) => return unauthorized_response(),
    };

    let auth = match str::from_utf8(&decoded) {
        Ok(s) => s,
        Err(_) => return unauthorized_response(),
    };

    let (username, password) = match auth.split_once(':') {
        Some((u, p)) => (u, p),
        None => return unauthorized_response(),
    };

    let dev_wallet_pubkey = std::env::var("DEVELOPER_WALLET_PUBKEY")
        .expect("DEVELOPER_WALLET_PUBKEY must be set");

    if username == "dev-wallet" && password == dev_wallet_pubkey {
        return next.call(req).await.map_err(Into::into);
    }

    unauthorized_response()
}

fn unauthorized_response() -> Result<Response<Body>, hyper::Error> {
    Ok(Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("WWW-Authenticate", "Basic realm=\"Nullblock Dev Access\"")
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Body::from(
            r#"<html>
            <head><title>Under Maintenance</title></head>
            <body>
                <h1>Nullblock System Under Maintenance</h1>
                <p>Access to development features is restricted to the architect’s wallet.</p>
                <p>For more information, contact: <strong>@pervySageDev</strong></p>
                <hr>
                <small>Nullblock v2.0 — The substrate awakens.</small>
            </body>
            </html>"#,
        ))
        .unwrap())
}