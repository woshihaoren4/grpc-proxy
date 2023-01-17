use crate::infra::server::HttpFilter;
use hyper::{Body, Request, Response};
use wd_run::Context;
use wd_sonyflake::SonyFlakeEntity;

const HTTP_REQUEST_ID: &'static str = "HTTP_REQUEST_ID";

pub struct RequestIdMiddle {
    rand: SonyFlakeEntity,
}

impl RequestIdMiddle {
    pub fn new() -> Self {
        let rand = SonyFlakeEntity::new_default();
        Self { rand }
    }
    pub async fn request_id(ctx: &Context) -> Option<i64> {
        ctx.copy(HTTP_REQUEST_ID).await
    }
}

#[async_trait::async_trait]
impl HttpFilter for RequestIdMiddle {
    async fn request(
        &self,
        ctx: Context,
        req: Request<Body>,
    ) -> Result<Request<Body>, Response<Body>> {
        let id = self.rand.get_id();
        ctx.set(HTTP_REQUEST_ID, id).await;
        Ok(req)
    }

    async fn response(&self, _ctx: Context, req: Response<Body>) -> Response<Body> {
        req
    }
}
