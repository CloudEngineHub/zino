use self::DataSourceConnector::*;
use super::Connector;
use toml::Table;
use zino_core::{Map, Record, bail, error::Error, extension::TomlTableExt};

#[cfg(feature = "connector-arrow")]
use super::ArrowConnector;
#[cfg(feature = "connector-http")]
use super::HttpConnector;
#[cfg(feature = "connector-mysql")]
use sqlx::mysql::MySqlPool;
#[cfg(feature = "connector-postgres")]
use sqlx::postgres::PgPool;
#[cfg(feature = "connector-sqlite")]
use sqlx::sqlite::SqlitePool;

/// Supported data source connectors.
#[non_exhaustive]
pub(super) enum DataSourceConnector {
    /// Apache Arrow
    #[cfg(feature = "connector-arrow")]
    Arrow(ArrowConnector),
    /// HTTP
    #[cfg(feature = "connector-http")]
    Http(HttpConnector),
    /// MySQL
    #[cfg(feature = "connector-mysql")]
    MySql(MySqlPool),
    /// Postgres
    #[cfg(feature = "connector-postgres")]
    Postgres(PgPool),
    /// SQLite
    #[cfg(feature = "connector-sqlite")]
    Sqlite(SqlitePool),
}

/// Data sources.
pub struct DataSource {
    /// Protocol.
    protocol: &'static str,
    /// Data souce type
    source_type: String,
    /// Name
    name: String,
    /// Catalog
    catalog: String,
    /// Connector
    connector: DataSourceConnector,
}

impl DataSource {
    /// Creates a new instance.
    #[inline]
    pub(super) fn new(
        protocol: &'static str,
        source_type: Option<String>,
        name: impl Into<String>,
        catalog: impl Into<String>,
        connector: DataSourceConnector,
    ) -> Self {
        Self {
            protocol,
            source_type: source_type.unwrap_or_else(|| protocol.to_owned()),
            name: name.into(),
            catalog: catalog.into(),
            connector,
        }
    }

    /// Constructs a new instance with the protocol and configuration,
    /// returning an error if it fails.
    ///
    /// Currently, we have built-in support for the following protocols:
    ///
    /// - `arrow`
    /// - `http`
    /// - `mssql`
    /// - `mysql`
    /// - `postgres`
    /// - `sqlite`
    pub fn try_new(protocol: &'static str, config: &Table) -> Result<DataSource, Error> {
        let mut data_source = match protocol {
            #[cfg(feature = "connector-arrow")]
            "arrow" => ArrowConnector::try_new_data_source(config)?,
            #[cfg(feature = "connector-http")]
            "http" => HttpConnector::try_new_data_source(config)?,
            #[cfg(feature = "connector-mysql")]
            "mysql" => MySqlPool::try_new_data_source(config)?,
            #[cfg(feature = "connector-postgres")]
            "postgres" => PgPool::try_new_data_source(config)?,
            #[cfg(feature = "connector-sqlite")]
            "sqlite" => SqlitePool::try_new_data_source(config)?,
            _ => {
                bail!("data source protocol `{}` is unsupported", protocol);
            }
        };
        let source_type = config.get_str("type").unwrap_or(protocol);
        data_source.source_type = source_type.to_owned();
        Ok(data_source)
    }

    /// Returns the protocol.
    #[inline]
    pub fn protocol(&self) -> &'static str {
        self.protocol
    }

    /// Returns the data source type.
    #[inline]
    pub fn source_type(&self) -> &str {
        self.source_type.as_str()
    }

    /// Returns the name.
    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the catalog.
    #[inline]
    pub fn catalog(&self) -> &str {
        self.catalog.as_str()
    }
}

impl Connector for DataSource {
    fn try_new_data_source(config: &Table) -> Result<DataSource, Error> {
        let source_type = config.get_str("type").unwrap_or("unkown");
        let protocol = match source_type {
            "arrow" => "arrow",
            "http" | "rest" | "graphql" => "http",
            "mysql" | "ceresdb" | "databend" | "mariadb" | "tidb" => "mysql",
            "postgres" | "citus" | "greptimedb" | "highgo" | "hologres" | "opengauss"
            | "postgis" | "timescaledb" => "postgres",
            "sqlite" => "sqlite",
            _ => {
                if let Some(protocol) = config.get_str("protocol") {
                    protocol.to_owned().leak()
                } else {
                    bail!("data source type `{}` is unsupported", source_type);
                }
            }
        };
        Self::try_new(protocol, config)
    }

    async fn execute(&self, query: &str, params: Option<&Map>) -> Result<Option<u64>, Error> {
        match &self.connector {
            #[cfg(feature = "connector-arrow")]
            Arrow(connector) => connector.execute(query, params).await,
            #[cfg(feature = "connector-http")]
            Http(connector) => connector.execute(query, params).await,
            #[cfg(feature = "connector-mysql")]
            MySql(pool) => pool.execute(query, params).await,
            #[cfg(feature = "connector-postgres")]
            Postgres(pool) => pool.execute(query, params).await,
            #[cfg(feature = "connector-sqlite")]
            Sqlite(pool) => pool.execute(query, params).await,
        }
    }

    async fn query(&self, query: &str, params: Option<&Map>) -> Result<Vec<Record>, Error> {
        match &self.connector {
            #[cfg(feature = "connector-arrow")]
            Arrow(connector) => connector.query(query, params).await,
            #[cfg(feature = "connector-http")]
            Http(connector) => connector.query(query, params).await,
            #[cfg(feature = "connector-mysql")]
            MySql(pool) => pool.query(query, params).await,
            #[cfg(feature = "connector-postgres")]
            Postgres(pool) => pool.query(query, params).await,
            #[cfg(feature = "connector-sqlite")]
            Sqlite(pool) => pool.query(query, params).await,
        }
    }

    async fn query_one(&self, query: &str, params: Option<&Map>) -> Result<Option<Record>, Error> {
        match &self.connector {
            #[cfg(feature = "connector-arrow")]
            Arrow(connector) => connector.query_one(query, params).await,
            #[cfg(feature = "connector-http")]
            Http(connector) => connector.query_one(query, params).await,
            #[cfg(feature = "connector-mysql")]
            MySql(pool) => pool.query_one(query, params).await,
            #[cfg(feature = "connector-postgres")]
            Postgres(pool) => pool.query_one(query, params).await,
            #[cfg(feature = "connector-sqlite")]
            Sqlite(pool) => pool.query_one(query, params).await,
        }
    }
}
