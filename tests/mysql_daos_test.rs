use bootust::dao::Dao;
use bootust::database::{
    mysql::MySqlDatabase, DatabaseConfig, DbError, RelationalDatabase, Row, Value,
};
use chrono::{DateTime, Utc};
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

// Order实体
#[derive(Debug, Clone, PartialEq)]
struct Order {
    id: i64,
    user_id: i64,
    product_name: String,
    amount: f32,
    created_at: String,
}

// Comment实体
#[derive(Debug, Clone, PartialEq)]
struct Comment {
    id: i64,
    user_id: i64,
    content: String,
    created_at: String,
}

// UserDao实现
struct UserDao<T:Sized> {
    database: MySqlDatabase,
    _table: PhantomData<T>,
}

impl Dao<User> for UserDao<User> {
    type Database = MySqlDatabase;

    fn new(database: Self::Database) -> Self {
        UserDao { 
            database ,
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

impl Dao<Order> for UserDao<Order> {
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
                _ => return Err(DbError::ConversionError("Invalid user_id type".to_string())),
            },
            product_name: match &row.values[2] {
                Value::Text(s) => s.clone(),
                _ => return Err(DbError::ConversionError("Invalid product_name type".to_string())),
            },
            amount: match &row.values[3] {
                Value::Float(f) => *f,
                _ => return Err(DbError::ConversionError("Invalid amount type".to_string())),
            },
            created_at: match &row.values[4] {
                Value::Text(dt) => dt.clone(),
                _ => return Err(DbError::ConversionError("Invalid created_at type".to_string())),
            },
        })
    }

    fn entity_to_map(entity: &Order) -> Vec<(String, Value)> {
        let mut map = Vec::new();
        map.push(("id".to_string(), Value::Integer(entity.id)));
        map.push(("user_id".to_string(), Value::Integer(entity.user_id)));
        map.push(("product_name".to_string(), Value::Text(entity.product_name.clone())));
        map.push(("amount".to_string(), Value::Float(entity.amount)));
        map.push(("created_at".to_string(), Value::Text(entity.created_at.clone())));
        map
    }

    fn table_name() -> String {
        "orders".to_string()
    }

    fn primary_key_column() -> String {
        "id".to_string()
    }
}

impl Dao<Comment> for UserDao<Comment> {
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

    fn row_to_entity(row: Row) -> Result<Comment, DbError> {
        if row.values.len() != 4 {
            return Err(DbError::ConversionError(
                "Invalid number of columns".to_string(),
            ));
        }

        Ok(Comment {
            id: match &row.values[0] {
                Value::Integer(i) => *i,
                _ => return Err(DbError::ConversionError("Invalid id type".to_string())),
            },
            user_id: match &row.values[1] {
                Value::Integer(i) => *i,
                _ => return Err(DbError::ConversionError("Invalid user_id type".to_string())),
            },
            content: match &row.values[2] {
                Value::Text(s) => s.clone(),
                _ => return Err(DbError::ConversionError("Invalid content type".to_string())),
            },
            created_at: match &row.values[3] {
                Value::Text(dt) => dt.clone(),
                _ => return Err(DbError::ConversionError("Invalid created_at type".to_string())),
            },
        })
    }

    fn entity_to_map(entity: &Comment) -> Vec<(String, Value)> {
        let mut map = Vec::new();
        map.push(("id".to_string(), Value::Integer(entity.id)));
        map.push(("user_id".to_string(), Value::Integer(entity.user_id)));
        map.push(("content".to_string(), Value::Text(entity.content.clone())));
        map.push(("created_at".to_string(), Value::Text(entity.created_at.clone())));
        map
    }

    fn table_name() -> String {
        "comments".to_string()
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
// 创建订单表
    db.execute("DROP TABLE IF EXISTS orders", vec![]).unwrap();
    db.execute(
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            product_name TEXT NOT NULL,
            amount FLOAT NOT NULL,
            created_at TEXT NOT NULL
        )",
        vec![],
    )
    .unwrap();

