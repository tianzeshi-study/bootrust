use bootrust::asyncdatabase::{sqlite::SqliteDatabase, DatabaseConfig, RelationalDatabase, Value};
use bootrust::entity::Entity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serial_test::serial;

// 商品实体
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Product {
    id: i64,
    name: String,
    description: Option<String>,
    price: f64,
    stock: i64,
    #[serde(with = "chrono::serde::ts_seconds")]
    created_at: DateTime<Utc>,
    log: Vec<u8>,
    history: Vec<String>,
    // count: Option<i64>,
}
impl Entity for Product {
    fn table() -> String {
        "products".to_string()
    }

    fn primary_key() -> String {
        "id".to_string()
    }
}

// 设置测试数据库
async fn setup_ecommerce_test_db() -> SqliteDatabase {
    let config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 3306,
        username: "root".to_string(),
        password: "root".to_string(),
        database_name: "test".to_string(),
        max_size: 10,
    };
    let db = SqliteDatabase::connect(config).await.unwrap();

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
            created_at INTEGER,
            log BLOB  NOT NULL,
            history BLOB NOT NULL
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
        // description: Some("This is a test product.".to_string()),
        description: None,
        price: 99.99,
        stock: 100,
        created_at: Utc::now(),
        log: b"0".to_vec(),
        history: vec!["0".to_string(), "1".to_string()],
        // count: None
    }
}

// 测试添加商品到购物车
#[tokio::test]
#[serial]
async fn test_entity_add_product_to_cart() {
    let db = setup_ecommerce_test_db().await;

    // 创建测试商品
    let product = create_test_product();
    Product::create(&db, &product).await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_stock_update() {
    let db = setup_ecommerce_test_db().await;

    // 创建测试商品
    let mut product = create_test_product();
    Product::create(&db, &product).await.unwrap();

    // 更新商品库存
    product.stock = 50;
    let result = Product::update(&db, &product).await;
    assert!(result.is_ok());

    // 验证商品库存是否更新
    let updated_product: Product = Product::find_by_id(&db, Value::Bigint(product.id))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated_product.stock, 50);
    assert_eq!(updated_product.log, b"0".to_vec());
    assert_eq!(
        updated_product.history,
        vec!["0".to_string(), "1".to_string()]
    );
}
