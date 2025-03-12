use crate::{
    State,
    lua::{eval, exec, reload},
};
use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::{
    Error, Method, Request, Response, StatusCode,
    body::{Bytes, Incoming},
};

pub async fn serve(
    request: Request<Incoming>,
    state: State,
) -> Result<Response<BoxBody<Bytes, Error>>, Error> {
    macro_rules! handle_code {
        ($handler:ident) => {
            match std::str::from_utf8(&request.into_body().collect().await?.to_bytes()) {
                Ok(code) => Response::new(full(format!(
                    "{:#?}",
                    $handler(&state.lua, code, None).await
                ))),
                Err(error) => status_code_response(
                    StatusCode::BAD_REQUEST,
                    full(format!("invalid utf-8 data received: {error:?}")),
                ),
            }
        };
    }

    Ok(match (request.method(), request.uri().path()) {
        (&Method::POST, "/reload") => {
            Response::new(full(format!("{:#?}", reload(&state.lua, None))))
        }
        (&Method::POST, "/eval") => handle_code!(eval),
        (&Method::POST, "/exec") => handle_code!(exec),
        (&Method::GET, "/ping") => Response::new(full("pong!")),
        _ => status_code_response(StatusCode::NOT_FOUND, empty()),
    })
}

fn status_code_response(
    status_code: StatusCode,
    bytes: BoxBody<Bytes, Error>,
) -> Response<BoxBody<Bytes, Error>> {
    let mut response = Response::new(bytes);
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
