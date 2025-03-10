use std::sync::Arc;
use bootrust::asyncdao::Dao;
use bootrust::asyncdatabase::{
    sqlite::SqliteDatabase, DatabaseConfig, DbError, RelationalDatabase, Row, Value,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serial_test::serial;
use std::marker::PhantomData;

// 商品实体
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Product {
    id: i64,
    name: String,
    description: String,
    price: f64,
    stock: i64,
    #[serde(with = "chrono::serde::ts_seconds")]
    created_at: DateTime<Utc>,
}

// 购物车实体
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CartItem {
    id: i64,
    user_id: i64,
    product_id: i64,
    quantity: i64,
    #[serde(with = "chrono::serde::ts_seconds")]
    added_at: DateTime<Utc>,
}

// 支付信息实体
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Payment {
    id: i64,
    order_id: i64,
    amount: f64,
    payment_method: String,
    transaction_id: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    paid_at: DateTime<Utc>,
}

// ECommerceDo实现
struct ECommerceDo<T: Sized, D: RelationalDatabase> {
    database: D,
    _table: PhantomData<T>,
}

impl<D: RelationalDatabase> Dao<Product> for ECommerceDo<Product, D> {
    type Database = D;

    fn new(database: Self::Database) -> Self {
        ECommerceDo {
            database,
            _table: PhantomData,
        }
    }

    fn database(&self) -> &Self::Database {
        &self.database
    }

    fn table_name() -> String {
        "products".to_string()
    }

    fn primary_key_column() -> String {
        "id".to_string()
    }
}

impl<D: RelationalDatabase> Dao<CartItem> for ECommerceDo<CartItem, D> {
    type Database = D;

    fn new(database: Self::Database) -> Self {
        ECommerceDo {
            database,
            _table: PhantomData,
        }
    }

    fn database(&self) -> &Self::Database {
        &self.database
    }

    fn table_name() -> String {
        "cart_items".to_string()
    }

    fn primary_key_column() -> String {
        "id".to_string()
    }
}

impl<D: RelationalDatabase> Dao<Payment> for ECommerceDo<Payment, D> {
    type Database = D;

    fn new(database: Self::Database) -> Self {
        ECommerceDo {
            database,
            _table: PhantomData,
        }
    }

    fn database(&self) -> &Self::Database {
        &self.database
    }

    fn table_name() -> String {
        "payments".to_string()
    }

    fn primary_key_column() -> String {
        "id".to_string()
    }
}

// 设置测试数据库
async fn setup_ecommerce_test_db() -> SqliteDatabase {
    let config = DatabaseConfig {
        database_name: ":memory:".to_string(),
        ..Default::default()
    };    let db = SqliteDatabase::connect(config).await.unwrap();

    // 创建商品表
    db.execute("DROP TABLE IF EXISTS products", vec![])
        .await
        .unwrap();
    db.execute(
        "CREATE TABLE products (
            id INTEGER PRIMARY KEY AUTOINCREMENT ,
            name TEXT NOT NULL,
            description TEXT,
            price FLOAT8 NOT NULL,
            stock INT8 NOT NULL,
            created_at TIMESTAMPTZ
        )",
        vec![],
    )
    .await
    .unwrap();

    // 创建购物车表
    db.execute("DROP TABLE IF EXISTS cart_items", vec![])
        .await
        .unwrap();
    db.execute(
        "CREATE TABLE cart_items (
            id BIGSERIAL   PRIMARY KEY,
            user_id INT8 NOT NULL,
            product_id INT8 NOT NULL,
            quantity INT8 NOT NULL,
            added_at TIMESTAMPTZ NOT NULL
        )",
        vec![],
    )
    .await
    .unwrap();

    // 创建支付信息表
    db.execute("DROP TABLE IF EXISTS payments", vec![])
        .await
        .unwrap();
    db.execute(
        "CREATE TABLE payments (
            id BIGSERIAL  PRIMARY KEY,
            order_id INT8 NOT NULL,
            amount FLOAT8 NOT NULL,
            payment_method TEXT NOT NULL,
            transaction_id TEXT NOT NULL,
            paid_at TIMESTAMP WITH TIME ZONE   NOT NULL
        )",
        vec![],
    )
    .await
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
#[tokio::test]
async fn test_add_product_to_cart() {
    let db = setup_ecommerce_test_db().await;
    let product_dao = ECommerceDo::new(db.clone());
    let cart_dao = ECommerceDo::new(db.clone());

    // 创建测试商品
    let product = create_test_product();
    product_dao.create(&product).await.unwrap();

    // 添加商品到购物车
    let mut cart_item = create_test_cart_item();
    cart_item.product_id = product.id;
    let result = cart_dao.create(&cart_item).await;
    assert!(result.is_ok());

    // 验证购物车项是否添加成功
    let added_item = cart_dao
        .find_by_id(Value::Bigint(cart_item.id))
        .await
        .unwrap();
    assert!(added_item.is_some());
    assert_eq!(added_item.unwrap().product_id, product.id);
}

