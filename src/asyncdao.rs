use crate::asyncdatabase::{DbError, RelationalDatabase, Row, Value};
use crate::serde::{EntityDeserializer,EntityConvertor};
use serde::{de::Deserialize, ser::Serialize};
use std::io::Cursor;
use std::marker::PhantomData;

#[async_trait::async_trait]
pub trait Dao<T>: Sized
where
    T: Sized + Sync + Serialize + for<'de> Deserialize<'de>,
{
    /// 关联的数据库类型
    type Database: RelationalDatabase;

    /// 数据库引用
    fn database(&self) -> &Self::Database;

    fn placeholders(&self, keys: &Vec<String>) -> Vec<String> {
        self.database().placeholders(keys)
    }

    /// 创建新的 DAO 实例
    fn new(database: Self::Database) -> Self;

    fn row_to_entity(row: Row) -> Result<T, DbError> {
        let values: Vec<Value> = row.values;
        let table: Vec<(String, Value)> = row
            .columns
            .into_iter()
            .enumerate()
            .map(|(i, s)| (s, values[i].clone()))
            .collect();
        let de = EntityDeserializer::from_value(Value::Table(table));
        T::deserialize(de).map_err(|e| DbError::ConversionError(e.to_string()))
    }

    fn entity_to_map(entity: &T) -> Vec<(String, Value)> {
        let cursor = Cursor::new(Vec::new());
        let mut convertor = EntityConvertor::new(cursor);
        let result = entity.serialize(&mut convertor);
        match result {
            Ok(Value::Table(table)) => table,
            _ => vec![("".to_string(), Value::Null)],
        }
    }

    /// 将实体对象转换为数据库值
    fn entity_to_values(&self, entity: &T) -> Vec<Value> {
        Self::entity_to_map(entity)
            .into_iter()
            .map(|kv| kv.1)
            .collect()
    }

    fn entity_to_keys(&self, entity: &T) -> Vec<String> {
        Self::entity_to_map(entity)
            .into_iter()
            .map(|kv| kv.0)
            .collect()
    }

    /// 获取表名
    fn table_name() -> String;

    fn table(&self) -> String {
        Self::table_name()
    }

    /// 获取主键列名
    fn primary_key_column() -> String;

    /// 创建新记录
    async fn create(&self, entity: &T) -> Result<u64, DbError> {
        let values = self.entity_to_values(entity);
        let keys = self.entity_to_keys(entity);
        let placeholders: Vec<String> = self.placeholders(&keys);

        let query = format!(
            "INSERT INTO {} VALUES ({})",
            Self::table_name(),
            placeholders.join(", ")
        );

        self.database().execute(&query, values).await
    }

    /// 根据ID查找记录
    async fn find_by_id(&self, id: Value) -> Result<Option<T>, DbError> {
        let placeholder = self.placeholders(&vec![Self::primary_key_column()])[0].clone();
        let query = format!(
            "SELECT * FROM {} WHERE {} = {}",
            Self::table_name(),
            Self::primary_key_column(),
            placeholder
        );

        let result = self.database().query_one(&query, vec![id]).await?;
        match result {
            Some(row) => Ok(Some(Self::row_to_entity(row)?)),
            None => Ok(None),
        }
    }

    /// 查找所有记录
    async fn find_all(&self) -> Result<Vec<T>, DbError> {
        let query = format!("SELECT * FROM {}", Self::table_name());
        let rows = self.database().query(&query, vec![]).await?;

        let mut entities = Vec::with_capacity(rows.len());
        for row in rows {
            entities.push(Self::row_to_entity(row)?);
        }
        Ok(entities)
    }

    /// 更新记录
    async fn update(&self, entity: &T) -> Result<u64, DbError> {
        let map = Self::entity_to_map(entity);
        let mut values: Vec<Value> = Vec::new();

        let mut primary_value = None;
        let update_columns: Vec<String> = map
            .iter()
            .inspect(|kv| {
                if kv.0 == Self::primary_key_column() {
                    primary_value = Some(kv.1.clone());
                }
            })
            .filter(|kv| kv.0 != Self::primary_key_column())
            .enumerate()
            .map(|(i, kv)| {
                let placeholder = self.placeholders(&vec![kv.0.clone(); i + 1])[i].clone();

                values.push(kv.1.clone());
                format!("{} = {}", kv.0, placeholder)
            })
            .collect();

        if let Some(id_value) = primary_value {
            values.push(id_value.clone());
        }

        let query = format!(
            "UPDATE {} SET {} WHERE {} = {}",
            Self::table_name(),
            update_columns.join(", "),
            Self::primary_key_column(),
            self.placeholders(&vec![Self::primary_key_column(); values.len()])[values.len() - 1]
                .clone(),
        );

        dbg!(&query);
        self.database().execute(&query, values).await
    }

    /// 删除记录
    async fn delete(&self, id: Value) -> Result<u64, DbError> {
        let placeholder = self.placeholders(&vec![Self::primary_key_column()])[0].clone();
        let query = format!(
            "DELETE FROM {} WHERE {} = {}",
            Self::table_name(),
            Self::primary_key_column(),
            placeholder
        );

        self.database().execute(&query, vec![id]).await
    }

    /// 自定义条件查询
    async fn find_by_condition(
        &self,
        condition: &str,
        params: Vec<Value>,
    ) -> Result<Vec<T>, DbError> {
        let query = format!("SELECT * FROM {} WHERE {}", Self::table_name(), condition);

        let rows = self.database().query(&query, params).await?;
        let mut entities = Vec::with_capacity(rows.len());
        for row in rows {
            entities.push(Self::row_to_entity(row)?);
        }
        Ok(entities)
    }

    async fn begin_transaction(&self) -> Result<(), DbError> {
        self.database().begin_transaction().await
    }

    async fn commit(&self) -> Result<(), DbError> {
        self.database().commit().await
    }

    async fn rollback(&self) -> Result<(), DbError> {
        self.database().rollback().await
    }

    fn prepare(&self) -> SqlExecutor<Self, T> {
        SqlExecutor::new(self)
    }
}

