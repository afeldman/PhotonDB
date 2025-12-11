//! RethinkDB 3.0 Server Binary
//!
//! Complete command-line interface for RethinkDB 3.0 with support for:
//! - Server management (start, stop, status)
//! - Database operations (create, drop, list)
//! - Table operations (create, drop, list)
//! - Data import/export
//! - Cluster administration
//!
//! # Examples
//!
//! ```bash
//! # Start server
//! rethinkdb serve --bind 0.0.0.0 --port 8080
//!
//! # Create database
//! rethinkdb admin create-db myapp
//!
//! # List databases
//! rethinkdb admin list-dbs
//!
//! # Export data
//! rethinkdb export --db myapp --output backup.json
//! ```

use clap::{Args, Parser, Subcommand};
use rethinkdb::server::{start_server, SecurityConfig, ServerConfig};
use rethinkdb::storage::{DefaultStorageEngine, StorageEngine};
use rethinkdb::Storage;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, warn};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// RethinkDB 3.0 - The Scientific Computing Database
#[derive(Parser, Debug)]
#[command(name = "rethinkdb")]
#[command(version = rethinkdb::VERSION)]
#[command(about = "RethinkDB 3.0 - The Scientific Computing Database", long_about = None)]
#[command(author = "Anton Feldmann <afeldman@lynqtech.com>")]
struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,

    /// Data directory path
    #[arg(
        long,
        global = true,
        default_value = "data/rethinkdb",
        env = "RETHINKDB_DATA"
    )]
    data_dir: PathBuf,

    /// Log directory path
    #[arg(long, global = true, default_value = "logs", env = "RETHINKDB_LOG_DIR")]
    log_dir: PathBuf,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, global = true, default_value = "info", env = "RUST_LOG")]
    log_level: String,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the RethinkDB server
    Serve(ServeArgs),

    /// Administrative commands
    Admin {
        #[command(subcommand)]
        command: AdminCommands,
    },

    /// Database operations
    Db {
        #[command(subcommand)]
        command: DbCommands,
    },

    /// Table operations
    Table {
        #[command(subcommand)]
        command: TableCommands,
    },

    /// Export data from database/table
    Export(ExportArgs),

    /// Import data into database/table
    Import(ImportArgs),

    /// Show server status
    Status,

    /// Show server version
    Version,
}

/// Server configuration arguments
#[derive(Args, Debug)]
struct ServeArgs {
    /// HTTP bind address
    #[arg(short, long, default_value = "127.0.0.1", env = "RETHINKDB_BIND")]
    bind: String,

    /// HTTP port
    #[arg(short, long, default_value = "8080", env = "RETHINKDB_PORT")]
    port: u16,

    /// Enable CORS
    #[arg(long, default_value = "true")]
    cors: bool,

    /// Disable security (development mode)
    #[arg(long)]
    dev_mode: bool,

    /// Request timeout in seconds
    #[arg(long, default_value = "30")]
    timeout: u64,

    /// Maximum request body size (MB)
    #[arg(long, default_value = "10")]
    max_body_size: usize,
}

/// Administrative commands
#[derive(Subcommand, Debug)]
enum AdminCommands {
    /// List all databases
    ListDbs,

    /// Create a new database
    CreateDb {
        /// Database name
        name: String,
    },

    /// Drop (delete) a database
    DropDb {
        /// Database name
        name: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Show database information
    DbInfo {
        /// Database name
        name: String,
    },

    /// Compact database storage
    Compact,

    /// Show storage statistics
    Stats,
}

/// Database commands
#[derive(Subcommand, Debug)]
enum DbCommands {
    /// Create a new database
    Create {
        /// Database name
        name: String,
    },

    /// Drop a database
    Drop {
        /// Database name
        name: String,
        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// List all databases
    List,

    /// Show database info
    Info {
        /// Database name
        name: String,
    },
}

/// Table commands
#[derive(Subcommand, Debug)]
enum TableCommands {
    /// Create a new table
    Create {
        /// Database name
        #[arg(short, long)]
        db: String,
        /// Table name
        name: String,
        /// Primary key field (default: "id")
        #[arg(short, long)]
        primary_key: Option<String>,
    },

