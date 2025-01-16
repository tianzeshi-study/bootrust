// 实体层

#[derive(Debug, Clone)]
pub struct UserEntity {
    pub id: Option<u32>,
    pub username: String,
    pub email: String,
    pub age: u8,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl UserEntity {
    pub fn new(username: String, email: String, age: u8) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: None,
            username,
            email,
            age,
            created_at: now,
            updated_at: now,
        }
    }
}
