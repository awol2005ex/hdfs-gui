use std::collections::HashMap;

use anyhow;
use hdfs_native::client::FileStatus;
use serde::{Deserialize, Serialize};

use super::hdfs_config::{self, HdfsConfig};

//hdfs配置
#[derive(Debug, Default, Deserialize, Serialize, sqlx::FromRow, Clone)]
pub struct HdfsFile {
    
    pub name: String,
    pub path: String, 
    pub parent_path: String, 
    pub owner :String,
    pub isdir: bool,
    pub group: String,
    pub permission: u16,
    pub modification_time: u64,
    pub access_time: u64,
    pub length: usize,
}
pub async fn get_hdfs_client(id:i64) -> anyhow_tauri::TAResult<hdfs_native::Client> {
    let  hdfsConfig :HdfsConfig=crate::commands::hdfs_config::get_one_hdfs_config(id).await?;

    let other_config= serde_json::from_str::<HashMap<String,String>>(&hdfsConfig.hdfs_config).unwrap_or_default();

    let hdfs_url=hdfsConfig.hdfs_url;
    let client = hdfs_native::Client::new_with_config(&hdfs_url, other_config).map_err(|e| anyhow::anyhow!(e.to_string()))?;

    return Ok(client);
}
//获取hdfs配置列表
#[tauri::command]
pub async fn get_hdfs_file_list(id:i64 ,parent_path: String) -> anyhow_tauri::TAResult<Vec<HdfsFile>> {
    println!("get_hdfs_file_list:parent_path:{}",&parent_path);
    let client = get_hdfs_client(id).await?;
    let files = client.list_status(&parent_path, false).await.map_err(|e| anyhow::anyhow!(e.to_string()))?;
    
    println!("get_hdfs_file_list:files:{:?}",&files);
    let hdfsFiles:Vec<HdfsFile> = files.iter().map(|file:&FileStatus | HdfsFile {
        //根据路径获取文件名

        name: std::path::Path::new( &file.path ).file_name().unwrap_or_default().to_str().unwrap_or_default().to_string(),
        path: file.path.clone().replace("\\", "/"),
        parent_path: parent_path.clone(),
        owner: file.owner.clone(),
        group: file.group.clone(),
        isdir: file.isdir.clone(),
        permission: file.permission.clone(),
        modification_time: file.modification_time.clone(),
        access_time: file.access_time.clone(),
        length: file.length.clone(),
    }).collect();
    println!("get_hdfs_file_list:hdfsFiles:{:?}",&hdfsFiles);
    Ok(hdfsFiles)
}
