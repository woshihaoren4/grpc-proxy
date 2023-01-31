use std::sync::Arc;
use crate::infra::dynamic::DynClient;

mod server;
mod response;
mod dyn_client_source;
mod dyn_map_simple;

pub use server::AppEntity;

pub trait DynMap:Send+Sync{
    fn get(&self,path:String)->Option<Arc<DynClient>>;
    fn set(&self,path:String,dc:DynClient);
}


