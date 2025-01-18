use bootust::dao::Dao;
use bootust::database::{
    sqlite::SqliteDatabase, DatabaseConfig, DbError, RelationalDatabase, Row, Value,
};
use chrono::{DateTime, Utc};
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
struct UserDao<T:Sized> {
    _marker: PhantomData<T>,
    database: SqliteDatabase,
}

impl Dao<User> for UserDao<User> {
    type Database = SqliteDatabase;

    fn new(database: Self::Database) -> Self {
        UserDao { 
            _marker: PhantomData,
            database 
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
                Value::Integer(i) => *i,
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
                Value::Integer(i) => *i,
                _ => return Err(DbError::ConversionError("Invalid active type".to_string())),
            },
        })
    }

    
    fn entity_to_map(entity: &User) -> Vec<(String, Value)> {
        let mut map = Vec::new();
        map.push(("id".to_string(), Value::Integer(entity.id)));
        map.push(("username".to_string(), Value::Text(entity.username.clone())));
        map.push(("email".to_string(), Value::Text(entity.email.clone())));
        map.push((
            "created_at".to_string(),
            Value::Text(entity.created_at.clone()),
        ));
        map.push(("active".to_string(), Value::Integer(entity.active)));
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
fn test_create_user() {
    let db = setup_test_db();
    // let dao = UserDao::new(db);
    let dao: UserDao<User> = UserDao::new(db);
    let user = create_test_user();

    let result = dao.create(&user);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
}

#[test]
fn test_find_user_by_id() {
    let db = setup_test_db();
    let dao = UserDao::new(db);
    let user = create_test_user();

    // 先创建用户
    dao.create(&user).unwrap();

    // 查找用户
    let found = dao.find_by_id(Value::Integer(1)).unwrap();
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
    let updated = dao.find_by_id(Value::Integer(1)).unwrap().unwrap();
    assert_eq!(updated.email, "updated@example.com");
}

#[test]
fn test_delete_user() {
    let db = setup_test_db();
    let dao = UserDao::new(db);
    let user = create_test_user();

    // 先创建用户
    dao.create(&user).unwrap();

    // 删除用户
    let result = dao.delete(Value::Integer(1));
    assert!(result.is_ok());

    // 验证删除
    let found = dao.find_by_id(Value::Integer(1)).unwrap();
    assert!(found.is_none());
}

#[test]
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



#[derive(Debug, Clone, PartialEq)]
struct VIPUser {
    id: i64,
    vip_username: String,
    email: String,
    // created_at: DateTime<Utc>,
    created_at: String,
    active: i64,
}

// UserDao实现
// struct UserDao {
    // database: SqliteDatabase,
// }

impl Dao<VIPUser> for UserDao<VIPUser> {
    type Database = SqliteDatabase;

    fn new(database: Self::Database) -> Self {
        UserDao { 
            _marker: PhantomData,
            database 
        }
    }

    fn database(&self) -> &Self::Database {
        &self.database
    }
    fn row_to_entity(row: Row) -> Result<VIPUser, DbError> {
        if row.values.len() != 5 {
            return Err(DbError::ConversionError(
                "Invalid number of columns".to_string(),
            ));
        }

        Ok(VIPUser {
            id: match &row.values[0] {
                Value::Integer(i) => *i,
                _ => return Err(DbError::ConversionError("Invalid id type".to_string())),
            },
            vip_username: match &row.values[1] {
                Value::Text(s) => s.clone(),
                _ => {
                    return Err(DbError::ConversionError(
                        "Invalid vip_username type".to_string(),
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
                Value::Integer(i) => *i,
                _ => return Err(DbError::ConversionError("Invalid active type".to_string())),
            },
        })
    }

    
    fn entity_to_map(entity: &VIPUser) -> Vec<(String, Value)> {
        let mut map = Vec::new();
        map.push(("id".to_string(), Value::Integer(entity.id)));
        map.push(("vip_username".to_string(), Value::Text(entity.vip_username.clone())));
        map.push(("email".to_string(), Value::Text(entity.email.clone())));
        map.push((
            "created_at".to_string(),
            Value::Text(entity.created_at.clone()),
        ));
        map.push(("active".to_string(), Value::Integer(entity.active)));
        map
    }

    fn table_name() -> String {
        "vip_users".to_string()
    }

    fn primary_key_column() -> String {
        "id".to_string()
    }
}

fn setup_test2_db() -> SqliteDatabase {
    let config = DatabaseConfig {
        database_name: ":memory:".to_string(),
        ..Default::default()
    };
    let db = SqliteDatabase::connect(config).unwrap();

    // 创建用户表
    db.execute(
        "CREATE TABLE vip_users (
            id INTEGER PRIMARY KEY,
            vip_username TEXT NOT NULL,
            email TEXT NOT NULL,
            created_at TEXT NOT NULL,
            active INTEGER NOT NULL
        )",
        vec![],
    )
    .unwrap();

    db
}

fn create_test_vip_user() -> VIPUser {
    VIPUser {
        id: 1,
        vip_username: "test_vip_user".to_string(),
        email: "test@example.com".to_string(),
        created_at: Utc::now().to_string(),
        active: 1,
    }
}

#[test]
fn test_create_vip_user() {
    let db = setup_test2_db();
    let dao = UserDao::new(db);
    let vip_user = create_test_vip_user();

    let result = dao.create(&vip_user);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
}

#[test]
fn test_find_vip_user_by_id() {
    let db = setup_test2_db();
    let dao = UserDao::new(db);
    let vip_user = create_test_vip_user();

    // 先创建用户
    dao.create(&vip_user).unwrap();

    // 查找用户
    let found = dao.find_by_id(Value::Integer(1)).unwrap();
    assert!(found.is_some());

    let found_vip_user = found.unwrap();
    assert_eq!(found_vip_user.id, vip_user.id);
    assert_eq!(found_vip_user.vip_username, vip_user.vip_username);
    assert_eq!(found_vip_user.email, vip_user.email);
    assert_eq!(found_vip_user.active, vip_user.active);
}

#[test]
fn test_find_all_vip_users() {
    let db = setup_test2_db();
    let dao = UserDao::new(db);

    // 创建多个用户
    let vip_user1 = create_test_vip_user();
    let mut vip_user2 = create_test_vip_user();
    vip_user2.id = 2;
    vip_user2.email = "test2@example.com".to_string();

    dao.create(&vip_user1).unwrap();
    dao.create(&vip_user2).unwrap();

    // 查找所有用户
    let vip_users = dao.find_all().unwrap();
    assert_eq!(vip_users.len(), 2);
}

#[test]
fn test_update_vip_user() {
    let db = setup_test2_db();
    let mut vip_user = create_test_vip_user();
    let dao = UserDao::new(db);

    // 先创建用户
    dbg!(&vip_user);
    dao.create(&vip_user).unwrap();

    // 更新用户信息
    vip_user.email = "updated@example.com".to_string();
    // let result = dao.update(&db, &vip_user);
    let result = dao.update(&vip_user);

    assert!(result.is_ok());

    // 验证更新
    let updated = dao.find_by_id(Value::Integer(1)).unwrap().unwrap();
    assert_eq!(updated.email, "updated@example.com");
}

#[test]
fn test_delete_vip_user() {
    let db = setup_test2_db();
    let dao = UserDao::new(db);
    let vip_user = create_test_vip_user();

    // 先创建用户
    dao.create(&vip_user).unwrap();

    // 删除用户
    let result = dao.delete(Value::Integer(1));
    assert!(result.is_ok());

    // 验证删除
    let found = dao.find_by_id(Value::Integer(1)).unwrap();
    assert!(found.is_none());
}

#[test]
fn test_find_by_vip_condition() {
    let db = setup_test2_db();
    let dao = UserDao::new(db);

    // 创建测试用户
    let vip_user = create_test_vip_user();
    dao.create(&vip_user).unwrap();

    // 按条件查询
    let vip_users = dao
        .find_by_condition("vip_username = ?", vec![Value::Text("test_vip_user".to_string())])
        .unwrap();

    assert_eq!(vip_users.len(), 1);
    assert_eq!(vip_users[0].vip_username, "test_vip_user");
}

#[derive(Debug, Clone, PartialEq)]
struct Order {
    id: i64,
    user_id: i64,
    product_name: String,
    amount: f64,
    order_time: String,
}

// OrderDao实现
impl Dao<Order> for UserDao<Order> {
    type Database = SqliteDatabase;

    fn new(database: Self::Database) -> Self {
        UserDao {
            _marker: PhantomData,
            database,
        }
    }

    fn database(&self) -> &Self::Database {
        &self.database
    }

    fn row_to_entity(row: Row) -> Result<Order, DbError> {
        if row.values.len() != 5 {
            return Err(DbError::ConversionError(
                "Invalid number of columns".to_string(),
            ));
        }

        Ok(Order {
            id: match &row.values[0] {
                Value::Integer(i) => *i,
                _ => return Err(DbError::ConversionError("Invalid id type".to_string())),
            },
            user_id: match &row.values[1] {
                Value::Integer(i) => *i,
                _ => {
                    return Err(DbError::ConversionError(
                        "Invalid user_id type".to_string(),
                    ))
                }
            },
            product_name: match &row.values[2] {
                Value::Text(s) => s.clone(),
                _ => {
                    return Err(DbError::ConversionError(
                        "Invalid product_name type".to_string(),
                    ))
                }
            },
            amount: match &row.values[3] {
                Value::Double(f) => *f,
                _ => {
                    return Err(DbError::ConversionError(
                        "Invalid amount type".to_string(),
                    ))
                }
            },
            order_time: match &row.values[4] {
                Value::Text(dt) => dt.clone(),
                _ => {
                    return Err(DbError::ConversionError(
                        "Invalid order_time type".to_string(),
                    ))
                }
            },
        })
    }

    fn entity_to_map(entity: &Order) -> Vec<(String, Value)> {
        let mut map = Vec::new();
        map.push(("id".to_string(), Value::Integer(entity.id)));
        map.push(("user_id".to_string(), Value::Integer(entity.user_id)));
        map.push((
            "product_name".to_string(),
            Value::Text(entity.product_name.clone()),
        ));
        map.push(("amount".to_string(), Value::Double(entity.amount)));
        map.push((
            "order_time".to_string(),
            Value::Text(entity.order_time.clone()),
        ));
        map
    }

    fn table_name() -> String {
        "orders".to_string()
    }

    fn primary_key_column() -> String {
        "id".to_string()
    }
}

fn setup_test3_db() -> SqliteDatabase {
    let config = DatabaseConfig {
        database_name: ":memory:".to_string(),
        ..Default::default()
    };
    let db = SqliteDatabase::connect(config).unwrap();

    // 创建订单表
    db.execute(
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            product_name TEXT NOT NULL,
            amount Float NOT NULL,
            order_time TEXT NOT NULL
        )",
        vec![],
    )
    .unwrap();

    db
}

fn create_test_order() -> Order {
    Order {
        id: 1,
        user_id: 1,
        product_name: "Test Product".to_string(),
        amount: 100.0,
        order_time: Utc::now().to_string(),
    }
}

#[test]
fn test_create_order() {
    let db = setup_test3_db();
    let dao = UserDao::new(db);
    let order = create_test_order();

    let result = dao.create(&order);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
}

#[test]
fn test_find_order_by_id() {
    let db = setup_test3_db();
    let dao = UserDao::new(db);
    let order = create_test_order();

    // 先创建订单
    dao.create(&order).unwrap();

    // 查找订单
    let found = dao.find_by_id(Value::Integer(1)).unwrap();
    assert!(found.is_some());

    let found_order = found.unwrap();
    assert_eq!(found_order.id, order.id);
    assert_eq!(found_order.user_id, order.user_id);
    assert_eq!(found_order.product_name, order.product_name);
    assert_eq!(found_order.amount, order.amount);
    assert_eq!(found_order.order_time, order.order_time);
}

#[test]
fn test_find_all_orders() {
    let db = setup_test3_db();
    let dao = UserDao::new(db);

    // 创建多个订单
    let order1 = create_test_order();
    let mut order2 = create_test_order();
    order2.id = 2;
    order2.product_name = "Another Product".to_string();

    dao.create(&order1).unwrap();
    dao.create(&order2).unwrap();

    // 查找所有订单
    let orders = dao.find_all().unwrap();
    assert_eq!(orders.len(), 2);
}

#[test]
fn test_update_order() {
    let db = setup_test3_db();
    let mut order = create_test_order();
    let dao = UserDao::new(db);

    // 先创建订单
    dao.create(&order).unwrap();

    // 更新订单信息
    order.product_name = "Updated Product".to_string();
    let result = dao.update(&order);

    assert!(result.is_ok());

    // 验证更新
    let updated = dao.find_by_id(Value::Integer(1)).unwrap().unwrap();
    assert_eq!(updated.product_name, "Updated Product");
}

#[test]
fn test_delete_order() {
    let db = setup_test3_db();
    let dao = UserDao::new(db);
    let order = create_test_order();

    // 先创建订单
    dao.create(&order).unwrap();

    // 删除订单
    let result = dao.delete(Value::Integer(1));
    assert!(result.is_ok());

    // 验证删除
    let found = dao.find_by_id(Value::Integer(1)).unwrap();
    assert!(found.is_none());
}

#[test]
fn test_find_by_order_condition() {
    let db = setup_test3_db();
    let dao = UserDao::new(db);

    // 创建测试订单
    let order = create_test_order();
    dao.create(&order).unwrap();

    // 按条件查询
    let orders = dao
        .find_by_condition(
            "product_name = ?",
            vec![Value::Text("Test Product".to_string())],
        )
        .unwrap();

    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].product_name, "Test Product");
}
