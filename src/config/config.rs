use serde::{Deserialize, Serialize};
use std::path::Path;

macro_rules! field_generate {
    ($cfg:tt; $($name:tt,$ty:ty,$default:expr,$default_desc:tt);*) => {
        #[derive(Debug,Serialize,Deserialize,Clone)]
        pub struct $cfg{
            $(
            #[serde(default=$default_desc)]
            pub $name : $ty,
            )*

        }
        impl $cfg{
            $(
            fn $name()->$ty{
                $default
            }
            )*
        }
        impl Default for $cfg{
            fn default() -> Self {
                Self{
                $(
                    $name : $default,
                )*
                }
            }
        }
    };
}

field_generate!(Server;
    name,String,String::from("rust-grpc-proxy"),"Server::name";
    addr,String,String::from("0.0.0.0:6789"),"Server::addr"
    // control_pre,String,String::from("/grpc/proxy"),"Server::control_pre"
);
field_generate!(Log;
    level,String,String::from("debug"),"Log::level";
    show_time,bool,true,"Log::show_time";
    show_file_line,bool,false,"Log::show_file_line";
    out_file_path,String,String::new(),"Log::out_file_path"
);

field_generate!(DynamicSink;
    enable,bool,false,"DynamicSink::enable";
    addr,String,String::from("0.0.0.0:6790"),"DynamicSink::addr"
);

field_generate!(ProxySink;
    name,String,String::from("default"),"ProxySink::name";
    addr,String,String::from(""),"ProxySink::addr";
    prefix,String,String::from("/"),"ProxySink::prefix"
);

field_generate!(MetadataFilter;
    prefix,Vec<String>,vec![String::from("md-")],"MetadataFilter::prefix";
    r#match,Vec<String>,vec![],"MetadataFilter::r#match";
    response_show_server,bool,true,"MetadataFilter::response_show_server";
    response_default_content_type,String,"application/json".into(),"MetadataFilter::response_default_content_type" //如果grpc response中未指明 `http-content-type`,则使用此字段作为返回内容类型
);

field_generate!(EnvSink;
    disable,bool,true,"EnvSink::disable";
    addr_env_key,String,String::from("RUST_GRPC_PROXY_ADDR"),"EnvSink::addr_env_key";
    wait_time_max_sec,u64,300,"EnvSink::wait_time_max_sec";
    interval_sec,u64,10,"EnvSink::interval_sec";
    prefix,String,String::from("/"),"EnvSink::prefix"
);

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "Server::default")]
    pub server: Server,
    #[serde(default = "Log::default")]
    pub log: Log,
    #[serde(default = "Vec::new")]
    pub proxy_sink: Vec<ProxySink>,
    #[serde(default = "DynamicSink::default")]
    pub dynamic_sink: DynamicSink,
    #[serde(default = "MetadataFilter::default")]
    pub metadata_filters: MetadataFilter,
    #[serde(default = "EnvSink::default")]
    pub env_sink: EnvSink,
}

impl Config {
    pub fn from_file_by_path(path: impl AsRef<Path>) -> anyhow::Result<Config> {
        match wd_run::load_config(path) {
            Err(e) => return Err(anyhow::anyhow!(e)),
            Ok(o) => Ok(o),
        }
    }
}

impl ToString for Config {
    fn to_string(&self) -> String {
        match serde_json::to_string(self) {
            Ok(o) => o,
            Err(e) => e.to_string(),
        }
    }
}
