use super::{JsonProtoTransition, PathIndex};
use hyper::body::Bytes;
use hyper::client::{connect::HttpConnector, Client};
use hyper::{Body, Method};
use protobuf::reflect::MessageDescriptor;
use std::collections::HashMap;

pub struct DynClient {
    name: String,
    host_port: String,
    protocol: String,
    client: Client<HttpConnector>,
    format: Box<dyn JsonProtoTransition + Send + Sync + 'static>,
    // restful: Box<dyn RestfulTransition + Send + Sync + 'static>,
    index: Box<dyn PathIndex + Send + Sync + 'static>,
}

impl DynClient {
    #[allow(dead_code)]
    pub fn new<J, P>(format: J, index: P) -> Self
    where
        J: JsonProtoTransition + Send + Sync + 'static,
        P: PathIndex + Send + Sync + 'static,
    {
        let name = "dyn-grpc-client".into();
        let client = hyper::Client::builder().http2_only(true).build_http();
        let format = Box::new(format);
        let index = Box::new(index);
        let host_port = String::from("127.0.0.1:443");
        let protocol = J::protocol();
        Self {
            name,
            host_port,
            protocol,
            client,
            format,
            index,
        }
    }
    #[allow(dead_code)]
    pub fn set_host_port<T: Into<String>>(mut self, host_post: T) -> Self {
        self.host_port = host_post.into();
        self
    }
    #[allow(dead_code)]
    pub fn name(&self) -> String {
        self.name.clone()
    }
    #[allow(dead_code)]
    pub fn set_name<T: ToString>(mut self, name: T) -> Self {
        self.name = name.to_string();
        self
    }
    #[allow(dead_code)]
    pub fn method_list(&self) -> Vec<(Method, String, String)> {
        self.index.list()
    }
    #[allow(dead_code)]
    fn error<T: Into<Vec<u8>>>(s: T) -> anyhow::Result<(HashMap<String, String>, Vec<u8>)> {
        return Ok((HashMap::new(), s.into()));
    }
}

// #[async_trait::async_trait]
impl DynClient {
    #[allow(dead_code)]
    pub async fn invoke(
        &self,
        method: Method,
        path: String,
        metadata: HashMap<String, String>,
        body: Vec<u8>,
        extend: Option<HashMap<String, String>>,
    ) -> anyhow::Result<(HashMap<String, String>, Vec<u8>)> {
        let (grpc_path, desc, restful) = if let Some(o) = self.index.search(method, path) {
            o
        } else {
            return DynClient::error("not found");
        };
        let extend = if let Some(mut mp) = extend {
            if let Some(rf) = restful {
                for (k, v) in rf {
                    mp.insert(k, v);
                }
            }
            Some(mp)
        } else {
            restful
        };
        let body = self.json_request_to_grpc(body, desc.input_type(), extend)?;
        let (status, md, resp_body) = self.do_grpc_request(grpc_path, metadata, body).await?;
        if status != 200 {
            let resp_result = String::from_utf8_lossy(resp_body.to_vec().as_slice()).to_string();
            return DynClient::error(format!("status:{} error:{}", status, resp_result));
        }
        let resp_json_body = self.grpc_response_to_json(resp_body, desc.output_type())?;
        return Ok((md, resp_json_body));
    }

    pub fn json_request_to_grpc(
        &self,
        // path: String,
        body: Vec<u8>,
        desc: MessageDescriptor,
        extend: Option<HashMap<String, String>>,
    ) -> anyhow::Result<Vec<u8>> {
        // let ps = self.restful.path(path);
        let body = self.format.json_to_proto(body, desc, extend)?;
        Ok(body)
        // let mut buf = vec![0];
        // let mut len = (body.len() as u32).to_be_bytes().to_vec();
        // buf.append(&mut len);
        // buf.append(&mut body);
        // return Ok(buf)
    }
    pub fn grpc_response_to_json(
        &self,
        body: Bytes,
        desc: MessageDescriptor,
    ) -> anyhow::Result<Vec<u8>> {
        if body.is_empty() {
            return Err(anyhow::anyhow!("response is nil"));
        }
        // body.advance(1);
        // let len = body.get_u32();
        // let buf = body.split_to(len as usize);
        let buf = body.to_vec();
        self.format.proto_to_json(buf, desc)
    }
    pub async fn do_grpc_request(
        &self,
        grpc_path: String,
        metadata: HashMap<String, String>,
        body: Vec<u8>,
    ) -> anyhow::Result<(u16, HashMap<String, String>, Bytes)> {
        let url = format!("http://{}/{}", self.host_port, grpc_path);
        // wd_log::log_debug_ln!("do_grpc_request url:{}",url);
        let mut req = hyper::Request::builder()
            .version(hyper::Version::HTTP_2)
            .method(Method::POST)
            .header(
                "content-type",
                format!("application/grpc+{}", &self.protocol),
            )
            .uri(url.as_str());
        for (k, v) in metadata.into_iter() {
            req = req.header(k, v);
        }
        let req = req.body(Body::from(body))?;
        let resp = self.client.request(req).await?;
        let status = resp.status().as_u16();
        let mut metadata = HashMap::new();
        for (k, v) in resp.headers() {
            let value = match v.to_str() {
                Ok(o) => o.to_string(),
                Err(e) => {
                    wd_log::log_error_ln!(
                        "Url:[{}] do_grpc_request metadata parse failed:{}",
                        url,
                        e
                    );
                    continue;
                }
            };
            metadata.insert(k.to_string(), value);
        }
        let body = hyper::body::to_bytes(resp.into_body()).await?;
        return Ok((status, metadata, body));
    }
}
