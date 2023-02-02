use std::sync::Arc;
use crate::app::DynMap;
use crate::config::ProxySink;
use crate::infra::dynamic::{DynClient, JsonProtoTransitionDefaultImpl, SimpleIndex};
use crate::infra::profiler::FileDescProfiler;

pub async fn init_proxy_sink(dm:Arc<dyn DynMap>,ps:Vec<ProxySink>){
    for i in ps.into_iter(){
        let client = wd_log::res_panic!(init_dyn_client(i.name.clone(),i.addr.clone()).await;"init_proxy_sink: init {} failed,addr=({})",i.name,i.addr);
        dm.set(i.name,i.prefix,client);
    }
}

pub async fn init_dyn_client(name:String,sink_addr:String)->anyhow::Result<DynClient>{
    let index = FileDescProfiler::new()
        .map(sink_addr.clone(), SimpleIndex::parse)
        .await?;
        // .expect("parse grpc index from reflect failed");
    let client = DynClient::new(JsonProtoTransitionDefaultImpl, index)
        .set_host_port(sink_addr)
        .set_name(name);
    return Ok(client)
}