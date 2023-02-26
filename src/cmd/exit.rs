use crate::infra::server::ShutDown;
use std::future::Future;
use std::pin::Pin;
use std::process::exit;
use wd_run::Context;

pub struct ExitApplication {
    server_sd: ShutDown,
}

impl ExitApplication {
    pub fn new(server_sd: ShutDown) -> Self {
        Self { server_sd }
    }
}

impl wd_run::EventHandle for ExitApplication {
    fn handle(&self, ctx: Context) -> Pin<Box<dyn Future<Output = Context> + Send>> {
        let sd = self.server_sd.clone();
        Box::pin(async move {
            wd_log::log_debug_ln!("exit signal is received, the application begins to exit");
            sd.close().await;
            wd_log::log_debug_ln!("application exit succeeded");
            exit(0);
            return ctx;
        })
    }
}
