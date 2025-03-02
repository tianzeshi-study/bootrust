use bootrust::dao::Dao;
use bootrust::database::{
    mysql::MySqlDatabase, DatabaseConfig, DbError, RelationalDatabase, Row, Value,
};
use chrono::Utc;
use serial_test::serial;
use std::marker::PhantomData;

// User实体
#[derive(Debug, Clone, PartialEq)]
struct User {
    id: i64,
    username: String,
    email: String,
    // created_at: DateTime<Utc>,
    created_at: String,
    active: i64,
}

// UserDao实现
struct UserDao<T: Sized> {
    database: MySqlDatabase,
    _table: PhantomData<T>,
}

impl Dao<User> for UserDao<User> {
    type Database = MySqlDatabase;

    fn new(database: Self::Database) -> Self {
        UserDao {
            database,
            _table: PhantomData,
        }
    }

    fn database(&self) -> &Self::Database {
        &self.database
    }
    fn row_to_entity(row: Row) -> Result<User, DbError> {
        if row.values.len() != 5 {
            return Err(DbError::ConversionError(
                "Invalid number of columns".to_string(),
            ));
        }

        Ok(User {
            id: match &row.values[0] {
                Value::Bigint(i) => *i,
                _ => return Err(DbError::ConversionError("Invalid id type".to_string())),
            },
            username: match &row.values[1] {
                Value::Text(s) => s.clone(),
                _ => {
                    return Err(DbError::ConversionError(
                        "Invalid username type".to_string(),
                    ))
                }
            },
            email: match &row.values[2] {
                Value::Text(s) => s.clone(),
                _ => return Err(DbError::ConversionError("Invalid email type".to_string())),
            },
            created_at: match &row.values[3] {
                // Value::DateTime(dt) => *dt,
                Value::Text(dt) => dt.clone(),
                _ => {
                    return Err(DbError::ConversionError(
                        "Invalid created_at type".to_string(),
                    ))
                }
            },
            active: match &row.values[4] {
                // Value::Boolean(b) => *b as i64,
                Value::Bigint(i) => *i,
                _ => return Err(DbError::ConversionError("Invalid active type".to_string())),
            },
        })
    }

    fn entity_to_map(entity: &User) -> Vec<(String, Value)> {
        let mut map = Vec::new();
        map.push(("id".to_string(), Value::Bigint(entity.id)));
        map.push(("username".to_string(), Value::Text(entity.username.clone())));
        map.push(("email".to_string(), Value::Text(entity.email.clone())));
        map.push((
            "created_at".to_string(),
            Value::Text(entity.created_at.clone()),
        ));
        map.push(("active".to_string(), Value::Bigint(entity.active)));
        map
    }

    fn table_name() -> String {
        "users".to_string()
    }

    fn primary_key_column() -> String {
        "id".to_string()
    }
}

fn setup_test_db() -> MySqlDatabase {
    let config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 3306,
        username: "root".to_string(),
        password: "root".to_string(),
        database_name: "test".to_string(),
        max_size: 15,
    };
    let db = MySqlDatabase::connect(config).unwrap();

    // 创建用户表
    db.execute("DROP TABLE IF EXISTS users", vec![]).unwrap();
    db.execute(
        "CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            email TEXT NOT NULL,
            created_at TEXT NOT NULL,
            active INTEGER NOT NULL
        )",
        vec![],
    )
    .unwrap();

    db
}

fn create_test_user() -> User {
    User {
        id: 1,
        username: "test_user".to_string(),
        email: "test@example.com".to_string(),
        created_at: Utc::now().to_string(),
        active: 1,
    }
}

#[test]
#[serial]
fn test_create_user() {
    let db = setup_test_db();
    let dao = UserDao::new(db);
    let user = create_test_user();

    let result = dao.create(&user);
    dbg!(&result);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
}

#[test]
#[serial]
fn test_find_user_by_id() {
    let db = setup_test_db();
    let dao = UserDao::new(db);
    let user = create_test_user();

    // 先创建用户
    dao.create(&user).unwrap();

    // 查找用户
    let found = dao.find_by_id(Value::Bigint(1)).unwrap();
    assert!(found.is_some());

    let found_user = found.unwrap();
    assert_eq!(found_user.id, user.id);
    assert_eq!(found_user.username, user.username);
    assert_eq!(found_user.email, user.email);
    assert_eq!(found_user.active, user.active);
}

#[test]
#[serial]
fn test_find_all_users() {
    let db = setup_test_db();
    let dao = UserDao::new(db);

    // 创建多个用户
    let user1 = create_test_user();
    let mut user2 = create_test_user();
    user2.id = 2;
    user2.email = "test2@example.com".to_string();

    dao.create(&user1).unwrap();
    dao.create(&user2).unwrap();

    // 查找所有用户
    let users = dao.find_all().unwrap();
    assert_eq!(users.len(), 2);
}

#[test]
#[serial]
fn test_update_user() {
    let db = setup_test_db();
    let mut user = create_test_user();
    let dao = UserDao::new(db);

    // 先创建用户
    dbg!(&user);
    dao.create(&user).unwrap();

    // 更新用户信息
    user.email = "updated@example.com".to_string();
    // let result = dao.update(&db, &user);
    let result = dao.update(&user);

    assert!(result.is_ok());

    // 验证更新
    let updated = dao.find_by_id(Value::Bigint(1)).unwrap().unwrap();
    assert_eq!(updated.email, "updated@example.com");
}

#[test]
#[serial]
fn test_delete_user() {
    let db = setup_test_db();
    let dao = UserDao::new(db);
    let user = create_test_user();

    // 先创建用户
    dao.create(&user).unwrap();

    // 删除用户
    let result = dao.delete(Value::Bigint(1));
    assert!(result.is_ok());

    // 验证删除
    let found = dao.find_by_id(Value::Bigint(1)).unwrap();
    assert!(found.is_none());
}

#[test]
#[serial]
fn test_find_by_condition() {
    let db = setup_test_db();
    let dao = UserDao::new(db);

    // 创建测试用户
    let user = create_test_user();
    dao.create(&user).unwrap();

    // 按条件查询
    let users = dao
        .find_by_condition("username = ?", vec![Value::Text("test_user".to_string())])
        .unwrap();

    assert_eq!(users.len(), 1);
    assert_eq!(users[0].username, "test_user");
}
