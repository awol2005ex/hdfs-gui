use commands::{
    hdfs_acls::*, hdfs_avro::*, hdfs_config::*, hdfs_file::*, hdfs_orc::*, hdfs_parquet::*,
};

mod commands;
mod db;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let current_dir = std::env::current_dir().map_err(|e| e.to_string()).unwrap();
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Webview,
                ))
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Folder {
                        path: current_dir.join("logs"),
                        file_name: None,
                    },
                ))
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            //获取hdfs配置列表
            get_hdfs_config_list,
            //保存hdfs配置
            save_hdfs_config,
            //获取hdfs配置
            get_hdfs_config,
            //获取hdfs文件列表
            get_hdfs_file_list,
            //删除hdfs配置
            delete_hdfs_config,
            //上传文件到hdfs
            upload_hdfs_file,
            //删除hdfs文件
            delete_hdfs_files,
            //新建目录
            create_hdfs_dir,
            //获取预览内容
            get_hdfs_file_content_preview,
            //获取内容
            get_hdfs_file_content,
            //下载文件
            download_file,
            //删除hdfs文件(跳过垃圾箱)
            delete_hdfs_files_force,
            //创建空白文件
            create_hdfs_empty_file,
            //设置权限
            set_hdfs_files_permissions,
            //写入文本
            write_text_hdfs_file,
            //获取单个hdfs文件
            get_hdfs_file,
            //改名
            rename_hdfs_file,
            //获取hdfs文件acl列表
            get_hdfs_file_acl_list,
            //添加acl
            add_acl,
            //删除acl
            delete_acl,
            //初始化连接
            init_connection,
            //删除默认acl
            delete_default_acl,
            //删除全部
            delete_all_acl,
            //获取orc文件字段列表
            get_hdfs_orc_file_field_list,
            //获取orc文件行数
            get_hdfs_orc_file_rows_count,
            //读取orc文件meta
            get_hdfs_orc_file_meta,
            //分页读取orc文件数据
            read_orc_file_data_by_page,
            //导出orc数据到csv
            export_orc_file_data_to_csv,
            //获取parquet文件字段列表
            get_hdfs_parquet_file_field_list,
            //获取parquet文件行数
            get_hdfs_parquet_file_rows_count,
            //读取parquet文件meta
            get_hdfs_parquet_file_meta,
            //分页读取parquet文件数据
            read_parquet_file_data_by_page,
            //导出parquet数据到csv
            export_parquet_file_data_to_csv,
            //下载hdfs目录到目标本地目录
            download_folder,
            //查看avro数据
            get_avro_content,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
