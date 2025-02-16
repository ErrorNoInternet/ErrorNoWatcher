use crate::{
    State,
    scripting::{eval, exec, reload},
};
use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::{Method, Request, Response, StatusCode, body::Bytes};

pub async fn handle(
    request: Request<hyper::body::Incoming>,
    state: State,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let path = request.uri().path().to_owned();

    match (request.method(), path.as_str()) {
        (&Method::POST, "/reload") => Ok(match reload(&state.lua.lock()) {
            Ok(()) => Response::new(empty()),
            Err(error) => status_code_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(full(format!("{error:?}"))),
            ),
        }),

        (&Method::POST, "/eval" | "/exec") => {
            let bytes = request.into_body().collect().await?.to_bytes();
            Ok(match std::str::from_utf8(&bytes) {
                Ok(code) => {
                    let lua = state.lua.lock();
                    Response::new(full(match path.as_str() {
                        "/eval" => format!("{:?}", eval(&lua, code)),
                        "/exec" => format!("{:?}", exec(&lua, code)),
                        _ => unreachable!(),
                    }))
                }
                Err(error) => {
                    return Ok(status_code_response(
                        StatusCode::BAD_REQUEST,
                        Some(full(format!("invalid utf-8 data received: {error:?}"))),
                    ));
                }
            })
        }

        (&Method::GET, "/ping") => Ok(Response::new(full("pong!"))),

        _ => Ok(status_code_response(StatusCode::NOT_FOUND, None)),
    }
}

fn status_code_response(
    status_code: StatusCode,
    bytes: Option<BoxBody<Bytes, hyper::Error>>,
) -> Response<BoxBody<Bytes, hyper::Error>> {
    let mut response = Response::new(bytes.unwrap_or(empty()));
    *response.status_mut() = status_code;
    response
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
