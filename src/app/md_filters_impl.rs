use crate::app::MetadataAnalysis;
use crate::config::MetadataFilter;
use hyper::header::HeaderValue;
use hyper::HeaderMap;
use std::collections::HashMap;

const HTTP_CONTENT_TYPE:&'static str = "http-content-type";
const CONTENT_TYPE:&'static str = "content-type";

pub struct MetadataAnalysisDefaultImpl {
    prefix: Vec<String>,
    has: HashMap<String, bool>,
    show_response_server: bool,
    response_default_content_type : String,
}

impl From<&MetadataFilter> for MetadataAnalysisDefaultImpl {
    fn from(mf: &MetadataFilter) -> Self {
        let mut has = HashMap::new();
        let mut iter = mf.r#match.iter();
        while let Some(key) = iter.next() {
            has.insert(key.clone(), true);
        }
        has.insert(HTTP_CONTENT_TYPE.to_string(),true);
        let prefix = mf.prefix.clone();
        let show_response_server = mf.response_show_server;
        let response_default_content_type = mf.response_default_content_type.clone();
        Self {
            prefix,
            has,
            show_response_server,
            response_default_content_type,
        }
    }
}

impl MetadataAnalysisDefaultImpl {
    fn allow(&self, key: &str) -> bool {
        if self.has.get(key).is_some() {
            return true;
        }
        for i in self.prefix.iter() {
            if key.starts_with(i.as_str()) {
                return true;
            }
        }
        return false;
    }
}

impl MetadataAnalysis for MetadataAnalysisDefaultImpl {
    fn request(&self, headers: &HeaderMap<HeaderValue>) -> HashMap<String, String> {
        let mut header = HashMap::new();
        for (k, v) in headers.iter() {
            if !self.allow(k.as_str()) {
                continue;
            }
            match v.to_str() {
                Ok(val) => {
                    header.insert(k.to_string(), val.into());
                }
                Err(e) => {
                    wd_log::log_warn_ln!("MetadataAnalysisDefaultImpl.request error:{}", e);
                }
            };
        }
        return header;
    }

    fn response(&self, header: HashMap<String, String>) -> HashMap<String, String> {
        let mut mp = HashMap::new();
        let mut header = header.into_iter();
        while let Some((key, value)) = header.next() {
            if self.allow(key.as_str()) {
                mp.insert(key, value);
            }
        }
        if self.show_response_server {
            mp.insert("proxy_server".into(), "rust-grpc-proxy".into());
        }
        if let Some(ty) = mp.remove(HTTP_CONTENT_TYPE) {
            mp.insert(CONTENT_TYPE.to_string(),ty);
        }else{
            mp.insert(CONTENT_TYPE.to_string(),self.response_default_content_type.clone());
        }
        return mp;
    }
}
