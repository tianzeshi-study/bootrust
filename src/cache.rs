use async_trait::async_trait;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::{AsyncCommands, ErrorKind, RedisError};
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;
use std::time::Duration;

#[async_trait]
pub trait Dco<T>
where
    T: 'static + Sized + Sync + Send + Serialize + DeserializeOwned,
{
    type Error;

    async fn get(&self, key: &str) -> Result<Option<T>, Self::Error>;
    async fn set(&self, key: &str, value: T, ttl: Option<Duration>) -> Result<(), Self::Error>;
    async fn del(&self, key: &str) -> Result<(), Self::Error>;
    async fn exists(&self, key: &str) -> Result<bool, Self::Error>;
}

pub struct RedisCache<T> {
    pool: Pool<RedisConnectionManager>,
    _table: PhantomData<T>,
}

impl<T> RedisCache<T> {
    pub async fn new(url: &str) -> Result<Self, RedisError> {
        let manager = RedisConnectionManager::new(url)?;
        let pool = Pool::builder().build(manager).await?;
        Ok(RedisCache {
            pool: pool,
            _table: PhantomData,
        })
    }
}

#[async_trait]
impl<T> Dco<T> for RedisCache<T>
where
    T: 'static + Sized + Sync + Send + Serialize + DeserializeOwned,
{
    type Error = RedisError;

    async fn get(&self, key: &str) -> Result<Option<T>, Self::Error> {
        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            _ => {
                return Err(RedisError::from((
                    ErrorKind::ClientError,
                    "error getting connect",
                )))
            }
        };

        let result: Option<Vec<u8>> = conn.get(key).await?;
        match result {
            Some(bytes) => {
                let value: T = bincode::deserialize(&bytes).map_err(|e| {
                    redis::RedisError::from((
                        redis::ErrorKind::TypeError,
                        "Deserialization error",
                        e.to_string(),
                    ))
                })?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    async fn set(&self, key: &str, value: T, ttl: Option<Duration>) -> Result<(), Self::Error> {
        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            _ => {
                return Err(RedisError::from((
                    ErrorKind::ClientError,
                    "error getting connect",
                )))
            }
        };

        let bytes = bincode::serialize(&value).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Serialization error",
                e.to_string(),
            ))
        })?;

        match ttl {
            Some(duration) => conn.set_ex(key, bytes, duration.as_secs() as u64).await,
            None => conn.set(key, bytes).await,
        }
    }

    async fn del(&self, key: &str) -> Result<(), Self::Error> {
        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            _ => {
                return Err(RedisError::from((
                    ErrorKind::ClientError,
                    "error getting connect",
                )))
            }
        };
        conn.del(key).await
    }

    async fn exists(&self, key: &str) -> Result<bool, Self::Error> {
        // let mut conn = self.pool.get().await.map_err(redis::RedisError::from)?;
        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            _ => {
                return Err(RedisError::from((
                    ErrorKind::ClientError,
                    "error getting connect",
                )))
            }
        };

        conn.exists(key).await
    }
}

trait CachedData = 'static + Sized + Sync + Send + Serialize + DeserializeOwned;
#[async_trait]
pub trait CacheDb {
    type Error;

    async fn get<T: CachedData>(&self, key: &str) -> Result<Option<T>, Self::Error>;
    async fn set<T: CachedData>(
        &self,
        key: &str,
        value: T,
        ttl: Option<Duration>,
    ) -> Result<(), Self::Error>;
    async fn del<T: CachedData>(&self, key: &str) -> Result<(), Self::Error>;
    async fn exists<T: CachedData>(&self, key: &str) -> Result<bool, Self::Error>;
}

pub struct Redis {
    pool: Pool<RedisConnectionManager>,
}

impl Redis {
    pub async fn new(url: &str) -> Result<Self, RedisError> {
        let manager = RedisConnectionManager::new(url)?;
        let pool = Pool::builder().build(manager).await?;
        Ok(Redis { pool: pool })
    }
}
#[async_trait]
impl CacheDb for Redis {
    type Error = RedisError;

