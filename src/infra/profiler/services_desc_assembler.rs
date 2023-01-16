use std::collections::HashMap;
use protobuf::descriptor::FileDescriptorProto;
use protobuf::reflect::ServiceDescriptor;
use crate::infra::profiler::ServiceDescriptorAssembler;

pub struct ServiceDescriptorAssemblerDefaultImpl;

impl ServiceDescriptorAssembler for ServiceDescriptorAssemblerDefaultImpl{
    fn assemble(&self, input: HashMap<String, Vec<FileDescriptorProto>>) -> anyhow::Result<HashMap<String, ServiceDescriptor>> {
        let mut map = HashMap::new();
        for (k,v) in input.into_iter(){
            let service_name = k.split('.').map(|x|{x.to_string()}).collect::<Vec<String>>();
            let files = protobuf::reflect::FileDescriptor::new_dynamic_fds(v, &[])?;
            for i in files.into_iter() {
                for x in i.services().into_iter() {
                    let sn = if let Some(ref sn) = x.proto().name {
                        sn
                    }else{continue};
                    if sn.eq(&service_name[1]) {
                        map.insert(k.to_string(),x);
                    }
                }
            }
        }
        Ok(map)
    }
}