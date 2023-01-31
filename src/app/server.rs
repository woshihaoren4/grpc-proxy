use std::collections::HashMap;
use std::sync::Arc;
use hyper::{Body, Request, Response};
use wd_run::Context;
use crate::app::DynMap;
use crate::infra::dynamic::DynClient;
use crate::infra::server::HttpHandle;

pub struct AppEntity{
    map: Arc<dyn DynMap>
}

impl AppEntity {
    pub fn new<T>(map: Arc<dyn DynMap>)->Self{
        Self{map}
    }
}

#[async_trait::async_trait]
impl HttpHandle for AppEntity{
    async fn handle(&self, ctx: Context, req: Request<Body>) -> anyhow::Result<Response<Body>> {
        let method = req.method().clone();
        let path = req.uri().path().to_string();
        let metadata = HashMap::new();
        let body = match hyper::body::to_bytes(req.body().into()).await{
            Ok(o)=>o.to_vec(),
            Err(e)=>return AppEntity::error(e.to_string()),
        };

        let client = match self.map.get(path.clone()){
            None => return AppEntity::response(404,"not found"),
            Some(c) => c,
        };

        let resp_content = match client.invoke(method, path, metadata, body).await{
            Ok((_,o)) => o,
            Err(e) => return AppEntity::error(e.to_string()),
        };
        AppEntity::response(200,resp_content)
    }
}