    async fn get<T: CachedData>(&self, key: &str) -> Result<Option<T>, Self::Error> {
        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            _ => {
                return Err(RedisError::from((
                    ErrorKind::ClientError,
                    "error getting connect",
                )))
            }
        };

        let result: Option<Vec<u8>> = conn.get(key).await?;
        match result {
            Some(bytes) => {
                let value: T = bincode::deserialize(&bytes).map_err(|e| {
                    redis::RedisError::from((
                        redis::ErrorKind::TypeError,
                        "Deserialization error",
                        e.to_string(),
                    ))
                })?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    async fn set<T: CachedData>(
        &self,
        key: &str,
        value: T,
        ttl: Option<Duration>,
    ) -> Result<(), Self::Error> {
        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            _ => {
                return Err(RedisError::from((
                    ErrorKind::ClientError,
                    "error getting connect",
                )))
            }
        };

        let bytes = bincode::serialize(&value).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Serialization error",
                e.to_string(),
            ))
        })?;

        match ttl {
            Some(duration) => conn.set_ex(key, bytes, duration.as_secs() as u64).await,
            None => conn.set(key, bytes).await,
        }
    }

    async fn del<T: CachedData>(&self, key: &str) -> Result<(), Self::Error> {
        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            _ => {
                return Err(RedisError::from((
                    ErrorKind::ClientError,
                    "error getting connect",
                )))
            }
        };
        conn.del(key).await
    }

    async fn exists<T: CachedData>(&self, key: &str) -> Result<bool, Self::Error> {
        // let mut conn = self.pool.get().await.map_err(redis::RedisError::from)?;
        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            _ => {
                return Err(RedisError::from((
                    ErrorKind::ClientError,
                    "error getting connect",
                )))
            }
        };

        conn.exists(key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use serial_test::serial;
    use std::time::Duration;
    use tokio::time::sleep; // Import sleep for testing TTL

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestData {
        a: i32,
        b: String,
    }

    async fn setup_cache<T>() -> RedisCache<T> {
        // Use a different database number for testing to avoid conflicts
        // with any existing data in the default database.
        RedisCache::new("redis://root@127.0.0.1:6379/1")
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_set_get_del() {
        let cache = setup_cache().await;
        let key = "test_key";
        let value = TestData {
            a: 42,
            b: "hello".to_string(),
        };

        // Set the value
        cache.set(key, value, None).await.unwrap();

        // Get the value and check it
        let retrieved_value: Option<TestData> = cache.get(key).await.unwrap();
        assert_eq!(
            retrieved_value,
            Some(TestData {
                a: 42,
                b: "hello".to_string()
            })
        );

        // Delete the value
        cache.del(key).await.unwrap();

        // Get the value again, should be None
        let retrieved_value: Option<TestData> = cache.get(key).await.unwrap();
        assert_eq!(retrieved_value, None);
    }

    #[tokio::test]
    async fn test_set_get_with_ttl() {
        let cache = setup_cache().await;
        let key = "test_key_ttl";
        let value = TestData {
            a: 123,
            b: "world".to_string(),
        };
        let ttl = Duration::from_secs(1); // Short TTL for testing

        // Set the value with TTL
        cache.set(key, value, Some(ttl)).await.unwrap();

        // Get the value immediately, should be Some
        let retrieved_value: Option<TestData> = cache.get(key).await.unwrap();
        assert_eq!(
            retrieved_value,
            Some(TestData {
                a: 123,
                b: "world".to_string()
            })
        );

        // Wait for longer than the TTL
        sleep(Duration::from_secs(2)).await;

        // Get the value again, should be None
        let retrieved_value: Option<TestData> = cache.get(key).await.unwrap();
        assert_eq!(retrieved_value, None);
    }

    #[tokio::test]
    async fn test_exists() {
        let cache = setup_cache().await;
        let key = "test_key_exists";
        let non_existent_key = "non_existent_key";
        let value = TestData {
            a: 1,
            b: "exists".to_string(),
        };

        // Key shouldn't exist yet
        assert!(!cache.exists(key).await.unwrap());

        // Set the value
        cache.set(key, value, None).await.unwrap();

        // Now the key should exist
        assert!(cache.exists(key).await.unwrap());

        // Non-existent key should not exist
        assert!(!cache.exists(non_existent_key).await.unwrap());

        // Delete Key
        cache.del(key).await.unwrap();
        assert!(!cache.exists(key).await.unwrap());
    }

    #[tokio::test]
    async fn test_get_nonexistent() {
        let cache = setup_cache().await;
        let key = "nonexistent_key";

        // Get a non-existent key, should return None
        let retrieved_value: Option<TestData> = cache.get(key).await.unwrap();
        assert_eq!(retrieved_value, None);
    }

    #[tokio::test]
    async fn test_del_nonexistent() {
        let cache: RedisCache<TestData> = setup_cache().await;
        let key = "nonexistent_key_to_delete";

        // Delete a non-existent key.  Should not error.
        let result = cache.del(key).await;
        assert!(result.is_ok());
    }

    async fn setup_cache_db() -> Redis {
        // Use a different database number for testing to avoid conflicts
        // with any existing data in the default database.
        Redis::new("redis://root@127.0.0.1:6379/2").await.unwrap()
    }

    #[tokio::test]
    async fn test_db_set_get_del() {
        let cache = setup_cache_db().await;
        let key = "test_key";
        let value = TestData {
            a: 42,
            b: "hello".to_string(),
        };

        // Set the value
        cache.set(key, value, None).await.unwrap();

        // Get the value and check it
        let retrieved_value: Option<TestData> = cache.get(key).await.unwrap();
        assert_eq!(
            retrieved_value,
            Some(TestData {
                a: 42,
                b: "hello".to_string()
            })
        );

        // Delete the value
        cache.del::<TestData>(key).await.unwrap();

        // Get the value again, should be None
        let retrieved_value: Option<TestData> = cache.get(key).await.unwrap();
        assert_eq!(retrieved_value, None);
    }

    #[tokio::test]
    async fn test_db_set_get_with_ttl() {
        let cache = setup_cache_db().await;
        let key = "test_key_ttl";
        let value = TestData {
            a: 123,
            b: "world".to_string(),
        };
        let ttl = Duration::from_secs(1); // Short TTL for testing

        // Set the value with TTL
        cache.set::<TestData>(key, value, Some(ttl)).await.unwrap();

        // Get the value immediately, should be Some
        let retrieved_value: Option<TestData> = cache.get::<TestData>(key).await.unwrap();
        assert_eq!(
            retrieved_value,
            Some(TestData {
                a: 123,
                b: "world".to_string()
            })
        );

        // Wait for longer than the TTL
        sleep(Duration::from_secs(2)).await;

        // Get the value again, should be None
        let retrieved_value: Option<TestData> = cache.get(key).await.unwrap();
        assert_eq!(retrieved_value, None);
    }

    #[tokio::test]
    #[serial]
    async fn test_db_exists() {
        let cache = setup_cache().await;
        let key = "test_key_exists";
        let non_existent_key = "non_existent_key";
        let value = TestData {
            a: 1,
            b: "exists".to_string(),
        };

        // Key shouldn't exist yet
        assert!(!cache.exists(key).await.unwrap());

        // Set the value
        cache.set(key, value, None).await.unwrap();

        // Now the key should exist
        assert!(cache.exists(key).await.unwrap());

        // Non-existent key should not exist
        assert!(!cache.exists(non_existent_key).await.unwrap());

        // Delete Key
        cache.del(key).await.unwrap();
        assert!(!cache.exists(key).await.unwrap());
    }

    #[tokio::test]
    async fn test_db_get_nonexistent() {
        let cache = setup_cache_db().await;
        let key = "nonexistent_key";

        // Get a non-existent key, should return None
        let retrieved_value: Option<TestData> = cache.get::<TestData>(key).await.unwrap();
        assert_eq!(retrieved_value, None);
    }

    #[tokio::test]
    async fn test_db_del_nonexistent() {
        let cache: Redis = setup_cache_db().await;
        let key = "nonexistent_key_to_delete";

        // Delete a non-existent key.  Should not error.
        let result = cache.del::<TestData>(key).await;
        assert!(result.is_ok());
    }
}
