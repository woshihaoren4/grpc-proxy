use crate::infra::dynamic::JsonProtoTransition;
use protobuf::reflect::{MessageDescriptor, ReflectValueBox};
use std::collections::HashMap;

pub struct JsonProtoTransitionDefaultImpl;

impl JsonProtoTransition for JsonProtoTransitionDefaultImpl {
    fn json_to_proto(
        &self,
        data: Vec<u8>,
        pt: MessageDescriptor,
        opt: Option<HashMap<String, String>>,
    ) -> anyhow::Result<Vec<u8>> {
        let s = String::from_utf8(data)?;
        let mut message_dyn = protobuf_json_mapping::parse_dyn_from_str(&pt, s.as_str())?;
        //将opt组装进去
        if let Some(mp) = opt {
            for (k,v) in mp.into_iter(){
                if let Some(field) = pt.field_by_name(k.as_str()) {
                    //todo 需要变成类型，第一版都为String
                    field.set_singular_field(&mut *message_dyn,ReflectValueBox::String(v));
                }
            }
        }
        let mut body = message_dyn.write_to_bytes_dyn()?;
        //组装
        let mut buf = vec![0];
        let mut len = (body.len() as u32).to_be_bytes().to_vec();
        buf.append(&mut len);
        buf.append(&mut body);
        return Ok(buf);
    }

    fn proto_to_json(&self, data: Vec<u8>, pt: MessageDescriptor) -> anyhow::Result<Vec<u8>> {
        if data.len() < 5 {
            return Err(anyhow::anyhow!("proto_to_json: data len < 5"));
        }
        let msg = pt.parse_from_bytes(&data[5..])?;
        let s = protobuf_json_mapping::print_to_string(&*msg)?;
        Ok(s.into_bytes())
    }
}
