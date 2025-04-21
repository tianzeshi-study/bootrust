use crate::asyncdatabase::{
    DatabaseConfig, DbError, QueryErrorKind, RelationalDatabase, Row, Value,
};
use async_trait::async_trait;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::{NoTls, Row as TokioRow};

#[derive(Debug, Clone)]
pub struct PostgresDatabase {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl From<tokio_postgres::Error> for DbError {
    fn from(e: tokio_postgres::Error) -> Self {
        DbError::ConnectionError(e.to_string())
    }
}

#[async_trait]
impl RelationalDatabase for PostgresDatabase {
    fn placeholders(&self, keys: &[String]) -> Vec<String> {
        keys.iter()
            .enumerate()
            .map(|(i, _)| format!("${}", i + 1))
            .collect()
    }

    async fn connect(config: DatabaseConfig) -> Result<Self, DbError> {
        let manager = PostgresConnectionManager::new_from_stringlike(
            format!(
                "host={} port={} user={} password={} dbname={}",
                config.host, config.port, config.username, config.password, config.database_name
            ),
            NoTls,
        )?;

        let pool = Pool::builder()
            .max_size(config.max_size) // 使用配置中的 max_size
            .build(manager)
            .await
            .map_err(|e| DbError::PoolError(e.to_string()))?;

        Ok(PostgresDatabase { pool })
    }

    async fn close(&self) -> Result<(), DbError> {
        // bb8 的 Pool 会在 Drop 时自动关闭连接，无需手动关闭
        Ok(())
    }

    async fn ping(&self) -> Result<(), DbError> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| DbError::PoolError(e.to_string()))?;
        conn.simple_query("")
            .await
            .map(|_| ())
            .map_err(|e| DbError::ConnectionError(e.to_string()))
    }

    async fn begin_transaction(&self) -> Result<(), DbError> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| DbError::PoolError(e.to_string()))?;
        conn.execute("BEGIN", &[])
            .await
            .map(|_| ())
            .map_err(|e| DbError::TransactionError(e.to_string()))
    }

    async fn commit(&self) -> Result<(), DbError> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| DbError::PoolError(e.to_string()))?;
        conn.execute("COMMIT", &[])
            .await
            .map(|_| ())
            .map_err(|e| DbError::TransactionError(e.to_string()))
    }

    async fn rollback(&self) -> Result<(), DbError> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| DbError::PoolError(e.to_string()))?;
        conn.execute("ROLLBACK", &[])
            .await
            .map(|_| ())
            .map_err(|e| DbError::TransactionError(e.to_string()))
    }

    async fn execute(&self, query: &str, params: Vec<Value>) -> Result<u64, DbError> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| DbError::PoolError(e.to_string()))?;

        let params = Self::params_to_postgres(&params);

        let stmt = conn.prepare(&query).await?;
        conn.execute(&stmt, &params).await.map_err(|e| {
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
                DbError::QueryError(QueryErrorKind::Other(format!("message: {}", e.to_string())))
            }
        })
    }

    async fn query(&self, query: &str, params: Vec<Value>) -> Result<Vec<Row>, DbError> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| DbError::PoolError(e.to_string()))?;
        let params = Self::params_to_postgres(&params);
        let stmt = conn.prepare(&query).await?;
        let rows = conn
            .query(&stmt, &params[..])
            .await
            .map_err(|e| DbError::QueryError(e.to_string().into()))?;
        Ok(Self::convert_rows(rows))
    }
    async fn query_one(&self, query: &str, params: Vec<Value>) -> Result<Option<Row>, DbError> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| DbError::PoolError(e.to_string()))?;
        let params = Self::params_to_postgres(&params);
        let stmt = conn.prepare(&query).await?;

        let row = conn
            .query_opt(&stmt, &params[..])
            .await
            .map_err(|e| DbError::QueryError(e.to_string().into()))?;
        Ok(row
            .map(|r| Self::convert_rows(vec![r]))
            .and_then(|mut v| v.pop()))
    }
}

