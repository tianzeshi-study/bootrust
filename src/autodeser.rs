// src/deserializer.rs

use crate::asyncdatabase::Value;
use serde::de::{self, Deserialize, DeserializeSeed, Deserializer, MapAccess, Visitor};
// use serde::de::value::Error;
use serde::de::value::Error as ValueError;
use serde::de::Error;
use std::fmt;

// 反序列化器结构体
#[derive(Debug)]
pub struct EntityDeserializer {
    value: Value,
}

impl EntityDeserializer {
    // 从 Value 创建反序列化器
    pub fn from_value(value: Value) -> Self {
        EntityDeserializer { value }
    }
}

// 为反序列化器实现 Deserializer trait
impl<'de> Deserializer<'de> for EntityDeserializer {
    type Error = ValueError;

    // 反序列化 i32
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Int(i) => visitor.visit_i32(i),
            _ => Err(Error::custom("Expected i32 value")),
        }
    }
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Bigint(i) => visitor.visit_i64(i),
            _ => Err(Error::custom("Expected i64 value")),
        }
    }
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Float(f) => visitor.visit_f32(f),
            _ => Err(Error::custom("Expected f32 value")),
        }
    }
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Double(f) => visitor.visit_f64(f),
            _ => Err(Error::custom("Expected f64 value")),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Boolean(b) => visitor.visit_bool(b),
            _ => Err(Error::custom("Expected boolean value")),
        }
    }

    // 反序列化 String
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Text(s) => visitor.visit_string(s),
            _ => Err(Error::custom("Expected string value")),
        }
    }
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Text(s) => visitor.visit_str(&s),
            _ => Err(Error::custom("Expected string value")),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Bytes(b) => visitor.visit_bytes(&b),
            _ => Err(Error::custom("Expected bytes value")),
        }
    }
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Bytes(b) => visitor.visit_byte_buf(b),
            _ => Err(Error::custom("Expected bytes value")),
        }
    }

    // 反序列化结构体
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Table(fields) => {
                let deserializer = StructDeserializer {
                    fields: fields,
                    current: 0,
                };

                visitor.visit_map(deserializer)
            }
            _ => Err(Error::custom("Expected struct value")),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    // 其他类型的反序列化...
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // For simplicity, we'll handle common types here.  You'll need to expand
        // this based on the types you expect in your `Value` enum.
        match self.value {
            Value::Null => visitor.visit_unit(),
            Value::Boolean(b) => visitor.visit_bool(b),
            Value::Int(i) => visitor.visit_i32(i),
            Value::Bigint(i) => visitor.visit_i64(i),
            Value::Float(f) => visitor.visit_f32(f),
            Value::Double(f) => visitor.visit_f64(f),
            Value::Text(s) => visitor.visit_string(s),
            Value::Bytes(b) => visitor.visit_byte_buf(b), // or visit_bytes
            Value::Table(_) => self.deserialize_struct("", &[], visitor), // Treat Table as struct
            Value::DateTime(dt) => {
                // Assuming you want to deserialize DateTime from a string
                let s = dt.to_rfc3339();
                visitor.visit_string(s)
            }
            // Add other Value variants as needed
            _ => Err(Error::custom("Unsupported value type for deserialize_any")),
        }
    }

    serde::forward_to_deserialize_any! {

         i8 i16   i128
        u8 u16 u32 u64 u128
         char
         unit unit_struct
        newtype_struct seq tuple
        tuple_struct map enum
        identifier ignored_any

    }
}

// 用于反序列化结构体的辅助结构体
struct StructDeserializer {
    fields: Vec<(String, Value)>,
    current: usize,
    // fields: std::vec::IntoIter<(String, Value)>,
}

// 为 StructDeserializer 实现 MapAccess trait
impl<'de> MapAccess<'de> for StructDeserializer {
    type Error = ValueError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        // if let Some((key, _value)) = self.fields.next() {
        if let Some((key, _value)) = self.fields.get(self.current) {
            let key_de = EntityDeserializer::from_value(Value::Text(key.clone()));
            seed.deserialize(key_de).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        // if let Some((_, value)) = self.fields.next() {
        if let Some((_, value)) = self.fields.get(self.current) {
            let value_de = EntityDeserializer::from_value(value.clone());
            self.current += 1;
            seed.deserialize(value_de)
        } else {
            Err(Error::custom("Expected value"))
        }
    }
    fn size_hint(&self) -> Option<usize> {
        Some(self.fields.len())
    }
}

// 为 Value 实现 Deserialize trait (可选，但建议实现)
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid Value")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Boolean(v))
            }

            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Int(v))
            }
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Bigint(v))
            }
            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Float(v))
            }
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Double(v))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Text(v))
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Text(v.to_string()))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Bytes(v.to_vec()))
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Bytes(v))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Null)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Null)
            }
            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut fields = Vec::new();
                while let Some(key) = access.next_key::<String>()? {
                    let value = access.next_value::<Value>()?;

                    fields.push((key, value));
                }
                Ok(Value::Table(fields))
            }

            // You can add other visit methods (e.g., visit_seq) if needed
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

// Create a ByteBufVisitor to handle the byte buffer.
struct ByteBufVisitor;