    /// Drop a table
    Drop {
        /// Database name
        #[arg(short, long)]
        db: String,
        /// Table name
        name: String,
        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// List all tables in a database
    List {
        /// Database name
        #[arg(short, long)]
        db: String,
    },

    /// Show table info
    Info {
        /// Database name
        #[arg(short, long)]
        db: String,
        /// Table name
        name: String,
    },
}

/// Export arguments
#[derive(Args, Debug)]
struct ExportArgs {
    /// Database name
    #[arg(short, long)]
    db: String,

    /// Table name (optional, exports all tables if not specified)
    #[arg(short, long)]
    table: Option<String>,

    /// Output file path
    #[arg(short, long)]
    output: PathBuf,

    /// Export format (json, csv)
    #[arg(short, long, default_value = "json")]
    format: String,
}

/// Import arguments
#[derive(Args, Debug)]
struct ImportArgs {
    /// Database name
    #[arg(short, long)]
    db: String,

    /// Table name
    #[arg(short, long)]
    table: String,

    /// Input file path
    #[arg(short, long)]
    input: PathBuf,

    /// Import format (json, csv)
    #[arg(short, long, default_value = "json")]
    format: String,

    /// Skip existing documents
    #[arg(long)]
    skip_existing: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Setup logging
    setup_logging(&cli)?;

    // Execute command
    match cli.command {
        Commands::Serve(args) => serve_command(cli.data_dir, args).await,
        Commands::Admin { command } => admin_command(cli.data_dir, command).await,
        Commands::Db { command } => db_command(cli.data_dir, command).await,
        Commands::Table { command } => table_command(cli.data_dir, command).await,
        Commands::Export(args) => export_command(cli.data_dir, args).await,
        Commands::Import(args) => import_command(cli.data_dir, args).await,
        Commands::Status => status_command(cli.data_dir).await,
        Commands::Version => {
            println!("RethinkDB {}", rethinkdb::VERSION);
            println!("Rust implementation by Anton Feldmann");
            Ok(())
        }
    }
}

/// Setup logging with rolling files and console output
fn setup_logging(cli: &Cli) -> anyhow::Result<()> {
    std::fs::create_dir_all(&cli.log_dir)?;

    let file_appender = RollingFileAppender::new(Rotation::DAILY, &cli.log_dir, "rethinkdb.log");

    let log_level = cli
        .log_level
        .parse::<tracing::Level>()
        .unwrap_or(tracing::Level::INFO);

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(std::io::stdout)
                .with_ansi(!cli.no_color)
                .pretty(),
        )
        .with(fmt::layer().with_writer(file_appender).with_ansi(false))
        .with(EnvFilter::from_default_env().add_directive(log_level.into()))
        .init();

