use std::sync::Arc;
use crate::infra::dynamic::DynClient;

mod server;
mod proxy_sink;
mod dyn_map_simple;

pub use server::AppEntity;
pub use proxy_sink::*;
use crate::app::dyn_map_simple::DynMapDefault;
use crate::config::{Config};
use crate::infra::server::{HyperHttpServerBuilder, LogMiddle, RequestIdMiddle, ShutDown, TimeMiddle};

pub trait DynMap:Send+Sync{
    fn get(&self,path:String)->Option<Arc<DynClient>>;
    fn set(&self,name:String,path:String,dc:DynClient);
}


pub async fn start(sd:ShutDown,cfg:Config){
    let map = Arc::new(DynMapDefault::default());
    let app = AppEntity::new(map.clone());
    init_proxy_sink(map,cfg.proxy_sink).await;
    //todo 开启新的服务动态除了grpc sink变化

    let _ = HyperHttpServerBuilder::new()
        .set_addr(cfg.server.addr.parse().expect("parse config server.addr error"))
        .handle(app)
        .append_filter(TimeMiddle)
        .append_filter(RequestIdMiddle::new())
        .append_filter(LogMiddle)
        // .run().await.expect("http服务报错");
        .set_shutdown_singe(sd)
        .async_run();
}

pub async fn show(cfg:Config){
    if cfg.proxy_sink.is_empty() {
        wd_log::log_warn_ln!("config[proxy_sink] is nil");
        return;
    }
    for i in cfg.proxy_sink.iter(){
        wd_log::log_info_ln!("---------> start refect grpc server[{}] <---------",i.name);
        let client = wd_log::res_panic!(init_dyn_client(i.name.clone(),i.addr.clone()).await;"init_proxy_sink: init {} failed,addr=({})",i.name,i.addr);
        let list = client.method_list();
        for i in list.iter(){
            wd_log::log_info_ln!("{} {} {}",i.0,i.1,i.2);
        }
    }
}

