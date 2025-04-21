use serde::ser::Error;
use serde::ser::{Impossible, Serialize, SerializeMap, SerializeSeq, SerializeStruct, Serializer};
// use std::error::Error;
use crate::asyncdatabase::Value;
use std::fmt::Display;
use std::io;
// 定义 Value 枚举，表示不同的数据类型
#[derive(Debug)]
// 实体转换器结构体
pub struct EntityConvertor<W> {
    _writer: W,                    // 写入器
    _fields: Vec<(String, Value)>, // 字段集合
}

// 为 EntityConvertor 实现构造函数
impl<W> EntityConvertor<W> {
    pub fn new(writer: W) -> Self {
        EntityConvertor {
            _writer: writer,
            _fields: Vec::new(),
        }
    }
}

// 为 EntityConvertor 实现 Serializer trait
impl<'a, W> Serializer for &'a mut EntityConvertor<W>
where
    W: io::Write,
{
    // 序列化成功时的返回类型
    type Ok = Value;

    // 序列化失败时的错误类型
    type Error = serde::de::value::Error;
    // type Error = DbError;

    // Used for now as placeholder, it should be replaced by a concrete type that implements the trait.
    // type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeSeq = EntitySerializeSeq<'a, W>;

    // Used for now as placeholder, it should be replaced by a concrete type that implements the trait.
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;

    // Used for now as placeholder, it should be replaced by a concrete type that implements the trait.
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;

    // Used for now as placeholder, it should be replaced by a concrete type that implements the trait.
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;

    // Used for now as placeholder, it should be replaced by a concrete type that implements the trait.
    type SerializeMap = EntitySerializeStruct<'a, W>;
    // type SerializeMap =Impossible<Self::Ok, Self::Error>;

    // Used for now as placeholder, it should be replaced by a concrete type that implements the trait.
    type SerializeStruct = EntitySerializeStruct<'a, W>;

    // Used for now as placeholder, it should be replaced by a concrete type that implements the trait.
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    // 序列化 bool 值
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Boolean(v))
    }

    // 序列化 i8 值
    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    // 序列化 i16 值
    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    // 序列化 i32 值
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Int(v))
    }

    // 序列化 i64 值
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Bigint(v))
    }

    // 序列化 i128 值
    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        let _ = v;
        Err(serde::de::value::Error::custom("i128 is not supported"))
    }

    // 序列化 u8 值
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Byte(v))
    }

    // 序列化 u16 值
    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    // 序列化 u32 值
    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    // 序列化 u64 值
    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    // 序列化 u128 值
    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        let _ = v;
        Err(serde::de::value::Error::custom("u128 is not supported"))
    }

    // 序列化 f32 值
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Float(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Double(v))
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Text(v.to_string()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Bytes(v.to_vec()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }

    fn serialize_some<T>(self, v: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        v.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    // 序列化单元结构体（例如：struct Unit;）
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    // 序列化单元变体（例如：enum E { A, B } 中的 E::A）
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    // 序列化 newtype 结构体（例如：struct Millimeters(u8);）
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    // 序列化 newtype 变体（例如：enum E { N(u8) } 中的 E::N）
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    // 序列化可变长度的序列（例如：Vec）
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        // unimplemented!()
        Ok(EntitySerializeSeq {
            entity_convertor: self,
            elements: Vec::new(),
        })
    }

    // 序列化固定长度的序列（例如：数组）
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        unimplemented!()
    }

    // 序列化元组结构体（例如：struct Rgb(u8, u8, u8);）
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        unimplemented!()
    }

    // 序列化元组变体
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        unimplemented!()
    }

    // 序列化 Map
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        unimplemented!()
    }

    // 序列化结构体
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(EntitySerializeStruct {
            entity_convertor: self,
            fields: Vec::new(),
        })
    }

    // 序列化结构体变体
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        unimplemented!()
    }

    // 将迭代器收集为序列
    fn collect_seq<I>(self, iter: I) -> Result<Self::Ok, Self::Error>
    where
        I: IntoIterator,
        <I as IntoIterator>::Item: Serialize,
    {
        let mut iter = iter.into_iter();
        let mut serializer = self.serialize_seq(iterator_len_hint(&iter))?;
        iter.try_for_each(|item| SerializeSeq::serialize_element(&mut serializer, &item))?;
        SerializeSeq::end(serializer)
    }

    // 将迭代器收集为 Map
    fn collect_map<K, V, I>(self, iter: I) -> Result<Self::Ok, Self::Error>
    where
        K: Serialize,
        V: Serialize,
        I: IntoIterator<Item = (K, V)>,
    {
        let mut iter = iter.into_iter();
        let mut serializer = self.serialize_map(iterator_len_hint(&iter))?;
        iter.try_for_each(|(key, value)| serializer.serialize_entry(&key, &value))?;
        SerializeMap::end(serializer)
    }

    // 收集字符串
    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Display,
    {
        Ok(Value::Text(value.to_string()))
    }

    // 是否是人类可读的格式
    fn is_human_readable(&self) -> bool {
        true
    }
}

