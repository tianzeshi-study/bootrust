use crate::asyncdatabase::{
    Connection, DatabaseConfig, DbError, QueryErrorKind, RelationalDatabase, Row, Value,
};
use async_trait::async_trait;
use chrono::{Datelike, NaiveDateTime, TimeZone, Timelike, Utc};
use mysql::OptsBuilder;
use r2d2::{Pool, PooledConnection};
use r2d2_mysql::mysql::{prelude::*, Value as MySqlValue};
use r2d2_mysql::MySqlConnectionManager;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct MySqlDatabase {
    pool: Arc<Pool<MySqlConnectionManager>>,
    current_transaction: Arc<Mutex<Option<PooledConnection<MySqlConnectionManager>>>>,
}

impl MySqlDatabase {
    async fn new_pool(
        config: &DatabaseConfig,
    ) -> Result<Pool<MySqlConnectionManager>, r2d2::Error> {
        let opts = OptsBuilder::new()
            .ip_or_hostname(Some(&config.host))
            .tcp_port(config.port)
            .user(Some(&config.username))
            .pass(Some(&config.password))
            .db_name(Some(&config.database_name));

        let manager = MySqlConnectionManager::new(opts);
        Pool::builder().max_size(config.max_size).build(manager)
    }

    fn value_to_mysql(value: &Value) -> MySqlValue {
        match value {
            Value::Null => MySqlValue::NULL,
            Value::Bigint(i) => MySqlValue::Int(*i),
            Value::Float(f) => MySqlValue::Float(*f as f32),
            Value::Double(f) => MySqlValue::Double(*f),
            Value::Text(s) => MySqlValue::Bytes(s.clone().into_bytes()),
            Value::Boolean(b) => MySqlValue::Int(if *b { 1 } else { 0 }),
            Value::Bytes(b) => MySqlValue::from(b),
            Value::DateTime(dt) => MySqlValue::Date(
                dt.year() as u16,
                dt.month() as u8,
                dt.day() as u8,
                dt.hour() as u8,
                dt.minute() as u8,
                dt.second() as u8,
                dt.timestamp_subsec_micros(),
            ),
            _ => unimplemented!(),
        }
    }

    fn convert_mysql_to_value(value: MySqlValue) -> Result<Value, DbError> {
        match value {
            MySqlValue::NULL => Ok(Value::Null),
            MySqlValue::Int(i) => Ok(Value::Bigint(i)),
            MySqlValue::Float(f) => Ok(Value::Float(f)),
            MySqlValue::Double(f) => Ok(Value::Double(f)),
            MySqlValue::Bytes(bytes) => Ok(Value::Text(
                String::from_utf8(bytes).map_err(|e| DbError::ConversionError(e.to_string()))?,
            )),
            MySqlValue::Date(year, month, day, hour, minute, second, micros) => {
                let naive = NaiveDateTime::new(
                    chrono::NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
                        .ok_or_else(|| DbError::ConversionError("Invalid date".to_string()))?,
                    chrono::NaiveTime::from_hms_micro_opt(
                        hour as u32,
                        minute as u32,
                        second as u32,
                        micros,
                    )
                    .ok_or_else(|| DbError::ConversionError("Invalid time".to_string()))?,
                );
                Ok(Value::DateTime(Utc.from_utc_datetime(&naive)))
            }
            _ => Err(DbError::ConversionError(
                "Unsupported MySQL type".to_string(),
            )),
        }
    }

    async fn execute_with_connection<F, T>(&self, f: F) -> Result<T, DbError>
    where
        F: FnOnce(&mut PooledConnection<MySqlConnectionManager>) -> Result<T, DbError>,
    {
        let mut transaction_guard = self
            .current_transaction
            .lock()
            .map_err(|e| DbError::TransactionError(e.to_string()))?;

        let mut conn = if let Some(conn) = &mut *transaction_guard {
            conn
        } else {
            &mut self
                .pool
                .get()
                .map_err(|e| DbError::ConnectionError(e.to_string()))?
        };

        // f(conn)
        f(&mut conn)
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

#[async_trait]
impl RelationalDatabase for MySqlDatabase {
    fn placeholders(&self, keys: &Vec<String>) -> Vec<String> {
        vec!["?".to_string(); keys.len()]
    }
    async fn connect(config: DatabaseConfig) -> Result<Self, DbError> {
        let pool = Self::new_pool(&config)
            .await
            .map_err(|e| DbError::ConnectionError(e.to_string()))?;

        Ok(MySqlDatabase {
            pool: Arc::new(pool),
            current_transaction: Arc::new(Mutex::new(None)),
        })
    }

    async fn close(&self) -> Result<(), DbError> {
        Ok(())
    }

    async fn ping(&self) -> Result<(), DbError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| DbError::ConnectionError(e.to_string()))?;
        conn.query_drop("SELECT 1")
            .map_err(|e| DbError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    async fn begin_transaction(&self) -> Result<(), DbError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| DbError::TransactionError(e.to_string()))?;

        conn.query_drop("START TRANSACTION")
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

        if let Some(mut conn) = guard.take() {
            conn.query_drop("COMMIT")
                .map_err(|e| DbError::TransactionError(e.to_string()))?;
        }
        Ok(())
    }

