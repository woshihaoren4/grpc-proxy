pub mod anno;
mod client;
mod compressed_dictionary_tree;
mod simple_index;
mod transition;

pub use client::DynClient;
pub use simple_index::SimpleIndex;
pub use transition::JsonProtoTransitionDefaultImpl;

use hyper::Method;
use protobuf::reflect::{MessageDescriptor, MethodDescriptor};
use std::collections::HashMap;
use std::sync::Arc;

pub trait JsonProtoTransition {
    fn protocol() -> String
    where
        Self: Sized,
    {
        "proto".into()
    } //proto json ...
    fn json_to_proto(
        &self,
        data: Vec<u8>,
        pt: MessageDescriptor,
        opt: Option<HashMap<String, String>>,
    ) -> anyhow::Result<Vec<u8>>;
    fn proto_to_json(&self, data: Vec<u8>, pt: MessageDescriptor) -> anyhow::Result<Vec<u8>>;
}

pub trait PathIndex {
    fn search(&self, method: Method, path: String) -> Option<(String, Arc<MethodDescriptor>,Option<HashMap<String,String>>)>; //返回grpc路径  package.services/method
    fn list(&self)->Vec<(Method,String,String)>{vec![]}
}

pub trait RestfulTransition {
    //grpc option中 restful语法的支持
    #[allow(unused_variables)]
    fn path(&self, path: String) -> Option<HashMap<String, String>> {
        return None;
    }
}

pub struct RestfulTransitionDefaultImpl;
impl RestfulTransition for RestfulTransitionDefaultImpl {}
