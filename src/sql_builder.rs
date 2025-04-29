use crate::asyncdatabase::{DbError, RelationalDatabase, Row, Value};
use crate::serde::EntityDeserializer;
use serde::{de::Deserialize, ser::Serialize};
use std::marker::PhantomData;

pub struct SqlExecutor<'a, D, T>
where
    D: RelationalDatabase,
    T: Sized + Sync + Serialize + for<'de> Deserialize<'de>,
{
    database: &'a D,
    _table: PhantomData<T>,
    query_type: Option<String>,
    table: Option<String>,
    columns: Vec<String>,
    set_clauses: Vec<String>,
    values: Vec<Value>,
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
    D: RelationalDatabase,
    T: Sized + Sync + Serialize + for<'de> Deserialize<'de>,
{
    /// 创建一个新的 SQL 生成器
    pub fn new(database: &'a D, tablename: String) -> Self {
        Self {
            database,
            _table: PhantomData,
            query_type: None,
            table: Some(tablename),
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
    pub fn where_clauses(mut self, condition: Vec<&str>) -> Self {
        match self.query_type.as_deref() {
            Some("UPDATE") => {
                let conditions: Vec<String> = condition.iter().map(|s| s.to_string()).collect();
                let total: Vec<String> = self
                    .set_clauses
                    .iter()
                    .cloned()
                    .chain(conditions.iter().cloned())
                    .collect();
                let placeholders = self.database.placeholders(&total);
                let where_clauses = conditions
                    .iter()
                    .enumerate()
                    .map(|(i, c)| format!("{} {}", c, placeholders[conditions.len() + i]))
                    .collect::<Vec<String>>();

                self.where_clauses = where_clauses;
                self
            }
            _ => {
                let conditions: Vec<String> = condition.iter().map(|s| s.to_string()).collect();
                let placeholders = self.database.placeholders(&conditions);
                let where_conditions: Vec<String> = conditions
                    .iter()
                    .enumerate()
                    .map(|(i, c)| format!("{} {}", c, placeholders[i]))
                    .collect::<Vec<String>>();

                self.where_clauses = where_conditions;
                self
            }
        }
    }

    /// 添加 ORDER BY 语句
    pub fn order_by(mut self, conditions: Vec<&str>) -> Self {
        self.order_by = conditions.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 设定 GROUP BY
    pub fn group_by(mut self, columns: Vec<&str>) -> Self {
        self.group_by = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 设定 HAVING 条件
    pub fn having(mut self, conditions: Vec<&str>) -> Self {
        let conditions: Vec<String> = conditions.iter().map(|s| s.to_string()).collect();
        let total: Vec<String> = self
            .where_clauses
            .iter()
            .cloned()
            .chain(conditions.iter().cloned())
            .collect();
        let placeholders = self.database.placeholders(&total);
        let having_condition = conditions
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{} {}", c, placeholders[self.where_clauses.len() + i]))
            .collect::<Vec<String>>();

        self.having = having_condition;

        self
    }

    /// 添加 JOIN
    pub fn join(mut self, table: &str, on_condition: &str) -> Self {
        self.joins
            .push(format!("JOIN {} ON {}", table, on_condition));
        self
    }

    pub fn left_join(mut self, table: &str, on_condition: &str) -> Self {
        self.joins
            .push(format!("LEFT JOIN {} ON {}", table, on_condition));
        self
    }

    pub fn cross_join(mut self, table: &str) -> Self {
        self.joins.push(format!("CROSS JOIN {} ", table));
        self
    }

    pub fn natural_join(mut self, table: &str) -> Self {
        self.joins.push(format!("NATURAL JOIN {} ", table));
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

    /// 设定插入的 VALUES
    pub fn values(mut self, values: Vec<impl Into<Value>>) -> Self {
        self.values = values.into_iter().map(|v| v.into()).collect();
        self
    }

    pub fn update(mut self, columns: &[&str]) -> Self {
        self.query_type = Some("UPDATE".to_string());
        let placeholders = self.database.placeholders(
            &columns
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
        );

        let set_clauses: Vec<String> = columns
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{} = {}", c, placeholders[i]))
            .collect::<Vec<String>>();
        self.set_clauses = set_clauses;

        // self.set_clauses = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn delete(mut self) -> Self {
        self.query_type = Some("DELETE".to_string());

        self
    }

    /// 生成最终的 SQL 语句
    pub async fn query(self) -> Result<Vec<T>, DbError> {
        let mut sql = String::new();

        match self.query_type.as_deref() {
            Some("SELECT") => {
                sql.push_str("SELECT ");
                sql.push_str(&self.columns.join(", "));
                sql.push_str(" FROM ");
                sql.push_str(&self.table.unwrap());

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
            }

            Some("INSERT") => {
                sql.push_str("INSERT INTO ");
                sql.push_str(&self.table.unwrap());
                sql.push_str(" (");
                sql.push_str(&self.columns.join(", "));
                sql.push_str(") VALUES (");
                let placeholders = self.database.placeholders(
                    &self
                        .columns
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>(),
                );
                sql.push_str(&placeholders.join(", "));
                // sql.push_str(&self.values.join(", "));
                sql.push(')');
            }
            Some("UPDATE") => {
                sql.push_str("UPDATE ");
                sql.push_str(&self.table.unwrap());
                sql.push_str(" SET ");
                sql.push_str(&self.set_clauses.join(", "));
                if !self.where_clauses.is_empty() {
                    sql.push_str(" WHERE ");
                    sql.push_str(&self.where_clauses.join(" AND "));
                }
            }
            Some("DELETE") => {
                sql.push_str("DELETE FROM ");
                sql.push_str(&self.table.unwrap());
                if !self.where_clauses.is_empty() {
                    sql.push_str(" WHERE ");
                    sql.push_str(&self.where_clauses.join(" AND "));
                }
            }

            _ => {}
        }
        dbg!(&sql);
        let rows: Vec<Row> = self.database.query(&sql, self.values).await?;

        // self.dao.convert_rows_to_entitys(rows);
        rows.iter()
            .map(|row| {
                let de = EntityDeserializer::from_value(row.to_table());
                T::deserialize(de).map_err(|e| DbError::ConversionError(e.to_string()))
            })
            .collect()
    }

    pub async fn execute(self) -> Result<u64, DbError> {
        let mut sql = String::new();

        match self.query_type.as_deref() {
            Some("SELECT") => {
                sql.push_str("SELECT ");
                sql.push_str(&self.columns.join(", "));
                sql.push_str(" FROM ");
                sql.push_str(&self.table.unwrap());

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
            }

            Some("INSERT") => {
                sql.push_str("INSERT INTO ");
                sql.push_str(&self.table.unwrap());
                sql.push_str(" (");
                sql.push_str(&self.columns.join(", "));
                sql.push_str(") VALUES (");
                let placeholders = self.database.placeholders(
                    &self
                        .columns
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>(),
                );
                sql.push_str(&placeholders.join(", "));
                // sql.push_str(&self.values.join(", "));
                sql.push(')');
            }
            Some("UPDATE") => {
                sql.push_str("UPDATE ");
                sql.push_str(&self.table.unwrap());
                sql.push_str(" SET ");
                sql.push_str(&self.set_clauses.join(", "));
                if !self.where_clauses.is_empty() {
                    sql.push_str(" WHERE ");
                    sql.push_str(&self.where_clauses.join(" AND "));
                }
            }
            Some("DELETE") => {
                sql.push_str("DELETE FROM ");
                sql.push_str(&self.table.unwrap());
                if !self.where_clauses.is_empty() {
                    sql.push_str(" WHERE ");
                    sql.push_str(&self.where_clauses.join(" AND "));
                }
            }

            _ => {}
        }
        dbg!(&sql);
        self.database.execute(&sql, self.values).await
    }
}
