use std::collections::HashMap;
use std::sync::Arc;
use hyper::{Body, Request, Response};
use wd_run::Context;
use crate::app::{DynMap, QueryAnalysis};
use crate::infra::server::HttpHandle;

pub struct AppEntity{
    map: Arc<dyn DynMap>,
    query: Arc<dyn QueryAnalysis>,
}

impl AppEntity {
    pub fn new(map: Arc<dyn DynMap>,query:Arc<dyn QueryAnalysis>)->Self{
        Self{map,query}
    }
    pub fn response<B>(status:u16,body:B)->anyhow::Result<Response<Body>>
    where Body: From<B>
    {
        let body = Body::from(body);
        let resp = Response::builder().status(status).body(body)?;Ok(resp)
    }

    pub fn error<T:ToString>(t:T)->anyhow::Result<Response<Body>>{
        let body = Body::from(t.to_string());
        let resp = Response::builder().status(500).body(body)?;Ok(resp)
    }

}

#[async_trait::async_trait]
impl HttpHandle for AppEntity{
    async fn handle(&self, _ctx: Context, req: Request<Body>) -> anyhow::Result<Response<Body>> {
        let method = req.method().clone();
        let path = req.uri().path().to_string();
        let query = req.uri().query().unwrap_or("");
        let query = self.query.analysis(query);
        let metadata = HashMap::new();
        let body = req.into_body();
        let body = match hyper::body::to_bytes(body).await{
            Ok(o)=>o.to_vec(),
            Err(e)=>return AppEntity::error(e.to_string()),
        };

        let client = match self.map.get(path.clone()){
            None => return AppEntity::response(404,"not found"),
            Some(c) => c,
        };

        //todo 需要将query 拼接到option中
        let resp_content = match client.invoke(method, path, metadata, body,query).await{
            Ok((_,o)) => o,
            Err(e) => return AppEntity::error(e.to_string()),
        };
        AppEntity::response(200,resp_content)
    }
}