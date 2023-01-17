use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use hyper::{Body, Request, Response};
use wd_run::{CmdInfo, Context};
use crate::infra::server::{HttpFilter, HyperHttpServerBuilder, LogMiddle, RequestIdMiddle, ShutDown, TimeMiddle};

#[derive(Default)]
pub struct RunApplication{
    sd: ShutDown,
}

impl RunApplication{
    pub fn new()->(Self,ShutDown){
        let app = Self{sd:ShutDown::default()};
        let sd = app.sd.clone();
        (app,sd)
    }
    pub fn args() -> CmdInfo {
        CmdInfo::new("run", "run application").add(
            "c",
            "./src/conf/config.toml",
            "config file path",
        )
    }
}

impl wd_run::EventHandle for RunApplication {
    fn handle(&self, ctx: Context) -> Pin<Box<dyn Future<Output=Context> + Send>> {
        let sd = self.sd.clone();
        Box::pin(async move {
            wd_log::log_debug_ln!("start run application");
            let sd = HyperHttpServerBuilder::new().handle(|_c,r:Request<Body>|async move{
                let method = r.method();
                let path = r.uri();
                wd_log::log_debug_ln!("method:[{}] path:[{}]",method,path);
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok(Response::new(Body::from("success")))
            })
                .append_filter(TimeMiddle)
                .append_filter(RequestIdMiddle::new())
                .append_filter(LogMiddle)
                .append_filter(FilterTest)
                // .append_filter(FilterAbort)
                // .run().await.expect("http服务报错");
                .set_shutdown_singe(sd)
                .async_run();
            wd_log::log_debug_ln!("run application success");
            sd.wait_close().await;
            return ctx
        })
    }
}

struct FilterAbort;
#[async_trait::async_trait]
impl HttpFilter for FilterAbort {
    async fn request(&self, _ctx: Context, _req: Request<Body>) -> Result<Request<Body>, Response<Body>> {
        Err(Response::new(Body::from("failed")))
        // Ok(req)
    }

    async fn response(&self, _ctx: Context, resp: Response<Body>) -> Response<Body> {
        wd_log::log_info_ln!("FilterAbort 终止了请求");
        resp
    }
}

struct FilterTest;
#[async_trait::async_trait]
impl HttpFilter for FilterTest {
    async fn request(&self, _ctx: Context, req: Request<Body>) -> Result<Request<Body>, Response<Body>> {
        wd_log::log_debug_ln!("FilterTest request");
        Ok(req)
    }

    async fn response(&self, _ctx: Context, resp: Response<Body>) -> Response<Body> {
        wd_log::log_debug_ln!("FilterTest response");
        resp
    }
}