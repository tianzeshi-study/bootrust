#[cfg(feature="postgres_async")]
pub mod database_injection;
pub mod postgres_async_daos;

#[cfg(feature="mysql")]
pub mod mysql_daos_test;
pub mod mysql_dao_test;
pub mod e_commerce_system_mysql_do_test;
pub mod e_commerce_system_mysql_daos_test;

#[cfg(feature="sqlite")]
pub mod sqlite_dao_test;