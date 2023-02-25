use crate::app::{DynMap, MetadataAnalysis, QueryAnalysis};
use crate::infra::server::HttpHandle;
use hyper::{Body, Request, Response};
use std::collections::HashMap;
use std::sync::Arc;
use wd_run::Context;

pub struct AppEntity {
    map: Arc<dyn DynMap>,
    query: Arc<dyn QueryAnalysis>,
    md_filters: Arc<dyn MetadataAnalysis>,
}

impl AppEntity {
    pub fn new(
        map: Arc<dyn DynMap>,
        query: Arc<dyn QueryAnalysis>,
        md_filters: Arc<dyn MetadataAnalysis>,
    ) -> Self {
        Self {
            map,
            query,
            md_filters,
        }
    }
    pub fn response<B>(
        status: u16,
        body: B,
        resp_headers: Option<HashMap<String, String>>,
    ) -> anyhow::Result<Response<Body>>
    where
        Body: From<B>,
    {
        let body = Body::from(body);
        let mut resp = Response::builder().status(status);
        if let Some(mp) = resp_headers {
            for (k, v) in mp.into_iter() {
                resp = resp.header(k, v);
            }
        }
        let resp = resp.body(body)?;
        Ok(resp)
    }

    pub fn error<T: ToString>(t: T) -> anyhow::Result<Response<Body>> {
        let body = Body::from(t.to_string());
        let resp = Response::builder().status(500).body(body)?;
        Ok(resp)
    }
}

#[async_trait::async_trait]
impl HttpHandle for AppEntity {
    async fn handle(&self, _ctx: Context, req: Request<Body>) -> anyhow::Result<Response<Body>> {
        let method = req.method().clone();
        let path = req.uri().path().to_string();
        let query = req.uri().query().unwrap_or("");
        let query = self.query.analysis(query);
        let metadata = self.md_filters.request(req.headers());

        let body = req.into_body();
        let body = match hyper::body::to_bytes(body).await {
            Ok(o) => o.to_vec(),
            Err(e) => return AppEntity::error(e.to_string()),
        };

        let client = match self.map.get(path.clone()) {
            None => return AppEntity::response(404, "not found", None),
            Some(c) => c,
        };

        let (resp_header, resp_body) =
            match client.invoke(method, path, metadata, body, query).await {
                Ok(o) => o,
                Err(e) => return AppEntity::error(e.to_string()),
            };
        let resp_header = self.md_filters.response(resp_header);
        AppEntity::response(200, resp_body, Some(resp_header))
    }
}
