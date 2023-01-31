use hyper::{Body, Response, StatusCode};
use crate::app::AppEntity;

impl AppEntity{
    pub fn response<B>(status:u16,body:B)->anyhow::Result<Response<Body>>{
        let body = Body::from(body);
        let resp = Response::builder().status(status).body(body)?;Ok(resp)
    }

    pub fn error<T:ToString>(t:T)->anyhow::Result<Response<Body>>{
        let body = Body::from(t.to_string());
        let resp = Response::builder().status(500).body(body)?;Ok(resp)
    }
}