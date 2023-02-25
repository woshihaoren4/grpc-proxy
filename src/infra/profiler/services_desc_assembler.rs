use crate::infra::profiler::ServiceDescriptorAssembler;
use protobuf::descriptor::FileDescriptorProto;
use protobuf::reflect::ServiceDescriptor;
use std::collections::HashMap;

pub struct ServiceDescriptorAssemblerDefaultImpl;

impl ServiceDescriptorAssembler for ServiceDescriptorAssemblerDefaultImpl {
    fn assemble(
        &self,
        input: HashMap<String, Vec<FileDescriptorProto>>,
    ) -> anyhow::Result<HashMap<String, ServiceDescriptor>> {
        let mut map = HashMap::new();
        // for (k, v) in input.into_iter() {
        //     let service_name = k.split('.').map(|x| x.to_string()).collect::<Vec<String>>();
        //     let files = protobuf::reflect::FileDescriptor::new_dynamic_fds(v, &[])?;
        //     for i in files.into_iter() {
        //         for x in i.services().into_iter() {
        //             let sn = if let Some(ref sn) = x.proto().name {
        //                 sn
        //             } else {
        //                 continue;
        //             };
        //             if sn.eq(&service_name[1]) {
        //                 map.insert(k.to_string(), x);
        //             }
        //         }
        //     }
        // }
        let mut services = HashMap::new();
        let mut files = HashMap::new();
        for (k, list) in input.into_iter() {
            let name = k.split('.').map(|x| x).collect::<Vec<&str>>();
            services.insert(name[1].to_string(), k);
            for i in list.into_iter() {
                files.insert(i.name().to_string(), i);
            }
        }
        let files = files
            .into_iter()
            .map(|x| x.1)
            .collect::<Vec<FileDescriptorProto>>();
        let desc_files = protobuf::reflect::FileDescriptor::new_dynamic_fds(files, &[])?;
        for i in desc_files.into_iter() {
            for sd in i.services().into_iter() {
                let sn = if let Some(ref sn) = sd.proto().name {
                    sn
                } else {
                    continue;
                };
                if let Some(s) = services.get(sn) {
                    map.insert(s.to_string(), sd);
                }
            }
        }
        Ok(map)
    }
}
