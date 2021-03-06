use super::super::controllers::auth;
use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use actix_web::{http, HttpResponse};
use futures::future::{ok, Either, FutureResult};
use futures::{Future, Poll};
// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct Authenticator;

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S> for Authenticator
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = Auth<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(Auth { service })
    }
}

pub struct Auth<S> {
    service: S,
}

impl<S, B> Service for Auth<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, FutureResult<Self::Response, Self::Error>>;
    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let access_token = req.headers().get("x-access-token").unwrap();
        println!(
            "Hi from start. You requested: {:?}",
            access_token.to_str().unwrap()
        );
        let token_data = auth::decode_token(access_token.to_str().unwrap());
        println!("{:?}", token_data);
        if token_data.is_ok() {
            println!("Token Verification Successful!");
        } else {
            return Either::B(ok(
                req.into_response(HttpResponse::Unauthorized().finish().into_body())
            ));
        }
        println!("{:?}", token_data.unwrap().claims);
        // Either::B(ok(req.into_response(
        //     HttpResponse::Found()
        //         .header(http::header::LOCATION, "/")
        //         .finish()
        //         .into_body(),
        // )))
        Either::A(self.service.call(req))
    }
}
