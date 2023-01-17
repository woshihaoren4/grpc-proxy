mod http_server;
mod filter_middle_req_id;
mod filter_middle_log;
mod filter_middle_time;
mod shutdown_control;

use std::future::Future;
use hyper::{Body, Request, Response};
use wd_run::Context;
pub use http_server::{HyperHttpServer, HyperHttpServerBuilder};
pub use filter_middle_req_id::RequestIdMiddle;
pub use filter_middle_log::LogMiddle;
pub use filter_middle_time::TimeMiddle;
pub use shutdown_control::ShutDown;

#[async_trait::async_trait]
pub trait HttpHandle{
    async fn handle(&self,ctx:Context,req:Request<Body>)->anyhow::Result<Response<Body>>;
}

#[allow(unused_variables)]
#[async_trait::async_trait]
pub trait HttpFilter : Sync{
    async fn request(&self,ctx:Context,req:Request<Body>)->Result<Request<Body>,Response<Body>>{
        Ok(req)
    }
    async fn response(&self,ctx:Context,resp:Response<Body>)->Response<Body>{
        resp
    }
}

#[async_trait::async_trait]
impl<T,F> HttpHandle for T
    where T: Fn(Context,Request<Body>)-> F + Send + Sync,
        F: Future<Output=anyhow::Result<Response<Body>>> + Send,
{
    async fn handle(&self, ctx: Context, req: Request<Body>) -> anyhow::Result<Response<Body>> {
        self(ctx,req).await
    }
}

#[cfg(test)]
mod test{
    use hyper::{Body, Request, Response};
    use crate::infra::server::HyperHttpServerBuilder;

    #[tokio::test(flavor ="multi_thread", worker_threads = 2)]
    async fn test_http_server(){
        HyperHttpServerBuilder::new().handle(|_c,_r:Request<Body>|async move{
            Ok(Response::new(Body::from("success")))
        }).run().await.expect("http服务报错");
    }
}