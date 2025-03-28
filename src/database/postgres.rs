use crate::database::{
    Connection, DatabaseConfig, DbError, QueryErrorKind, RelationalDatabase, Row, Value,
};
use chrono::{DateTime, Utc};
use postgres::{config::Config as PostgresConfig, NoTls};
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::PostgresConnectionManager;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct PostgresDatabase {
    pool: Arc<Pool<PostgresConnectionManager<NoTls>>>,
    current_transaction: Arc<Mutex<Option<PooledConnection<PostgresConnectionManager<NoTls>>>>>,
}

impl PostgresDatabase {
    fn new_pool(
        config: &DatabaseConfig,
    ) -> Result<Pool<PostgresConnectionManager<NoTls>>, r2d2::Error> {
        let mut pg_config = PostgresConfig::new();
        pg_config
            .host(&config.host)
            .port(config.port)
            .user(&config.username)
            .password(&config.password)
            .dbname(&config.database_name);

        let manager = PostgresConnectionManager::new(pg_config, NoTls);
        Pool::builder().max_size(config.max_size).build(manager)
    }

    fn params_to_postgres(params: &Vec<Value>) -> Vec<&(dyn postgres::types::ToSql + Sync)> {
        params
            .iter()
            .map(|v| match v {
                Value::Int(i) => i as &(dyn postgres::types::ToSql + Sync),
                Value::Bigint(i) => i as &(dyn postgres::types::ToSql + Sync),
                Value::Text(s) => s as &(dyn postgres::types::ToSql + Sync),
                Value::Varchar(s) => s as &(dyn postgres::types::ToSql + Sync),
                Value::Float(f) => f as &(dyn postgres::types::ToSql + Sync),
                Value::Double(d) => d as &(dyn postgres::types::ToSql + Sync),
                Value::Boolean(b) => b as &(dyn postgres::types::ToSql + Sync),
                Value::Bytes(by) => by as &(dyn postgres::types::ToSql + Sync),
                Value::DateTime(dt) => dt as &(dyn postgres::types::ToSql + Sync),
                Value::Null => &None::<&str> as &(dyn postgres::types::ToSql + Sync),
                _ => unimplemented!(),
            })
            .collect::<Vec<_>>()
    }

    fn convert_postgres_to_value(
        value: &postgres::row::Row,
        index: usize,
    ) -> Result<Value, DbError> {
        let column = &value.columns()[index];
        match *column.type_() {
            postgres::types::Type::VOID => Ok(Value::Null),
            postgres::types::Type::INT8 => {
                let val: i64 = value.get(index);
                Ok(Value::Bigint(val))
            }
            postgres::types::Type::INT4 => {
                let val: i32 = value.get(index);
                Ok(Value::Int(val))
            }
            postgres::types::Type::FLOAT4 => {
                let val: f32 = value.get(index);
                Ok(Value::Float(val))
            }
            postgres::types::Type::FLOAT8 => {
                let val: f64 = value.get(index);
                Ok(Value::Double(val))
            }
            postgres::types::Type::TEXT => {
                let val: String = value.get(index);
                Ok(Value::Text(val))
            }
            postgres::types::Type::VARCHAR => {
                let val: String = value.get(index);
                Ok(Value::Varchar(val))
            }
            postgres::types::Type::BOOL => {
                let val: bool = value.get(index);
                Ok(Value::Boolean(val))
            }
            postgres::types::Type::BYTEA => {
                let val: Vec<u8> = value.get(index);
                Ok(Value::Bytes(val))
            }
            postgres::types::Type::TIMESTAMPTZ => {
                let val: DateTime<Utc> = value.get(index);
                Ok(Value::DateTime(val))
            }
            _ => Err(DbError::ConversionError(
                "Unsupported Postgres type".to_string(),
            )),
        }
    }

    fn execute_with_connection<F, T>(&self, f: F) -> Result<T, DbError>
    where
        F: FnOnce(&mut PooledConnection<PostgresConnectionManager<NoTls>>) -> Result<T, DbError>,
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

        f(&mut conn)
    }
}

#[cfg(all(not(feature = "full"), feature = "postgresql"))]
impl From<postgres::Error> for DbError {
    fn from(err: postgres::Error) -> DbError {
        DbError::QueryError(err.to_string().into())
    }
}

