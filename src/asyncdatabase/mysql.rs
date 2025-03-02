use async_trait::async_trait;
use tokio_mysql::{NoTls, Row as TokioRow};
use bb8::Pool;
use bb8_mysql::MysqlConnectionManager;
use crate::asyncdatabase::{DatabaseConfig, DbError, RelationalDatabase, Row, Value};

#[derive(Debug, Clone)]
pub struct MySqlDatabase { // Renamed from PostgresDatabase
    pool: Pool<MysqlConnectionManager<NoTls>>,
}

impl From<tokio_mysql::Error> for DbError {
    fn from(e: tokio_mysql::Error) -> Self {
        DbError::ConnectionError(e.to_string())
    }
}

#[async_trait]
impl RelationalDatabase for MySqlDatabase { // Renamed
    fn placeholders(&self, keys: &Vec<String>) -> Vec<String> {
        keys.iter().map(|_| "?".to_string()).collect()
    }

    async fn connect(config: DatabaseConfig) -> Result<Self, DbError> {
        let manager = MysqlConnectionManager::new_from_stringlike(
            format!("mysql://{}:{}@{}:{}/{}",
                    config.username, config.password, config.host, config.port, config.database_name),
            NoTls,
        )?;

        let pool = Pool::builder()
            .max_size(config.max_size)
            .build(manager)
            .await
            .map_err(|e| DbError::PoolError(e.to_string()))?;

        Ok(MySqlDatabase { pool }) // Renamed
    }

    async fn close(&self) -> Result<(), DbError> {
        // bb8's Pool handles closing connections on Drop.
        Ok(())
    }

    async fn ping(&self) -> Result<(), DbError> {
        let conn = self.pool.get().await.map_err(|e| DbError::PoolError(e.to_string()))?;
        conn.ping().await.map_err(|e| DbError::ConnectionError(e.to_string()))
    }

    async fn begin_transaction(&self) -> Result<(), DbError> {
        let conn = self.pool.get().await.map_err(|e| DbError::PoolError(e.to_string()))?;
        conn.query_drop("START TRANSACTION").await.map_err(|e| DbError::TransactionError(e.to_string()))
    }

    async fn commit(&self) -> Result<(), DbError> {
        let conn = self.pool.get().await.map_err(|e| DbError::PoolError(e.to_string()))?;
        conn.query_drop("COMMIT").await.map_err(|e| DbError::TransactionError(e.to_string()))
    }

    async fn rollback(&self) -> Result<(), DbError> {
        let conn = self.pool.get().await.map_err(|e| DbError::PoolError(e.to_string()))?;
        conn.query_drop("ROLLBACK").await.map_err(|e| DbError::TransactionError(e.to_string()))
    }
    async fn execute(&self, query: &str, params: Vec<Value>) -> Result<u64, DbError> {
        let conn = self.pool.get().await.map_err(|e| DbError::PoolError(e.to_string()))?;
        let params = params.iter().map(|v| match v {
            Value::Integer(i) => i as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::Bigint(i) => i as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::Text(s) => s as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::Float(f) => f as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::Double(d) => d as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::Boolean(b) => b as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::Bytes(by) => by as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::DateTime(dt) => dt as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::Null => &() as &(dyn tokio_mysql::prelude::ToValue + Sync), // Handle Null
        }).collect::<Vec<_>>();

        conn.exec_drop(query, params).await.map_err(|e| DbError::QueryError(e.to_string()))?;
        Ok(conn.affected_rows())
    }

    async fn query(&self, query: &str, params: Vec<Value>) -> Result<Vec<Row>, DbError> {
        let conn = self.pool.get().await.map_err(|e| DbError::PoolError(e.to_string()))?;
        let params = params.iter().map(|v| match v {
            Value::Integer(i) => i as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::Bigint(i) => i as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::Text(s) => s as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::Float(f) => f as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::Double(d) => d as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::Boolean(b) => b as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::Bytes(by) => by as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::DateTime(dt) => dt as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::Null => &() as &(dyn tokio_mysql::prelude::ToValue + Sync), // Handle Null
        }).collect::<Vec<_>>();

        let rows = conn.exec(query, params).await.map_err(|e| DbError::QueryError(e.to_string()))?;
        Ok(Self::convert_rows(rows))
    }

    async fn query_one(&self, query: &str, params: Vec<Value>) -> Result<Option<Row>, DbError> {
        let conn = self.pool.get().await.map_err(|e| DbError::PoolError(e.to_string()))?;
        let params = params.iter().map(|v| match v {
            Value::Integer(i) => i as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::Bigint(i) => i as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::Text(s) => s as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::Float(f) => f as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::Double(d) => d as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::Boolean(b) => b as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::Bytes(by) => by as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::DateTime(dt) => dt as &(dyn tokio_mysql::prelude::ToValue + Sync),
            Value::Null => &() as &(dyn tokio_mysql::prelude::ToValue + Sync), // Handle Null
        }).collect::<Vec<_>>();

        let row = conn.exec_first(query, params).await.map_err(|e| DbError::QueryError(e.to_string()))?;
        Ok(row.map(|r| Self::convert_rows(vec![r])).and_then(|mut v| v.pop()))
    }
}

