use tauri_plugin_sql::{ Migration, MigrationKind};
use once_cell::sync::OnceCell;
use sqlx::{Pool, Sqlite,migrate::MigrateDatabase};

pub static DB_FILE: &str="sqlite://hdfs-gui.db";

pub static DB_POOL: OnceCell<Pool<Sqlite>> = OnceCell::new();

pub async fn init_db() -> Result<(), sqlx::Error> {
    if !Sqlite::database_exists(DB_FILE).await.unwrap_or(false) {
        println!("Creating database {}", DB_FILE);
        match Sqlite::create_database(DB_FILE).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }
    
    let pool = Pool::<Sqlite>::connect(DB_FILE).await?;
    DB_POOL.set(pool).unwrap();
    Ok(())
}
//数据库初始化
pub fn migrations () -> Vec<Migration> {
    return vec![
        // Define your migrations here
        Migration {
            version: 1,
            description: "create_initial_tables",
            sql: "CREATE TABLE hdfs_config (id INTEGER PRIMARY KEY   AUTOINCREMENT, name TEXT, hdfs_url TEXT,hdfs_config TEXT, del_flag INTEGER)",
            kind: MigrationKind::Up,
        }
    ];
}