use super::ArrowArrayExt;
use datafusion::{arrow::util, dataframe::DataFrame};
use serde::de::DeserializeOwned;
use zino_core::{Map, Record, error::Error};

/// Executor trait for [`DataFrame`](datafusion::dataframe::DataFrame).
pub trait DataFrameExecutor {
    /// Executes the `DataFrame` and returns the total number of rows affected.
    async fn execute(self) -> Result<Option<u64>, Error>;

    /// Executes the `DataFrame` and parses it as `Vec<Record>`.
    async fn query(self) -> Result<Vec<Record>, Error>;

    /// Executes the `DataFrame` and parses it as `Vec<T>`.
    async fn query_as<T: DeserializeOwned>(self) -> Result<Vec<T>, Error>;

    /// Executes the `DataFrame` and parses it as a `Record`.
    async fn query_one(self) -> Result<Option<Record>, Error>;

    /// Executes the `DataFrame` and parses it as an instance of type `T`.
    async fn query_one_as<T: DeserializeOwned>(self) -> Result<Option<T>, Error>;

    /// Executes the `DataFrame` and creates a visual representation of record batches.
    async fn output(self) -> Result<String, Error>;
}

impl DataFrameExecutor for DataFrame {
    async fn execute(self) -> Result<Option<u64>, Error> {
        self.collect().await?;
        Ok(None)
    }

    async fn query(self) -> Result<Vec<Record>, Error> {
        let batches = self.collect().await?;
        let mut records = Vec::new();
        let mut max_rows = 0;
        for batch in batches {
            let num_rows = batch.num_rows();
            if num_rows > max_rows {
                records.resize_with(num_rows, Record::new);
                max_rows = num_rows;
            }
            for field in &batch.schema().fields {
                let field_name = field.name().as_str();
                if let Some(array) = batch.column_by_name(field_name) {
                    for (index, record) in records.iter_mut().enumerate() {
                        let value = array.parse_avro_value(index)?;
                        record.push((field_name.to_owned(), value));
                    }
                }
            }
        }
        Ok(records)
    }

    async fn query_as<T: DeserializeOwned>(self) -> Result<Vec<T>, Error> {
        let batches = self.collect().await?;
        let mut data = Vec::new();
        let mut max_rows = 0;
        for batch in batches {
            let num_rows = batch.num_rows();
            if num_rows > max_rows {
                data.resize_with(num_rows, Map::new);
                max_rows = num_rows;
            }
            for field in &batch.schema().fields {
                let field_name = field.name().as_str();
                if let Some(array) = batch.column_by_name(field_name) {
                    for (index, map) in data.iter_mut().enumerate() {
                        let value = array.parse_json_value(index)?;
                        map.insert(field_name.to_owned(), value);
                    }
                }
            }
        }
        serde_json::from_value(data.into()).map_err(Error::from)
    }

    async fn query_one(self) -> Result<Option<Record>, Error> {
        let batches = self.limit(0, Some(1))?.collect().await?;
        let mut record = Record::new();
        for batch in batches {
            for field in &batch.schema().fields {
                let field_name = field.name().as_str();
                if let Some(array) = batch.column_by_name(field_name) {
                    let value = array.parse_avro_value(0)?;
                    record.push((field_name.to_owned(), value));
                }
            }
        }
        Ok(Some(record))
    }

    async fn query_one_as<T: DeserializeOwned>(self) -> Result<Option<T>, Error> {
        let batches = self.limit(0, Some(1))?.collect().await?;
        let mut map = Map::new();
        for batch in batches {
            for field in &batch.schema().fields {
                let field_name = field.name().as_str();
                if let Some(array) = batch.column_by_name(field_name) {
                    let value = array.parse_json_value(0)?;
                    map.insert(field_name.to_owned(), value);
                }
            }
        }
        serde_json::from_value(map.into()).map_err(Error::from)
    }

    async fn output(self) -> Result<String, Error> {
        let batches = self.collect().await?;
        let data = util::pretty::pretty_format_batches(&batches)?;
        Ok(data.to_string())
    }
}
