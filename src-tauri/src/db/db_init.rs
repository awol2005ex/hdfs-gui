use once_cell::sync::OnceCell;
use sqlx::{migrate::MigrateDatabase, Pool, Sqlite};

pub static DB_FILE: &str = "sqlite://hdfs-gui.db";

pub static DB_POOL: OnceCell<Pool<Sqlite>> = OnceCell::new();
//数据库连接池初始化
pub async fn init_db() -> Result<(), anyhow::Error> {
    //创建数据库
    if !Sqlite::database_exists(DB_FILE).await.unwrap_or(false) {
        println!("Creating database {}", DB_FILE);
        match Sqlite::create_database(DB_FILE).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }

    if DB_POOL.get().is_none() {
        let pool = Pool::<Sqlite>::connect(DB_FILE).await?;
         DB_POOL.set(pool).map_err(|_| anyhow::anyhow!("set pool fail".to_string()))?;
         if let Some(init_pool) = DB_POOL.get() {
            sqlx::query("CREATE TABLE if not exists hdfs_config (id INTEGER PRIMARY KEY   AUTOINCREMENT, name TEXT, hdfs_url TEXT,hdfs_config TEXT, del_flag INTEGER)")
        .execute(init_pool).await.map_err(|e| anyhow::anyhow!(e.to_string()))?;
        }
    }
    
    
    Ok(())
}
