use serde::ser::Error;
use serde::ser::{
    Impossible, Serialize, SerializeMap, SerializeSeq, SerializeStruct, Serializer,
};
// use std::error::Error;
use crate::asyncdatabase::Value;
use std::fmt::Display;
use std::io;
// 定义 Value 枚举，表示不同的数据类型
#[derive(Debug)]
// 实体转换器结构体
pub struct EntityConvertor<W> {
    writer: W,                    // 写入器
    fields: Vec<(String, Value)>, // 字段集合
}

// 为 EntityConvertor 实现构造函数
impl<W> EntityConvertor<W> {
    pub fn new(writer: W) -> Self {
        EntityConvertor {
            writer,
            fields: Vec::new(),
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
    type SerializeSeq = Impossible<Self::Ok, Self::Error>;

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
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    // 序列化 i16 值
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
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
        unimplemented!()
    }

    // 序列化 u16 值
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    // 序列化 u32 值
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    // 序列化 u64 值
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
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

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
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

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    // 序列化单元结构体（例如：struct Unit;）
    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    // 序列化单元变体（例如：enum E { A, B } 中的 E::A）
    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    // 序列化 newtype 结构体（例如：struct Millimeters(u8);）
    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    // 序列化 newtype 变体（例如：enum E { N(u8) } 中的 E::N）
    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    // 序列化可变长度的序列（例如：Vec）
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        unimplemented!()
    }

    // 序列化固定长度的序列（例如：数组）
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        unimplemented!()
    }

    // 序列化元组结构体（例如：struct Rgb(u8, u8, u8);）
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        unimplemented!()
    }

    // 序列化元组变体
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        unimplemented!()
    }

    // 序列化 Map
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        unimplemented!()
    }

    // 序列化结构体
    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(EntitySerializeStruct {
            entity_convertor: self,
            fields: Vec::new(),
        })
    }

    // 序列化结构体变体
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
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
        // println!("{}", &value);
        // unimplemented!()
        // use std::str::FromStr;
        //
        // let datetime_utc = chrono::DateTime::<chrono::Utc>::from_str(value).expect("Invalid datetime format");
        // let datetime_fixed = chrono::DateTime::parse_from_rfc3339(value).expect("Invalid datetime format");
        // let datetime_utc: chrono::DateTime<chrono::Utc> = datetime_fixed.into();
        // Ok(Value::DateTime(datetime_utc))
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
pub trait SerializeStructVariant {
    /// 必须与我们的 `Serializer` 的 `Ok` 类型匹配。
    type Ok;

    /// 必须与我们的 `Serializer` 的 `Error` 类型匹配。
    type Error: Error;

    /// Serialize a struct variant field.
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize;

    /// Indicate that a struct variant field has been skipped.
    ///
    /// The default implementation does nothing.
    #[inline]
    fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
        let _ = key;
        Ok(())
    }

    /// Finish serializing a struct variant.
    fn end(self) -> Result<Self::Ok, Self::Error>;
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
    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }
    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
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

// 为 Value 实现 Serialize trait，以便进行递归序列化
impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Value::Null => serializer.serialize_unit(),
            Value::Int(i) => serializer.serialize_i32(i),
            Value::Bigint(i) => serializer.serialize_i64(i),
            Value::Float(f) => serializer.serialize_f32(f),
            Value::Double(f) => serializer.serialize_f64(f),
            Value::Text(ref s) => serializer.serialize_str(s),
            Value::Boolean(b) => serializer.serialize_bool(b),
            Value::Bytes(ref b) => serializer.serialize_bytes(b),
            // Value::DateTime(ref dt) => serializer.collect_str(dt), // 需要 Display trait
            Value::DateTime(ref dt) => serializer.serialize_str(&dt.to_rfc3339()), // 使用 to_rfc3339 格式化
            _ => unimplemented!(),
        }
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
        dbg!(&result);

        // 检查序列化是否成功
        assert!(result.is_ok());

        // 这里添加更多断言来检查序列化的结果
        // 例如，检查 convertor.fields 中的内容是否符合预期
        // 这取决于你希望 Value::Struct 如何表示
        // assert_eq!(convertor.fields, ...);
    }

    #[test]
    fn test_serialize_value() {
        // 创建一个 Cursor 作为写入器
        let cursor = Cursor::new(Vec::new());

        // 创建 EntityConvertor
        let mut convertor = EntityConvertor::new(cursor);

        // 测试各种 Value 类型的序列化
        let values = vec![
            Value::Int(42),
            Value::Text("hello".to_string()),
            Value::Boolean(true),
            // 添加其他 Value 类型的测试用例
        ];

        for value in values {
            let result = value.serialize(&mut convertor);
            assert!(result.is_ok());
            // 可以根据需要添加更多断言
        }
    }
}
