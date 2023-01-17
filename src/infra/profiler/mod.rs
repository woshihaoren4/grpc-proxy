mod reflect;
mod reflect_file_desc;
mod services_desc_assembler;

pub use reflect::*;
pub use reflect_file_desc::FileDescProfiler;
pub use services_desc_assembler::ServiceDescriptorAssemblerDefaultImpl;

use protobuf::descriptor::FileDescriptorProto;
use protobuf::reflect::ServiceDescriptor;
use std::collections::HashMap;

pub trait ServicesFilter {
    fn filter(&self, _: &mut Vec<String>);
}

pub trait ServiceDescriptorAssembler {
    fn assemble(
        &self,
        input: HashMap<String, Vec<FileDescriptorProto>>,
    ) -> anyhow::Result<HashMap<String, ServiceDescriptor>>;
}

pub struct ServicesFilterDefaultImpl;
impl ServicesFilter for ServicesFilterDefaultImpl {
    fn filter(&self, services: &mut Vec<String>) {
        let mut i = 0;
        while i < services.len() {
            if services[i].starts_with("grpc.") {
                //grpc.开头的服务是官方接口 理论上都是不需要的
                services.remove(i);
                i += 1;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::infra::profiler::FileDescProfiler;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_file_desc_profiler() {
        let profiler = FileDescProfiler::new();
        let result = profiler.reflect("127.0.0.1:666".into()).await;
        if let Err(e) = result {
            wd_log::log_error_ln!("profiler.reflect 127.0.0.1:666 error:{}", e);
            return;
        }
        wd_log::log_debug_ln!("---> profiler.reflect success start show");
        for (k, v) in result.unwrap().into_iter() {
            wd_log::log_info_ln!("service:{} => desc:{}", k, v.proto().name.as_ref().unwrap());
            for i in v.methods() {
                wd_log::log_info_ln!(
                    "      method:{} input:{} output:{}",
                    i.proto().name.as_ref().unwrap(),
                    i.input_type().name(),
                    i.output_type().name()
                );
                // wd_log::log_info_ln!("      method:{} option:{}",i.proto().name.as_ref().unwrap(),i.proto().options.to_string());
            }
        }
        wd_log::log_debug_ln!("---> over");
    }
}
