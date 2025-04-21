mod autode;
mod autoser;
pub use autode::EntityDeserializer;
pub use autoser::EntityConvertor;

#[cfg(test)]
mod test {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::io::Cursor;

    #[test]
    fn test_bytes_serde() {
        let cursor = Cursor::new(Vec::new());
        let mut convertor = EntityConvertor::new(cursor);
        let bytes: Vec<u8> = vec![1];

        let s = bytes.serialize(&mut convertor).unwrap();

        // let s = Value::Bytes(vec![1]);
        let de = EntityDeserializer::from_value(s);

        let _result = Vec::<u8>::deserialize(de).unwrap();

        // let d1 = EntityDeserializer::from_value(d);
    }
}
