use crate::database::{
    Connection as DbConnection, Connection, DatabaseConfig, DbError, RelationalDatabase, Row, Value,
};
use chrono::{DateTime, Datelike, NaiveDateTime, TimeZone, Timelike, Utc};
use mysql::{Opts, OptsBuilder};
use r2d2::{Pool, PooledConnection};
use r2d2_mysql::mysql::{prelude::*, Value as MySqlValue};
use r2d2_mysql::MysqlConnectionManager;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct MySqlDatabase {
    pool: Arc<Pool<MysqlConnectionManager>>,
    current_transaction: Arc<Mutex<Option<PooledConnection<MysqlConnectionManager>>>>,
}

impl MySqlDatabase {
    fn new_pool(config: &DatabaseConfig) -> Result<Pool<MysqlConnectionManager>, r2d2::Error> {
        let opts = OptsBuilder::new()
            .ip_or_hostname(Some(&config.host))
            .tcp_port(config.port)
            .user(Some(&config.username))
            .pass(Some(&config.password))
            .db_name(Some(&config.database_name));

        let manager = MysqlConnectionManager::new(opts);
        Pool::builder().max_size(config.max_size).build(manager)
    }

    fn value_to_mysql(value: &Value) -> MySqlValue {
        match value {
            Value::Null => MySqlValue::NULL,
            Value::Integer(i) => MySqlValue::Int(*i),
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
        }
    }

