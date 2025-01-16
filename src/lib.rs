pub mod entity;
pub mod repository;
pub mod service;
pub mod controller;
mod database;
mod dao;

use std::sync::{Arc, Mutex};

// 重新导出主要的类型
pub use entity::UserEntity;
pub use repository::UserRepository;
pub use service::UserService;
pub use controller::UserController;

// 定义错误类型
#[derive(Debug)]
pub enum UserError {
    NotFound,
    InvalidInput(String),
    DatabaseError(String),
}

// 定义结果类型别名
pub type UserResult<T> = Result<T, UserError>;

// 添加测试模块
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_crud() {
        // 创建依赖关系
        let repository = Arc::new(UserRepository::new());
        let service = Arc::new(UserService::new(repository.clone()));
        let controller = UserController::new(service.clone());

        // 测试创建用户
        let create_request = controller::CreateUserRequest {
            username: "张三".to_string(),
            email: "zhangsan@example.com".to_string(),
            age: 25,
        };
        let user = controller.create_user(create_request).unwrap();
        assert_eq!(user.username, "张三");

        // 测试查询用户
        let found_user = controller.get_user(user.id).unwrap();
        assert_eq!(found_user.username, "张三");
        assert_eq!(found_user.email, "zhangsan@example.com");

        // 测试更新用户
        let update_request = controller::CreateUserRequest {
            username: "李四".to_string(),
            email: "lisi@example.com".to_string(),
            age: 26,
        };
        let updated_user = controller.update_user(user.id, update_request).unwrap();
        assert_eq!(updated_user.username, "李四");

        // 测试删除用户
        assert!(controller.delete_user(user.id).is_ok());
        assert!(controller.get_user(user.id).is_err());
    
      assert!(controller.get_user(user.id).is_err());
    }
}