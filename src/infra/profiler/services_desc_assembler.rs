use crate::infra::profiler::ServiceDescriptorAssembler;
use protobuf::descriptor::FileDescriptorProto;
use protobuf::reflect::ServiceDescriptor;
use std::collections::HashMap;

pub struct ServiceDescriptorAssemblerDefaultImpl;

impl ServiceDescriptorAssembler for ServiceDescriptorAssemblerDefaultImpl {
    fn assemble(
        &self,
        input: HashMap<String, Vec<FileDescriptorProto>>,
        deps : Vec<Vec<FileDescriptorProto>>,
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
        // let mut dep_des = vec![];
        // for i in deps.into_iter().rev(){
        //     println!("---1");
        //     let des = protobuf::reflect::FileDescriptor::new_dynamic_fds(i, &dep_des)?;
        //     for j in des.into_iter(){
        //     println!("---2 {}",j.name());
        //         if !dep_des.iter().any(|x|x.name() == j.name()) {
        //             dep_des.push(j);
        //         }
        //     }
        // }
        let mut des = vec![];
        for i in deps.into_iter().rev(){
            for j in i.into_iter(){
                if !des.iter().any(|x:&FileDescriptorProto|x.name() == j.name()) {
                    des.push(j);
                }
            }

        }
        let deps = protobuf::reflect::FileDescriptor::new_dynamic_fds(des, &[])?;

        let mut services = HashMap::new();
        let mut files = HashMap::new();
        for (k, list) in input.into_iter() {
            let name = k.split('.').map(|x| x).collect::<Vec<&str>>();
            services.insert(name[1].to_string(), k);
            for i in list.into_iter() {
                if !files.contains_key(i.name()) {
                    files.insert(i.name().to_string(), i);
                }
            }
        }
        let files = files
            .into_iter()
            .map(|x| x.1)
            .collect::<Vec<FileDescriptorProto>>();

        let desc_files = protobuf::reflect::FileDescriptor::new_dynamic_fds(files, &deps)?;
        // println!("---> ok");
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
