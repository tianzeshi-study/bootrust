use bootust::database::{
    RelationalDatabase,
    sqlite::SqliteDatabase,
    Value, DbError, Row, DatabaseConfig
};
use bootust::dao::Dao;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

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
struct UserDao;

impl Dao<User> for UserDao {
    type Database = SqliteDatabase;
    
    fn row_to_entity(row: Row) -> Result<User, DbError> {
        if row.values.len() != 5 {
            return Err(DbError::ConversionError("Invalid number of columns".to_string()));
        }
        
        Ok(User {
            id: match &row.values[0] {
                Value::Integer(i) => *i,
                _ => return Err(DbError::ConversionError("Invalid id type".to_string())),
            },
            username: match &row.values[1] {
                Value::Text(s) => s.clone(),
                _ => return Err(DbError::ConversionError("Invalid username type".to_string())),
            },
            email: match &row.values[2] {
                Value::Text(s) => s.clone(),
                _ => return Err(DbError::ConversionError("Invalid email type".to_string())),
            },
            created_at: match &row.values[3] {
                // Value::DateTime(dt) => *dt,
                Value::Text(dt) => dt.clone(),
                _ => return Err(DbError::ConversionError("Invalid created_at type".to_string())),
            },
            active: match &row.values[4] {
                // Value::Boolean(b) => *b as i64,
                Value::Integer(i) => *i,  
                _ => return Err(DbError::ConversionError("Invalid active type".to_string())),
            },
        })
    }
    
    fn entity_to_values(entity: &User) -> Vec<Value> {
        vec![
            Value::Integer(entity.id),
            Value::Text(entity.username.clone()),
            Value::Text(entity.email.clone()),
            // Value::DateTime(entity.created_at),
            Value::Text(entity.created_at.clone()),
            Value::Integer(entity.active),
        ]
    }

    fn entity_to_map(entity: &User) -> HashMap<String, Value> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), Value::Integer(entity.id));
        map.insert("username".to_string(), Value::Text(entity.username.clone()));
        map.insert("email".to_string(), Value::Text(entity.email.clone()));
        map.insert("created_at".to_string(), Value::Text(entity.created_at.clone()));
        map.insert("active".to_string(), Value::Integer(entity.active));
        map
    }
    
    fn table_name() -> String {
        "users".to_string()
    }
    
    fn primary_key_column() -> String {
        "id".to_string()
    }
}

fn setup_test_db() -> SqliteDatabase {
    let config = DatabaseConfig {
        database_name: ":memory:".to_string(),
        ..Default::default()
    };
    let db = SqliteDatabase::connect(config).unwrap();
    
    // 创建用户表
    db.execute(
        "CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            email TEXT NOT NULL,
            created_at TEXT NOT NULL,
            active INTEGER NOT NULL
        )",
        vec![]
    ).unwrap();
    
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
fn test_create_user() {
    let db = setup_test_db();
    let dao = UserDao;
    let user = create_test_user();
    
    let result = dao.create(&db, &user);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
}

#[test]
fn test_find_user_by_id() {
    let db = setup_test_db();
    let dao = UserDao;
    let user = create_test_user();
    
    // 先创建用户
    dao.create(&db, &user).unwrap();
    
    // 查找用户
    let found = dao.find_by_id(&db, Value::Integer(1)).unwrap();
    assert!(found.is_some());
    
    let found_user = found.unwrap();
    assert_eq!(found_user.id, user.id);
    assert_eq!(found_user.username, user.username);
    assert_eq!(found_user.email, user.email);
    assert_eq!(found_user.active, user.active);
}


    #[test]
    fn test_find_all_users() {
        let db = setup_test_db();
        let dao = UserDao;
        
        // 创建多个用户
        let user1 = create_test_user();
        let mut user2 = create_test_user();
        user2.id = 2;
        user2.email = "test2@example.com".to_string();
        
        dao.create(&db, &user1).unwrap();
        dao.create(&db, &user2).unwrap();
        
        // 查找所有用户
        let users = dao.find_all(&db).unwrap();
        assert_eq!(users.len(), 2);
    }

    #[test]
    fn test_update_user() {
        let db = setup_test_db();
        let mut user = create_test_user();
        let dao = UserDao;
        
        // 先创建用户
        dbg!(&user);
        dao.create(&db, &user).unwrap();

        
        // 更新用户信息
        user.email = "updated@example.com".to_string();
        let result = dao.update(&db, &user);

        assert!(result.is_ok());
        
        // 验证更新
        let updated = dao.find_by_id(&db, Value::Integer(1)).unwrap().unwrap();
        assert_eq!(updated.email, "updated@example.com");
    }

    #[test]
    fn test_delete_user() {
        let db = setup_test_db();
        let dao = UserDao;
        let user = create_test_user();
        
        // 先创建用户
        dao.create(&db, &user).unwrap();
        
        // 删除用户
        let result = dao.delete(&db, Value::Integer(1));
        assert!(result.is_ok());
        
        // 验证删除
        let found = dao.find_by_id(&db, Value::Integer(1)).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_find_by_condition() {
        let db = setup_test_db();
        let dao = UserDao;
        
        // 创建测试用户
        let user = create_test_user();
        dao.create(&db, &user).unwrap();
        
        // 按条件查询
        let users = dao.find_by_condition(
            &db,
            "username = ?",
            vec![Value::Text("test_user".to_string())]
        ).unwrap();
        
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].username, "test_user");
    }
