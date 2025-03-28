use bootrust::dao::Dao;
use bootrust::database::{
    mysql::MySqlDatabase, DatabaseConfig, DbError, RelationalDatabase, Row, Value,
};
use chrono::{DateTime, Utc};
use serial_test::serial;
use std::marker::PhantomData;

// 商品实体
#[derive(Debug, Clone, PartialEq)]
struct Product {
    id: i64,
    name: String,
    description: String,
    price: f64,
    stock: i64,
    created_at: DateTime<Utc>,
}

// 购物车实体
#[derive(Debug, Clone, PartialEq)]
struct CartItem {
    id: i64,
    user_id: i64,
    product_id: i64,
    quantity: i64,
    added_at: DateTime<Utc>,
}

// 支付信息实体
#[derive(Debug, Clone, PartialEq)]
struct Payment {
    id: i64,
    order_id: i64,
    amount: f64,
    payment_method: String,
    transaction_id: String,
    paid_at: DateTime<Utc>,
}

// ECommerceDo实现
struct ECommerceDo<T: Sized> {
    database: MySqlDatabase,
    _table: PhantomData<T>,
}

impl Dao<Product> for ECommerceDo<Product> {
    type Database = MySqlDatabase;

    fn new(database: Self::Database) -> Self {
        ECommerceDo {
            database,
            _table: PhantomData,
        }
    }

    fn database(&self) -> &Self::Database {
        &self.database
    }

    fn row_to_entity(row: Row) -> Result<Product, DbError> {
        if row.values.len() != 6 {
            return Err(DbError::ConversionError(
                "Invalid number of columns".to_string(),
            ));
        }

        Ok(Product {
            id: match &row.values[0] {
                Value::Bigint(i) => *i,
                _ => return Err(DbError::ConversionError("Invalid id type".to_string())),
            },
            name: match &row.values[1] {
                Value::Text(s) => s.clone(),
                _ => return Err(DbError::ConversionError("Invalid name type".to_string())),
            },
            description: match &row.values[2] {
                Value::Text(s) => s.clone(),
                _ => {
                    return Err(DbError::ConversionError(
                        "Invalid description type".to_string(),
                    ))
                }
            },
            price: match &row.values[3] {
                Value::Double(f) => *f,
                _ => return Err(DbError::ConversionError("Invalid price type".to_string())),
            },
            stock: match &row.values[4] {
                Value::Bigint(i) => *i,
                _ => return Err(DbError::ConversionError("Invalid stock type".to_string())),
            },
            created_at: match &row.values[5] {
                Value::DateTime(dt) => *dt,
                _ => {
                    return Err(DbError::ConversionError(
                        "Invalid created_at type".to_string(),
                    ))
                }
            },
        })
    }

    fn entity_to_map(entity: &Product) -> Vec<(String, Value)> {
        let mut map = Vec::new();
        map.push(("id".to_string(), Value::Bigint(entity.id)));
        map.push(("name".to_string(), Value::Text(entity.name.clone())));
        map.push((
            "description".to_string(),
            Value::Text(entity.description.clone()),
        ));
        map.push(("price".to_string(), Value::Double(entity.price)));
        map.push(("stock".to_string(), Value::Bigint(entity.stock)));
        map.push(("created_at".to_string(), Value::DateTime(entity.created_at)));
        map
    }

    fn table_name() -> String {
        "products".to_string()
    }

    fn primary_key_column() -> String {
        "id".to_string()
    }
}

impl Dao<CartItem> for ECommerceDo<CartItem> {
    type Database = MySqlDatabase;

    fn new(database: Self::Database) -> Self {
        ECommerceDo {
            database,
            _table: PhantomData,
        }
    }

    fn database(&self) -> &Self::Database {
        &self.database
    }