// 测试从购物车移除商品
#[tokio::test]
async fn test_remove_product_from_cart() {
    let db = setup_ecommerce_test_db().await;
    let cart_dao = ECommerceDo::new(db.clone());

    // 添加商品到购物车
    let cart_item = create_test_cart_item();
    cart_dao.create(&cart_item).await.unwrap();

    // 从购物车移除商品
    let result = cart_dao.delete(Value::Bigint(cart_item.id)).await;
    dbg!(&result);
    assert!(result.is_ok());

    // 验证购物车项是否已移除
    let removed_item = cart_dao
        .find_by_id(Value::Bigint(cart_item.id))
        .await
        .unwrap();
    assert!(removed_item.is_none());
}

// 测试更新购物车商品数量
#[tokio::test]
async fn test_update_cart_item_quantity() {
    let db = setup_ecommerce_test_db().await;
    let cart_dao = ECommerceDo::new(db.clone());

    // 添加商品到购物车
    let mut cart_item = create_test_cart_item();
    cart_dao.create(&cart_item).await.unwrap();

    // 更新购物车商品数量
    cart_item.quantity = 3;
    let result = cart_dao.update(&cart_item).await;
    assert!(result.is_ok());

    // 验证购物车商品数量是否更新
    let updated_item = cart_dao
        .find_by_id(Value::Bigint(cart_item.id))
        .await
        .unwrap();
    assert_eq!(updated_item.unwrap().quantity, 3);
}

// 测试支付流程
#[tokio::test]
async fn test_payment_process() {
    let db = setup_ecommerce_test_db().await;
    let payment_dao = ECommerceDo::new(db.clone());

    // 创建测试订单
    let order_id = 1;

    // 进行支付
    let mut payment = create_test_payment();
    payment.order_id = order_id;
    let result = payment_dao.create(&payment).await;
    assert!(result.is_ok());

    // 验证支付信息是否保存成功
    let saved_payment = payment_dao
        .find_by_id(Value::Bigint(payment.id))
        .await
        .unwrap();
    assert!(saved_payment.is_some());
    assert_eq!(saved_payment.unwrap().order_id, order_id);
}

// 测试库存更新
#[tokio::test]
async fn test_stock_update() {
    let db = setup_ecommerce_test_db().await;
    let product_dao = ECommerceDo::new(db.clone());

    // 创建测试商品
    let mut product = create_test_product();
    product_dao.create(&product).await.unwrap();

    // 更新商品库存
    product.stock = 50;
    let result = product_dao.update(&product).await;
    assert!(result.is_ok());

    // 验证商品库存是否更新
    let updated_product = product_dao
        .find_by_id(Value::Bigint(product.id))
        .await
        .unwrap();
    assert_eq!(updated_product.unwrap().stock, 50);
}

// 测试事务处理
#[tokio::test]
async fn test_transaction() {
    let db = setup_ecommerce_test_db().await;
    let product_dao = ECommerceDo::new(db.clone());
    let cart_dao = ECommerceDo::new(db.clone());
    let payment_dao = ECommerceDo::new(db.clone());

    // 开始事务
    let result = product_dao.begin_transaction().await;
    assert!(result.is_ok());

    // 创建商品
    let product = create_test_product();
    let result = product_dao.create(&product).await;
    assert!(result.is_ok());

    // 添加商品到购物车
    let mut cart_item = create_test_cart_item();
    cart_item.product_id = product.id;
    let result = cart_dao.create(&cart_item).await;
    assert!(result.is_ok());

    // 进行支付
    let payment = create_test_payment();
    let result = payment_dao.create(&payment).await;
    assert!(result.is_ok());

    // 提交事务
    let result = product_dao.commit().await;
    assert!(result.is_ok());

    // 验证商品、购物车项和支付信息是否已创建
    let found_product = product_dao
        .find_by_id(Value::Bigint(product.id))
        .await
        .unwrap();
    assert!(found_product.is_some());

    let found_cart_item = cart_dao
        .find_by_id(Value::Bigint(cart_item.id))
        .await
        .unwrap();
    assert!(found_cart_item.is_some());

    let found_payment = payment_dao
        .find_by_id(Value::Bigint(payment.id))
        .await
        .unwrap();
    assert!(found_payment.is_some());
}

