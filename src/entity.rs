use crate::sql_builder::SqlExecutor;
use crate::asyncdatabase::{DbError, RelationalDatabase, Row, Value};
use crate::serde::{EntityDeserializer,EntityConvertor};
use serde::{de::{Deserialize, DeserializeOwned}, Serialize};
use std::io::Cursor;


pub trait EntityData = 'static + Sized + Sync + Send + Serialize + DeserializeOwned+Clone;

#[async_trait::async_trait]
pub trait Entity: Sized + Sync + Serialize + for<'de> Deserialize<'de>{





    fn row_to_entity<T: EntityData>(row: Row) -> Result<T, DbError> {
        let de = EntityDeserializer::from_value(row.to_table());
        T::deserialize(de).map_err(|e| DbError::ConversionError(e.to_string()))
    }
    
    fn convert_row_to_entity<T: EntityData>(&self,  row: Row) ->Result<T, DbError> {
        Self::row_to_entity(row)
    } 
    
fn convert_rows_to_entitys<T: EntityData>(&self, rows: Vec<Row>) -> Result<Vec<T>, DbError> {
    rows.into_iter()
        .map(|row|
        Self::row_to_entity(row)
        ).collect()
}

    fn entity_to_map<T: EntityData>(entity: &T) -> Vec<(String, Value)> {
        let cursor = Cursor::new(Vec::new());
        let mut convertor = EntityConvertor::new(cursor);
        let result = entity.serialize(&mut convertor);
        match result {
            Ok(Value::Table(table)) => table,
            _ => vec![("".to_string(), Value::Null)],
        }
    }
    
    fn convert_entity_to_table<T: EntityData>(&self, entity: &T) -> Value {
        let map = Self::entity_to_map(entity);
        Value::Table(map)
    }
    
    fn table() -> String;

    fn primary_key() -> String;


    async fn create<T: EntityData, D: RelationalDatabase>(db: &D, entity: &T) -> Result<u64, DbError> {
        let map: Vec<(String, Value)>   = Self::entity_to_map(entity);
        let (keys, values): (Vec<String>, Vec<Value>) = map.into_iter()
        .unzip();

        let placeholders: Vec<String> = db.placeholders(&keys);

        let query = format!(
            "INSERT INTO {} VALUES ({})",
            Self::table(),
            placeholders.join(", ")
        );

        db.execute(&query, values).await
    }
    
    async fn create_without<T: EntityData, D: RelationalDatabase>(
    db: &D,
    entity: &T,
    exclude_fields: Vec<&str>, // 传入要排除的字段
) -> Result<u64, DbError> {
    // 获取实体字段的键值对
    let map: Vec<(String, Value)> = Self::entity_to_map(entity)
        .into_iter()
        // 过滤掉需要排除的字段
        .filter(|(key, _)| !exclude_fields.contains(&key.as_str()))
        .collect();

    // 拆分为字段名和对应的值
    let (keys, values): (Vec<String>, Vec<Value>) = map.into_iter().unzip();

    // 生成 SQL 占位符
    let placeholders: Vec<String> = db.placeholders(&keys);

    // 生成 SQL 语句
    let query = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        Self::table(),
        keys.join(", "), // 仅插入被保留的字段
        placeholders.join(", ")
    );

    // 执行 SQL 语句
    db.execute(&query, values).await
}



    async fn find_by_id<T: EntityData, D: RelationalDatabase>(db: &D, id: Value) -> Result<Option<T>, DbError> {
        let placeholder = db.placeholders(&vec![Self::primary_key()])[0].clone();
        let query = format!(
            "SELECT * FROM {} WHERE {} = {}",
            Self::table(),
            Self::primary_key(),
            placeholder
        );

        let result = db.query_one(&query, vec![id]).await?;
        match result {
            Some(row) => Ok(Some(Self::row_to_entity(row)?)),
            None => Ok(None),
        }
    }


    async fn find_all<T: EntityData, D: RelationalDatabase>(db:& D) -> Result<Vec<T>, DbError> {
        let query = format!("SELECT * FROM {}", Self::table());
        let rows = db.query(&query, vec![]).await?;

        let mut entities = Vec::with_capacity(rows.len());
        for row in rows {
            entities.push(Self::row_to_entity(row)?);
        }
        Ok(entities)
    }


    async fn update<T: EntityData, D: RelationalDatabase>(db: &D, entity: &T) -> Result<u64, DbError> {
        let map: Vec<(String, Value)>   = Self::entity_to_map(entity);
        let mut values: Vec<Value> = Vec::new();


        let mut primary_value = None;
        let update_columns: Vec<String> = map
            .iter()
            .inspect(|kv| {
                if kv.0 == Self::primary_key() {
                    primary_value = Some(kv.1.clone());
                }
            })
            .filter(|kv| kv.0 != Self::primary_key())
            .enumerate()
            .map(|(i, kv)| {
                let placeholder = db.placeholders(&vec![kv.0.clone(); i + 1])[i].clone();

                values.push(kv.1.clone());
                format!("{} = {}", kv.0, placeholder)
            })
            .collect();

        if let Some(id_value) = primary_value {
            values.push(id_value.clone());
        }

        let query = format!(
            "UPDATE {} SET {} WHERE {} = {}",
            Self::table(),
            update_columns.join(", "),
            Self::primary_key(),
            db.placeholders(&vec![Self::primary_key(); values.len()])[values.len() - 1]
                .clone(),
        );

        dbg!(&query);
        db.execute(&query, values).await
    }


    async fn delete<T: EntityData, D: RelationalDatabase>(db: &D, id: Value) -> Result<u64, DbError> {
        let placeholder = db.placeholders(&vec![Self::primary_key()])[0].clone();
        let query = format!(
            "DELETE FROM {} WHERE {} = {}",
            Self::table(),
            Self::primary_key(),
            placeholder
        );

        db.execute(&query, vec![id]).await
    }


    async fn find_by_condition<T: EntityData, D: RelationalDatabase>(
        db: &D,
        condition: Vec<&str>,
        params: Vec<Value>,
    ) -> Result<Vec<T>, DbError> {
        let  conditions: Vec<String>  = condition.iter().map(|s| s.to_string()).collect();
        let placeholders = db.placeholders(&conditions);
        let where_condition: String =  conditions.iter()
        .enumerate()
        .map(|(i, c)| format!("{} {}", c,  placeholders[i]))
        .collect::<Vec<String>>()
        .join(" AND ");
        let query = format!("SELECT * FROM {} WHERE {}", Self::table(), where_condition);

        let rows = db.query(&query, params).await?;
        let mut entities = Vec::with_capacity(rows.len());
        for row in rows {
            entities.push(Self::row_to_entity(row)?);
        }
        Ok(entities)
    }

    async fn begin_transaction<D: RelationalDatabase>(db: &D) -> Result<(), DbError> {
        db.begin_transaction().await
    }

    async fn commit<D: RelationalDatabase>(db: &D) -> Result<(), DbError> {
        db.commit().await
    }

    async fn rollback<D: RelationalDatabase>(db: &D) -> Result<(), DbError> {
        db.rollback().await
    }

    fn prepare<D: RelationalDatabase, T: EntityData>(db: &D) -> SqlExecutor<D, T> {
        SqlExecutor::new(&db, Self::table())
}


}

