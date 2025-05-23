use bootrust::asyncdatabase::{mysql::MySqlDatabase, DatabaseConfig, RelationalDatabase, Value};
use bootrust::entity::Entity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serial_test::serial;
use std::sync::Arc;

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
impl Entity for Product {
    fn table() -> String {
        "products".to_string()
    }

    fn primary_key() -> String {
        "id".to_string()
    }
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

impl Entity for CartItem {
    fn table() -> String {
        "cart_items".to_string()
    }

    fn primary_key() -> String {
        "id".to_string()
    }
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

impl Entity for Payment {
    fn table() -> String {
        "payments".to_string()
    }

    fn primary_key() -> String {
        "id".to_string()
    }
}

// 设置测试数据库
async fn setup_test_db() -> MySqlDatabase {
    let config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 3306,
        username: "root".to_string(),
        password: "root".to_string(),
        database_name: "test".to_string(),
        max_size: 10,
    };
    let db = MySqlDatabase::connect(config).await.unwrap();

    // 创建商品表
    db.execute("DROP TABLE IF EXISTS products", vec![])
        .await
        .unwrap();
    db.execute(
        "CREATE TABLE products (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            price DOUBLE NOT NULL,
            stock INTEGER NOT NULL,
            created_at Bigint NOT NULL
            -- created_at TEXT NOT NULL
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
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            product_id INTEGER NOT NULL,
            quantity INTEGER NOT NULL,
            added_at INTEGER NOT NULL
            -- added_at TEXT  NOT NULL
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
            id INTEGER PRIMARY KEY,
            order_id INTEGER NOT NULL,
            amount DOUBLE NOT NULL,
            payment_method TEXT NOT NULL,
            transaction_id TEXT NOT NULL,
            paid_at INTEGER NOT NULL
            -- paid_at TEXT  NOT NULL
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
#[serial]
async fn test_entity_add_product_to_cart() {
    let db = setup_test_db().await;

    // 创建测试商品
    let product = create_test_product();
    Product::create(&db, &product).await.unwrap();

    // 添加商品到购物车
    let mut cart_item = create_test_cart_item();
    cart_item.product_id = product.id;
    let result = CartItem::create(&db, &cart_item).await;
    assert!(result.is_ok());

    // 验证购物车项是否添加成功
    let added_item: Option<CartItem> = CartItem::find_by_id(&db, Value::Bigint(cart_item.id))
        .await
        .unwrap();
    assert!(added_item.is_some());
    let item = added_item.unwrap();
    dbg!(&item);
    assert_eq!(item.product_id, product.id);
}

// 测试从购物车移除商品
#[tokio::test]
#[serial]
async fn test_remove_product_from_cart() {
    let db = setup_test_db().await;
    // 添加商品到购物车
    let cart_item = create_test_cart_item();
    CartItem::create(&db, &cart_item).await.unwrap();

    // 从购物车移除商品
    let result = CartItem::delete(&db, Value::Bigint(cart_item.id)).await;

    assert!(result.is_ok());

    // 验证购物车项是否已移除
    let removed_item: Option<CartItem> = CartItem::find_by_id(&db, Value::Bigint(cart_item.id))
        .await
        .unwrap();
    assert!(removed_item.is_none());
}

// 测试更新购物车商品数量
#[tokio::test]
#[serial]
async fn test_update_cart_item_quantity() {
    let db = setup_test_db().await;
    // 添加商品到购物车
    let mut cart_item = create_test_cart_item();
    CartItem::create(&db, &cart_item).await.unwrap();

    // 更新购物车商品数量
    cart_item.quantity = 3;
    let result = CartItem::update(&db, &cart_item).await;
    assert!(result.is_ok());

    // 验证购物车商品数量是否更新
    let updated_item: Option<CartItem> = CartItem::find_by_id(&db, Value::Bigint(cart_item.id))
        .await
        .unwrap();
    assert_eq!(updated_item.unwrap().quantity, 3);
}

// 测试支付流程
#[tokio::test]
#[serial]
async fn test_payment_process() {
    let db = setup_test_db().await;

    // 创建测试订单
    let order_id = 1;

    // 进行支付
    let mut payment = create_test_payment();
    payment.order_id = order_id;
    let result = Payment::create(&db, &payment).await;
    assert!(result.is_ok());

    // 验证支付信息是否保存成功
    let saved_payment: Option<Payment> = Payment::find_by_id(&db, Value::Bigint(payment.id))
        .await
        .unwrap();
    assert!(saved_payment.is_some());
    assert_eq!(saved_payment.unwrap().order_id, order_id);
}

// 测试库存更新
#[tokio::test]
#[serial]
async fn test_stock_update() {
    let db = setup_test_db().await;

    // 创建测试商品
    let mut product = create_test_product();
    Product::create(&db, &product).await.unwrap();

    // 更新商品库存
    product.stock = 50;
    let result = Product::update(&db, &product).await;
    assert!(result.is_ok());

    // 验证商品库存是否更新
    let updated_product: Option<Product> = Product::find_by_id(&db, Value::Bigint(product.id))
        .await
        .unwrap();
    assert_eq!(updated_product.unwrap().stock, 50);
}

// 测试事务处理
#[tokio::test]
#[serial]
async fn test_transaction() {
    let db = setup_test_db().await;
    // 开始事务
    let result = Product::begin_transaction(&db).await;
    assert!(result.is_ok());

    // 创建商品
    let product = create_test_product();
    let result = Product::create(&db, &product).await;
    assert!(result.is_ok());

    // 添加商品到购物车
    let mut cart_item = create_test_cart_item();
    cart_item.product_id = product.id;
    let result = CartItem::create(&db, &cart_item).await;
    assert!(result.is_ok());

    // 进行支付
    let payment = create_test_payment();
    let result = Payment::create(&db, &payment).await;
    assert!(result.is_ok());

    // 提交事务
    let result = Product::commit(&db).await;
    assert!(result.is_ok());

    // 验证商品、购物车项和支付信息是否已创建
    let found_product: Option<Product> = Product::find_by_id(&db, Value::Bigint(product.id))
        .await
        .unwrap();
    assert!(found_product.is_some());

    let found_cart_item: Option<CartItem> = CartItem::find_by_id(&db, Value::Bigint(cart_item.id))
        .await
        .unwrap();
    assert!(found_cart_item.is_some());

    let found_payment: Option<Payment> = Payment::find_by_id(&db, Value::Bigint(payment.id))
        .await
        .unwrap();
    assert!(found_payment.is_some());
}

// 测试事务回滚
#[tokio::test]
#[serial]
async fn test_transaction_rollback() {
    let db = setup_test_db().await;
    let _arc_db = Arc::new(&db);

    let result = Product::begin_transaction(&db).await;
    assert!(result.is_ok());

    // 创建商品
    let product = create_test_product();
    let result = Product::create(&db, &product).await;
    assert!(result.is_ok());

    // 添加商品到购物车 (故意制造错误, 例如商品ID不存在)
    let mut cart_item = create_test_cart_item();
    cart_item.product_id = 999; // 不存在的商品ID
    let _result = CartItem::create(&db, &cart_item);
    // assert!(result.is_err()); // 应该返回错误

    // 回滚事务
    let result = Product::rollback(&db).await;
    assert!(result.is_ok());

    // 验证商品和购物车项是否未创建
    let found_product: Option<Product> = Product::find_by_id(&db, Value::Bigint(product.id))
        .await
        .unwrap();
    assert!(found_product.is_none());

    let found_cart_item: Option<CartItem> = CartItem::find_by_id(&db, Value::Bigint(cart_item.id))
        .await
        .unwrap();
    assert!(found_cart_item.is_none());
}

#[tokio::test]
#[serial]
async fn test_arc_db() {
    let db = setup_test_db().await;

    let product = create_test_product();
    Product::create(&db, &product).await.unwrap();

    let added_item: Option<Product> = Product::find_by_id(&db, Value::Bigint(product.id))
        .await
        .unwrap();
    assert!(added_item.is_some());
    assert_eq!(added_item.unwrap().id, product.id);
}

#[tokio::test]
#[serial]
async fn test_multi_condition_query() {
    let db = setup_test_db().await;

    // 创建测试订单
    let order_id = 1;

    // 进行支付
    let mut payment = create_test_payment();
    payment.order_id = order_id;
    let result = Payment::create(&db, &payment).await;
    assert!(result.is_ok());
    let mut payment1 = create_test_payment();
    payment1.amount = 100.0;
    payment1.id = 2;
    Payment::create(&db, &payment1).await.unwrap();

    // 验证支付信息是否保存成功
    let saved_payment: Option<Payment> = Payment::find_by_id(&db, Value::Bigint(payment.id))
        .await
        .unwrap();
    assert!(saved_payment.is_some());
    assert_eq!(saved_payment.unwrap().order_id, order_id);

    let saved_payment: Vec<Payment> = Payment::find_by_conditions(
        &db,
        &["id =", "order_id =", "amount <"],
        vec![
            Value::Bigint(payment.id),
            Value::Bigint(payment.order_id),
            Value::Double(200.00),
        ],
    )
    .await
    .unwrap();
    assert_eq!(saved_payment[0].order_id, order_id);

    let saved_payment: Vec<Payment> = Payment::find_by_conditions(
        &db,
        &["id <", "order_id <", "amount >="],
        vec![Value::Bigint(10), Value::Bigint(10), Value::Double(100.0)],
    )
    .await
    .unwrap();
    assert_eq!(saved_payment.len(), 2);
}

#[tokio::test]
#[serial]
async fn test_complex_query() {
    let db = setup_test_db().await;

    // 创建测试订单
    let order_id = 1;

    // 进行支付
    let mut payment = create_test_payment();
    payment.order_id = order_id;
    let result = Payment::create(&db, &payment).await;
    assert!(result.is_ok());
    let mut payment1 = create_test_payment();
    payment1.amount = 100.0;
    payment1.id = 2;
    payment1.order_id = 2;
    Payment::create(&db, &payment1).await.unwrap();

    // 验证支付信息是否保存成功
    let saved_payment: Option<Payment> = Payment::find_by_id(&db, Value::Bigint(payment.id))
        .await
        .unwrap();
    assert!(saved_payment.is_some());
    assert_eq!(saved_payment.unwrap().order_id, order_id);

    let result: Vec<Payment> = Payment::prepare(&db)
        .find()
        .where_clauses(vec!["id <", "order_id <", "amount >="])
        .order_by(vec!["amount  asc"])
        .group_by(vec!["id"])
        .having(vec!["order_id ="])
        .values(vec![
            Value::Bigint(10),
            Value::Bigint(10),
            Value::Double(100.00),
            Value::Bigint(2),
        ])
        .query()
        .await
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].order_id, 2);
    dbg!(&result);
}

#[tokio::test]
#[serial]
async fn test_complex_delete() {
    let db = setup_test_db().await;

    // 创建测试订单
    let order_id = 1;

    // 进行支付
    let mut payment = create_test_payment();
    payment.order_id = order_id;
    let result = Payment::create(&db, &payment).await;
    assert!(result.is_ok());
    let mut payment1 = create_test_payment();
    payment1.amount = 100.0;
    payment1.id = 2;
    payment1.order_id = 2;
    Payment::create(&db, &payment1).await.unwrap();

    let result: Vec<Payment> = Payment::prepare(&db)
        .delete()
        .where_clauses(vec!["id <", "order_id <", "amount >"])
        .values(vec![
            Value::Bigint(10),
            Value::Bigint(10),
            Value::Double(100.00),
        ])
        .query()
        .await
        .unwrap();
    let left: Vec<Payment> = Payment::find_all(&db).await.unwrap();
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].amount, 100.0);
    dbg!(&result);
}

