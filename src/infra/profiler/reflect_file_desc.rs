use crate::infra::profiler::server_reflection_client::ServerReflectionClient;
use crate::infra::profiler::server_reflection_request::MessageRequest;
use crate::infra::profiler::server_reflection_response::MessageResponse;
use crate::infra::profiler::services_desc_assembler::ServiceDescriptorAssemblerDefaultImpl;
use crate::infra::profiler::{
    ServerReflectionRequest, ServiceDescriptorAssembler, ServicesFilter, ServicesFilterDefaultImpl,
};
use protobuf::descriptor::FileDescriptorProto;
use protobuf::reflect::ServiceDescriptor;
use protobuf::Message;
use std::collections::HashMap;
use tokio_stream::StreamExt;
use tonic::transport::Channel;

pub struct FileDescProfiler {
    services_filter: Vec<Box<dyn ServicesFilter + Send + Sync + 'static>>,
    services_desc_assemble: Box<dyn ServiceDescriptorAssembler + Send + Sync + 'static>,
}

impl FileDescProfiler {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let services_filter: Vec<Box<dyn ServicesFilter + Send + Sync + 'static>> =
            vec![Box::new(ServicesFilterDefaultImpl)];
        let services_desc_assemble = Box::new(ServiceDescriptorAssemblerDefaultImpl);
        Self {
            services_filter,
            services_desc_assemble,
        }
    }
    #[allow(dead_code)]
    pub fn set_services_filter<
        F: FnOnce(&mut Vec<Box<dyn ServicesFilter + Send + Sync + 'static>>),
    >(
        mut self,
        function: F,
    ) -> Self {
        function(&mut self.services_filter);
        self
    }
    #[allow(dead_code)]
    pub fn set_assemble<S: ServiceDescriptorAssembler + Send + Sync + 'static>(
        mut self,
        assembler: S,
    ) -> Self {
        self.services_desc_assemble = Box::new(assembler);
        self
    }
}

impl FileDescProfiler {
    //reflect grpc desc
    #[allow(dead_code)]
    pub async fn reflect(
        &self,
        server: String,
    ) -> anyhow::Result<HashMap<String, ServiceDescriptor>> {
        let mut client = ServerReflectionClient::connect(format!("http://{}", &server)).await?;
        // wd_log::log_debug_ln!("FileDescProfiler: connect target:{} success",server);
        let mut services = FileDescProfiler::get_service_list(&mut client).await?;
        for i in self.services_filter.iter() {
            i.filter(&mut services);
        }
        wd_log::log_debug_ln!("[{}] reflect success --> {:?}",&server,services);
        let map = FileDescProfiler::get_file_desc_by_services(&mut client, services).await?;
        let services = self.services_desc_assemble.assemble(map)?;
        Ok(services)
    }
    #[allow(dead_code)]
    pub async fn map<T, F>(&self, server: String, f: F) -> anyhow::Result<T>
    where
        F: Fn(HashMap<String, ServiceDescriptor>) -> anyhow::Result<T> + Send,
        T: Send,
    {
        let list = self.reflect(server).await?;
        f(list)
    }
}
impl FileDescProfiler {
    async fn get_service_list(
        client: &mut ServerReflectionClient<Channel>,
    ) -> anyhow::Result<Vec<String>> {
        let req_stream = tokio_stream::once(ServerReflectionRequest {
            host: String::default(),
            message_request: Some(MessageRequest::ListServices("*".into())),
        });
        let mut resp = client
            .server_reflection_info(req_stream)
            .await
            .unwrap()
            .into_inner();
        let mut list = vec![];
        while let Some(s) = resp.next().await {
            match s {
                Ok(o)=>{
                    if let MessageResponse::ListServicesResponse(s) = o.message_response.unwrap() {
                        for i in s.service.into_iter() {
                            list.push(i.name)
                        }
                    }
                    break
                }
                Err(e)=>{
                    return Err(anyhow::anyhow!("get_service_list error: status:[{}] message:{}",e.code(),e.message()));
                }
            };
            // if let MessageResponse::ListServicesResponse(s) = s.unwrap().message_response.unwrap() {
            //     for i in s.service.into_iter() {
            //         list.push(i.name)
            //     }
            // }
        }
        return Ok(list);
    }
    async fn get_file_desc_by_services(
        client: &mut ServerReflectionClient<Channel>,
        services: Vec<String>,
    ) -> anyhow::Result<HashMap<String, Vec<FileDescriptorProto>>> {
        let mut map = HashMap::new();
        let stream = services
            .iter()
            .map(|x| ServerReflectionRequest {
                host: String::default(),
                message_request: Some(MessageRequest::FileContainingSymbol(x.to_string())),
            })
            .collect::<Vec<ServerReflectionRequest>>();
        let req_stream = tokio_stream::iter(stream);
        let mut resp = client
            .server_reflection_info(req_stream)
            .await?
            .into_inner();
        let mut i = 0;
        while let Some(s) = resp.next().await {
            let resp = match s {
                Ok(o) => o,
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "get_file_desc_by_services error,status:{},message:{}",
                        e.code(),
                        e.message()
                    ));
                }
            };
            let file_desc_resp = resp.message_response.unwrap();
            match file_desc_resp {
                MessageResponse::FileDescriptorResponse(s) => {
                    let mut list = vec![];
                    for buf in s.file_descriptor_proto.into_iter() {
                        let fdp = FileDescriptorProto::parse_from_bytes(buf.as_slice())?;
                        list.push(fdp);
                    }
                    map.insert(services[i].clone(), list);
                }
                _ => {}
            }
            i += 1;
        }
        Ok(map)
    }
}