impl RelationalDatabase for PostgresDatabase {
    fn placeholders(&self, keys: &Vec<String>) -> Vec<String> {
        keys.iter()
            .enumerate()
            .map(|(i, _)| format!("${}", i + 1))
            .collect()
    }

    fn connect(config: DatabaseConfig) -> Result<Self, DbError> {
        let pool = Self::new_pool(&config).map_err(|e| DbError::ConnectionError(e.to_string()))?;

        Ok(PostgresDatabase {
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
        conn.execute("SELECT 1", &[])
            .map_err(|e| DbError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    fn begin_transaction(&self) -> Result<(), DbError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| DbError::TransactionError(e.to_string()))?;

        conn.execute("START TRANSACTION", &[])
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
            conn.execute("COMMIT", &[])
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
            conn.execute("ROLLBACK", &[])
                .map_err(|e| DbError::TransactionError(e.to_string()))?;
        }
        Ok(())
    }

    fn execute(&self, query: &str, params: Vec<Value>) -> Result<u64, DbError> {
        self.execute_with_connection(|conn| {
            let stmt = conn.prepare(query)?;
            let params = Self::params_to_postgres(&params);

            // let rows_affected = conn.execute(&stmt, &params[..])?;

            // Ok(rows_affected)
            conn.execute(&stmt, &params).map_err(|e| {
                if let Some(db_err) = e.as_db_error() {
                    match db_err.code().code() {
                        "23503" => {
                            // 外键约束错误
                            DbError::QueryError(QueryErrorKind::ForeignKeyViolation(
                                db_err.message().to_string(),
                            ))
                        }
                        "23505" => {
                            // 唯一约束错误（包括主键冲突）
                            DbError::QueryError(QueryErrorKind::UniqueViolation(
                                db_err.message().to_string(),
                            ))
                        }
                        "23502" => {
                            // 非空约束错误
                            DbError::QueryError(QueryErrorKind::NotNullViolation(
                                db_err.message().to_string(),
                            ))
                        }
                        "23514" => {
                            // 检查约束错误
                            DbError::QueryError(QueryErrorKind::CheckViolation(
                                db_err.message().to_string(),
                            ))
                        }
                        "23P01" => {
                            // 排他约束错误
                            DbError::QueryError(QueryErrorKind::ExclusionViolation(
                                db_err.message().to_string(),
                            ))
                        }
                        _ => {
                            // 其他数据库错误
                            DbError::QueryError(QueryErrorKind::Other(format!(
                                "code: {}, message: {}",
                                db_err.code().code(),
                                db_err.message().to_string()
                            )))
                        }
                    }
                } else {
                    // 如果不是数据库错误，比如 IO 错误等
                    DbError::QueryError(QueryErrorKind::Other(format!(
                        "message: {}",
                        e.to_string()
                    )))
                }
            })
        })
    }

    fn query(&self, query: &str, params: Vec<Value>) -> Result<Vec<Row>, DbError> {
        self.execute_with_connection(|conn| {
            let stmt = conn.prepare(query)?;
            let params = Self::params_to_postgres(&params);
            let result = conn.query(&stmt, &params[..])?;

            let mut rows = Vec::new();
            for row in result {
                let mut values = Vec::new();
                let columns = row.columns();

                for (i, _column) in columns.iter().enumerate() {
                    values.push(Self::convert_postgres_to_value(&row, i)?);
                }

                rows.push(Row {
                    columns: columns.iter().map(|c| c.name().to_string()).collect(),
                    values,
                });
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

    fn setup_test_db() -> PostgresDatabase {
        let config = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            username: "root".to_string(),
            password: "root".to_string(),
            database_name: "test".to_string(),
            max_size: 10,
        };
        PostgresDatabase::connect(config).unwrap()
    }

    #[test]
    // #[ignore] // 需要PostgreSQL服务器才能运行
    #[serial]
    fn test_basic_connection() {
        let db = setup_test_db();
        assert!(db.ping().is_ok());
    }

    #[test]
    #[serial]
    fn test_execute() {
        let db = setup_test_db();
        db.execute("DROP TABLE IF EXISTS users", vec![]).unwrap();
        db.execute(
            "CREATE TABLE users (id SERIAL PRIMARY KEY, name VARCHAR(255), age INT)",
            vec![],
        )
        .unwrap();

        let affected_rows = db
            .execute(
                "INSERT INTO users (name, age) VALUES ($1, $2)",
                vec![Value::Text("Alice".to_string()), Value::Int(30)],
            )
            .unwrap();
        assert_eq!(affected_rows, 1);

        let affected_rows = db
            .execute(
                "UPDATE users SET age = $1 WHERE name = $2",
                vec![Value::Int(31), Value::Text("Alice".to_string())],
            )
            .unwrap();
        assert_eq!(affected_rows, 1);

        db.execute("DROP TABLE users", vec![]).unwrap();
    }

    #[test]
    #[serial]
    fn test_query() {
        let db = setup_test_db();
        db.execute("DROP TABLE IF EXISTS users", vec![]).unwrap();
        db.execute(
            "CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT, age INT, created_at TIMESTAMP WITH TIME ZONE)",
            vec![],
        )
        .unwrap();

        let now = Utc::now();
        db.execute(
            "INSERT INTO users (name, age, created_at) VALUES ($1, $2, $3)",
            vec![
                Value::Text("Alice".to_string()),
                Value::Int(30),
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
        assert!(matches!(rows[0].values[0], Value::Int(_)));
        assert!(matches!(rows[0].values[1], Value::Text(_)));
        assert!(matches!(rows[0].values[2], Value::Int(_)));
        assert!(matches!(rows[0].values[3], Value::DateTime(_)));

        if let Value::Text(name) = &rows[0].values[1] {
            assert_eq!(name, "Alice");
        } else {
            panic!("Expected name to be a string");
        }

        if let Value::Int(age) = &rows[0].values[2] {
            assert_eq!(age, &30);
        } else {
            panic!("Expected age to be an integer");
        }

        db.execute("DROP TABLE users", vec![]).unwrap();
    }

    #[test]
    #[serial]
    fn test_query_one() {
        let db = setup_test_db();
        db.execute("DROP TABLE IF EXISTS users", vec![]).unwrap();
        db.execute(
            "CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT)",
            vec![],
        )
        .unwrap();

        db.execute(
            "INSERT INTO users (name) VALUES ($1), ($2)",
            vec![
                Value::Text("Alice".to_string()),
                Value::Text("Bob".to_string()),
            ],
        )
        .unwrap();

        let row = db
            .query_one(
                "SELECT id, name FROM users WHERE name = $1",
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
                "SELECT * FROM users WHERE name = $1",
                vec![Value::Text("Charlie".to_string())],
            )
            .unwrap();
        assert!(none.is_none());

        db.execute("DROP TABLE users", vec![]).unwrap();
    }

    #[test]
    #[serial]
    fn test_transaction() {
        let db = setup_test_db();
        db.execute("DROP TABLE IF EXISTS users", vec![]).unwrap();
        db.execute(
            "CREATE TABLE users (id SERIAL PRIMARY KEY, name VARCHAR(255))",
            vec![],
        )
        .unwrap();

        db.begin_transaction().unwrap();
        db.execute(
            "INSERT INTO users (name) VALUES ($1)",
            vec![Value::Text("Alice".to_string())],
        )
        .unwrap();
        db.rollback().unwrap();

        let rows = db.query("SELECT * FROM users", vec![]).unwrap();
        assert_eq!(rows.len(), 0);

        db.begin_transaction().unwrap();
        db.execute(
            "INSERT INTO users (name) VALUES ($1)",
            vec![Value::Text("Bob".to_string())],
        )
        .unwrap();
        db.commit().unwrap();

        let rows = db.query("SELECT * FROM users", vec![]).unwrap();
        assert_eq!(rows.len(), 1);

        db.execute("DROP TABLE users", vec![]).unwrap();
    }
    
            #[test]
    #[serial]
    fn test_execute_foreign_key_violation() {
        let db = setup_test_db();

        // 创建父表和子表，子表中设置外键约束
        db.execute("DROP TABLE IF EXISTS child", vec![]).unwrap();
        db.execute("DROP TABLE IF EXISTS parent", vec![]).unwrap();
        db.execute(
            "CREATE TABLE parent (
                id SERIAL PRIMARY KEY
            )",
            vec![],
        )
        
        .unwrap();
        db.execute(
            "CREATE TABLE child (
                id SERIAL PRIMARY KEY,
                parent_id INT,
                CONSTRAINT fk_parent FOREIGN KEY (parent_id) REFERENCES parent(id)
            )",
            vec![],
        )
        
        .unwrap();

        // 插入子表时，使用一个不存在的 parent_id 触发外键约束错误
        let res = db
            .execute(
                "INSERT INTO child (parent_id) VALUES ($1)",
                vec![Value::Int(9999)], // 假设9999不存在
            )
            ;
        match res {
            Err(DbError::QueryError(QueryErrorKind::ForeignKeyViolation(msg))) => {
                println!("Foreign key violation error: {}", msg);
            }
            Err(e) => panic!("期望 ForeignKeyViolation, 但得到了其他错误: {:?}", e),
            Ok(_) => panic!("期望错误, 但执行成功"),
        }

        // 清理表
        db.execute("DROP TABLE child", vec![]).unwrap();
        db.execute("DROP TABLE parent", vec![]).unwrap();
    }

    #[test]
    #[serial]
    fn test_execute_unique_violation() {
        let db = setup_test_db();

        db.execute("DROP TABLE IF EXISTS unique_test", vec![])
            
            .unwrap();
        db.execute(
            "CREATE TABLE unique_test (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255) UNIQUE
            )",
            vec![],
        )
        
        .unwrap();

        // 插入第一条数据
        db.execute(
            "INSERT INTO unique_test (name) VALUES ($1)",
            vec![Value::Text("Alice".to_string())],
        )
        
        .unwrap();
        // 重复插入相同数据，触发唯一约束错误
        let res = db
            .execute(
                "INSERT INTO unique_test (name) VALUES ($1)",
                vec![Value::Text("Alice".to_string())],
            )
            ;
        match res {
            Err(DbError::QueryError(QueryErrorKind::UniqueViolation(msg))) => {
                println!("Unique violation error: {}", msg);
            }
            Err(e) => panic!("期望 UniqueViolation, 但得到了其他错误: {:?}", e),
            Ok(_) => panic!("期望错误, 但执行成功"),
        }

        db.execute("DROP TABLE unique_test", vec![]).unwrap();
    }

    #[test]
    #[serial]
    fn test_execute_not_null_violation() {
        let db = setup_test_db();

        db.execute("DROP TABLE IF EXISTS notnull_test", vec![])
            
            .unwrap();
        db.execute(
            "CREATE TABLE notnull_test (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )",
            vec![],
        )
        
        .unwrap();

        // 尝试插入 NULL 到 NOT NULL 列中
        let res = db
            .execute(
                "INSERT INTO notnull_test (name) VALUES ($1)",
                vec![Value::Null],
            )
            ;
        match res {
            Err(DbError::QueryError(QueryErrorKind::NotNullViolation(msg))) => {
                println!("Not null violation error: {}", msg);
            }
            Err(e) => panic!("期望 NotNullViolation, 但得到了其他错误: {:?}", e),
            Ok(_) => panic!("期望错误, 但执行成功"),
        }

        db.execute("DROP TABLE notnull_test", vec![]).unwrap();
    }

    #[test]
    #[serial]
    fn test_execute_check_violation() {
        let db = setup_test_db();

        db.execute("DROP TABLE IF EXISTS check_test", vec![])
            
            .unwrap();
        db.execute(
            "CREATE TABLE check_test (
                id SERIAL PRIMARY KEY,
                age INT,
                CONSTRAINT age_positive CHECK (age > 0)
            )",
            vec![],
        )
        
        .unwrap();

        // 插入不满足 check 条件的数据
        let res = db
            .execute(
                "INSERT INTO check_test (age) VALUES ($1)",
                vec![Value::Int(0)],
            )
            ;
        match res {
            Err(DbError::QueryError(QueryErrorKind::CheckViolation(msg))) => {
                println!("Check violation error: {}", msg);
            }
            Err(e) => panic!("期望 CheckViolation, 但得到了其他错误: {:?}", e),
            Ok(_) => panic!("期望错误, 但执行成功"),
        }

        db.execute("DROP TABLE check_test", vec![]).unwrap();
    }


}
