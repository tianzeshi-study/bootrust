use crate::database::{DbError, RelationalDatabase, Row, Value};

/// 通用的数据访问对象trait
pub trait Dao<T>
where
    T: Sized,
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

    fn row_to_entity(row: Row) -> Result<T, DbError>;

    fn entity_to_map(entity: &T) -> Vec<(String, Value)>;

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

    /// 获取主键列名
    fn primary_key_column() -> String;

    /// 创建新记录
    fn create(&self, entity: &T) -> Result<u64, DbError> {
        let values = self.entity_to_values(entity);
        let keys = self.entity_to_keys(entity);
        // let placeholders: Vec<String> = (1..=values.len())
        // .map(|i| format!("${}", i))
        // .map(|i| "?".to_string() )
        // .collect();

        let placeholders: Vec<String> = self.placeholders(&keys);

        let query = format!(
            "INSERT INTO {} VALUES ({})",
            Self::table_name(),
            placeholders.join(", ")
        );

        self.database().execute(&query, values)
    }

    /// 根据ID查找记录
    fn find_by_id(&self, id: Value) -> Result<Option<T>, DbError> {

        let placeholder = self.placeholders(&vec![Self::primary_key_column()])[0].clone();
        let query = format!(

            "SELECT * FROM {} WHERE {} = {}",
            Self::table_name(),
            Self::primary_key_column(),
            placeholder
        );

        let result = self.database().query_one(&query, vec![id])?;
        match result {
            Some(row) => Ok(Some(Self::row_to_entity(row)?)),
            None => Ok(None),
        }
    }

    /// 查找所有记录
    fn find_all(&self) -> Result<Vec<T>, DbError> {
        let query = format!("SELECT * FROM {}", Self::table_name());
        let rows = self.database().query(&query, vec![])?;

        let mut entities = Vec::with_capacity(rows.len());
        for row in rows {
            entities.push(Self::row_to_entity(row)?);
        }
        Ok(entities)
    }

    /// 更新记录
    fn update(&self, entity: &T) -> Result<u64, DbError> {

        let map = Self::entity_to_map(entity);
        let mut values: Vec<Value> = Vec::new();

        let mut primary_value = None;
        let update_columns: Vec<String> = map
            .iter()
            .map(|kv| {
                if kv.0 == Self::primary_key_column() {
                    primary_value = Some(kv.1.clone());
                }
                kv
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
        self.database().execute(&query, values)
    }

    /// 删除记录
    fn delete(&self, id: Value) -> Result<u64, DbError> {
        let placeholder = self.placeholders(&vec![Self::primary_key_column()])[0].clone();
        let query = format!(
            "DELETE FROM {} WHERE {} = {}",
            Self::table_name(),
            Self::primary_key_column(),
            placeholder
        );

        self.database().execute(&query, vec![id])
    }

    /// 自定义条件查询
    fn find_by_condition(&self, condition: &str, params: Vec<Value>) -> Result<Vec<T>, DbError> {
        let query = format!("SELECT * FROM {} WHERE {}", Self::table_name(), condition);

        let rows = self.database().query(&query, params)?;
        let mut entities = Vec::with_capacity(rows.len());
        for row in rows {
            entities.push(Self::row_to_entity(row)?);
        }
        Ok(entities)
    }

    fn begin_transaction(&self) -> Result<(), DbError> {
        self.database().begin_transaction()
    }

    fn commit(&self) -> Result<(), DbError>{
        self.database().commit()
    }

    fn rollback(&self) -> Result<(), DbError>{
        self.database().rollback()
    }
}
