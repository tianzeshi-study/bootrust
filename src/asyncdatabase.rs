#[cfg(feature = "postgresql_async")]
pub mod postgres;

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
        host: "localhost".to_string(),
        port: 3306,
        username: "root".to_string(),
        password: "root".to_string(),
        database_name: "DefaultDatabase".to_string(),
        max_size: 16,
    }
    }
}

// 定义数据库连接池类型
pub enum DatabaseType {
    Postgres,
    MySQL,
    SQLite,
    // 可以继续添加其他数据库类型
}

#[async_trait::async_trait]
pub trait RelationalDatabase {
    fn placeholders(&self, keys: &Vec<String>) -> Vec<String>;
    // 连接相关
    async fn connect(config: DatabaseConfig) -> Result<Self, DbError>
    where
        Self: Sized;
    async fn close(&self) -> Result<(), DbError>;
    async fn ping(&self) -> Result<(), DbError>;

    // 事务相关
    async fn begin_transaction(&self) -> Result<(), DbError>;
    async fn commit(&self) -> Result<(), DbError>;
    async fn rollback(&self) -> Result<(), DbError>;

    // 查询相关
    async fn execute(&self, query: &str, params: Vec<Value>) -> Result<u64, DbError>;
    async fn query(&self, query: &str, params: Vec<Value>) -> Result<Vec<Row>, DbError>;
    async fn query_one(&self, query: &str, params: Vec<Value>) -> Result<Option<Row>, DbError>;

    // 连接池相关
    // async fn get_connection(&self) -> Result<Connection, DbError>;
    // async fn release_connection(&self, conn: Connection) -> Result<(), DbError>;
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
