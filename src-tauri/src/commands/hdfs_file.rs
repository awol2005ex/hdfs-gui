use std::collections::HashMap;

use anyhow;
use bytes::Bytes;
use hdfs_native::client::FileStatus;
use hdfs_native::WriteOptions;
use serde::{Deserialize, Serialize};

use super::hdfs_config::{self, HdfsConfig};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
//hdfs配置
#[derive(Debug, Default, Deserialize, Serialize, sqlx::FromRow, Clone)]
pub struct HdfsFile {
    pub name: String,
    pub path: String,
    pub parent_path: String,
    pub owner: String,
    pub isdir: bool,
    pub group: String,
    pub permission: u16,
    pub modification_time: u64,
    pub access_time: u64,
    pub length: usize,
}
pub async fn get_hdfs_client(id: i64) -> Result<hdfs_native::Client, String> {
    let hdfsConfig: HdfsConfig = crate::commands::hdfs_config::get_one_hdfs_config(id).await?;

    let other_config = serde_json::from_str::<HashMap<String, String>>(&hdfsConfig.hdfs_config)
        .unwrap_or_default();

    let hdfs_url = hdfsConfig.hdfs_url;
    let client = hdfs_native::Client::new_with_config(&hdfs_url, other_config)
        .map_err(|e| e.to_string())?;

    return Ok(client);
}
//获取hdfs配置列表
#[tauri::command]
pub async fn get_hdfs_file_list(
    id: i64,
    parent_path: String,
) -> Result<Vec<HdfsFile>, String>  {
    println!("get_hdfs_file_list:parent_path:{}", &parent_path);
    let client = get_hdfs_client(id).await?;
    let files = client
        .list_status(&parent_path, false)
        .await
        .map_err(|e| e.to_string())?;

    println!("get_hdfs_file_list:files:{:?}", &files);
    let hdfsFiles: Vec<HdfsFile> = files
        .iter()
        .map(|file: &FileStatus| HdfsFile {
            //根据路径获取文件名
            name: std::path::Path::new(&file.path)
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
                .to_string(),
            path: file.path.clone().replace("\\", "/"),
            parent_path: parent_path.clone(),
            owner: file.owner.clone(),
            group: file.group.clone(),
            isdir: file.isdir.clone(),
            permission: file.permission.clone(),
            modification_time: file.modification_time.clone(),
            access_time: file.access_time.clone(),
            length: file.length.clone(),
        })
        .collect();
    println!("get_hdfs_file_list:hdfsFiles:{:?}", &hdfsFiles);
    Ok(hdfsFiles)
}

//上传文件
#[tauri::command]
pub async fn upload_hdfs_file(
    id: i64,
    parent_path: String,
    local_file_path: String,
) -> Result<bool, String> {
    let client = get_hdfs_client(id).await.map_err(|e| e.to_string())?;
    //获取文件名
    let local_file_name = std::path::Path::new(&local_file_path)
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_string();

    let local_file = File::open(&local_file_path).map_err(|e| e.to_string())?;
    let mut local_file_buf_reader = BufReader::new(local_file);

    let mut hdfs_file_writer = client
        .create(
            &format!("{}/{}", &parent_path, &local_file_name),
            WriteOptions::default(),
        )
        .await
        .map_err(|e| e.to_string())?;

    loop {
        let mut buf = [0u8; 1024];
        if let Ok(nbytes_read) = local_file_buf_reader.read(&mut buf[..]) {
            //println!("nbytes_read:{}", nbytes_read);
            // 如果没有字节可读，跳出循环
            if nbytes_read == 0 {
                break;
            }
            let s = &buf[0..nbytes_read];
            let b = bytes::Bytes::copy_from_slice(s);
            // 从buffer构造字符串
            let writeSize =hdfs_file_writer
                .write(b)
                .await
                .map_err(|e| e.to_string())?;
            //println!("writeSize:{}", writeSize);
        } else {
            break;
        }
    }
    hdfs_file_writer.close().await.map_err(|e| e.to_string())?;

    Ok(true)
}


//删除文件
#[tauri::command]
pub async fn delete_hdfs_files(
    id: i64,
    file_path_list: Vec<String>,
) ->  Result<bool, String>  {
    let client = get_hdfs_client(id).await.map_err(|e| e.to_string())?;
    for file_path in file_path_list {
        client
            .delete(&file_path, true)
            .await
            .map_err(|e|e.to_string())?;
    }
    Ok(true)
}

//新建目录
#[tauri::command]
pub async fn create_hdfs_dir(
    id: i64,
    parent_path: String,
    dir_name: String,
) -> Result<bool, String>  {
    let client = get_hdfs_client(id).await.map_err(|e| e.to_string())?;
    client
        .mkdirs(&format!("{}/{}", &parent_path, &dir_name),509, false)
        .await
        .map_err(|e| e.to_string())?;
    Ok(true)
}

//获取文件预览二进制内容
#[tauri::command]
pub async fn get_hdfs_file_content_preview(
    id: i64,
    file_path: String,
) -> Result<String, String>  {
    let client = get_hdfs_client(id).await.map_err(|e| e.to_string())?;
    let mut hdfs_file_reader = client
        .read(&file_path)
        .await
        .map_err(|e| e.to_string())?;

    let buf:Bytes=hdfs_file_reader
        .read(1*1024*1024)
        .await
        .map_err(|e| e.to_string())?;
    Ok(buf.to_vec().into_iter().map(|x| x as char).collect())
}