impl MySqlDatabase { // Renamed
    fn convert_rows(rows: Vec<TokioRow>) -> Vec<Row> {
        let mut result_rows = Vec::new();
        for row in rows {
            let mut columns = Vec::new();
            let mut values = Vec::new();

            if let Some(column_names) = row.columns_ref().map(|cols| cols.iter().map(|col| col.name().to_string()).collect::<Vec<_>>()) {
                columns = column_names;
            }

            for value in row.values_iter() {
                let converted_value = match value.data_type() {
                    tokio_mysql::Value::NULL => Value::Null,
                    tokio_mysql::Value::Bytes => Value::Bytes(value.as_sql().unwrap().as_bytes().unwrap().to_vec()),
                    tokio_mysql::Value::Int => Value::Integer(value.as_sql().unwrap().as_i64().unwrap()),
                    tokio_mysql::Value::UInt => Value::Bigint(value.as_sql().unwrap().as_u64().unwrap() as i64),
                    tokio_mysql::Value::Float => Value::Float(value.as_sql().unwrap().as_f64().unwrap() as f32),
                    tokio_mysql::Value::Double => Value::Double(value.as_sql().unwrap().as_f64().unwrap()),
                    tokio_mysql::Value::Date => {
                        let date_str = String::from_utf8(value.as_sql().unwrap().as_bytes().unwrap().to_vec()).unwrap();
                        // Use NaiveDateTime for MySQL DATETIME
                        Value::DateTime(chrono::NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S").unwrap().and_utc())
                    },
                    tokio_mysql::Value::Time => {
                        let time_str = String::from_utf8(value.as_sql().unwrap().as_bytes().unwrap().to_vec()).unwrap();
                        Value::Text(time_str) // Or convert to a more specific time type
                    },
                    tokio_mysql::Value::Text => Value::Text(String::from_utf8(value.as_sql().unwrap().as_bytes().unwrap().to_vec()).unwrap()),
                    _ => unimplemented!("Type conversion not implemented for {:?}", value.data_type()),
                };
                values.push(converted_value);
            }
            result_rows.push(Row { columns, values });
        }
        result_rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Utc, Duration};
    use serial_test::serial;

    async fn setup_test_db() -> MySqlDatabase { // Renamed
        let config = DatabaseConfig {
            host: "localhost".to_string(),
            port: 3306,
            username: "root".to_string(),
            password: "root".to_string(),
            database_name: "test".to_string(),
            max_size: 10,
        };
        MySqlDatabase::connect(config).await.unwrap() // Renamed
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
        db.execute("DROP TABLE IF EXISTS users", vec![]).await.unwrap();
        db.execute(
            "CREATE TABLE users (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(255), age INT)",
            vec![],
        )
        .await
        .unwrap();

        let affected_rows = db
            .execute(
                "INSERT INTO users (name, age) VALUES (?, ?)",
                vec![Value::Text("Alice".to_string()), Value::Integer(30)],
            )
            .await
            .unwrap();
        assert_eq!(affected_rows, 1);

        let affected_rows = db
            .execute(
                "UPDATE users SET age = ? WHERE name = ?",
                vec![Value::Integer(31), Value::Text("Alice".to_string())],
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
        db.execute("DROP TABLE IF EXISTS users", vec![]).await.unwrap();
        db.execute(
            "CREATE TABLE users (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(255), age BIGINT, created_at DATETIME)",
            vec![],
        )
        .await
        .unwrap();

        let now = Utc::now().naive_utc();
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
        assert!(matches!(rows[0].values[0], Value::Integer(_)));
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

        // More robust datetime comparison, accounting for potential millisecond differences.
        if let Value::DateTime(created_at) = &rows[0].values[3] {
            let diff = *created_at - now.and_utc();
            assert!(diff < Duration::milliseconds(1), "Time difference should be less than 1ms");
        } else {
            panic!("Expected created_at to be a DateTime");
        }

        db.execute("DROP TABLE users", vec![]).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_query_one() {
        let db = setup_test_db().await;
        db.execute("DROP TABLE IF EXISTS users", vec![]).await.unwrap();
        db.execute(
            "CREATE TABLE users (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(255))",
            vec![],
        )
        .await
        .unwrap();

        db.execute(
            "INSERT INTO users (name) VALUES (?) , (?)",
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
        db.execute("DROP TABLE IF EXISTS users", vec![]).await.unwrap();
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
        let db = setup_test_db().await;

        let now = Utc::now();
        let row = db.query_one("SELECT ?", vec![Value::DateTime(now)]).await.unwrap().unwrap();
        //Since the select statement will return the same value and type, we can compare them directly.
        if let Value::DateTime(dt) = &row.values[0] {
            assert!((dt.timestamp_micros() - now.timestamp_micros()).abs() <= 1000); //Increased tolerance

        } else {
             panic!("Expected DateTime");
        }
    }
}