pub struct DataAccessory<T: Sized, D: RelationalDatabase> {
    database: D,
    _table: PhantomData<T>,
}
impl<T, D> Dao<T> for DataAccessory<T, D>
where
    T: Sized + Sync + Serialize + for<'de> Deserialize<'de>,
    D: RelationalDatabase,
{
    type Database = D;
    fn database(&self) -> &Self::Database {
        &self.database
    }

    fn new(database: Self::Database) -> Self {
        Self {
            database,
            _table: PhantomData,
        }
    }

    
    fn entity_to_map(entity: &T) -> Vec<(String, Value)> {

        let cursor = Cursor::new(Vec::new());
        let mut convertor = EntityConvertor::new(cursor);
        let result = entity.serialize(&mut convertor);
        match result {
            Ok(Value::Table(table)) => table,
            _ => vec![("".to_string(), Value::Null)],
        }
    }
    fn table_name() -> String {
        "user".to_string()
    }
    fn primary_key_column() -> String {
        "id".to_string()
    }
}

struct SqlExecutor<'a, D, T>
where
    D: Dao<T>,
    T: Sized + Sync + Serialize + for<'de> Deserialize<'de>,
{
    dao: &'a D,
    _table: PhantomData<T>,
    query_type: Option<String>,
    table: Option<String>,
    columns: Vec<String>,
    set_clauses: Vec<String>,
    values: Vec<String>,
    where_clauses: Vec<String>,
    order_by: Vec<String>,
    group_by: Vec<String>,
    having: Vec<String>,
    joins: Vec<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl<'a, D, T> SqlExecutor<'a, D, T>
where
    D: Dao<T>,
    T: Sized + Sync + Serialize + for<'de> Deserialize<'de>,
{
    /// 创建一个新的 SQL 生成器
    pub fn new(dao: &'a D) -> Self {
        Self {
            dao,
            _table: PhantomData,
            query_type: None,
            table: Some(dao.table()),
            columns: vec![],
            set_clauses: vec![],
            values: vec![],
            where_clauses: vec![],
            order_by: vec![],
            group_by: vec![],
            having: vec![],
            joins: vec![],
            limit: None,
            offset: None,
        }
    }
    pub fn find(mut self) -> Self {
        self.query_type = Some("SELECT".to_string());
        self.columns = vec!["*".to_string()];
        self
    }
    /// 选择表和列
    pub fn select(mut self, columns: &[&str]) -> Self {
        self.query_type = Some("SELECT".to_string());
        self.columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 选择要操作的表
    pub fn from(mut self, table: &str) -> Self {
        self.table = Some(table.to_string());
        self
    }

    /// 设定 WHERE 条件
    pub fn r#where(mut self, condition: &str) -> Self {
        self.where_clauses.push(condition.to_string());
        self
    }

    /// 添加 ORDER BY 语句
    pub fn order_by(mut self, column: &str, desc: bool) -> Self {
        let order = if desc { "DESC" } else { "ASC" };
        self.order_by.push(format!("{} {}", column, order));
        self
    }

    /// 设定 GROUP BY
    pub fn group_by(mut self, column: &str) -> Self {
        // self.group_by = columns.iter().map(|s| s.to_string()).collect();
        self.group_by.push(column.to_string());
        self
    }

    /// 设定 HAVING 条件
    pub fn having(mut self, condition: &str) -> Self {
        self.having.push(condition.to_string());
        self
    }

    /// 添加 JOIN
    pub fn join(mut self, table: &str, on_condition: &str) -> Self {
        self.joins
            .push(format!("JOIN {} ON {}", table, on_condition));
        self
    }

    /// 设置 LIMIT
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// 设置 OFFSET
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn insert(mut self, columns: &[&str]) -> Self {
        self.query_type = Some("INSERT".to_string());

        self.columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 设定 INSERT INTO 语句
    pub fn insert_into(mut self, table: &str, columns: &[&str]) -> Self {
        self.query_type = Some("INSERT".to_string());
        self.table = Some(table.to_string());
        self.columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 设定插入的 VALUES
    pub fn values(mut self, values: &[&str]) -> Self {
        self.values = values.iter().map(|s| format!("'{}'", s)).collect();
        self
    }

    pub fn update(mut self) -> Self {
        self.query_type = Some("UPDATE".to_string());

        self
    }

    /// 设定 UPDATE 语句
    pub fn update_to(mut self, table: &str) -> Self {
        self.query_type = Some("UPDATE".to_string());
        self.table = Some(table.to_string());
        self
    }

    /// 设定 SET 语句
    pub fn set(mut self, column: &str, value: &str) -> Self {
        self.set_clauses.push(format!("{} = '{}'", column, value));
        self
    }
    pub fn delete(mut self) -> Self {
        self.query_type = Some("DELETE".to_string());

        self
    }

    /// 设定 DELETE 语句
    pub fn delete_from(mut self, table: &str) -> Self {
        self.query_type = Some("DELETE".to_string());
        self.table = Some(table.to_string());
        self
    }
    /*
    /// 生成最终的 SQL 语句
    pub fn build(self) -> String {
        match self.query_type.as_deref() {
            Some("SELECT") => {
                let columns = if self.columns.is_empty() {
                    "*".to_string()
                } else {
                    self.columns.join(", ")
                };
                let mut sql = format!("SELECT {} FROM {}", columns, self.table.unwrap());

                if !self.joins.is_empty() {
                    sql.push(' ');
                    sql.push_str(&self.joins.join(" "));
                }
                if !self.where_clauses.is_empty() {
                    sql.push_str(" WHERE ");
                    sql.push_str(&self.where_clauses.join(" AND "));
                }
                if !self.group_by.is_empty() {
                    sql.push_str(" GROUP BY ");
                    sql.push_str(&self.group_by.join(", "));
                }
                if !self.having.is_empty() {
                    sql.push_str(" HAVING ");
                    sql.push_str(&self.having.join(" AND "));
                }
                if !self.order_by.is_empty() {
                    sql.push_str(" ORDER BY ");
                    sql.push_str(&self.order_by.join(", "));
                }
                if let Some(limit) = self.limit {
                    sql.push_str(&format!(" LIMIT {}", limit));
                }
                if let Some(offset) = self.offset {
                    sql.push_str(&format!(" OFFSET {}", offset));
                }

                sql
            }
            Some("INSERT") => format!(
                "INSERT INTO {} ({}) VALUES ({})",
                self.table.unwrap(),
                self.columns.join(", "),
                self.values.join(", ")
            ),
            Some("UPDATE") => {
                let mut sql = format!("UPDATE {} SET {}", self.table.unwrap(), self.set_clauses.join(", "));
                if !self.where_clauses.is_empty() {
                    sql.push_str(" WHERE ");
                    sql.push_str(&self.where_clauses.join(" AND "));
                }
                sql
            }
            Some("DELETE") => {
                let mut sql = format!("DELETE FROM {}", self.table.unwrap());
                if !self.where_clauses.is_empty() {
                    sql.push_str(" WHERE ");
                    sql.push_str(&self.where_clauses.join(" AND "));
                }
                sql
            }
            _ => "INVALID SQL".to_string(),
        }
    }
    */
}
