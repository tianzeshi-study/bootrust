// 服务层
use super::*;

pub struct UserService {
    repository: Arc<UserRepository>,
}

impl UserService {
    pub fn new(repository: Arc<UserRepository>) -> Self {
        Self { repository }
    }

    pub fn create_user(&self, username: String, email: String, age: u8) -> UserResult<UserEntity> {
        // 输入验证
        if username.is_empty() {
            return Err(UserError::InvalidInput(
                "Username cannot be empty".to_string(),
            ));
        }
        if email.is_empty() {
            return Err(UserError::InvalidInput("Email cannot be empty".to_string()));
        }
        if age < 1 {
            return Err(UserError::InvalidInput("Age must be positive".to_string()));
        }

        let user = UserEntity::new(username, email, age);
        self.repository.create(user)
    }

    pub fn get_user(&self, id: u32) -> UserResult<UserEntity> {
        self.repository.find_by_id(id)
    }

    pub fn get_all_users(&self) -> UserResult<Vec<UserEntity>> {
        self.repository.find_all()
    }

    pub fn update_user(
        &self,
        id: u32,
        username: String,
        email: String,
        age: u8,
    ) -> UserResult<UserEntity> {
        let user = UserEntity::new(username, email, age);
        self.repository.update(id, user)
    }

    pub fn delete_user(&self, id: u32) -> UserResult<()> {
        self.repository.delete(id)
    }
}