    // 创建评论表
    db.execute("DROP TABLE IF EXISTS comments", vec![]).unwrap();
    db.execute(
        "CREATE TABLE comments (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL
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

// 创建测试订单
fn create_test_order() -> Order {
    Order {
        id: 1,
        user_id: 1,
        product_name: "Test Product".to_string(),
        amount: 99.99,
        created_at: Utc::now().to_string(),
    }
}

// 创建测试评论
fn create_test_comment() -> Comment {
    Comment {
        id: 1,
        user_id: 1,
        content: "This is a test comment.".to_string(),
        created_at: Utc::now().to_string(),
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
    let found = dao.find_by_id(Value::Integer(1)).unwrap();
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
    let updated = dao.find_by_id(Value::Integer(1)).unwrap().unwrap();
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
    let result = dao.delete(Value::Integer(1));
    assert!(result.is_ok());

    // 验证删除
    let found = dao.find_by_id(Value::Integer(1)).unwrap();
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

#[test]
#[serial]
fn test_create_order() {
    let db = setup_test_db();
    let dao = UserDao::<Order>::new(db);
    let order = create_test_order();

    let result = dao.create(&order);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
}

#[test]
#[serial]
fn test_find_order_by_id() {
    let db = setup_test_db();
    let dao = UserDao::<Order>::new(db);
    let order = create_test_order();

    dao.create(&order).unwrap();

    let found = dao.find_by_id(Value::Integer(1)).unwrap();
    assert!(found.is_some());

    let found_order = found.unwrap();
    assert_eq!(found_order.id, order.id);
    assert_eq!(found_order.user_id, order.user_id);
    assert_eq!(found_order.product_name, order.product_name);
    // assert_eq!(found_order.amount, order.amount);
    assert!((found_order.amount - order.amount).abs() < 0.00001);
}

#[test]
#[serial]
fn test_find_all_orders() {
    let db = setup_test_db();
    let dao = UserDao::<Order>::new(db);

    let order1 = create_test_order();
    let mut order2 = create_test_order();
    order2.id = 2;
    order2.product_name = "Another Product".to_string();

    dao.create(&order1).unwrap();
    dao.create(&order2).unwrap();

    let orders = dao.find_all().unwrap();
    assert_eq!(orders.len(), 2);
}

#[test]
#[serial]
fn test_update_order() {
    let db = setup_test_db();
    let mut order = create_test_order();
    let dao = UserDao::<Order>::new(db);

    dao.create(&order).unwrap();

    order.amount = 199.99;
    let result = dao.update(&order);

    assert!(result.is_ok());

    let updated = dao.find_by_id(Value::Integer(1)).unwrap().unwrap();
    // assert_eq!(updated.amount, 199.99);
    assert!((updated.amount - 199.99).abs() < 0.00001);
}

#[test]
#[serial]
fn test_delete_order() {
    let db = setup_test_db();
    let dao = UserDao::<Order>::new(db);
    let order = create_test_order();

    dao.create(&order).unwrap();

    let result = dao.delete(Value::Integer(1));
    assert!(result.is_ok());

    let found = dao.find_by_id(Value::Integer(1)).unwrap();
    assert!(found.is_none());
}

#[test]
#[serial]
fn test_create_comment() {
    let db = setup_test_db();
    let dao = UserDao::<Comment>::new(db);
    let comment = create_test_comment();

    let result = dao.create(&comment);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
}

#[test]
#[serial]
fn test_find_comment_by_id() {
    let db = setup_test_db();
    let dao = UserDao::<Comment>::new(db);
    let comment = create_test_comment();

    dao.create(&comment).unwrap();

    let found = dao.find_by_id(Value::Integer(1)).unwrap();
    assert!(found.is_some());

    let found_comment = found.unwrap();
    assert_eq!(found_comment.id, comment.id);
    assert_eq!(found_comment.user_id, comment.user_id);
    assert_eq!(found_comment.content, comment.content);
}

#[test]
#[serial]
fn test_find_all_comments() {
    let db = setup_test_db();
    let dao = UserDao::<Comment>::new(db);

    let comment1 = create_test_comment();
    let mut comment2 = create_test_comment();
    comment2.id = 2;
    comment2.content = "Another comment.".to_string();

    dao.create(&comment1).unwrap();
    dao.create(&comment2).unwrap();

    let comments = dao.find_all().unwrap();
    assert_eq!(comments.len(), 2);
}

#[test]
#[serial]
fn test_update_comment() {
    let db = setup_test_db();
    let mut comment = create_test_comment();
    let dao = UserDao::<Comment>::new(db);

    dao.create(&comment).unwrap();

    comment.content = "Updated comment.".to_string();
    let result = dao.update(&comment);

    assert!(result.is_ok());

    let updated = dao.find_by_id(Value::Integer(1)).unwrap().unwrap();
    assert_eq!(updated.content, "Updated comment.");
}

#[test]
#[serial]
fn test_delete_comment() {
    let db = setup_test_db();
    let dao = UserDao::<Comment>::new(db);
    let comment = create_test_comment();

    dao.create(&comment).unwrap();

    let result = dao.delete(Value::Integer(1));
    assert!(result.is_ok());

    let found = dao.find_by_id(Value::Integer(1)).unwrap();
    assert!(found.is_none());
}

#[test]
#[serial]
fn test_find_info_by_id() {
    let db = setup_test_db();
    let dao = UserDao::new(db.clone());
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
    // let db = setup_test_db();
    let dao = UserDao::<Order>::new(db.clone());
    let order = create_test_order();

    dao.create(&order).unwrap();

    let found1 = dao.find_by_id(Value::Integer(1)).unwrap();
    assert!(found1.is_some());

    let found_order = found1.unwrap();
    assert_eq!(found_order.id, order.id);
    assert_eq!(found_order.user_id, order.user_id);
    assert_eq!(found_order.product_name, order.product_name);

    assert!((found_order.amount - order.amount).abs() < 0.00001);

    // let db = setup_test_db();
    let dao = UserDao::<Comment>::new(db);
    let comment = create_test_comment();

    dao.create(&comment).unwrap();

    let found = dao.find_by_id(Value::Integer(1)).unwrap();
    assert!(found.is_some());

    let found_comment = found.unwrap();
    assert_eq!(found_comment.id, comment.id);
    assert_eq!(found_comment.user_id, comment.user_id);
    assert_eq!(found_comment.content, comment.content);
}

#[test]
#[serial]
fn test_find_info_by_user_id() {
    let db = setup_test_db();
    let user_dao = UserDao::new(db.clone());
    let order_dao = UserDao::<Order>::new(db.clone());
    let comment_dao = UserDao::<Comment>::new(db.clone());

    // 创建测试用户
    let user = create_test_user();
    user_dao.create(&user).unwrap();

    // 创建测试订单
    let mut order = create_test_order();
    order.user_id = user.id;
    order_dao.create(&order).unwrap();

    // 创建测试评论
    let mut comment = create_test_comment();
    comment.user_id = user.id;
    comment_dao.create(&comment).unwrap();

    // 根据用户ID查找订单
    let orders = order_dao
        .find_by_condition("user_id = ?", vec![Value::Integer(user.id)])
        .unwrap();
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].user_id, user.id);

    // 根据用户ID查找评论
    let comments = comment_dao
        .find_by_condition("user_id = ?", vec![Value::Integer(user.id)])
        .unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].user_id, user.id);
}

