use anyhow::Result;
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_postgres::NoTls;
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
    config.trust_cert();

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
    let base = format!(
        "host={} port={} dbname={} user={} password={}",
        host, port, db, user, pass
    );

    let builder = TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()?;
    let connector = MakeTlsConnector::new(builder);
    let tls_config = format!("{} sslmode=require", base);

    match tokio_postgres::connect(&tls_config, connector).await {
        Ok((client, connection)) => {
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("postgres connection error: {}", e);
                }
            });
            Ok(DatabaseRef::Postgres(Arc::new(client)))
        }
        Err(e) if e.to_string().contains("server does not support TLS") => {
            let plain_config = format!("{} sslmode=disable", base);
            let (client, connection) = tokio_postgres::connect(&plain_config, NoTls).await?;
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("postgres connection error: {}", e);
                }
            });
            Ok(DatabaseRef::Postgres(Arc::new(client)))
        }
        Err(e) => Err(e.into()),
    }
}
