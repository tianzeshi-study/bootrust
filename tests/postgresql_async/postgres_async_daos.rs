use bootrust::asyncdao::Dao;
use bootrust::asyncdatabase::{
    postgres::PostgresDatabase, DatabaseConfig, DbError, RelationalDatabase, Row, Value,
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
async fn setup_ecommerce_test_db() -> PostgresDatabase {
    let config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        username: "root".to_string(),
        password: "root".to_string(),
        database_name: "test".to_string(),
        max_size: 10,
    };
    let db = PostgresDatabase::connect(config).await.unwrap();

    // 创建商品表
    db.execute("DROP TABLE IF EXISTS products", vec![])
        .await
        .unwrap();
    db.execute(
        "CREATE TABLE products (
            id BIGSERIAL PRIMARY KEY,
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
async fn test_transaction_rollback() {
    let db = setup_ecommerce_test_db().await;
    let product_dao = ECommerceDo::new(db.clone());
    let cart_dao = ECommerceDo::new(db.clone());

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
