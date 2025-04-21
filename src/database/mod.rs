#[cfg(feature = "mysql")]
pub mod mysql;
#[cfg(feature = "postgresql")]
pub mod postgres;
#[cfg(feature = "sqlite")]
pub mod sqlite;

pub use crate::common::{Connection, DatabaseConfig, DbError, QueryErrorKind, Row, Value};

#[cfg(all(not(feature = "full"), feature = "mysql"))]
pub fn auto_config() -> mysql::MySqlDatabase {
    let config = DatabaseConfig::default();
    mysql::MySqlDatabase::connect(config).unwrap()
}

#[cfg(all(not(feature = "full"), feature = "postgresql"))]
pub fn auto_config() -> postgres::PostgresDatabase {
    let config = DatabaseConfig::default();
    postgres::PostgresDatabase::connect(config).unwrap()
}

#[cfg(all(not(feature = "full"), feature = "sqlite"))]
pub fn auto_config() -> sqlite::SqliteDatabase {
    let config = DatabaseConfig::default();
    sqlite::SqliteDatabase::connect(config).unwrap()
}
// 定义关系型数据库通用接口
pub trait RelationalDatabase: Clone {
    fn placeholders(&self, keys: &[String]) -> Vec<String>;
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