    async fn rollback(&self) -> Result<(), DbError> {
        let mut guard = self
            .current_transaction
            .lock()
            .map_err(|e| DbError::TransactionError(e.to_string()))?;

        if let Some(mut conn) = guard.take() {
            conn.query_drop("ROLLBACK")
                .map_err(|e| DbError::TransactionError(e.to_string()))?;
        }
        Ok(())
    }

    async fn execute(&self, query: &str, params: Vec<Value>) -> Result<u64, DbError> {
        self.execute_with_connection(|conn| {
            let params: Vec<mysql::Value> =
                params.iter().map(MySqlDatabase::value_to_mysql).collect();

            let stmt = conn
                .prep(query)
                .map_err(|e| DbError::ConversionError(e.to_string()))?;

            conn.exec_drop(&stmt, &params).map_err(|e| {
                match e {
                    mysql::Error::MySqlError(ref mysql_err) => {
                        // 获取 MySQL 错误码
                        match mysql_err.code {
                            1451 | 1452 => {
                                // 外键约束错误
                                DbError::QueryError(QueryErrorKind::ForeignKeyViolation(
                                    mysql_err.message.clone(),
                                ))
                            }
                            1062 => {
                                // 唯一约束错误
                                DbError::QueryError(QueryErrorKind::UniqueViolation(
                                    mysql_err.message.clone(),
                                ))
                            }
                            1048 => {
                                // 非空约束错误
                                DbError::QueryError(QueryErrorKind::NotNullViolation(
                                    mysql_err.message.clone(),
                                ))
                            }
                            // 其他错误
                            other_code => DbError::QueryError(QueryErrorKind::Other(format!(
                                "code: {}, message: {}",
                                other_code, mysql_err.message
                            ))),
                        }
                    }
                    // 其他类型的错误（比如连接错误、IO错误等）
                    _ => DbError::QueryError(QueryErrorKind::Other(format!("message: {}", e))),
                }
            })?;
            Ok(conn.affected_rows() as u64)
        })
        .await
    }

