use crate::app::DynMap;
use crate::config::{EnvSink, ProxySink};
use crate::infra::dynamic::{DynClient, JsonProtoTransitionDefaultImpl, SimpleIndex};
use crate::infra::profiler::FileDescProfiler;
use std::sync::Arc;

pub async fn init_proxy_sink(dm: Arc<dyn DynMap>, ps: Vec<ProxySink>) {
    for i in ps.into_iter() {
        let client = wd_log::res_panic!(init_dyn_client(i.name.clone(),i.addr.clone()).await;"init_proxy_sink: init {} failed,addr=({})",i.name,i.addr);
        dm.set(i.name, i.prefix, client);
    }
}

pub async fn init_env_sink(dm: Arc<dyn DynMap>, es: EnvSink, name: String) {
    if es.disable {
        return;
    }
    let mut i = es.interval_sec;
    let addr = match std::env::var(es.addr_env_key.as_str()) {
        Ok(o) => o,
        Err(_) => {
            return;
        }
    };
    wd_log::log_debug_ln!("load env addr: {} start init dyn client", addr);
    while i < es.wait_time_max_sec {
        tokio::time::sleep(std::time::Duration::from_secs(es.interval_sec)).await;
        let result = init_dyn_client(name.clone(), addr.clone()).await;
        match result {
            Ok(o) => {
                for i in o.method_list().iter() {
                    wd_log::log_debug_ln!("env sink ->{} {} {}", i.0, i.1, i.2);
                }
                dm.set(name, es.prefix, o);
                return;
            }
            Err(e) => {
                wd_log::log_debug_ln!("init_env_sink failed={}", e);
            }
        }
        i += es.interval_sec;
    }
    wd_log::log_error_ln!("init dyn client failed,please src server is normal");
}

pub async fn init_dyn_client(name: String, sink_addr: String) -> anyhow::Result<DynClient> {
    let index = FileDescProfiler::new()
        .map(sink_addr.clone(), SimpleIndex::parse)
        .await?;
    // .expect("parse grpc index from reflect failed");
    let client = DynClient::new(JsonProtoTransitionDefaultImpl, index)
        .set_host_port(sink_addr)
        .set_name(name);
    return Ok(client);
}