#[tokio::test]
#[serial]
async fn test_complex_select() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Pay {
        id: i64,
        order_id: i64,
        amount: f64,
        payment_method: String,
        transaction_id: String,
        // #[serde(with = "chrono::serde::ts_seconds")]
        // paid_at: DateTime<Utc>,
    }

    let db = setup_test_db().await;

    // 创建测试订单
    let order_id = 1;

    // 进行支付
    let mut payment = create_test_payment();
    payment.order_id = order_id;
    let result = Payment::create(&db, &payment).await;
    assert!(result.is_ok());
    let mut payment1 = create_test_payment();
    payment1.amount = 100.0;
    payment1.id = 2;
    payment1.order_id = 2;
    Payment::create(&db, &payment1).await.unwrap();

    let result: Vec<Pay> = Payment::prepare(&db)
        .select(&[
            "id",
            "order_id",
            "amount",
            "payment_method",
            "transaction_id",
            // "paid_at"
        ])
        .where_clauses(vec!["id <", "order_id <", "amount >="])
        .order_by(vec!["amount  asc"])
        .group_by(vec!["id"])
        .having(vec!["order_id ="])
        .values(vec![
            Value::Bigint(10),
            Value::Bigint(10),
            Value::Double(100.00),
            Value::Bigint(2),
        ])
        .query()
        .await
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].order_id, 2);

    dbg!(&result);
}