    fn row_to_entity(row: Row) -> Result<CartItem, DbError> {
        if row.values.len() != 5 {
            return Err(DbError::ConversionError(
                "Invalid number of columns".to_string(),
            ));
        }

        Ok(CartItem {
            id: match &row.values[0] {
                Value::Bigint(i) => *i,
                _ => return Err(DbError::ConversionError("Invalid id type".to_string())),
            },
            user_id: match &row.values[1] {
                Value::Bigint(i) => *i,
                _ => return Err(DbError::ConversionError("Invalid user_id type".to_string())),
            },
            product_id: match &row.values[2] {
                Value::Bigint(i) => *i,
                _ => {
                    return Err(DbError::ConversionError(
                        "Invalid product_id type".to_string(),
                    ))
                }
            },
            quantity: match &row.values[3] {
                Value::Bigint(i) => *i,
                _ => {
                    return Err(DbError::ConversionError(
                        "Invalid quantity type".to_string(),
                    ))
                }
            },
            added_at: match &row.values[4] {
                Value::DateTime(dt) => *dt,
                _ => {
                    return Err(DbError::ConversionError(
                        "Invalid added_at type".to_string(),
                    ))
                }
            },
        })
    }

    fn entity_to_map(entity: &CartItem) -> Vec<(String, Value)> {
        let mut map = Vec::new();
        map.push(("id".to_string(), Value::Bigint(entity.id)));
        map.push(("user_id".to_string(), Value::Bigint(entity.user_id)));
        map.push(("product_id".to_string(), Value::Bigint(entity.product_id)));
        map.push(("quantity".to_string(), Value::Bigint(entity.quantity)));
        map.push(("added_at".to_string(), Value::DateTime(entity.added_at)));
        map
    }

    fn table_name() -> String {
        "cart_items".to_string()
    }

    fn primary_key_column() -> String {
        "id".to_string()
    }
}

impl Dao<Payment> for ECommerceDo<Payment> {
    type Database = MySqlDatabase;

    fn new(database: Self::Database) -> Self {
        ECommerceDo {
            database,
            _table: PhantomData,
        }
    }

    fn database(&self) -> &Self::Database {
        &self.database
    }

    fn row_to_entity(row: Row) -> Result<Payment, DbError> {
        if row.values.len() != 6 {
            return Err(DbError::ConversionError(
                "Invalid number of columns".to_string(),
            ));
        }

        Ok(Payment {
            id: match &row.values[0] {
                Value::Bigint(i) => *i,
                _ => return Err(DbError::ConversionError("Invalid id type".to_string())),
            },
            order_id: match &row.values[1] {
                Value::Bigint(i) => *i,
                _ => {
                    return Err(DbError::ConversionError(
                        "Invalid order_id type".to_string(),
                    ))
                }
            },
            amount: match &row.values[2] {
                Value::Double(f) => *f,
                _ => return Err(DbError::ConversionError("Invalid amount type".to_string())),
            },
            payment_method: match &row.values[3] {
                Value::Text(s) => s.clone(),
                _ => {
                    return Err(DbError::ConversionError(
                        "Invalid payment_method type".to_string(),
                    ))
                }
            },
            transaction_id: match &row.values[4] {
                Value::Text(s) => s.clone(),
                _ => {
                    return Err(DbError::ConversionError(
                        "Invalid transaction_id type".to_string(),
                    ))
                }
            },
            paid_at: match &row.values[5] {
                Value::DateTime(dt) => *dt,
                _ => return Err(DbError::ConversionError("Invalid paid_at type".to_string())),
            },
        })
    }

    fn entity_to_map(entity: &Payment) -> Vec<(String, Value)> {
        let mut map = Vec::new();
        map.push(("id".to_string(), Value::Bigint(entity.id)));
        map.push(("order_id".to_string(), Value::Bigint(entity.order_id)));
        map.push(("amount".to_string(), Value::Double(entity.amount)));
        map.push((
            "payment_method".to_string(),
            Value::Text(entity.payment_method.clone()),
        ));
        map.push((
            "transaction_id".to_string(),
            Value::Text(entity.transaction_id.clone()),
        ));
        map.push(("paid_at".to_string(), Value::DateTime(entity.paid_at)));
        map
    }

    fn table_name() -> String {
        "payments".to_string()
    }

    fn primary_key_column() -> String {
        "id".to_string()
    }
}

