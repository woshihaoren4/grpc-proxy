use crate::infra::dynamic::{DynClient, JsonProtoTransitionDefaultImpl, SimpleIndex};
use crate::infra::profiler::FileDescProfiler;
use hyper::Method;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use wd_run::{CmdInfo, Context};

#[derive(Default)]
pub struct TestExampleBuilder {
    examples: HashMap<String, Box<dyn wd_run::EventHandle + Send + Sync + 'static>>,
}
impl TestExampleBuilder {
    pub fn new() -> Self {
        Self {
            examples: HashMap::new(),
        }
    }

    pub fn init_examples(mut self) -> Self {
        self.examples
            .insert("dyn".into(), Box::new(TestExampleDynClient));
        self
    }
    pub fn build(self) -> TestExample {
        TestExample {
            examples: Arc::new(self.examples),
        }
    }
}

#[derive(Default)]
pub struct TestExample {
    examples: Arc<HashMap<String, Box<dyn wd_run::EventHandle + Send + Sync + 'static>>>,
}
impl TestExample {
    pub fn args(&self) -> CmdInfo {
        let mut ms: String = "all".into();
        for (i, _) in self.examples.iter() {
            ms.push_str(",");
            ms.push_str(i)
        }
        CmdInfo::new("test", "test example").add("m", "all", format!("test models: {}", ms))
    }
}

impl wd_run::EventHandle for TestExample {
    fn handle(&self, ctx: Context) -> Pin<Box<dyn Future<Output = Context> + Send>> {
        let examples = self.examples.clone();
        Box::pin(async move {
            let order = ctx.copy::<_, String>("m").await.unwrap();
            let ods = order
                .split(",")
                .map(|x| x.to_string())
                .collect::<Vec<String>>();
            wd_log::log_info_ln!("start test");
            if order.eq("all") {
                //测试全部
                for (m, func) in examples.iter() {
                    wd_log::log_info_ln!("------> module:[{}] test start", m);
                    func.handle(ctx.clone()).await;
                    wd_log::log_info_ln!("------> module:[{}] test success", m);
                }
            } else {
                for (m, func) in examples.iter() {
                    if ods.contains(m) {
                        wd_log::log_info_ln!("------> module:[{}] test start", m);
                        func.handle(ctx.clone()).await;
                        wd_log::log_info_ln!("------> module:[{}] test success", m);
                    }
                }
            }
            wd_log::log_info_ln!("test over");
            return ctx;
        })
    }
}

struct TestExampleDynClient;

impl wd_run::EventHandle for TestExampleDynClient {
    fn handle(&self, ctx: Context) -> Pin<Box<dyn Future<Output = Context> + Send>> {
        Box::pin(async move {
            let index = FileDescProfiler::new()
                .map("127.0.0.1:666".into(), SimpleIndex::parse)
                .await
                .expect("parse grpc index from reflect failed");
            wd_log::log_info_ln!("FileDescProfiler reflect success");
            let client = DynClient::new(JsonProtoTransitionDefaultImpl, index)
                .set_host_port("127.0.0.1:666");
            let request_body = Vec::from(r#"{"request":"hello"}"#);
            let (_, resp) = client
                .invoke(
                    Method::POST,
                    "/api/v2/hello".into(),
                    HashMap::new(),
                    request_body,
                )
                .await
                .expect("invoke grpc request failed");
            let resp_body = String::from_utf8_lossy(resp.as_slice()).to_string();
            wd_log::log_info_ln!("response body:{}", resp_body);
            assert_eq!(
                resp_body, r#"{"response": "hello world"}"#,
                "grpc response body is error"
            );
            return ctx;
        })
    }
}
