use crate::asyncdatabase::{Connection, DatabaseConfig, DbError, RelationalDatabase, Row, Value};

use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::ToSql;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct SqliteDatabase {
    pool: Arc<Pool<SqliteConnectionManager>>,
    current_transaction: Arc<Mutex<Option<PooledConnection<SqliteConnectionManager>>>>,
}

impl SqliteDatabase {
    async fn new_pool(
        path: &str,
        max_size: u32,
    ) -> Result<Pool<SqliteConnectionManager>, r2d2::Error> {
        let manager = SqliteConnectionManager::file(path);
        Pool::builder().max_size(max_size).build(manager)
    }

    fn value_to_sql(value: &Value) -> Box<dyn ToSql> {
        match value {
            Value::Null => Box::new(None::<String>),
            Value::Int(i) => Box::new(*i),
            Value::Bigint(i) => Box::new(*i),
            Value::Float(f) => Box::new(*f),
            Value::Double(f) => Box::new(*f),
            Value::Text(s) => Box::new(s.clone()),
            Value::Boolean(b) => Box::new(*b),
            Value::Bytes(b) => Box::new(b.to_vec()),
            Value::DateTime(dt) => Box::new(dt.to_rfc3339()),
            _ => unimplemented!(),
        }
    }

    fn convert_sql_to_value(value: rusqlite::types::ValueRef) -> Result<Value, rusqlite::Error> {
        match value {
            rusqlite::types::ValueRef::Null => Ok(Value::Null),
            rusqlite::types::ValueRef::Integer(i) => Ok(Value::Bigint(i)),
            rusqlite::types::ValueRef::Real(f) => Ok(Value::Double(f)),
            rusqlite::types::ValueRef::Text(s) => {
                Ok(Value::Text(String::from_utf8_lossy(s).into_owned()))
            }
            rusqlite::types::ValueRef::Blob(b) => Ok(Value::Bytes(b.to_vec())),
        }
    }

    async fn execute_with_connection<F, T>(&self, f: F) -> Result<T, DbError>
    where
        F: FnOnce(&PooledConnection<SqliteConnectionManager>) -> Result<T, DbError>,
    {
        let transaction_guard = self
            .current_transaction
            .lock()
            .map_err(|e| DbError::TransactionError(e.to_string()))?;

        let conn = if let Some(ref conn) = *transaction_guard {
            conn
        } else {
            &self
                .pool
                .get()
                .map_err(|e| DbError::ConnectionError(e.to_string()))?
        };

        f(conn)
    }
    pub async fn get_connection(&self) -> Result<Connection, DbError> {
        let _conn = self
            .pool
            .get()
            .map_err(|e| DbError::PoolError(e.to_string()))?;
        Ok(Connection {})
    }

    pub async fn release_connection(&self, _conn: Connection) -> Result<(), DbError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl RelationalDatabase for SqliteDatabase {
    fn placeholders(&self, keys: &[String]) -> Vec<String> {
        let placeholders: Vec<String> = (1..=keys.len()).map(|i| format!("${}", i)).collect();
        placeholders
    }
    async fn connect(config: DatabaseConfig) -> Result<Self, DbError> {
        let pool = Self::new_pool(&config.database_name, config.max_size)
            .await
            .map_err(|e| DbError::ConnectionError(e.to_string()))?;

        Ok(SqliteDatabase {
            pool: Arc::new(pool),
            current_transaction: Arc::new(Mutex::new(None)),
        })
    }

    async fn close(&self) -> Result<(), DbError> {
        Ok(())
    }

    async fn ping(&self) -> Result<(), DbError> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::ConnectionError(e.to_string()))?;
        conn.prepare("SELECT 1")
            .map_err(|e| DbError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    async fn begin_transaction(&self) -> Result<(), DbError> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::TransactionError(e.to_string()))?;

        conn.execute("BEGIN TRANSACTION", [])
            .map_err(|e| DbError::TransactionError(e.to_string()))?;

        let mut guard = self
            .current_transaction
            .lock()
            .map_err(|e| DbError::TransactionError(e.to_string()))?;
        *guard = Some(conn);