// 为迭代器提供长度提示的辅助函数
fn iterator_len_hint<I>(iter: &I) -> Option<usize>
where
    I: Iterator,
{
    match iter.size_hint() {
        (lower, Some(upper)) if lower == upper => Some(lower),
        _ => None,
    }
}

// 用于辅助序列化结构体的结构体
pub struct EntitySerializeStruct<'a, W: 'a> {
    entity_convertor: &'a mut EntityConvertor<W>, // 实体转换器的可变引用
    fields: Vec<(String, Value)>,                 // 字段集合
}

// 为 EntitySerializeStruct 实现 SerializeStruct trait
impl<W> SerializeStruct for EntitySerializeStruct<'_, W>
where
    W: io::Write,
{
    // 成功时的返回类型
    type Ok = Value;
    // 错误类型
    type Error = serde::de::value::Error;

    // 序列化字段
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // 递归地使用序列化器将每个字段转换为 Value
        let value = value.serialize(&mut *self.entity_convertor)?;
        self.fields.push((key.to_string(), value));
        Ok(())
    }

    // 结束序列化
    fn end(self) -> Result<Self::Ok, Self::Error> {
        // 将字段组合成一个单一的 Value::Struct 或类似的类型
        // 为简单起见，这里返回 Value::Null，你需要根据实际情况构建正确的 Value 变体
        // Ok(Value::Null) // 占位符，替换为实际逻辑
        Ok(Value::Table(self.fields))
    }
}

impl<W> SerializeMap for EntitySerializeStruct<'_, W>
where
    W: io::Write,
{
    // 成功时的返回类型
    type Ok = Value;
    // 错误类型
    type Error = serde::de::value::Error;

    // 序列化字段
    fn serialize_key<T>(&mut self, _key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }
    fn serialize_value<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }
    // 结束序列化
    fn end(self) -> Result<Self::Ok, Self::Error> {
        // 将字段组合成一个单一的 Value::Struct 或类似的类型
        // 为简单起见，这里返回 Value::Null，你需要根据实际情况构建正确的 Value 变体
        Ok(Value::Null) // 占位符，替换为实际逻辑
    }
}

pub struct EntitySerializeSeq<'a, W: 'a> {
    entity_convertor: &'a mut EntityConvertor<W>, // 实体转换器的可变引用
    elements: Vec<Value>,                         // 存储序列化后的元素集合
}

// 为 EntitySerializeSeq 实现 SerializeSeq trait
impl<W> SerializeSeq for EntitySerializeSeq<'_, W>
where
    W: io::Write,
{
    // 成功时返回的类型
    type Ok = Value;
    // 错误类型
    type Error = serde::de::value::Error;

    // 序列化单个元素
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // 递归调用转换器将元素转换为 Value
        let serialized = value.serialize(&mut *self.entity_convertor)?;
        self.elements.push(serialized);
        // self.elements.push(value);
        Ok(())
    }

    // 结束序列化并返回最终结果
    fn end(self) -> Result<Self::Ok, Self::Error> {
        // 组合所有元素为一个 Value::Array 类型
        // Ok(Value::Array(self.elements))
        let bytes =
            bincode::serialize(&self.elements).map_err(|e| serde::de::value::Error::custom(&e))?;
        Ok(Value::Bytes(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use std::io::Cursor;

    #[test]
    fn test_serialize_struct() {
        // 定义一个简单的结构体用于测试
        #[derive(Serialize)]
        struct TestStruct {
            b: String,
            a: i32,
        }

        // 创建一个 Cursor 作为写入器
        let cursor = Cursor::new(Vec::new());

        // 创建 EntityConvertor
        let mut convertor = EntityConvertor::new(cursor);

        // 创建测试结构体的实例
        let test_struct = TestStruct {
            a: 42,
            b: "hello".to_string(),
        };

        // 序列化结构体
        let result = test_struct.serialize(&mut convertor);

        // 检查序列化是否成功
        assert!(result.is_ok());

        // 这里添加更多断言来检查序列化的结果
        // 例如，检查 convertor.fields 中的内容是否符合预期
        // 这取决于你希望 Value::Struct 如何表示
        // assert_eq!(convertor.fields, ...);
    }

    #[test]
    fn test_serialize_bytes() {
        let cursor = Cursor::new(Vec::new());
        let mut convertor = EntityConvertor::new(cursor);
        let bytes: Vec<u8> = vec![1; 256];
        // let bytes = vec!["1".to_string()];
        let result = bytes.serialize(&mut convertor);
    }
}
