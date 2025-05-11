use std::{error::Error, fmt};

pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
    pub max_size: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: std::env::var("BOOTRUST_DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("BOOTRUST_DB_PORT")
                .unwrap_or_else(|_| "3306".to_string())
                .parse::<u16>()
                .expect("DB_PORT must be a number"),
            username: std::env::var("BOOTRUST_DB_USERNAME").unwrap_or_else(|_| "root".to_string()),
            password: std::env::var("BOOTRUST_DB_PASSWORD")
                .unwrap_or_else(|_| "password".to_string()),
            database_name: std::env::var("BOOTRUST_DB_DATABASE")
                .unwrap_or_else(|_| "bootrust_default_db".to_string()),
            max_size: std::env::var("DB_MAX_SIZE")
                .unwrap_or_else(|_| "20".to_string())
                .parse::<u32>()
                .expect("DB_MAX_SIZE must be a number"),
        }
    }
}

#[derive(Debug)]
pub enum QueryErrorKind {
    SyntaxError(String),
    ForeignKeyViolation(String),
    UniqueViolation(String),
    NotNullViolation(String),
    CheckViolation(String),
    ExclusionViolation(String),
    Other(String),
}

impl From<String> for QueryErrorKind {
    fn from(s: String) -> Self {
        QueryErrorKind::Other(s)
    }
}

impl fmt::Display for QueryErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryErrorKind::SyntaxError(msg) => write!(f, "syntax  error: {}", msg),
            QueryErrorKind::ForeignKeyViolation(msg) => write!(f, "ForeignKeyViolation: {}", msg),
            QueryErrorKind::UniqueViolation(msg) => write!(f, "UniqueViolation: {}", msg),
            QueryErrorKind::NotNullViolation(msg) => write!(f, "NotNullViolation: {}", msg),
            QueryErrorKind::CheckViolation(msg) => write!(f, "CheckViolation: {}", msg),
            QueryErrorKind::ExclusionViolation(msg) => write!(f, "ExclusionViolation: {}", msg),
            QueryErrorKind::Other(msg) => write!(f, "Pool error: {}", msg),
        }
    }
}

// 定义通用的数据库错误类型
#[derive(Debug)]
pub enum DbError {
    ConnectionError(String),
    QueryError(QueryErrorKind),
    TransactionError(String),
    PoolError(String),
    ConversionError(String),
    // 其他错误类型...
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            DbError::QueryError(msg) => write!(f, "Query error: {}", msg),
            DbError::TransactionError(msg) => write!(f, "Transaction error: {}", msg),
            DbError::PoolError(msg) => write!(f, "Pool error: {}", msg),
            DbError::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
        }
    }
}

impl Error for DbError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Value {
    Null,
    Table(Vec<(String, Value)>),
    Int(i32),
    Bigint(i64),
    Float(f32),
    Double(f64),
    Text(String),
    Varchar(String),
    Boolean(bool),
    Byte(u8),
    Bytes(Vec<u8>),
    DateTime(chrono::DateTime<chrono::Utc>),
    // 其他数据类型...
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(v: Option<T>) -> Self {
        if let Some(val) = v {
            val.into()
        } else {
            Value::Null
        }
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::Int(v)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Bigint(v)
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::Float(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Double(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::Text(v)
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Boolean(v)
    }
}

impl From<u8> for Value {
    fn from(v: u8) -> Self {
        Value::Byte(v)
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Value::Bytes(v)
    }
}

impl From<chrono::DateTime<chrono::Utc>> for Value {
    fn from(v: chrono::DateTime<chrono::Utc>) -> Self {
        Value::DateTime(v)
    }
}

// 定义通用的结果行类型
#[derive(Debug)]
pub struct Row {
    pub columns: Vec<String>,
    pub values: Vec<Value>,
}

impl Row {
    pub fn to_table(&self) -> Value {
        let table: Vec<(String, Value)> = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, s)| (s.clone(), self.values[i].clone()))
            .collect();
        Value::Table(table)
    }
}

// 定义连接类型（可以根据需要扩展）
pub struct Connection {
    // 连接相关字段
}

// 定义数据库连接池类型
pub enum _DatabaseType {
    Postgres,
    MySQL,
    SQLite,
}
