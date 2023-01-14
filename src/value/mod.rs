mod de;
mod ser;

use std::collections::HashMap;

#[derive(Debug)]
pub enum Value {
    I64(i64),
    I32(i32),
    I16(i16),
    I8(i8),
    U64(u64),
    U32(u32),
    U16(u16),
    U8(u8),
    F64(f64),
    String(String),
    Bytes(Vec<u8>),
    Bool(bool),
    Object(HashMap<String, Value>),
    Seq(Vec<Value>),
}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(obj) => obj.get(key),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        match self {
            Value::Object(obj) => obj.get_mut(key),
            _ => None,
        }
    }
}
