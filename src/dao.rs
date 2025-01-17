use crate::database::{RelationalDatabase, Value, DbError, Row};
use std::collections::HashMap;


/// 通用的数据访问对象trait
pub trait Dao<T> where T: Sized {
    /// 关联的数据库类型
    type Database: RelationalDatabase;
    
    /// 数据库引用
    fn database(&self) -> &Self::Database;

    fn placeholders(&self, vals: &Vec<Value>) -> Vec<String> {
        self.database().placeholders(vals )
    }

    /// 创建新的 DAO 实例
    fn new(database: Self::Database) -> Self;
    
    fn row_to_entity(row: Row) -> Result<T, DbError>;
    
    /// 将实体对象转换为数据库值
    fn entity_to_values(entity: &T) -> Vec<Value>;
    fn entity_to_map(entity: &T) -> HashMap<String, Value>;
    
    /// 获取表名
    fn table_name() -> String;
    
    /// 获取主键列名
    fn primary_key_column() -> String;
    
    /// 创建新记录
    fn create(&self, entity: &T) -> Result<u64, DbError> {
        let values = Self::entity_to_values(entity);
        // let placeholders: Vec<String> = (1..=values.len())
            // .map(|i| format!("${}", i))
            // .map(|i| "?".to_string() )
            // .collect();
            // let placeholders: Vec<String> =  vec!["?".to_string();values.len()];
            let placeholders: Vec<String> = self.placeholders(&values );
            
        let query = format!(
            "INSERT INTO {} VALUES ({})",
            Self::table_name(),
            placeholders.join(", ")
        );

        
        self.database().execute(&query, values)
    }
    
    /// 根据ID查找记录
    fn find_by_id(&self, id: Value) -> Result<Option<T>, DbError> {
        let placeholder = self.placeholders(&vec![id.clone() ])[0].clone(); 
        let query = format!(
            // "SELECT * FROM {} WHERE {} = ?",
            "SELECT * FROM {} WHERE {} = {}",
            Self::table_name(),
            Self::primary_key_column(),
            placeholder
        );
        
        let result = self.database().query_one(&query, vec![id])?;
        match result {
            Some(row) => Ok(Some(Self::row_to_entity(row)?)),
            None => Ok(None)
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
        let map = Self::entity_to_map(entity.clone());
        let mut values: Vec<Value> = Vec::new();
        // let values: Vec<Value> = self.entity_to_values(entity);
        
        let update_columns: Vec<String> = map.iter()
            .filter(|(k, _)| *k != &Self::primary_key_column())
            .enumerate()
            .map(|(i, (k, v))| {
                let placeholder = self.placeholders(&vec![v.clone();i+1])[i].clone(); 

                values.push(v.clone());
                // format!("{} = ${}", k, i + 1)
                                // format!("{} = ?", k)
                                format!("{} = {}", k, placeholder)
            })
            .collect();
        
        if let Some(id_value) = map.get(&Self::primary_key_column()) {
            values.push(id_value.clone());
        }
        let mut placeholders = self.placeholders(&values);
        
        let query = format!(
            // "UPDATE {} SET {} WHERE {} = ?",
            "UPDATE {} SET {} WHERE {} = {}",
            Self::table_name(),
            update_columns.join(", "),
            Self::primary_key_column(),
            // values.len()
            if let Some(primary_key_placeholder) = placeholders.pop() {
                primary_key_placeholder
            } else {
                panic!("primary_key {}", &Self::primary_key_column());
            }

        );
        
        self.database().execute(&query, values)
    }
    
    /// 删除记录
    fn delete(&self, id: Value) -> Result<u64, DbError> {
        let query = format!(
            "DELETE FROM {} WHERE {} = ?",
            Self::table_name(),
            Self::primary_key_column()
        );
        
        self.database().execute(&query, vec![id])
    }
    
    /// 自定义条件查询
    fn find_by_condition(
        &self,
        condition: &str,
        params: Vec<Value>
    ) -> Result<Vec<T>, DbError> {
        let query = format!(
            "SELECT * FROM {} WHERE {}",
            Self::table_name(),
            condition
        );
        
        let rows = self.database().query(&query, params)?;
        let mut entities = Vec::with_capacity(rows.len());
        for row in rows {
            entities.push(Self::row_to_entity(row)?);
        }
        Ok(entities)
    }
}