    fn convert_mysql_to_value(value: MySqlValue) -> Result<Value, DbError> {
        match value {
            MySqlValue::NULL => Ok(Value::Null),
            MySqlValue::Int(i) => Ok(Value::Integer(i)),
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

    fn execute_with_connection<F, T>(&self, f: F) -> Result<T, DbError>
    where
        // F: FnOnce(&mut PooledConnection<MysqlConnectionManager>) -> Result<T, DbError>
        F: FnOnce(&mut PooledConnection<MysqlConnectionManager>) -> Result<T, DbError>,
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
}

impl RelationalDatabase for MySqlDatabase {
    fn placeholders(&self, keys: &Vec<String>) -> Vec<String> {
        vec!["?".to_string(); keys.len()]
    }
    fn connect(config: DatabaseConfig) -> Result<Self, DbError> {
        let pool = Self::new_pool(&config).map_err(|e| DbError::ConnectionError(e.to_string()))?;

        Ok(MySqlDatabase {
            pool: Arc::new(pool),
            current_transaction: Arc::new(Mutex::new(None)),
        })
    }

    fn close(&self) -> Result<(), DbError> {
        Ok(())
    }

    fn ping(&self) -> Result<(), DbError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| DbError::ConnectionError(e.to_string()))?;
        conn.query_drop("SELECT 1")
            .map_err(|e| DbError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    fn begin_transaction(&self) -> Result<(), DbError> {
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

    fn commit(&self) -> Result<(), DbError> {
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

    fn rollback(&self) -> Result<(), DbError> {
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

    fn execute(&self, query: &str, params: Vec<Value>) -> Result<u64, DbError> {
        self.execute_with_connection(|conn| {
            let params: Vec<mysql::Value> =
                params.iter().map(MySqlDatabase::value_to_mysql).collect();

            conn.exec_drop(query, params)
                .map_err(|e| DbError::QueryError(e.to_string()))?;

            Ok(conn.affected_rows() as u64)
        })
    }

    fn query(&self, query: &str, params: Vec<Value>) -> Result<Vec<Row>, DbError> {
        self.execute_with_connection(|conn| {
            let params: Vec<mysql::Value> =
                params.iter().map(MySqlDatabase::value_to_mysql).collect();

            let result = conn
                .exec_map(query, params, |row: mysql::Row| {
                    let mut values = Vec::new();
                    let columns = row.columns();

                    for (i, column) in columns.iter().enumerate() {
                        let value = row.get(i).ok_or_else(|| {
                            DbError::QueryError("Missing column value".to_string())
                        })?;
                        values.push(Self::convert_mysql_to_value(value)?);
                    }

                    Ok(Row {
                        columns: columns.iter().map(|c| c.name_str().to_string()).collect(),
                        values,
                    })
                })
                .map_err(|e| DbError::QueryError(e.to_string()))?;

            let mut rows = Vec::new();
            for row_result in result {
                rows.push(row_result?);
            }
            Ok(rows)
        })
    }

    fn query_one(&self, query: &str, params: Vec<Value>) -> Result<Option<Row>, DbError> {
        let mut rows = self.query(query, params)?;
        Ok(rows.pop())
    }

    fn get_connection(&self) -> Result<Connection, DbError> {
        let _conn = self
            .pool
            .get()
            .map_err(|e| DbError::PoolError(e.to_string()))?;
        Ok(Connection {})
    }

    fn release_connection(&self, _conn: Connection) -> Result<(), DbError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serial_test::serial;

    fn setup_test_db() -> MySqlDatabase {
        let config = DatabaseConfig {
            host: "localhost".to_string(),
            port: 3306,
            username: "root".to_string(),
            password: "root".to_string(),
            database_name: "test".to_string(),
            max_size: 10,
        };
        MySqlDatabase::connect(config).unwrap()
    }

    #[test]
    // #[ignore] // 需要MySQL服务器才能运行
    #[serial]
    fn test_basic_connection() {
        let db = setup_test_db();
        assert!(db.ping().is_ok());
    }

    #[test]
    // #[ignore]
    #[serial]
    fn test_execute() {
        let db = setup_test_db();
        db.execute("DROP TABLE IF EXISTS users", vec![]).unwrap();
        db.execute(
            "CREATE TABLE users (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(255), age INT)",
            vec![],
        )
        .unwrap();

        let affected_rows = db
            .execute(
                "INSERT INTO users (name, age) VALUES (?, ?)",
                vec![Value::Text("Alice".to_string()), Value::Integer(30)],
            )
            .unwrap();
        assert_eq!(affected_rows, 1);

        let affected_rows = db
            .execute(
                "UPDATE users SET age = ? WHERE name = ?",
                vec![Value::Integer(31), Value::Text("Alice".to_string())],
            )
            .unwrap();
        assert_eq!(affected_rows, 1);

        db.execute("DROP TABLE users", vec![]).unwrap();
    }

    #[test]
    // #[ignore]
    #[serial]
    fn test_query() {
        let db = setup_test_db();
        db.execute("DROP TABLE IF EXISTS users", vec![]).unwrap();
        db.execute(
            "CREATE TABLE users (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(255), age INT, created_at DATETIME)",
            vec![],
        )
        .unwrap();

        let now = Utc::now();
        db.execute(
            "INSERT INTO users (name, age, created_at) VALUES (?, ?, ?)",
            vec![
                Value::Text("Alice".to_string()),
                Value::Integer(30),
                Value::DateTime(now),
            ],
        )
        .unwrap();

        let rows = db
            .query("SELECT id, name, age, created_at FROM users", vec![])
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].columns, vec!["id", "name", "age", "created_at"]);
        assert_eq!(rows[0].values.len(), 4);
        assert!(matches!(rows[0].values[0], Value::Integer(_)));
        assert!(matches!(rows[0].values[1], Value::Text(_)));
        assert!(matches!(rows[0].values[2], Value::Integer(_)));
        assert!(matches!(rows[0].values[3], Value::DateTime(_)));

        if let Value::Text(name) = &rows[0].values[1] {
            assert_eq!(name, "Alice");
        } else {
            panic!("Expected name to be a string");
        }

        if let Value::Integer(age) = &rows[0].values[2] {
            assert_eq!(age, &30);
        } else {
            panic!("Expected age to be an integer");
        }

        db.execute("DROP TABLE users", vec![]).unwrap();
    }

    #[test]
    // #[ignore]
    #[serial]
    fn test_query_one() {
        let db = setup_test_db();
        db.execute("DROP TABLE IF EXISTS users", vec![]).unwrap();
        db.execute(
            "CREATE TABLE users (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(255))",
            vec![],
        )
        .unwrap();

        db.execute(
            "INSERT INTO users (name) VALUES (?), (?)",
            vec![
                Value::Text("Alice".to_string()),
                Value::Text("Bob".to_string()),
            ],
        )
        .unwrap();

        let row = db
            .query_one(
                "SELECT id, name FROM users WHERE name = ?",
                vec![Value::Text("Alice".to_string())],
            )
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
            .unwrap();
        assert!(none.is_none());

        db.execute("DROP TABLE users", vec![]).unwrap();
    }

    #[test]
    // #[ignore]
    #[serial]
    fn test_transaction() {
        let db = setup_test_db();
        db.execute("DROP TABLE IF EXISTS users", vec![]).unwrap();
        db.execute(
            "CREATE TABLE users (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(255))",
            vec![],
        )
        .unwrap();

        db.begin_transaction().unwrap();
        db.execute(
            "INSERT INTO users (name) VALUES (?)",
            vec![Value::Text("Alice".to_string())],
        )
        .unwrap();
        db.rollback().unwrap();

        let rows = db.query("SELECT * FROM users", vec![]).unwrap();
        assert_eq!(rows.len(), 0);

        db.begin_transaction().unwrap();
        db.execute(
            "INSERT INTO users (name) VALUES (?)",
            vec![Value::Text("Bob".to_string())],
        )
        .unwrap();
        db.commit().unwrap();

        let rows = db.query("SELECT * FROM users", vec![]).unwrap();
        assert_eq!(rows.len(), 1);

        db.execute("DROP TABLE users", vec![]).unwrap();
    }

    #[test]
    // #[ignore]
    #[serial]
    fn test_value_conversion() {
        let db = setup_test_db();

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
