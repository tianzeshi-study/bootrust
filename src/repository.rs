// 仓储层

use super::*;
use std::collections::HashMap;

pub struct UserRepository {
    users: Arc<Mutex<HashMap<u32, UserEntity>>>,
    next_id: Arc<Mutex<u32>>,
}

impl UserRepository {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    pub fn create(&self, mut user: UserEntity) -> UserResult<UserEntity> {
        let mut next_id = self.next_id.lock().unwrap();
        let mut users = self.users.lock().unwrap();

        user.id = Some(*next_id);
        users.insert(*next_id, user.clone());
        *next_id += 1;

        Ok(user)
    }

    pub fn find_by_id(&self, id: u32) -> UserResult<UserEntity> {
        let users = self.users.lock().unwrap();
        users.get(&id).cloned().ok_or(UserError::NotFound)
    }

    pub fn find_all(&self) -> UserResult<Vec<UserEntity>> {
        let users = self.users.lock().unwrap();
        Ok(users.values().cloned().collect())
    }

    pub fn update(&self, id: u32, user: UserEntity) -> UserResult<UserEntity> {
        let mut users = self.users.lock().unwrap();

        if let Some(existing_user) = users.get_mut(&id) {
            existing_user.username = user.username;
            existing_user.email = user.email;
            existing_user.age = user.age;
            existing_user.updated_at = chrono::Utc::now();
            Ok(existing_user.clone())
        } else {
            Err(UserError::NotFound)
        }
    }

    pub fn delete(&self, id: u32) -> UserResult<()> {
        let mut users = self.users.lock().unwrap();
        if users.remove(&id).is_some() {
            Ok(())
        } else {
            Err(UserError::NotFound)
        }
    }
}

pub trait Repository {
    // 关联类型，用于指定具体的实体类型
    type DomainObject;
    type Error;

    // CRUD 基本操作
    fn create(&self, DomainObject: Self::DomainObject) -> Result<Self::DomainObject, Self::Error>;
    fn find_by_id(&self, id: u32) -> Result<Self::DomainObject, Self::Error>;
    fn find_all(&self) -> Result<Vec<Self::DomainObject>, Self::Error>;
    fn update(
        &self,
        id: u32,
        DomainObject: Self::DomainObject,
    ) -> Result<Self::DomainObject, Self::Error>;
    fn delete(&self, id: u32) -> Result<(), Self::Error>;
}

// 然后为 UserRepository 实现这个 trait
impl Repository for UserRepository {
    type DomainObject = UserEntity;
    type Error = UserError;

    fn create(&self, DomainObject: Self::DomainObject) -> Result<Self::DomainObject, Self::Error> {
        self.create(DomainObject)
    }

    fn find_by_id(&self, id: u32) -> Result<Self::DomainObject, Self::Error> {
        self.find_by_id(id)
    }

    fn find_all(&self) -> Result<Vec<Self::DomainObject>, Self::Error> {
        self.find_all()
    }

    fn update(
        &self,
        id: u32,
        DomainObject: Self::DomainObject,
    ) -> Result<Self::DomainObject, Self::Error> {
        self.update(id, DomainObject)
    }

    fn delete(&self, id: u32) -> Result<(), Self::Error> {
        self.delete(id)
    }
}
