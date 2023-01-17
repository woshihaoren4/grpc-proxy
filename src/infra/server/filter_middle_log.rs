use std::sync::Arc;
use hyper::{Body, Method, Request, Response};
use wd_run::Context;
use crate::infra::server::{HttpFilter, RequestIdMiddle};
use crate::infra::server::filter_middle_time::TimeMiddle;

const HTTP_LOG_METHOD:&'static str = "HTTP_LOG_METHOD";
const HTTP_LOG_PATH:&'static str = "HTTP_LOG_PATH";

pub struct LogMiddle;

impl LogMiddle{
    pub async fn method_string(ctx: &Context)->String{
        ctx.get(HTTP_LOG_METHOD).await.map(|x:Arc<Method>|{
            x.to_string()
        }).unwrap_or("unknown".into())
        // let opt = ctx.get::<_,Method>(HTTP_LOG_METHOD).await;
        // if opt.is_none() {
        //     return String::new();
        // }
        // opt.unwrap().to_string()
    }
    pub async fn path(ctx: &Context)->Arc<String>{
        ctx.get(HTTP_LOG_PATH).await.unwrap_or(Arc::new(String::default()))
    }
}

#[async_trait::async_trait]
impl HttpFilter for LogMiddle{
    async fn request(&self, ctx: Context, req: Request<Body>) -> Result<Request<Body>, Response<Body>> {
        let method = req.method().clone();
        let path = req.uri().to_string();
        ctx.set(HTTP_LOG_METHOD,method).await;
        ctx.set(HTTP_LOG_PATH,path).await;
        Ok(req)
    }

    async fn response(&self, ctx: Context, resp: Response<Body>) -> Response<Body> {
        let request_id = RequestIdMiddle::request_id(&ctx).await.unwrap_or(0);
        let method = LogMiddle::method_string(&ctx).await;
        let path = LogMiddle::path(&ctx).await;
        let time = TimeMiddle::elapsed(&ctx).await.as_millis();
        let status = resp.status();
        wd_log::log_info_ln!("request[{}:{}ms] method:[{}] path:{}  response:{}",request_id,time,method,path,status);
        resp
    }
}