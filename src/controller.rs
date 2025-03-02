/*
// 控制层

use super::*;

pub struct UserController {
    service: Arc<UserService>,
}

// 用于API响应的DTO
#[derive(Debug, serde::Serialize)]
pub struct UserResponse {
    pub id: u32,
    pub username: String,
    pub email: String,
    pub age: u8,
}

// 用于创建用户的DTO
#[derive(Debug, serde::Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub age: u8,
}

impl UserController {
    pub fn new(service: Arc<UserService>) -> Self {
        Self { service }
    }

    pub fn create_user(&self, request: CreateUserRequest) -> UserResult<UserResponse> {
        let user = self
            .service
            .create_user(request.username, request.email, request.age)?;

        Ok(UserResponse {
            id: user.id.unwrap(),
            username: user.username,
            email: user.email,
            age: user.age,
        })
    }

    pub fn get_user(&self, id: u32) -> UserResult<UserResponse> {
        let user = self.service.get_user(id)?;
        Ok(UserResponse {
            id: user.id.unwrap(),
            username: user.username,
            email: user.email,
            age: user.age,
        })
    }

    pub fn get_all_users(&self) -> UserResult<Vec<UserResponse>> {
        let users = self.service.get_all_users()?;
        Ok(users
            .into_iter()
            .map(|user| UserResponse {
                id: user.id.unwrap(),
                username: user.username,
                email: user.email,
                age: user.age,
            })
            .collect())
    }

    pub fn update_user(&self, id: u32, request: CreateUserRequest) -> UserResult<UserResponse> {
        let user = self
            .service
            .update_user(id, request.username, request.email, request.age)?;

        Ok(UserResponse {
            id: user.id.unwrap(),
            username: user.username,
            email: user.email,
            age: user.age,
        })
    }

    pub fn delete_user(&self, id: u32) -> UserResult<()> {
        self.service.delete_user(id)
    }
}

*/
