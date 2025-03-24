use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::{
    Error, Method, Request, Response, StatusCode,
    body::{Bytes, Incoming},
};

use crate::{
    State,
    lua::{eval, exec, reload},
};

pub async fn serve(
    request: Request<Incoming>,
    state: State,
) -> Result<Response<BoxBody<Bytes, Error>>, Error> {
    Ok(match (request.method(), request.uri().path()) {
        (&Method::POST, "/reload") => Response::new(
            reload(&state.lua, None).map_or_else(|error| full(error.to_string()), |()| empty()),
        ),
        (&Method::POST, "/eval") => Response::new(full(
            eval(
                &state.lua,
                &String::from_utf8_lossy(&request.into_body().collect().await?.to_bytes()),
                None,
            )
            .await
            .unwrap_or_else(|error| error.to_string()),
        )),
        (&Method::POST, "/exec") => Response::new(
            exec(
                &state.lua,
                &String::from_utf8_lossy(&request.into_body().collect().await?.to_bytes()),
                None,
            )
            .await
            .map_or_else(|error| full(error.to_string()), |()| empty()),
        ),
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