// 设置测试数据库
fn setup_ecommerce_test_db() -> MySqlDatabase {
    let config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 3306,
        username: "root".to_string(),
        password: "root".to_string(),
        database_name: "ecommerce_test".to_string(),
        max_size: 10,
    };
    let db = MySqlDatabase::connect(config).unwrap();

    // 创建商品表
    db.execute("DROP TABLE IF EXISTS products", vec![]).unwrap();
    db.execute(
        "CREATE TABLE products (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            price DOUBLE NOT NULL,
            stock INTEGER NOT NULL,
            created_at DATETIME NOT NULL
        )",
        vec![],
    )
    .unwrap();

    // 创建购物车表
    db.execute("DROP TABLE IF EXISTS cart_items", vec![])
        .unwrap();
    db.execute(
        "CREATE TABLE cart_items (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            product_id INTEGER NOT NULL,
            quantity INTEGER NOT NULL,
            added_at DATETIME NOT NULL
        )",
        vec![],
    )
    .unwrap();

    // 创建支付信息表
    db.execute("DROP TABLE IF EXISTS payments", vec![]).unwrap();
    db.execute(
        "CREATE TABLE payments (
            id INTEGER PRIMARY KEY,
            order_id INTEGER NOT NULL,
            amount DOUBLE NOT NULL,
            payment_method TEXT NOT NULL,
            transaction_id TEXT NOT NULL,
            paid_at DATETIME NOT NULL
        )",
        vec![],
    )
    .unwrap();

    db
}

// 创建测试商品
fn create_test_product() -> Product {
    Product {
        id: 1,
        name: "Test Product".to_string(),
        description: "This is a test product.".to_string(),
        price: 99.99,
        stock: 100,
        created_at: Utc::now(),
    }
}

// 创建测试购物车项
fn create_test_cart_item() -> CartItem {
    CartItem {
        id: 1,
        user_id: 1,
        product_id: 1,
        quantity: 2,
        added_at: Utc::now(),
    }
}

// 创建测试支付信息
fn create_test_payment() -> Payment {
    Payment {
        id: 1,
        order_id: 1,
        amount: 199.98,
        payment_method: "Credit Card".to_string(),
        transaction_id: "tx12345".to_string(),
        paid_at: Utc::now(),
    }
}

// 测试添加商品到购物车
#[test]
#[serial]
fn test_add_product_to_cart() {
    let db = setup_ecommerce_test_db();
    let product_dao = ECommerceDo::new(db.clone());
    let cart_dao = ECommerceDo::new(db.clone());

    // 创建测试商品
    let product = create_test_product();
    product_dao.create(&product).unwrap();

    // 添加商品到购物车
    let mut cart_item = create_test_cart_item();
    cart_item.product_id = product.id;
    let result = cart_dao.create(&cart_item);
    assert!(result.is_ok());

    // 验证购物车项是否添加成功
    let added_item = cart_dao.find_by_id(Value::Bigint(cart_item.id)).unwrap();
    assert!(added_item.is_some());
    assert_eq!(added_item.unwrap().product_id, product.id);
}

// 测试从购物车移除商品
#[test]
#[serial]
fn test_remove_product_from_cart() {
    let db = setup_ecommerce_test_db();
    let cart_dao = ECommerceDo::new(db.clone());

    // 添加商品到购物车
    let cart_item = create_test_cart_item();
    cart_dao.create(&cart_item).unwrap();

    // 从购物车移除商品
    let result = cart_dao.delete(Value::Bigint(cart_item.id));
    assert!(result.is_ok());

    // 验证购物车项是否已移除
    let removed_item = cart_dao.find_by_id(Value::Bigint(cart_item.id)).unwrap();
    assert!(removed_item.is_none());
}

// 测试更新购物车商品数量
#[test]
#[serial]
fn test_update_cart_item_quantity() {
    let db = setup_ecommerce_test_db();
    let cart_dao = ECommerceDo::new(db.clone());

    // 添加商品到购物车
    let mut cart_item = create_test_cart_item();
    cart_dao.create(&cart_item).unwrap();

    // 更新购物车商品数量
    cart_item.quantity = 3;
    let result = cart_dao.update(&cart_item);
    assert!(result.is_ok());

    // 验证购物车商品数量是否更新
    let updated_item = cart_dao.find_by_id(Value::Bigint(cart_item.id)).unwrap();
    assert_eq!(updated_item.unwrap().quantity, 3);
}

