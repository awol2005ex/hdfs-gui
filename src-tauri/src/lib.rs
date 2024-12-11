mod commands;
mod db;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
            //数据库初始化
            .add_migrations(db::db_init::DB_FILE, db::db_init::migrations())
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            //获取hdfs配置列表
            commands::hdfs_config::get_hdfs_config_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