impl PostgresDatabase {
    fn convert_rows(rows: Vec<TokioRow>) -> Vec<Row> {
        let mut result_rows = Vec::new();
        for row in rows {
            let mut columns = Vec::new();
            let mut values = Vec::new();
            for (i, column) in row.columns().iter().enumerate() {
                columns.push(column.name().to_string());
                // 根据列的类型进行值的转换
                let value = match column.type_() {
                    &tokio_postgres::types::Type::INT4 => Value::Int(row.get(i)),
                    &tokio_postgres::types::Type::INT8 => {
                        let v: Option<i64> = row.get(i);

                        Value::Bigint(v.unwrap_or(0))
                    }
                    &tokio_postgres::types::Type::TEXT => {
                        let v: Option<String> = row.get(i);
                        // Value::Text(v.unwrap_or("1900-01-01T00:00:00.000000000Z".to_string()))
                        Value::Text(v.unwrap_or("".to_string()))
                    }
                    &tokio_postgres::types::Type::VARCHAR => Value::Text(row.get(i)),
                    &tokio_postgres::types::Type::BPCHAR => Value::Text(row.get(i)),
                    &tokio_postgres::types::Type::FLOAT4 => Value::Float(row.get(i)),
                    &tokio_postgres::types::Type::FLOAT8 => Value::Double(row.get(i)),
                    &tokio_postgres::types::Type::BOOL => Value::Boolean(row.get(i)),
                    &tokio_postgres::types::Type::BYTEA => Value::Bytes(row.get(i)),
                    &tokio_postgres::types::Type::TIMESTAMPTZ => {
                        Value::DateTime(row.get(i)) // 对应 Rust 中的 chrono::DateTime<chrono::Utc>
                    }
                    &tokio_postgres::types::Type::VOID => Value::Null,
                    // ... 其他类型的处理
                    _ => {
                        unimplemented!()
                    }
                };
                values.push(value);
            }
            result_rows.push(Row { columns, values });
        }
        result_rows
    }

    fn params_to_postgres(params: &Vec<Value>) -> Vec<&(dyn tokio_postgres::types::ToSql + Sync)> {
        params
            .iter()
            .map(|v| match v {
                Value::Int(i) => i as &(dyn tokio_postgres::types::ToSql + Sync),
                Value::Bigint(i) => i as &(dyn tokio_postgres::types::ToSql + Sync),
                Value::Text(s) => s as &(dyn tokio_postgres::types::ToSql + Sync),
                Value::Varchar(s) => s as &(dyn tokio_postgres::types::ToSql + Sync),
                Value::Float(f) => f as &(dyn tokio_postgres::types::ToSql + Sync),
                Value::Double(d) => d as &(dyn tokio_postgres::types::ToSql + Sync),
                Value::Boolean(b) => b as &(dyn tokio_postgres::types::ToSql + Sync),
                Value::Bytes(by) => by as &(dyn tokio_postgres::types::ToSql + Sync),
                Value::DateTime(dt) => dt as &(dyn tokio_postgres::types::ToSql + Sync),
                Value::Null => &None::<&str> as &(dyn tokio_postgres::types::ToSql + Sync),
                // ... 其他 Value 类型的处理
                _ => unimplemented!(),
            })
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serial_test::serial;

    async fn setup_test_db() -> PostgresDatabase {
        let config = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            username: "root".to_string(),
            password: "root".to_string(),
            database_name: "test".to_string(),
            max_size: 10,
        };
        PostgresDatabase::connect(config).await.unwrap()
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
            "CREATE TABLE users (id SERIAL PRIMARY KEY, name VARCHAR(255), age INT)",
            vec![],
        )
        .await
        .unwrap();

        let affected_rows = db
            .execute(
                "INSERT INTO users (name, age) VALUES ($1, $2)",
                vec![Value::Text("Alice".to_string()), Value::Int(30)],
            )
            .await
            .unwrap();
        assert_eq!(affected_rows, 1);

        let affected_rows = db
            .execute(
                "UPDATE users SET age = $1 WHERE name = $2",
                vec![Value::Int(31), Value::Text("Alice".to_string())],
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
            "CREATE TABLE users (id SERIAL PRIMARY KEY, name VARCHAR(255), age INT8, created_at TIMESTAMP WITH TIME ZONE)",
            vec![],
        )
        .await
        .unwrap();

        let now = Utc::now();
        db.execute(
            "INSERT INTO users (name, age, created_at) VALUES ($1, $2, $3)",
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
        assert!(matches!(rows[0].values[0], Value::Int(_)));
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
            "CREATE TABLE users (id SERIAL PRIMARY KEY, name VARCHAR(255))",
            vec![],
        )
        .await
        .unwrap();

        db.execute(
            "INSERT INTO users (name) VALUES ($1), ($2)",
            vec![
                Value::Text("Alice".to_string()),
                Value::Text("Bob".to_string()),
            ],
        )
        .await
        .unwrap();

        let row = db
            .query_one(
                "SELECT id, name FROM users WHERE name = $1",
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
                "SELECT * FROM users WHERE name = $1",
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
            "CREATE TABLE users (id SERIAL PRIMARY KEY, name VARCHAR(255))",
            vec![],
        )
        .await
        .unwrap();

        db.begin_transaction().await.unwrap();
        db.execute(
            "INSERT INTO users (name) VALUES ($1)",
            vec![Value::Text("Alice".to_string())],
        )
        .await
        .unwrap();
        db.rollback().await.unwrap();

        let rows = db.query("SELECT * FROM users", vec![]).await.unwrap();
        assert_eq!(rows.len(), 0);

