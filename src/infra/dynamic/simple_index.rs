use crate::infra::dynamic::anno::http_rule::Pattern;
use crate::infra::dynamic::anno::HttpRule;
use crate::infra::dynamic::PathIndex;
use hyper::Method;
use protobuf::reflect::{MethodDescriptor, ServiceDescriptor};
use protobuf::UnknownValueRef;
use std::collections::HashMap;
use std::sync::Arc;

struct Node {
    method: Method,
    http_path: String,
    http_path_list: Vec<String>,
    http_path_match: Vec<String>,
    grpc_path: String,
    desc: Arc<MethodDescriptor>,
}

impl Node {
    // pub fn path_match(
    //     &self,
    //     method: &Method,
    //     path: &String,
    // ) -> Option<(String, Arc<MethodDescriptor>)> {
    //     if self.http_path.eq(path) && self.method.eq(method) {
    //         Some((self.grpc_path.clone(), self.desc.clone()))
    //     } else {
    //         None
    //     }
    // }

    pub fn path_match_restful(
        &self,
        method: &Method,
        path: &Vec<&str>,
    ) -> Option<(
        String,
        Arc<MethodDescriptor>,
        Option<HashMap<String, String>>,
    )> {
        if method != self.method {
            return None;
        }
        let mut opt: Option<HashMap<String, String>> = None;
        for (i, v) in self.http_path_match.iter().enumerate() {
            let p = if let Some(p) = path.get(i) {
                *p
            } else {
                return None;
            };
            if v.as_str() == "*" {
                let key = self.http_path_list[i].clone();
                let value = p.to_string();
                if let Some(ref mut mp) = &mut opt {
                    mp.insert(key, value);
                } else {
                    let mp = HashMap::from([(key, value)]);
                    opt = Some(mp);
                }
            } else if v != p {
                return None;
            }
        }
        return Some((self.grpc_path.clone(), self.desc.clone(), opt));
    }

    //SimpleIndex 只是一个简单实现的路由模块,所有此处魔数硬编码 详见如下文档
    //https://github.com/googleapis/googleapis/blob/master/google/api/annotations.proto
    #[allow(dead_code)]
    fn from_method_descriptor(
        service_name: String,
        desc: MethodDescriptor,
    ) -> anyhow::Result<Option<Self>> {
        let method_name = desc.proto().name.clone().unwrap_or(String::from("none"));
        let option = if let Some(s) = desc.proto().options.as_ref() {
            s
        } else {
            return Ok(None);
        };
        let value = match option.special_fields.unknown_fields().get(72295728) {
            // UnknownValueRef::LengthDelimited(b) => b,
            // _=>return None,
            None => return Ok(None),
            Some(s) => s,
        };
        let value = match value {
            UnknownValueRef::LengthDelimited(s) => s,
            _ => return Ok(None),
        };
        let hp: HttpRule = prost::Message::decode(value)?;
        Ok(Node::from_http_rule(service_name, method_name, hp, desc))
    }
    #[allow(dead_code)]
    fn from_http_rule(
        service_name: String,
        method_name: String,
        hp: HttpRule,
        desc: MethodDescriptor,
    ) -> Option<Self> {
        let (method, http_path) = match hp.pattern? {
            Pattern::Get(p) => (Method::GET, p),
            Pattern::Put(p) => (Method::PUT, p),
            Pattern::Post(p) => (Method::POST, p),
            Pattern::Delete(p) => (Method::DELETE, p),
            Pattern::Patch(p) => (Method::PATCH, p),
            Pattern::Custom(_) => return None, //SimpleIndex 暂时不支持自定义方法名
        };
        let mut http_path_list = vec![];
        let mut http_path_match = vec![];
        let list: Vec<&str> = http_path.split('/').collect();
        for i in list.into_iter() {
            if i.starts_with('{') && i.ends_with('}') {
                let filters: &[_] = &['{', '}'];
                http_path_match.push("*".into());
                http_path_list.push(i.trim_matches(filters).to_string());
            } else {
                http_path_match.push(i.to_string());
                http_path_list.push(i.into());
            }
        }
        let grpc_path = format!("{}/{}", service_name, method_name);
        let desc = Arc::new(desc);
        Some(Self {
            method,
            http_path,
            http_path_list,
            http_path_match,
            grpc_path,
            desc,
        })
    }
}

//简单的路径分类工具
//暂时不支持restful
pub struct SimpleIndex {
    nodes: Vec<Node>,
}

impl SimpleIndex {
    #[allow(dead_code)]
    fn new() -> Self {
        Self { nodes: vec![] }
    }
    #[allow(dead_code)]
    fn append_node(&mut self, node: Node) {
        self.nodes.push(node);
    }
}

impl SimpleIndex {
    //解析
    #[allow(dead_code)]
    pub(crate) fn parse(mp: HashMap<String, ServiceDescriptor>) -> anyhow::Result<Self> {
        let mut index = Self::new();
        for (service, desc) in mp.into_iter() {
            for i in desc.methods() {
                if let Some(node) = Node::from_method_descriptor(service.clone(), i)? {
                    index.append_node(node);
                }
            }
        }
        Ok(index)
    }
}

impl PathIndex for SimpleIndex {
    fn search(
        &self,
        method: Method,
        path: String,
    ) -> Option<(
        String,
        Arc<MethodDescriptor>,
        Option<HashMap<String, String>>,
    )> {
        let path_list: Vec<&str> = path.split('/').collect();
        for n in self.nodes.iter() {
            if let Some(s) = n.path_match_restful(&method, &path_list) {
                return Some(s);
            }
        }
        None
    }

    fn list(&self) -> Vec<(Method, String, String)> {
        let mut list = vec![];
        for i in self.nodes.iter() {
            let node = (i.method.clone(), i.http_path.clone(), i.grpc_path.clone());
            list.push(node);
        }
        list
    }
}