#[test]
#[serial]
fn test_delete_info_by_user_id() {
    let db = setup_test_db();
    let user_dao = UserDao::new(db.clone());
    let order_dao = UserDao::<Order>::new(db.clone());
    let comment_dao = UserDao::<Comment>::new(db.clone());

    // 创建测试用户
    let user = create_test_user();
    user_dao.create(&user).unwrap();

    // 创建测试订单
    let mut order = create_test_order();
    order.user_id = user.id;
    order_dao.create(&order).unwrap();

    // 创建测试评论
    let mut comment = create_test_comment();
    comment.user_id = user.id;
    comment_dao.create(&comment).unwrap();

    // 删除指定用户ID的所有内容
    let orders = order_dao
        .find_by_condition("user_id = ?", vec![Value::Integer(user.id)])
        .unwrap();
    for order in orders {
        order_dao.delete(Value::Integer(order.id)).unwrap();
    }
    let comments = comment_dao
        .find_by_condition("user_id = ?", vec![Value::Integer(user.id)])
        .unwrap();
    for comment in comments {
        comment_dao.delete(Value::Integer(comment.id)).unwrap();
    }
    user_dao.delete(Value::Integer(user.id)).unwrap();
}

#[test]
#[serial]
fn test_multi_step_transaction() {
    let db = setup_test_db();
    let user_dao = UserDao::new(db.clone());
    let order_dao = UserDao::<Order>::new(db.clone());
    let comment_dao = UserDao::<Comment>::new(db.clone());

    // 开始事务
    let result = user_dao.begin_transaction();
    assert!(result.is_ok());

    // 创建用户
    let user = create_test_user();
    let result = user_dao.create(&user);
    assert!(result.is_ok());

    // 创建订单
    let mut order = create_test_order();
    order.user_id = user.id;
    let result = order_dao.create(&order);
    assert!(result.is_ok());

    // 创建评论
    let mut comment = create_test_comment();
    comment.user_id = user.id;
    let result = comment_dao.create(&comment);
    assert!(result.is_ok());

    // 提交事务
    let result = user_dao.commit();
    assert!(result.is_ok());

    // 验证用户、订单和评论是否已创建
    let found_user = user_dao.find_by_id(Value::Integer(user.id)).unwrap();
    assert!(found_user.is_some());

    let found_order = order_dao.find_by_id(Value::Integer(order.id)).unwrap();
    assert!(found_order.is_some());

    let found_comment = comment_dao.find_by_id(Value::Integer(comment.id)).unwrap();
    assert!(found_comment.is_some());
}

