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

// 定义通用的数据库错误类型
#[derive(Debug)]
pub enum DbError {
    ConnectionError(String),
    QueryError(String),
    TransactionError(String),
    PoolError(String),
    ConversionError(String),
    // 其他错误类型...
}

// 定义通用的数据库值类型
#[derive(Debug, Clone, PartialEq)]
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
    Bytes(Vec<u8>),
    DateTime(chrono::DateTime<chrono::Utc>),
    // 其他数据类型...
}

// 定义通用的结果行类型
#[derive(Debug)]
pub struct Row {
    pub columns: Vec<String>,
    pub values: Vec<Value>,
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