// 测试事务回滚
#[tokio::test]
async fn test_transaction_rollback() {
    let db = setup_ecommerce_test_db().await;
    let arc_db = Arc::new(db);
    let product_dao = ECommerceDo::new(Arc::clone(&arc_db));
let cart_dao = ECommerceDo::new(Arc::clone(&arc_db));
    // let product_dao = ECommerceDo::new(db.clone());
    // let cart_dao = ECommerceDo::new(db.clone());

    // 开始事务
    let result = product_dao.begin_transaction().await;
    assert!(result.is_ok());

    // 创建商品
    let product = create_test_product();
    let result = product_dao.create(&product).await;
    assert!(result.is_ok());

    // 添加商品到购物车 (故意制造错误, 例如商品ID不存在)
    let mut cart_item = create_test_cart_item();
    cart_item.product_id = 999; // 不存在的商品ID
    let result = cart_dao.create(&cart_item);
    // assert!(result.is_err()); // 应该返回错误

    // 回滚事务
    let result = product_dao.rollback().await;
    assert!(result.is_ok());

    // 验证商品和购物车项是否未创建
    let found_product = product_dao
        .find_by_id(Value::Bigint(product.id))
        .await
        .unwrap();
    assert!(found_product.is_none());

    let found_cart_item = cart_dao
        .find_by_id(Value::Bigint(cart_item.id))
        .await
        .unwrap();
    assert!(found_cart_item.is_none());
}

#[tokio::test]
async fn test_arc_db() {
    let db = setup_ecommerce_test_db().await;
    let arc_db = Arc::new(db);
    let product_dao = ECommerceDo::<Product, _>::new(Arc::clone(&arc_db));

let product = create_test_product();
    product_dao.create(&product).await.unwrap();

let added_item = product_dao
        .find_by_id(Value::Bigint(product.id))
        .await
        .unwrap();
    assert!(added_item.is_some());
    assert_eq!(added_item.unwrap().id, product.id);

}

#[tokio::test]
async fn test_complex_query() {
    let db = setup_ecommerce_test_db().await;
    let payment_dao = ECommerceDo::new(db.clone());

    // 创建测试订单
    let order_id = 1;

    // 进行支付
    let mut payment = create_test_payment();
    payment.order_id = order_id;
    let result = payment_dao.create(&payment).await;
    assert!(result.is_ok());

    // 验证支付信息是否保存成功
    let saved_payment = payment_dao
        .find_by_id(Value::Bigint(payment.id))
        .await
        .unwrap();
    assert!(saved_payment.is_some());
    assert_eq!(saved_payment.unwrap().order_id, order_id);
    
    let saved_payment = payment_dao
        .find_by_condition(
        vec!["id =", "order_id =", "amount <"],
        vec![Value::Bigint(payment.id), Value::Bigint(payment.order_id), Value::Bigint(200)]
        ).await
        .unwrap();
        assert_eq!(saved_payment[0].order_id, order_id);
        let mut payment1 = create_test_payment();
        payment1.amount =100.0;
        payment1.id =2;
payment1.order_id =2;        
payment_dao.create(&payment1).await.unwrap();        
let saved_payment = payment_dao
        .find_by_condition(
        vec!["id <", "order_id <", "amount >="],
        vec![Value::Bigint(10), Value::Bigint(10), Value::Bigint(100)]
        ).await
        .unwrap();
        assert_eq!(saved_payment.len(), 2);
        
        let result = payment_dao
        .prepare()
        .find()
        .where_clauses(vec!["id <", "order_id <", "amount >="])
        .order_by(vec!["amount  asc"])
        .group_by(vec!["id"])
        .having(vec!["order_id ="])
        .values(vec![Value::Bigint(10), Value::Bigint(10), Value::Double(100.00), Value::Bigint(2)])
        .execute()
        .await
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].order_id, 2);

}