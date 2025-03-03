pub trait Cache<T> {
async fn get(key: &str)-> Option<T>;
async fn set(key: &str, value: T, time: i64)->Result<(), Err>;
}