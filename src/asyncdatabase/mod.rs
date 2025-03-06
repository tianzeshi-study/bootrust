#[cfg(feature = "mysql_async")]
pub mod mysql;
#[cfg(feature = "postgresql_async")]
pub mod postgres;
#[cfg(feature = "sqlite_async")]
pub mod sqlite;

pub use crate::common::{Connection, DatabaseConfig, DbError, Row, Value};

#[async_trait::async_trait]
pub trait RelationalDatabase: Sync + Clone {
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

#[cfg(all(not(feature = "full"), feature = "postgresql_async"))]
pub async fn auto_config() -> postgres::PostgresDatabase {
    let config = DatabaseConfig::default();
    postgres::PostgresDatabase::connect(config).await.unwrap()
}

#[cfg(all(not(feature = "full"), feature = "mysql_async"))]
pub async fn auto_config() -> mysql::MySqlDatabase {
    let config = DatabaseConfig::default();
    mysql::MySqlDatabase::connect(config).await.unwrap()
}

#[cfg(all(not(feature = "full"), feature = "sqlite_async"))]
pub async fn auto_config() -> sqlite::SqliteDatabase {
    let config = DatabaseConfig::default();
    sqlite::SqliteDatabase::connect(config).await.unwrap()
}
