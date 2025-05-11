#[cfg(all(
    not(feature = "full"),
    any(
        feature = "sqlite_async",
        feature = "mysql_async",
        feature = "postgresql_async"
    )
))]
use bootrust::{
    asyncdatabase::{auto_config, RelationalDatabase},
    entity::Entity,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    id: i64,
    name: String,
    // 将 DateTime<Utc> 序列化为 Unix 秒数（i64）
    #[serde(with = "chrono::serde::ts_seconds")]
    created_at: DateTime<Utc>,
}

#[cfg(all(
    not(feature = "full"),
    any(
        feature = "sqlite_async",
        feature = "mysql_async",
        feature = "postgresql_async"
    )
))]
impl Entity for User {
    fn table() -> String {
        "example".to_string()
    }
    fn primary_key() -> String {
        "id".to_string()
    }
}

#[tokio::main]
async fn main() {
    #[cfg(all(
        not(feature = "full"),
        any(
            feature = "sqlite_async",
            feature = "mysql_async",
            feature = "postgresql_async"
        )
    ))]
    simple_example().await.unwrap();
}

#[cfg(all(
    not(feature = "full"),
    any(
        feature = "sqlite_async",
        feature = "mysql_async",
        feature = "postgresql_async"
    )
))]
async fn simple_example() -> Result<(), Box<dyn std::error::Error>> {
    // 根据 URL 自动选择数据库驱动
    let db = auto_config().await;

    // create table
    // suggest write sql in init.sql, hard code  here is just example
    db.execute(
        "
    drop table if exists  example
",
        vec![],
    )
    .await?;

    db.execute(
        "
    create table  example(
id BIGINT  PRIMARY KEY,
name TEXT,
created_at BIGINT
)",
        vec![],
    )
    .await?;

    // create new users
    let new_user = User {
        id: 1,
        name: "Alice".into(),
        created_at: Utc::now(),
    };

    let another_user = User {
        id: 2,
        name: "Bob".into(),
        created_at: Utc::now(),
    };

    User::create(&db, &new_user).await?;
    User::create(&db, &another_user).await?;

    // find all
    let users: Vec<User> = User::find_all(&db).await?;
    println!("{:?}", users);
    let user1: User = User::find_by_id(&db, 1).await?.expect("cant find");
    println!("{:?}", user1);

    Ok(())
}