#[tokio::test]
#[serial]
async fn test_join() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct PaymentDetail {
        id: i64,
        order_id: i64,
        amount: f64,
        payment_method: String,
        transaction_id: String,
        #[serde(with = "chrono::serde::ts_seconds")]
        paid_at: DateTime<Utc>,
        product_id: i64,
        quantity: i64,
        name: String,
        description: String,
        price: f64,
        x_stock: i64,
    }

    let db = setup_test_db().await;

    let product = create_test_product();
    // 创建测试订单
    let mut cart_item = create_test_cart_item();
    cart_item.product_id = product.id;
    // 进行支付
    let mut payment = create_test_payment();
    payment.order_id = cart_item.id;

    Product::create(&db, &product).await.unwrap();
    Payment::create(&db, &payment).await.unwrap();
    CartItem::create(&db, &cart_item).await.unwrap();

    let result: Vec<PaymentDetail> = Payment::prepare(&db)
        .select(&[
            "payments.id",
            "payments.order_id",
            "payments.amount",
            "payments.payment_method",
            "payments.transaction_id",
            "payments.paid_at",
            "cart_items.product_id",
            "cart_items.quantity",
            "products.name",
            "products.description",
            "products.price",
            "products.stock as x_stock",
        ])
        .join("cart_items", "cart_items.id = payments.order_id")
        .join("products", "products.id = cart_items.product_id")
        .query()
        .await
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].order_id, 1);

    dbg!(&result);
}

