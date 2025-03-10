use crate::{
    State,
    lua::{eval, exec, reload},
};
use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::{Method, Request, Response, StatusCode, body::Bytes};

pub async fn serve(
    request: Request<hyper::body::Incoming>,
    state: State,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let path = request.uri().path().to_owned();

    Ok(match (request.method(), path.as_str()) {
        (&Method::POST, "/reload") => {
            Response::new(full(format!("{:#?}", reload(&state.lua, None))))
        }

        (&Method::POST, "/eval" | "/exec") => {
            let bytes = request.into_body().collect().await?.to_bytes();
            match std::str::from_utf8(&bytes) {
                Ok(code) => Response::new(full(match path.as_str() {
                    "/eval" => format!("{:#?}", eval(&state.lua, code, None).await),
                    "/exec" => format!("{:#?}", exec(&state.lua, code, None).await),
                    _ => unreachable!(),
                })),
                Err(error) => status_code_response(
                    StatusCode::BAD_REQUEST,
                    full(format!("invalid utf-8 data received: {error:?}")),
                ),
            }
        }

        (&Method::GET, "/ping") => Response::new(full("pong!")),

        _ => status_code_response(StatusCode::NOT_FOUND, empty()),
    })
}

fn status_code_response(
    status_code: StatusCode,
    bytes: BoxBody<Bytes, hyper::Error>,
) -> Response<BoxBody<Bytes, hyper::Error>> {
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
