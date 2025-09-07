use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

pub enum DbKind {
    Mssql,
    Postgres,
}

pub enum DatabaseRef {
    Mssql(Arc<Mutex<tiberius::Client<Compat<TcpStream>>>>),
    Postgres(Arc<tokio_postgres::Client>),
}

impl DatabaseRef {
    pub fn kind(&self) -> DbKind {
        match self {
            DatabaseRef::Mssql(_) => DbKind::Mssql,
            DatabaseRef::Postgres(_) => DbKind::Postgres,
        }
    }
}

pub async fn connect_mssql(
    host: &str,
    port: u16,
    db: &str,
    user: &str,
    pass: &str,
) -> Result<DatabaseRef> {
    let mut config = tiberius::Config::new();
    config.host(host);
    config.port(port);
    config.database(db);
    config.authentication(tiberius::AuthMethod::sql_server(user, pass));

    let tcp = TcpStream::connect((host, port)).await?;
    tcp.set_nodelay(true)?;
    let client = tiberius::Client::connect(config, tcp.compat_write()).await?;
    Ok(DatabaseRef::Mssql(Arc::new(Mutex::new(client))))
}

pub async fn connect_postgres(
    host: &str,
    port: u16,
    db: &str,
    user: &str,
    pass: &str,
) -> Result<DatabaseRef> {
    let config = format!(
        "host={} port={} dbname={} user={} password={}",
        host, port, db, user, pass
    );
    let (client, connection) = tokio_postgres::connect(&config, tokio_postgres::NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("postgres connection error: {}", e);
        }
    });
    Ok(DatabaseRef::Postgres(Arc::new(client)))
}