#[tokio::test]
#[serial]
async fn test_complex_update() {
    let db = setup_test_db().await;

    // 创建测试商品
    let product = create_test_product();
    Product::create(&db, &product).await.unwrap();

    // 更新商品库存
    // product.stock = 50;
    let result: u64 = Product::prepare::<Product>(&db)
        .update(&["stock"])
        .where_clauses(vec!["id ="])
        .values(vec![Value::Bigint(50), Value::Bigint(1)])
        .execute()
        .await
        .unwrap();
    assert_eq!(result, 1);

    // 验证商品库存是否更新
    let updated_product: Option<Product> = Product::find_by_id(&db, Value::Bigint(product.id))
        .await
        .unwrap();
    assert_eq!(updated_product.unwrap().stock, 50);
}

#[tokio::test]
#[serial]
async fn test_complex_insert() {
    let db = setup_test_db().await;

    // 创建测试商品
    let product = create_test_product();
    Product::prepare::<Product>(&db)
        .insert(&["id", "name", "description", "price", "stock", "created_at"])
        .values(vec![
            Value::Bigint(1),
            Value::Text("Test Product".to_string()),
            Value::Text("This is a test product.".to_string()),
            Value::Double(99.99),
            Value::Bigint(100),
            Value::Bigint(Utc::now().timestamp()),
        ])
        .execute()
        .await
        .unwrap();

    // 添加商品到购物车
    let mut cart_item = create_test_cart_item();
    cart_item.product_id = product.id;
    let result = CartItem::create(&db, &cart_item).await;
    assert!(result.is_ok());

    // 验证购物车项是否添加成功
    let added_item: Option<CartItem> = CartItem::find_by_id(&db, Value::Bigint(cart_item.id))
        .await
        .unwrap();
    assert!(added_item.is_some());
    let item = added_item.unwrap();
    dbg!(&item);
    assert_eq!(item.product_id, product.id);
}

#[tokio::test]
#[serial]
async fn test_complex_sql() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Pay {
        id: i64,
        order_id: i64,
        amount: f64,
        payment_method: String,
        transaction_id: String,
        // #[serde(with = "chrono::serde::ts_seconds")]
        // paid_at: DateTime<Utc>,
    }

    let db = setup_test_db().await;

    // 创建测试订单
    let order_id = 1;

    // 进行支付
    let mut payment = create_test_payment();
    payment.order_id = order_id;
    let result = Payment::create(&db, &payment).await;
    assert!(result.is_ok());
    let mut payment1 = create_test_payment();
    payment1.amount = 100.0;
    payment1.id = 2;
    payment1.order_id = 2;
    Payment::create(&db, &payment1).await.unwrap();

    let result: Vec<Pay> = Payment::prepare(&db)
        .select(&[
            "id",
            "order_id",
            "amount",
            "payment_method",
            "transaction_id",
            // "paid_at"
        ])
        .where_clauses(vec!["id <", "order_id <", "amount >="])
        .order_by(vec!["amount  desc"])
        // .group_by(vec!["id"])
        // .having(vec!["order_id ="])
        .limit(2)
        .offset(1)
        .values(vec![
            Value::Bigint(10),
            Value::Bigint(10),
            Value::Double(100.00),
            // Value::Bigint(2),
        ])
        .query()
        .await
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].order_id, 2);

    dbg!(&result);
}
