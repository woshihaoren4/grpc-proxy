use crate::infra::dynamic::JsonProtoTransition;
use protobuf::descriptor::field_descriptor_proto::Type;
use protobuf::reflect::{MessageDescriptor, ReflectValueBox};
use std::collections::HashMap;
use protobuf_json_mapping::PrintOptions;

pub struct JsonProtoTransitionDefaultImpl;

impl JsonProtoTransition for JsonProtoTransitionDefaultImpl {
    fn json_to_proto(
        &self,
        data: Vec<u8>,
        pt: MessageDescriptor,
        opt: Option<HashMap<String, String>>,
    ) -> anyhow::Result<Vec<u8>> {
        let s = String::from_utf8(data)?;
        let mut message_dyn = if s.is_empty() {
            pt.new_instance()
        } else {
            protobuf_json_mapping::parse_dyn_from_str(&pt, s.as_str())?
        };

        //将opt组装进去
        if let Some(mp) = opt {
            for (k, v) in mp.into_iter() {
                if let Some(field) = pt.field_by_name(k.as_str()) {
                    let value = match field.proto().type_() {
                        Type::TYPE_DOUBLE => ReflectValueBox::F64(v.parse().unwrap_or(0f64)),
                        Type::TYPE_FLOAT => ReflectValueBox::F32(v.parse().unwrap_or(0f32)),
                        Type::TYPE_INT64 => ReflectValueBox::I64(v.parse().unwrap_or(0i64)),
                        Type::TYPE_UINT64 => ReflectValueBox::U64(v.parse().unwrap_or(0u64)),
                        Type::TYPE_INT32 => ReflectValueBox::I32(v.parse().unwrap_or(0i32)),
                        Type::TYPE_FIXED64 => ReflectValueBox::I64(v.parse().unwrap_or(0i64)),
                        Type::TYPE_FIXED32 => ReflectValueBox::I32(v.parse().unwrap_or(0i32)),
                        Type::TYPE_BOOL => ReflectValueBox::Bool(v.parse().unwrap_or(false)),
                        Type::TYPE_STRING => ReflectValueBox::String(v),
                        // Type::TYPE_GROUP =>{},
                        // Type::TYPE_MESSAGE =>{},
                        Type::TYPE_BYTES => ReflectValueBox::Bytes(v.into_bytes()),
                        Type::TYPE_UINT32 => ReflectValueBox::U32(v.parse().unwrap_or(0u32)),
                        // Type::TYPE_ENUM =>{},
                        // Type::TYPE_SFIXED32 =>{},
                        // Type::TYPE_SFIXED64 => {},
                        Type::TYPE_SINT32 => ReflectValueBox::I32(v.parse().unwrap_or(0i32)),
                        Type::TYPE_SINT64 => ReflectValueBox::I64(v.parse().unwrap_or(0i64)),
                        _ => continue,
                    };
                    // wd_log::log_debug_ln!("set_singular_field {}:{:?}",field.name(),value);
                    field.set_singular_field(&mut *message_dyn, value);
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
        let s = protobuf_json_mapping::print_to_string_with_options(&*msg,&PrintOptions{proto_field_name:true,..Default::default()})?;
        Ok(s.into_bytes())
    }
}
