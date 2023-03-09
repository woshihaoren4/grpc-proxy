use crate::app;
use crate::config::{Config, Log};
use crate::infra::server::ShutDown;
use std::future::Future;
use std::pin::Pin;
use wd_log::Level;
use wd_run::{CmdInfo, Context};

#[derive(Default)]
pub struct RunApplication {
    sd: ShutDown,
}

impl RunApplication {
    pub fn new() -> (Self, ShutDown) {
        let app = Self {
            sd: ShutDown::default(),
        };
        let sd = app.sd.clone();
        (app, sd)
    }
    pub fn args() -> CmdInfo {
        CmdInfo::new("run", "run application").add(
            "c",
            "./src/config/config.toml",
            "config file path",
        )
    }
    pub async fn load_config(ctx: &Context) -> Config {
        let path = ctx
            .copy::<_, String>("c")
            .await
            .expect("load config failed");
        let cfg = match Config::from_file_by_path(&path) {
            Ok(o) => o,
            Err(e) => {
                wd_log::log_info_ln!("from file:[{}] load config error={}", path, e);
                Config::default()
            }
        };
        wd_log::log_debug_ln!("config file load success:{}", cfg.to_string());
        return cfg;
    }
    pub fn init_log(app: String, log: Log) {
        wd_log::set_level(Level::from(log.level));
        // unsafe {
        let name: &'static str = Box::leak(app.into());
        wd_log::set_prefix(name);
        // }
        wd_log::show_time(log.show_time);
        wd_log::show_file_line(log.show_file_line);
        if !log.out_file_path.is_empty() {
            wd_log::output_to_file(log.out_file_path).expect("init_log output_to_file error")
        }
        wd_log::log_debug_ln!("log config init success");
    }

    pub async fn launch(cfg: Config, sd: ShutDown) {
        wd_log::log_debug_ln!("start run application");
        app::start(sd, cfg).await;
        wd_log::log_debug_ln!("run application success");
    }
    // pub async fn launch(_cfg:Config,sd:ShutDown) {
    //     let _ = HyperHttpServerBuilder::new()
    //         .handle(|_c, r: Request<Body>| async move {
    //             let method = r.method();
    //             let path = r.uri();
    //             wd_log::log_debug_ln!("method:[{}] path:[{}]", method, path);
    //             tokio::time::sleep(Duration::from_millis(100)).await;
    //             Ok(Response::new(Body::from("success")))
    //         })
    //         .append_filter(TimeMiddle)
    //         .append_filter(RequestIdMiddle::new())
    //         .append_filter(LogMiddle)
    //         // .run().await.expect("http服务报错");
    //         .set_shutdown_singe(sd)
    //         .async_run();
    // }
}

impl wd_run::EventHandle for RunApplication {
    fn handle(&self, ctx: Context) -> Pin<Box<dyn Future<Output = Context> + Send>> {
        let sd = self.sd.clone();
        Box::pin(async move {
            let cfg = RunApplication::load_config(&ctx).await;
            RunApplication::init_log(cfg.server.name.clone(), cfg.log.clone());
            RunApplication::launch(cfg, sd.clone()).await; //启动客户端
            sd.wait_close().await;
            return ctx;
        })
    }
}