    async fn query(&self, query: &str, params: Vec<Value>) -> Result<Vec<Row>, DbError> {
        self.execute_with_connection(|conn| {
            let params: Vec<mysql::Value> =
                params.iter().map(MySqlDatabase::value_to_mysql).collect();

            let result = conn
                .exec_map(query, params, |row: mysql::Row| {
                    let mut values = Vec::new();
                    let columns = row.columns();

                    for (i, _column) in columns.iter().enumerate() {
                        let value = row.get(i).ok_or_else(|| {
                            DbError::QueryError("Missing column value".to_string().into())
                        })?;
                        values.push(Self::convert_mysql_to_value(value)?);
                    }

                    Ok::<Row, DbError>(Row {
                        columns: columns.iter().map(|c| c.name_str().to_string()).collect(),
                        values,
                    })
                })
                .map_err(|e| DbError::QueryError(e.to_string().into()))?;

            let mut rows = Vec::new();
            for row_result in result {
                rows.push(row_result?);
            }
            Ok(rows)
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
    use serial_test::serial;

    async fn setup_test_db() -> MySqlDatabase {
        let config = DatabaseConfig {
            host: "localhost".to_string(),
            port: 3306,
            username: "root".to_string(),
            password: "root".to_string(),
            database_name: "test".to_string(),
            max_size: 10,
        };
        MySqlDatabase::connect(config).await.unwrap()
    }

    #[tokio::test]
    #[serial]
    async fn test_basic_connection() {
        let db = setup_test_db().await;
        assert!(db.ping().await.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_execute() {
        let db = setup_test_db().await;
        db.execute("DROP TABLE IF EXISTS users", vec![])
            .await
            .unwrap();
        db.execute(
            "CREATE TABLE users (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(255), age INT)",
            vec![],
        )
        .await
        .unwrap();

        let affected_rows = db
            .execute(
                "INSERT INTO users (name, age) VALUES (?, ?)",
                vec![Value::Text("Alice".to_string()), Value::Bigint(30)],
            )
            .await
            .unwrap();
        assert_eq!(affected_rows, 1);

        let affected_rows = db
            .execute(
                "UPDATE users SET age = ? WHERE name = ?",
                vec![Value::Bigint(31), Value::Text("Alice".to_string())],
            )
            .await
            .unwrap();
        assert_eq!(affected_rows, 1);

        db.execute("DROP TABLE users", vec![]).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_query() {
        let db = setup_test_db().await;
        db.execute("DROP TABLE IF EXISTS users", vec![])
            .await
            .unwrap();
        db.execute(
            "CREATE TABLE users (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(255), age INT, created_at DATETIME)",
            vec![],
        )
        .await
        .unwrap();

        let now = Utc::now();
        db.execute(
            "INSERT INTO users (name, age, created_at) VALUES (?, ?, ?)",
            vec![
                Value::Text("Alice".to_string()),
                Value::Bigint(30),
                Value::DateTime(now),
            ],
        )
        .await
        .unwrap();

        let rows = db
            .query("SELECT id, name, age, created_at FROM users", vec![])
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].columns, vec!["id", "name", "age", "created_at"]);
        assert_eq!(rows[0].values.len(), 4);
        assert!(matches!(rows[0].values[0], Value::Bigint(_)));
        assert!(matches!(rows[0].values[1], Value::Text(_)));
        assert!(matches!(rows[0].values[2], Value::Bigint(_)));
        assert!(matches!(rows[0].values[3], Value::DateTime(_)));

        if let Value::Text(name) = &rows[0].values[1] {
            assert_eq!(name, "Alice");
        } else {
            panic!("Expected name to be a string");
        }

        if let Value::Bigint(age) = &rows[0].values[2] {
            assert_eq!(age, &30);
        } else {
            panic!("Expected age to be an integer");
        }

        db.execute("DROP TABLE users", vec![]).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_query_one() {
        let db = setup_test_db().await;
        db.execute("DROP TABLE IF EXISTS users", vec![])
            .await
            .unwrap();
        db.execute(
            "CREATE TABLE users (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(255))",
            vec![],
        )
        .await
        .unwrap();

        db.execute(
            "INSERT INTO users (name) VALUES (?), (?)",
            vec![
                Value::Text("Alice".to_string()),
                Value::Text("Bob".to_string()),
            ],
        )
        .await
        .unwrap();

        let row = db
            .query_one(
                "SELECT id, name FROM users WHERE name = ?",
                vec![Value::Text("Alice".to_string())],
            )
            .await
            .unwrap();
        assert!(row.is_some());

        if let Some(row) = row {
            assert_eq!(row.columns, vec!["id", "name"]);
            if let Value::Text(name) = &row.values[1] {
                assert_eq!(name, "Alice");
            } else {
                panic!("Expected name to be a string");
            }
        }

        let none = db
            .query_one(
                "SELECT * FROM users WHERE name = ?",
                vec![Value::Text("Charlie".to_string())],
            )
            .await
            .unwrap();
        assert!(none.is_none());

        db.execute("DROP TABLE users", vec![]).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_transaction() {
        let db = setup_test_db().await;
        db.execute("DROP TABLE IF EXISTS users", vec![])
            .await
            .unwrap();
        db.execute(
            "CREATE TABLE users (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(255))",
            vec![],
        )
        .await
        .unwrap();

        db.begin_transaction().await.unwrap();
        db.execute(
            "INSERT INTO users (name) VALUES (?)",
            vec![Value::Text("Alice".to_string())],
        )
        .await
        .unwrap();
        db.rollback().await.unwrap();

        let rows = db.query("SELECT * FROM users", vec![]).await.unwrap();
        assert_eq!(rows.len(), 0);

        db.begin_transaction().await.unwrap();
        db.execute(
            "INSERT INTO users (name) VALUES (?)",
            vec![Value::Text("Bob".to_string())],
        )
        .await
        .unwrap();
        db.commit().await.unwrap();

        let rows = db.query("SELECT * FROM users", vec![]).await.unwrap();
        assert_eq!(rows.len(), 1);

        db.execute("DROP TABLE users", vec![]).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_value_conversion() {
        let _db = setup_test_db().await;

        let now = Utc::now();
        let mysql_now = MySqlDatabase::value_to_mysql(&Value::DateTime(now));
        let converted_now = MySqlDatabase::convert_mysql_to_value(mysql_now).unwrap();

        if let Value::DateTime(dt) = converted_now {
            assert_eq!(dt.date_naive(), now.date_naive());
            // assert_eq!(dt.time(), now.time());
            // 比较时间时，允许1微秒的误差
            assert!((dt.timestamp_micros() - now.timestamp_micros()).abs() <= 1);
        } else {
            panic!("Expected DateTime");
        }
    }
}
