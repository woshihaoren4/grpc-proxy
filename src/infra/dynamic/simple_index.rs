use std::collections::HashMap;
use std::sync::Arc;
use hyper::Method;
use protobuf::{  UnknownValueRef};
use protobuf::reflect::{ MethodDescriptor, ServiceDescriptor};
use crate::infra::dynamic::anno::{ HttpRule};
use crate::infra::dynamic::anno::http_rule::Pattern;
use crate::infra::dynamic::PathIndex;

struct Node{
    method: Method,
    http_path: String,
    grpc_path: String,
    desc: Arc<MethodDescriptor>,
}

impl Node {
    pub fn path_match(&self,method:&Method,path:&String)-> Option<(String,Arc<MethodDescriptor>)>{
        if self.http_path.eq(path) && self.method.eq(method){
         Some((self.grpc_path.clone(),self.desc.clone()))
        }else {
            None
        }
    }

    //SimpleIndex 只是一个简单实现的路由模块,所有此处魔数硬编码 详见如下文档
    //https://github.com/googleapis/googleapis/blob/master/google/api/annotations.proto
    #[allow(dead_code)]
    fn from_method_descriptor(service_name:String, desc: MethodDescriptor) -> anyhow::Result<Option<Self>> {
        let method_name = desc.proto().name.clone().unwrap_or(String::from("none"));
        let option = if let Some(s) = desc.proto().options.as_ref(){s}else{return Ok(None)};
        let value = match option.special_fields.unknown_fields().get(72295728) {
            // UnknownValueRef::LengthDelimited(b) => b,
            // _=>return None,
            None => return Ok(None),
            Some(s) => s,
        };
        let value = match value {
            UnknownValueRef::LengthDelimited(s) => {s}
            _=>return Ok(None)
        };
        let hp:HttpRule = prost::Message::decode(value)?;
        Ok(Node::from_http_rule(service_name,method_name,hp,desc))
    }
    #[allow(dead_code)]
    fn from_http_rule(service_name:String,method_name:String,hp:HttpRule,desc:MethodDescriptor)->Option<Self>{
        let (method,http_path) = match hp.pattern? {
            Pattern::Get(p) => (Method::GET,p),
            Pattern::Put(p) => (Method::PUT,p),
            Pattern::Post(p) => (Method::POST,p),
            Pattern::Delete(p) => (Method::DELETE,p),
            Pattern::Patch(p) => (Method::PATCH,p),
            Pattern::Custom(_) => return None, //SimpleIndex 暂时不支持自定义方法名
        };
        let grpc_path = format!("{}/{}",service_name,method_name);
        let desc = Arc::new(desc);
        Some(Self{method,http_path,grpc_path,desc})
    }
}

//简单的路径分类工具
//暂时不支持restful
pub struct SimpleIndex{
    nodes : Vec<Node>
}

impl SimpleIndex {
    #[allow(dead_code)]
    fn new()->Self{
        Self{nodes:vec![]}
    }
    #[allow(dead_code)]
    fn append_node(&mut self,node:Node){
        self.nodes.push(node);
    }
}

impl SimpleIndex{ //解析
    #[allow(dead_code)]
pub(crate) fn parse(mp:HashMap<String,ServiceDescriptor>) ->anyhow::Result<Self>{
        let mut index = Self::new();
        for (service,desc) in mp.into_iter() {
            for i in desc.methods() {
                if let Some(node) = Node::from_method_descriptor(service.clone(),i)?{
                    index.append_node(node);
                }
            }
        }
        Ok(index)
    }
}

impl PathIndex for SimpleIndex {
    fn search(&self,method:Method, path: String) -> Option<(String, Arc<MethodDescriptor>)> {
        for n in self.nodes.iter() {
            if let Some(s) = n.path_match(&method,&path) {
                return Some(s)
            }
        }
        None
    }
}
