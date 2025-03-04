pub trait Cache<T> {
    type TTL;
async fn get(key: &str)-> Option<T>;
async fn set(key: &str, value: T)->Result<(), Err>;
}