        Ok(())
    }

    async fn commit(&self) -> Result<(), DbError> {
        let mut guard = self
            .current_transaction
            .lock()
            .map_err(|e| DbError::TransactionError(e.to_string()))?;

        if let Some(conn) = guard.take() {
            conn.execute("COMMIT", [])
                .map_err(|e| DbError::TransactionError(e.to_string()))?;
        }
        Ok(())
    }

    async fn rollback(&self) -> Result<(), DbError> {
        let mut guard = self
            .current_transaction
            .lock()
            .map_err(|e| DbError::TransactionError(e.to_string()))?;

        if let Some(conn) = guard.take() {
            conn.execute("ROLLBACK", [])
                .map_err(|e| DbError::TransactionError(e.to_string()))?;
        }
        Ok(())
    }

    async fn execute(&self, query: &str, params: Vec<Value>) -> Result<u64, DbError> {
        self.execute_with_connection(|conn| {
            let params: Vec<Box<dyn ToSql>> =
                params.iter().map(SqliteDatabase::value_to_sql).collect();
            let mut stmt = conn
                .prepare(query)
                .map_err(|e| DbError::ConversionError(e.to_string()))?;

            stmt.execute(rusqlite::params_from_iter(params.iter()))
                .map(|rows| rows as u64)
                .map_err(|e| DbError::QueryError(e.to_string().into()))
        })
        .await
    }

    async fn query(&self, query: &str, params: Vec<Value>) -> Result<Vec<Row>, DbError> {
        self.execute_with_connection(|conn| {
            let mut stmt = conn
                .prepare(query)
                .map_err(|e| DbError::QueryError(e.to_string().into()))?;

            let column_names: Vec<String> = stmt
                .column_names()
                .iter()
                .map(|&name| name.to_string())
                .collect();

            let column_count = stmt.column_count();

            let params: Vec<Box<dyn ToSql>> =
                params.iter().map(SqliteDatabase::value_to_sql).collect();

            let rows = stmt
                .query_map(rusqlite::params_from_iter(params.iter()), |row| {
                    let mut values = Vec::new();
                    for i in 0..column_count {
                        let value = Self::convert_sql_to_value(row.get_ref(i).map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                i,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?)
                        .map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                i,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?;
                        values.push(value);
                    }
                    Ok(Row {
                        columns: column_names.clone(),
                        values,
                    })
                })
                .map_err(|e| DbError::QueryError(e.to_string().into()))?;

            let mut results = Vec::new();
            for row in rows {
                results.push(row.map_err(|e| DbError::QueryError(e.to_string().into()))?);
            }
            Ok(results)
        })
        .await
    }

    async fn query_one(&self, query: &str, params: Vec<Value>) -> Result<Option<Row>, DbError> {
        let mut rows = self.query(query, params).await?;
        Ok(rows.pop())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    async fn setup_test_db() -> SqliteDatabase {
        // 使用内存数据库进行测试
        let config = DatabaseConfig {
            database_name: ":memory:".to_string(),
            ..Default::default()
        };
        SqliteDatabase::connect(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_basic_connection() {
        let db = setup_test_db().await;

        assert!(db.ping().await.is_ok());
    }

    #[tokio::test]
    async fn test_execute_query() {
        let db = setup_test_db().await;

        // 创建测试表
        let create_table = "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)";
        assert!(db.execute(create_table, vec![]).await.is_ok());

        // 插入数据
        let insert = "INSERT INTO test (name, age) VALUES ($1, $2)";
        let result = db
            .execute(
                insert,
                vec![Value::Text("Alice".to_string()), Value::Bigint(25)],
            )
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_query() {
        let db = setup_test_db().await;

        // 创建并填充测试表
        db.execute(
            "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)",
            vec![],
        )
        .await
        .unwrap();

        db.execute(
            "INSERT INTO test (name, age) VALUES ($1, $2)",
            vec![Value::Text("Bob".to_string()), Value::Bigint(30)],
        )
        .await
        .unwrap();

        // 测试查询
        let rows = db.query("SELECT * FROM test", vec![]).await.unwrap();
        assert_eq!(rows.len(), 1);

        let row = &rows[0];
        assert_eq!(row.columns.len(), 3);
        assert_eq!(row.values.len(), 3);

        match &row.values[1] {
            Value::Text(name) => assert_eq!(name, "Bob"),
            _ => panic!("Expected Text value"),
        }

        match &row.values[2] {
            Value::Bigint(age) => assert_eq!(*age, 30),
            _ => panic!("Expected Integer value"),
        }
    }

    #[tokio::test]
    async fn test_transaction() {
        let db = setup_test_db().await;

        // 设置测试表
        db.execute(
            "CREATE TABLE test (id INTEGER PRIMARY KEY, value TEXT)",
            vec![],
        )
        .await
        .unwrap();
        db.query("SELECT * FROM test", vec![]).await.unwrap();

        // 测试成功的事务
        db.begin_transaction().await.unwrap();
        db.execute(
            "INSERT INTO test (value) VALUES ($1)",
            vec![Value::Text("transaction_test".to_string())],
        )
        .await
        .unwrap();
        db.commit().await.unwrap();

        let rows = db.query("SELECT * FROM test", vec![]).await.unwrap();
        assert_eq!(rows.len(), 1);

        // 测试回滚
        db.begin_transaction().await.unwrap();
        db.execute(
            "INSERT INTO test (value) VALUES ($1)",
            vec![Value::Text("will_rollback".to_string())],
        )
        .await
        .unwrap();
        db.rollback().await.unwrap();

        let rows = db.query("SELECT * FROM test", vec![]).await.unwrap();
        assert_eq!(rows.len(), 1); // 应该还是1条记录
    }

    #[tokio::test]
    async fn test_value_conversions() {
        let db = setup_test_db().await;

        db.execute(
            "CREATE TABLE test_types (
                id INTEGER PRIMARY KEY,
                int_val INTEGER,
                float_val REAL,
                text_val TEXT,
                null_val TEXT,
                datetime_val TEXT
            )",
            vec![],
        )
        .await
        .unwrap();

        let now = Utc::now();

        db.execute(
            "INSERT INTO test_types (int_val, float_val, text_val, null_val, datetime_val) 
             VALUES ($1, $2, $3, $4, $5)",
            vec![
                Value::Bigint(42),
                Value::Double(3.14),
                Value::Text("hello".to_string()),
                Value::Null,
                Value::DateTime(now),
            ],
        )
        .await
        .unwrap();

        let rows = db.query("SELECT * FROM test_types", vec![]).await.unwrap();
        assert_eq!(rows.len(), 1);

        let row = &rows[0];
        match &row.values[1] {
            Value::Bigint(i) => assert_eq!(*i, 42),
            _ => panic!("Expected Integer"),
        }

        match &row.values[2] {
            Value::Double(f) => assert!((f - 3.14).abs() < f64::EPSILON),
            _ => panic!("Expected Float"),
        }

        match &row.values[3] {
            Value::Text(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected Text"),
        }

        match &row.values[4] {
            Value::Null => (),
            _ => panic!("Expected Null"),
        }
    }
}
