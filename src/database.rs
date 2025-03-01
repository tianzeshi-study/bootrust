#[cfg(feature = "postgresql")]
pub mod  postgres;
#[cfg(feature = "mysql")]
pub mod mysql;
#[cfg(feature = "sqlite")]
pub mod sqlite;
// 定义数据库连接配置
#[derive(Clone)]
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
        password: std::env::var("BOOTRUST_DB_PASSWORD").unwrap_or_else(|_| "password".to_string()),
        database_name: std::env::var("BOOTRUST_DB_DATABASE").unwrap_or_else(|_| "bootrust_default_db".to_string()),
        max_size: std::env::var("DB_MAX_SIZE")
        .unwrap_or_else(|_| "20".to_string())
        .parse::<u32>()
        .expect("DB_MAX_SIZE must be a number"),
    }
    }
}

// 定义数据库连接池类型
#[derive(Debug)]
pub enum DatabaseType {
    Postgres,
    MySQL,
    SQLite,
    // 可以继续添加其他数据库类型
}

impl DatabaseType {
    pub fn from_env() -> Option<Self> {
        match std::env::var("DATABASE_TYPE").ok()?.to_lowercase().as_str() {
            "postgres" => Some(DatabaseType::Postgres),
            "mysql" => Some(DatabaseType::MySQL),
            "sqlite" => Some(DatabaseType::SQLite),
            _ => None, 
        }
    }
}

#[cfg(all(not(feature="full"), feature="mysql"))]
pub fn auto_config() -> mysql::MySqlDatabase {
    let config = DatabaseConfig::default();
mysql::MySqlDatabase::connect(config).unwrap()
}

#[cfg(all(not(feature="full"), feature="sqlite"))]
pub fn auto_config() -> sqlite::SqliteDatabase {
    let config = DatabaseConfig::default();
sqlite::SqliteDatabase::connect(config).unwrap()
}
// 定义关系型数据库通用接口
pub trait RelationalDatabase {
    fn placeholders(&self, keys: &Vec<String>) -> Vec<String>;
    // 连接相关
    fn connect(config: DatabaseConfig) -> Result<Self, DbError>
    where
        Self: Sized;
    fn close(&self) -> Result<(), DbError>;
    fn ping(&self) -> Result<(), DbError>;

    // 事务相关
    fn begin_transaction(&self) -> Result<(), DbError>;
    fn commit(&self) -> Result<(), DbError>;
    fn rollback(&self) -> Result<(), DbError>;

    // 查询相关
    fn execute(&self, query: &str, params: Vec<Value>) -> Result<u64, DbError>;
    fn query(&self, query: &str, params: Vec<Value>) -> Result<Vec<Row>, DbError>;
    fn query_one(&self, query: &str, params: Vec<Value>) -> Result<Option<Row>, DbError>;

    // 连接池相关
    fn get_connection(&self) -> Result<Connection, DbError>;
    fn release_connection(&self, conn: Connection) -> Result<(), DbError>;
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
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Integer(i32),
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
