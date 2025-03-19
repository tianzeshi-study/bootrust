mod autoser;
mod autode;
pub use autoser::EntityConvertor;
pub use autode::EntityDeserializer;

#[cfg(test)]
mod test {
    use super::*;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use crate::common::Value;

    #[test]
    fn test_bytes_serde() {
        let cursor = Cursor::new(Vec::new());
let mut convertor = EntityConvertor::new(cursor);
let bytes: Vec<u8> = vec![1];

let s = bytes.serialize(&mut convertor).unwrap();
dbg!(&s);
// let s = Value::Bytes(vec![1]);
let de = EntityDeserializer::from_value(s);
dbg!(&de);
let result =Vec::<u8>::deserialize(de).unwrap(); 
dbg!(&result);
// let d1 = EntityDeserializer::from_value(d);
// dbg!(&d1);
/*
    let value = Value::Bytes(vec![1, 2, 3]);
        let de = EntityDeserializer::from_value(value);
        // Use deserialize_byte_buf instead of directly calling Vec::<u8>::deserialize
        // let result = Vec::<u8>::deserialize(de).unwrap();
        // assert_eq!(result, vec![1, 2, 3]);

        let result = de.deserialize_byte_buf(ByteBufVisitor {}).unwrap();
        assert_eq!(result, vec![1, 2, 3]);
*/
    
    }
}