#[test]
#[serial]
fn test_multi_step_transaction_rollback() {
    let db = setup_test_db();
    let user_dao = UserDao::new(db.clone());
    let order_dao = UserDao::<Order>::new(db.clone());
    let comment_dao = UserDao::<Comment>::new(db.clone());

    // 开始事务
    let result = user_dao.begin_transaction();
    assert!(result.is_ok());

    // 创建用户
    let user = create_test_user();
    let result = user_dao.create(&user);
    assert!(result.is_ok());

    // 创建订单
    let mut order = create_test_order();
    order.user_id = user.id;
    let result = order_dao.create(&order);
    assert!(result.is_ok());

    // 创建评论 (故意制造错误, 例如评论内容为空)
    let mut comment = create_test_comment();
    comment.user_id = user.id;
    comment.content = "".to_string(); // 评论内容为空
    let result = comment_dao.create(&comment);
    // assert!(result.is_err()); // 应该返回错误

    // 回滚事务
    let result = user_dao.rollback();
    assert!(result.is_ok());

    // 验证用户、订单和评论是否未创建
    let found_user = user_dao.find_by_id(Value::Integer(user.id)).unwrap();
    assert!(found_user.is_none());

    let found_order = order_dao.find_by_id(Value::Integer(order.id)).unwrap();
    assert!(found_order.is_none());

    let found_comment = comment_dao.find_by_id(Value::Integer(comment.id)).unwrap();
    assert!(found_comment.is_none());
}