use std::future::Future;
use std::pin::Pin;
use wd_run::{CmdInfo, Context};

#[derive(Default)]
pub struct RunApplication{}

impl RunApplication{
    pub fn new()->Self{
        Self{}
    }
    pub fn args() -> CmdInfo {
        CmdInfo::new("run", "run application").add(
            "c",
            "./src/conf/config.toml",
            "config file path",
        )
    }
}

impl wd_run::EventHandle for RunApplication {
    fn handle(&self, ctx: Context) -> Pin<Box<dyn Future<Output=Context> + Send>> {
        Box::pin(async move {
            wd_log::log_debug_ln!("run application success");
            return ctx
        })
    }
}