        db.begin_transaction().await.unwrap();
        db.execute(
            "INSERT INTO users (name) VALUES ($1)",
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
        let row = db
            .query_one(
                "SELECT $1::timestamp with time zone",
                vec![Value::DateTime(now)],
            )
            .await
            .unwrap()
            .unwrap();
        if let Value::DateTime(dt) = &row.values[0] {
            assert!((dt.timestamp_micros() - now.timestamp_micros()).abs() <= 1);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_execute_foreign_key_violation() {
        let db = setup_test_db().await;

        // 创建父表和子表，子表中设置外键约束
        db.execute("DROP TABLE IF EXISTS child", vec![])
            .await
            .unwrap();
        db.execute("DROP TABLE IF EXISTS parent", vec![])
            .await
            .unwrap();
        db.execute(
            "CREATE TABLE parent (
                id SERIAL PRIMARY KEY
            )",
            vec![],
        )
        .await
        .unwrap();
        db.execute(
            "CREATE TABLE child (
                id SERIAL PRIMARY KEY,
                parent_id INT,
                CONSTRAINT fk_parent FOREIGN KEY (parent_id) REFERENCES parent(id)
            )",
            vec![],
        )
        .await
        .unwrap();

        // 插入子表时，使用一个不存在的 parent_id 触发外键约束错误
        let res = db
            .execute(
                "INSERT INTO child (parent_id) VALUES ($1)",
                vec![Value::Int(9999)], // 假设9999不存在
            )
            .await;
        match res {
            Err(DbError::QueryError(QueryErrorKind::ForeignKeyViolation(msg))) => {
                println!("Foreign key violation error: {}", msg);
            }
            Err(e) => panic!("期望 ForeignKeyViolation, 但得到了其他错误: {:?}", e),
            Ok(_) => panic!("期望错误, 但执行成功"),
        }

        // 清理表
        db.execute("DROP TABLE child", vec![]).await.unwrap();
        db.execute("DROP TABLE parent", vec![]).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_execute_unique_violation() {
        let db = setup_test_db().await;

        db.execute("DROP TABLE IF EXISTS unique_test", vec![])
            .await
            .unwrap();
        db.execute(
            "CREATE TABLE unique_test (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255) UNIQUE
            )",
            vec![],
        )
        .await
        .unwrap();

        // 插入第一条数据
        db.execute(
            "INSERT INTO unique_test (name) VALUES ($1)",
            vec![Value::Text("Alice".to_string())],
        )
        .await
        .unwrap();
        // 重复插入相同数据，触发唯一约束错误
        let res = db
            .execute(
                "INSERT INTO unique_test (name) VALUES ($1)",
                vec![Value::Text("Alice".to_string())],
            )
            .await;
        match res {
            Err(DbError::QueryError(QueryErrorKind::UniqueViolation(msg))) => {
                println!("Unique violation error: {}", msg);
            }
            Err(e) => panic!("期望 UniqueViolation, 但得到了其他错误: {:?}", e),
            Ok(_) => panic!("期望错误, 但执行成功"),
        }

        db.execute("DROP TABLE unique_test", vec![]).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_execute_not_null_violation() {
        let db = setup_test_db().await;

        db.execute("DROP TABLE IF EXISTS notnull_test", vec![])
            .await
            .unwrap();
        db.execute(
            "CREATE TABLE notnull_test (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )",
            vec![],
        )
        .await
        .unwrap();

        // 尝试插入 NULL 到 NOT NULL 列中
        let res = db
            .execute(
                "INSERT INTO notnull_test (name) VALUES ($1)",
                vec![Value::Null],
            )
            .await;
        match res {
            Err(DbError::QueryError(QueryErrorKind::NotNullViolation(msg))) => {
                println!("Not null violation error: {}", msg);
            }
            Err(e) => panic!("期望 NotNullViolation, 但得到了其他错误: {:?}", e),
            Ok(_) => panic!("期望错误, 但执行成功"),
        }

        db.execute("DROP TABLE notnull_test", vec![]).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_execute_check_violation() {
        let db = setup_test_db().await;

        db.execute("DROP TABLE IF EXISTS check_test", vec![])
            .await
            .unwrap();
        db.execute(
            "CREATE TABLE check_test (
                id SERIAL PRIMARY KEY,
                age INT,
                CONSTRAINT age_positive CHECK (age > 0)
            )",
            vec![],
        )
        .await
        .unwrap();

        // 插入不满足 check 条件的数据
        let res = db
            .execute(
                "INSERT INTO check_test (age) VALUES ($1)",
                vec![Value::Int(0)],
            )
            .await;
        match res {
            Err(DbError::QueryError(QueryErrorKind::CheckViolation(msg))) => {
                println!("Check violation error: {}", msg);
            }
            Err(e) => panic!("期望 CheckViolation, 但得到了其他错误: {:?}", e),
            Ok(_) => panic!("期望错误, 但执行成功"),
        }

        db.execute("DROP TABLE check_test", vec![]).await.unwrap();
    }
}