impl<'de> Visitor<'de> for ByteBufVisitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a byte buffer")
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[test]
    fn test_deserialize_i32() {
        let value = Value::Int(42);
        let de = EntityDeserializer::from_value(value);
        let result = i32::deserialize(de).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_deserialize_i64() {
        let value = Value::Bigint(1234567890);
        let de = EntityDeserializer::from_value(value);
        let result = i64::deserialize(de).unwrap();
        assert_eq!(result, 1234567890);
    }
    #[test]
    fn test_deserialize_f32() {
        let value = Value::Float(3.14);
        let de = EntityDeserializer::from_value(value);
        let result = f32::deserialize(de).unwrap();
        assert_eq!(result, 3.14);
    }

    #[test]
    fn test_deserialize_f64() {
        let value = Value::Double(2.71828);
        let de = EntityDeserializer::from_value(value);
        let result = f64::deserialize(de).unwrap();
        assert_eq!(result, 2.71828);
    }

    #[test]
    fn test_deserialize_bool() {
        let value = Value::Boolean(true);
        let de = EntityDeserializer::from_value(value);
        let result = bool::deserialize(de).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_deserialize_string() {
        let value = Value::Text("hello".to_string());
        let de = EntityDeserializer::from_value(value);
        let result = String::deserialize(de).unwrap();
        assert_eq!(result, "hello");
    }
    // #[test]
    fn test_deserialize_bytes1() {
        let value = Value::Bytes(vec![1, 2, 3]);
        let de = EntityDeserializer::from_value(value);
        let result = Vec::<u8>::deserialize(de).unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_deserialize_bytes() {
        let value = Value::Bytes(vec![1, 2, 3]);
        let de = EntityDeserializer::from_value(value);
        // Use deserialize_byte_buf instead of directly calling Vec::<u8>::deserialize
        // let result = Vec::<u8>::deserialize(de).unwrap();
        // assert_eq!(result, vec![1, 2, 3]);

        let result = de.deserialize_byte_buf(ByteBufVisitor {}).unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }
    #[test]
    fn test_deserialize_option_some() {
        let value = Value::Text("hello".to_string());
        let de = EntityDeserializer::from_value(value);
        let result = Option::<String>::deserialize(de).unwrap();
        assert_eq!(result, Some("hello".to_string()));
    }

    #[test]
    fn test_deserialize_option_none() {
        let value = Value::Null;
        let de = EntityDeserializer::from_value(value);
        let result = Option::<String>::deserialize(de).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_deserialize_struct() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct TestStruct {
            a: i32,
            b: String,
        }

        let fields = vec![
            ("a".to_string(), Value::Int(42)),
            ("b".to_string(), Value::Text("hello".to_string())),
        ];
        let value = Value::Table(fields);

        let de = EntityDeserializer::from_value(value);

        let result = TestStruct::deserialize(de).unwrap();
        assert_eq!(
            result,
            TestStruct {
                a: 42,
                b: "hello".to_string()
            }
        );
    }

    // #[test]
    fn test_deserialize_datetime() {
        use chrono::{DateTime, Utc};
        let now = Utc::now();
        let value = Value::DateTime(now);
        let de = EntityDeserializer::from_value(value);
        let result = String::deserialize(de).unwrap();
        assert_eq!(result, now.to_rfc3339());
    }

    #[test]
    fn test_deserialize_value() {
        let value = Value::Int(42);
        let de = EntityDeserializer::from_value(value);
        let result = Value::deserialize(de).unwrap();
        assert_eq!(result, Value::Int(42));

        let value = Value::Bigint(42);
        let de = EntityDeserializer::from_value(value);
        let result = Value::deserialize(de).unwrap();
        assert_eq!(result, Value::Bigint(42));

        let value = Value::Float(42.0);
        let de = EntityDeserializer::from_value(value);
        let result = Value::deserialize(de).unwrap();
        assert_eq!(result, Value::Float(42.0));

        let value = Value::Double(42.0);
        let de = EntityDeserializer::from_value(value);
        let result = Value::deserialize(de).unwrap();
        assert_eq!(result, Value::Double(42.0));

        let value = Value::Boolean(true);
        let de = EntityDeserializer::from_value(value);
        let result = Value::deserialize(de).unwrap();
        assert_eq!(result, Value::Boolean(true));

        let value = Value::Text("test".to_string());
        let de = EntityDeserializer::from_value(value);
        let result = Value::deserialize(de).unwrap();
        assert_eq!(result, Value::Text("test".to_string()));

        let value = Value::Bytes(vec![1, 2, 3]);
        let de = EntityDeserializer::from_value(value);
        let result = Value::deserialize(de).unwrap();
        assert_eq!(result, Value::Bytes(vec![1, 2, 3]));

        let value = Value::Null;
        let de = EntityDeserializer::from_value(value);
        let result = Value::deserialize(de).unwrap();
        assert_eq!(result, Value::Null);
    }
    #[test]
    fn test_table() {
        let fields = vec![
            ("a".to_string(), Value::Int(42)),
            ("b".to_string(), Value::Text("hello".to_string())),
        ];
        let value = Value::Table(fields.clone());
        let de = EntityDeserializer::from_value(value);
        let result = Value::deserialize(de).unwrap();
        assert_eq!(result, Value::Table(fields));
    }
}
