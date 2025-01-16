use crate::database::{RelationalDatabase, Value, DbError, Row};


/// 通用的数据访问对象trait
pub trait Dao<T> where T: Sized {
    /// 关联的数据库类型
    type Database: RelationalDatabase;
    
    /// 将数据库行转换为实体对象
    fn row_to_entity(row: Row) -> Result<T, DbError>;
    
    /// 将实体对象转换为数据库值
    fn entity_to_values(entity: &T) -> Vec<Value>;
    
    /// 获取表名
    fn table_name() -> String;
    
    /// 获取主键列名
    fn primary_key_column() -> String;
    
    /// 创建新记录
    fn create(&self, db: &Self::Database, entity: &T) -> Result<u64, DbError> {
        let values = Self::entity_to_values(entity);
        let placeholders: Vec<String> = (1..=values.len())
            .map(|i| format!("${}", i))
            .collect();
            
        let query = format!(
            "INSERT INTO {} VALUES ({})",
            Self::table_name(),
            placeholders.join(", ")
        );
        
        db.execute(&query, values)
    }
    
    /// 根据ID查找记录
    fn find_by_id(&self, db: &Self::Database, id: Value) -> Result<Option<T>, DbError> {
        let query = format!(
            "SELECT * FROM {} WHERE {} = $1",
            Self::table_name(),
            Self::primary_key_column()
        );
        
        let result = db.query_one(&query, vec![id])?;
        match result {
            Some(row) => Ok(Some(Self::row_to_entity(row)?)),
            None => Ok(None)
        }
    }
    
    /// 查找所有记录
    fn find_all(&self, db: &Self::Database) -> Result<Vec<T>, DbError> {
        let query = format!("SELECT * FROM {}", Self::table_name());
        let rows = db.query(&query, vec![])?;
        
        let mut entities = Vec::with_capacity(rows.len());
        for row in rows {
            entities.push(Self::row_to_entity(row)?);
        }
        Ok(entities)
    }
    
    /// 更新记录
    fn update(&self, db: &Self::Database, entity: &T) -> Result<u64, DbError> {
        let values = Self::entity_to_values(entity);
        let update_columns: Vec<String> = (1..values.len())
            .map(|i| format!("{} = ${}", Self::primary_key_column(), i))
            .collect();
            
        let query = format!(
            "UPDATE {} SET {} WHERE {} = ${}",
            Self::table_name(),
            update_columns.join(", "),
            Self::primary_key_column(),
            values.len()
        );
        
        db.execute(&query, values)
    }
    
    /// 删除记录
    fn delete(&self, db: &Self::Database, id: Value) -> Result<u64, DbError> {
        let query = format!(
            "DELETE FROM {} WHERE {} = $1",
            Self::table_name(),
            Self::primary_key_column()
        );
        
        db.execute(&query, vec![id])
    }
    
    /// 自定义条件查询
    fn find_by_condition(
        &self,
        db: &Self::Database,
        condition: &str,
        params: Vec<Value>
    ) -> Result<Vec<T>, DbError> {
        let query = format!(
            "SELECT * FROM {} WHERE {}",
            Self::table_name(),
            condition
        );
        
        let rows = db.query(&query, params)?;
        let mut entities = Vec::with_capacity(rows.len());
        for row in rows {
            entities.push(Self::row_to_entity(row)?);
        }
        Ok(entities)
    }
}
