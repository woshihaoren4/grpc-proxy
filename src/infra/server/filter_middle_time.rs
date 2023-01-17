use std::time::{Duration, Instant};
use hyper::{Body, Request, Response};
use wd_run::Context;
use crate::infra::server::HttpFilter;

const HTTP_REQUEST_ELAPSED_TIME:&'static str = "HTTP_REQUEST_ELAPSED_TIME";

pub struct TimeMiddle;

impl TimeMiddle{
    pub async fn elapsed(ctx:&Context)->Duration{
        let instant = if let Some(s) = ctx.get::<_,Instant>(HTTP_REQUEST_ELAPSED_TIME).await{
            s
        }else{
            return Duration::default()
        };
        instant.elapsed()
    }
}

#[async_trait::async_trait]
impl HttpFilter for TimeMiddle{
    async fn request(&self, ctx: Context, req: Request<Body>) -> Result<Request<Body>, Response<Body>> {
        let instant = Instant::now();
        ctx.set(HTTP_REQUEST_ELAPSED_TIME, instant).await;
        Ok(req)
    }

    // async fn response(&self, ctx: Context, resp: Response<Body>) -> Response<Body> {
    //     resp
    // }
}