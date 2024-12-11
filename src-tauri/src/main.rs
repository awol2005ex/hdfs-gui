mod db;

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[tokio::main]
async fn main()-> Result<(),anyhow::Error > {

    //初始化数据库连接
    db::db_init::init_db().await?;


    //启动UI
    hdfs_gui_lib::run();

    Ok(())

}
