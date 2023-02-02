use std::future::Future;
use std::pin::Pin;
use wd_run::{CmdInfo, Context};
use crate::app;
use crate::cmd::run::RunApplication;

#[derive(Default)]
pub struct Show;

impl Show {
    pub fn args() -> CmdInfo {
        CmdInfo::new("show", "show config file proxy sink method list").add(
            "c",
            "./src/config/config.toml",
            "config file path",
        )
    }
}

impl wd_run::EventHandle for Show {
    fn handle(&self, ctx: Context) -> Pin<Box<dyn Future<Output = Context> + Send>> {
        Box::pin(async move {
            let cfg = RunApplication::load_config(&ctx).await;
            RunApplication::init_log(cfg.server.name.clone(),cfg.log.clone());
            app::show(cfg).await;
            return ctx;
        })
    }
}