// 测试支付流程
#[test]
#[serial]
fn test_payment_process() {
    let db = setup_ecommerce_test_db();
    let payment_dao = ECommerceDo::new(db.clone());

    // 创建测试订单
    let order_id = 1;

    // 进行支付
    let mut payment = create_test_payment();
    payment.order_id = order_id;
    let result = payment_dao.create(&payment);
    assert!(result.is_ok());

    // 验证支付信息是否保存成功
    let saved_payment = payment_dao.find_by_id(Value::Bigint(payment.id)).unwrap();
    assert!(saved_payment.is_some());
    assert_eq!(saved_payment.unwrap().order_id, order_id);
}

// 测试库存更新
#[test]
#[serial]
fn test_stock_update() {
    let db = setup_ecommerce_test_db();
    let product_dao = ECommerceDo::new(db.clone());

    // 创建测试商品
    let mut product = create_test_product();
    product_dao.create(&product).unwrap();

    // 更新商品库存
    product.stock = 50;
    let result = product_dao.update(&product);
    assert!(result.is_ok());

    // 验证商品库存是否更新
    let updated_product = product_dao.find_by_id(Value::Bigint(product.id)).unwrap();
    assert_eq!(updated_product.unwrap().stock, 50);
}

// 测试事务处理
#[test]
#[serial]
fn test_transaction() {
    let db = setup_ecommerce_test_db();
    let product_dao = ECommerceDo::new(db.clone());
    let cart_dao = ECommerceDo::new(db.clone());
    let payment_dao = ECommerceDo::new(db.clone());

    // 开始事务
    let result = product_dao.begin_transaction();
    assert!(result.is_ok());

    // 创建商品
    let product = create_test_product();
    let result = product_dao.create(&product);
    assert!(result.is_ok());

    // 添加商品到购物车
    let mut cart_item = create_test_cart_item();
    cart_item.product_id = product.id;
    let result = cart_dao.create(&cart_item);
    assert!(result.is_ok());

    // 进行支付
    let payment = create_test_payment();
    let result = payment_dao.create(&payment);
    assert!(result.is_ok());

    // 提交事务
    let result = product_dao.commit();
    assert!(result.is_ok());

    // 验证商品、购物车项和支付信息是否已创建
    let found_product = product_dao.find_by_id(Value::Bigint(product.id)).unwrap();
    assert!(found_product.is_some());

    let found_cart_item = cart_dao.find_by_id(Value::Bigint(cart_item.id)).unwrap();
    assert!(found_cart_item.is_some());

    let found_payment = payment_dao.find_by_id(Value::Bigint(payment.id)).unwrap();
    assert!(found_payment.is_some());
}

// 测试事务回滚
#[test]
#[serial]
fn test_transaction_rollback() {
    let db = setup_ecommerce_test_db();
    let product_dao = ECommerceDo::new(db.clone());
    let cart_dao = ECommerceDo::new(db.clone());

    // 开始事务
    let result = product_dao.begin_transaction();
    assert!(result.is_ok());

    // 创建商品
    let product = create_test_product();
    let result = product_dao.create(&product);
    assert!(result.is_ok());

    // 添加商品到购物车 (故意制造错误, 例如商品ID不存在)
    let mut cart_item = create_test_cart_item();
    cart_item.product_id = 999; // 不存在的商品ID
    let _result = cart_dao.create(&cart_item);
    // assert!(result.is_err()); // 应该返回错误

    // 回滚事务
    let result = product_dao.rollback();
    assert!(result.is_ok());

    // 验证商品和购物车项是否未创建
    let found_product = product_dao.find_by_id(Value::Bigint(product.id)).unwrap();
    assert!(found_product.is_none());

    let found_cart_item = cart_dao.find_by_id(Value::Bigint(cart_item.id)).unwrap();
    assert!(found_cart_item.is_none());
}