    Ok(())
}

/// Serve command - start the RethinkDB server
async fn serve_command(data_dir: PathBuf, args: ServeArgs) -> anyhow::Result<()> {
    info!("🚀 RethinkDB 3.0 starting...");
    info!(version = %rethinkdb::VERSION, "Version information");

    // Initialize storage
    let storage_engine = DefaultStorageEngine::with_defaults(data_dir.to_str().unwrap())?;
    let storage = Arc::new(Storage::new(Box::new(storage_engine)));
    info!("✅ Storage initialized at {}", data_dir.display());

    // Security configuration
    let security_config = if !args.dev_mode {
        info!("🔒 Production mode: Security enabled");
        Some(SecurityConfig::default())
    } else {
        warn!("⚠️  Development mode: Security disabled");
        None
    };

    // Server configuration
    let server_config = ServerConfig {
        http_addr: args.bind.clone(),
        http_port: args.port,
        enable_cors: args.cors,
        timeout_secs: args.timeout,
        max_body_size: args.max_body_size * 1024 * 1024,
    };

    info!("🌐 HTTP API starting on {}:{}", args.bind, args.port);

    // Start TCP protocol server (port 28015)
    let tcp_storage = storage.clone();
    let tcp_handle = tokio::spawn(async move {
        use rethinkdb::network::{ProtocolServer, ServerConfig as TcpConfig};
        
        let tcp_config = TcpConfig {
            bind_addr: "0.0.0.0:28015".parse().unwrap(),
            max_connections: 1024,
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
        };
        
        let tcp_server = ProtocolServer::new(tcp_config, tcp_storage);
        info!("🔌 TCP protocol server starting on port 28015");
        
        if let Err(e) = tcp_server.serve().await {
            error!("TCP server error: {}", e);
        }
    });

    // Start QUIC protocol server (port 28016) if feature enabled
    #[cfg(feature = "quic")]
    let quic_handle = {
        let quic_storage = storage.clone();
        tokio::spawn(async move {
            use rethinkdb::network::{QuicProtocolServer, QuicServerConfig};
            
            let quic_config = QuicServerConfig {
                bind_addr: "0.0.0.0:28016".parse().unwrap(),
                max_connections: 1024,
                cert_path: None,
                key_path: None,
                auto_cert: true,
            };
            
            let quic_server = QuicProtocolServer::new(quic_config, quic_storage);
            info!("⚡ QUIC protocol server starting on port 28016");
            
            if let Err(e) = quic_server.serve().await {
                error!("QUIC server error: {}", e);
            }
        })
    };

    // Start HTTP server
    let http_result = start_server(server_config, storage, security_config).await;

    // Wait for protocol servers
    #[cfg(feature = "quic")]
    {
        tokio::select! {
            _ = tcp_handle => {},
            _ = quic_handle => {},
        }
    }
    #[cfg(not(feature = "quic"))]
    {
        tcp_handle.await?;
    }

    http_result
}

/// Administrative commands
async fn admin_command(data_dir: PathBuf, command: AdminCommands) -> anyhow::Result<()> {
    let engine = DefaultStorageEngine::with_defaults(data_dir.to_str().unwrap())?;

    match command {
        AdminCommands::ListDbs => {
            info!("Listing all databases...");
            let dbs = engine.list_databases().await?;
            if dbs.is_empty() {
                println!("No databases found.");
            } else {
                println!("Databases ({})", dbs.len());
                println!("───────────────────────────────");
                for db in dbs {
                    println!("  • {}", db);
                }
            }
            Ok(())
        }
        AdminCommands::CreateDb { name } => {
            info!(db = %name, "Creating database...");
            engine.create_database(&name).await?;
            println!("✅ Database '{}' created", name);
            Ok(())
        }
        AdminCommands::DropDb { name, force } => {
            if !force {
                print!(
                    "Are you sure you want to drop database '{}'? (yes/no): ",
                    name
                );
                use std::io::{self, Write};
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if input.trim().to_lowercase() != "yes" {
                    println!("Aborted.");
                    return Ok(());
                }
            }
            info!(db = %name, "Dropping database...");
            engine.drop_database(&name).await?;
            println!("✅ Database '{}' dropped", name);
            Ok(())
        }
        AdminCommands::DbInfo { name } => {
            info!(db = %name, "Getting database info...");
            // Check if database exists by listing all databases
            let dbs = engine.list_databases().await?;
            if dbs.contains(&name) {
                println!("Database: {}", name);
                println!("ID: db_{}", name);
                // Count tables in database
                let tables = engine.list_tables_in_db(&name).await?;
                println!("Tables: {}", tables.len());
            } else {
                println!("❌ Database '{}' not found", name);
            }
            Ok(())
        }
        AdminCommands::Compact => {
            info!("Compacting storage...");
            // TODO: Implement compaction
            println!("⚠️  Compaction not yet implemented");
            Ok(())
        }
        AdminCommands::Stats => {
            info!("Getting storage statistics...");
            // TODO: Implement stats
            println!("⚠️  Statistics not yet implemented");
            Ok(())
        }
    }
}

/// Database commands
async fn db_command(data_dir: PathBuf, command: DbCommands) -> anyhow::Result<()> {
    let engine = DefaultStorageEngine::with_defaults(data_dir.to_str().unwrap())?;

    match command {
        DbCommands::Create { name } => {
            engine.create_database(&name).await?;
            println!("✅ Created database '{}'", name);
            Ok(())
        }
        DbCommands::Drop { name, force } => {
            if !force {
                print!("Drop database '{}'? (yes/no): ", name);
                use std::io::{self, Write};
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if input.trim().to_lowercase() != "yes" {
                    return Ok(());
                }
            }
            engine.drop_database(&name).await?;
            println!("✅ Dropped database '{}'", name);
            Ok(())
        }
        DbCommands::List => {
            let dbs = engine.list_databases().await?;
            if dbs.is_empty() {
                println!("No databases.");
            } else {
                for db in dbs {
                    println!("{}", db);
                }
            }
            Ok(())
        }
        DbCommands::Info { name } => {
            let dbs = engine.list_databases().await?;
            if dbs.contains(&name) {
                println!("Name: {}", name);
                println!("ID: db_{}", name);
                let tables = engine.list_tables_in_db(&name).await?;
                println!("Tables: {}", tables.len());
            } else {
                println!("Database not found");
            }
            Ok(())
        }
    }
}

/// Table commands
async fn table_command(data_dir: PathBuf, command: TableCommands) -> anyhow::Result<()> {
    let engine = DefaultStorageEngine::with_defaults(data_dir.to_str().unwrap())?;

    match command {
        TableCommands::Create {
            db,
            name,
            primary_key,
        } => {
            let pk = primary_key.as_deref().unwrap_or("id");
            engine.create_table(&db, &name, pk).await?;
            println!("✅ Created table '{}.{}'", db, name);
            Ok(())
        }
        TableCommands::Drop { db, name, force } => {
            if !force {
                print!("Drop table '{}.{}'? (yes/no): ", db, name);
                use std::io::{self, Write};
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if input.trim().to_lowercase() != "yes" {
                    return Ok(());
                }
            }
            engine.drop_table(&db, &name).await?;
            println!("✅ Dropped table '{}.{}'", db, name);
            Ok(())
        }
        TableCommands::List { db } => {
            let tables = engine.list_tables_in_db(&db).await?;
            if tables.is_empty() {
                println!("No tables in database '{}'", db);
            } else {
                println!("Tables in '{}'", db);
                for table in tables {
                    println!("  {}", table);
                }
            }
            Ok(())
        }
        TableCommands::Info { db, name } => {
            let full_name = format!("{}.{}", db, name);
            if let Some(info) = engine.get_table_info(&full_name).await? {
                println!("Table: {}.{}", db, info.name);
                println!("Database: {}", info.db);
                println!("Primary Key: {}", info.primary_key);
                println!("Documents: {}", info.doc_count);
                println!("Indexes: {:?}", info.indexes);
            } else {
                println!("Table not found");
            }
            Ok(())
        }
    }
}

/// Export command
async fn export_command(_data_dir: PathBuf, _args: ExportArgs) -> anyhow::Result<()> {
    println!("⚠️  Export functionality not yet implemented");
    // TODO: Implement export
    Ok(())
}

/// Import command
async fn import_command(_data_dir: PathBuf, _args: ImportArgs) -> anyhow::Result<()> {
    println!("⚠️  Import functionality not yet implemented");
    // TODO: Implement import
    Ok(())
}

/// Status command
async fn status_command(_data_dir: PathBuf) -> anyhow::Result<()> {
    println!("⚠️  Status command not yet implemented");
    // TODO: Check if server is running, show stats
    Ok(())
}
