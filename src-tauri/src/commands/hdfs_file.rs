use std::collections::HashMap;

use bytes::Bytes;
use hdfs_native::{Client, WriteOptions};
use serde::{Deserialize, Serialize};

use super::hdfs_config::{get_hdfs_username, HdfsConfig};
use futures::future::BoxFuture;
use futures_util::FutureExt;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;
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

    pub file_count: Option<u64>,
    pub directory_count: Option<u64>,
    pub quota: Option<u64>,
    pub space_consumed: Option<u64>,
    pub space_quota: Option<u64>,
}
pub async fn get_hdfs_client(id: i64) -> Result<hdfs_native::Client, String> {
    let hdfs_config_instance: HdfsConfig =
        crate::commands::hdfs_config::get_one_hdfs_config(id).await?;

    let other_config =
        serde_json::from_str::<HashMap<String, String>>(&hdfs_config_instance.hdfs_config)
            .unwrap_or_default();

    let hdfs_url = hdfs_config_instance.hdfs_url;
    /*TODO 自动kerberos登录*/

    let client =
        hdfs_native::Client::new_with_config(&hdfs_url, other_config).map_err(|e| e.to_string())?;

    return Ok(client);
}
//获取hdfs文件列表
#[tauri::command]
pub async fn get_hdfs_file_list(
    id: i64,
    parent_path: String,
    show_content_summary: bool,
) -> Result<Vec<HdfsFile>, String> {
    //log::info!("get_hdfs_file_list:parent_path:{}", &parent_path);
    let client = get_hdfs_client(id).await?;
    let files = client
        .list_status(&parent_path, false)
        .await
        .map_err(|e| e.to_string())?;

    let mut hdfs_files = Vec::new();

    for file in files.iter() {
        let mut hdfs_file = HdfsFile {
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
            file_count: None,
            directory_count: None,
            quota: None,
            space_consumed: None,
            space_quota: None,
        };

        if file.isdir && show_content_summary {
            let content_summary: hdfs_native::client::ContentSummary = client
                .get_content_summary(&file.path)
                .await
                .map_err(|e| e.to_string())?;
            // 更新hdfs_file的内容摘要字段
            hdfs_file.length = content_summary.length as usize;
            hdfs_file.file_count = Some(content_summary.file_count);
            hdfs_file.directory_count = Some(content_summary.directory_count);
            hdfs_file.quota = Some(content_summary.quota);
            hdfs_file.space_consumed = Some(content_summary.space_consumed);
            hdfs_file.space_quota = Some(content_summary.space_quota);
        }

        hdfs_files.push(hdfs_file);
    }

    Ok(hdfs_files)
}

//获取hdfs单个文件状态
#[tauri::command]
pub async fn get_hdfs_file(id: i64, file_path: String) -> Result<HdfsFile, String> {
    let client = get_hdfs_client(id).await?;
    let file = client
        .get_file_info(&file_path)
        .await
        .map_err(|e| e.to_string())?;

    //log::info!("get_hdfs_file_list:files:{:?}", &files);
    let hdfs_file = HdfsFile {
        //根据路径获取文件名
        name: std::path::Path::new(&file.path)
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string(),
        path: file.path.clone().replace("\\", "/"),
        parent_path: std::path::Path::new(&file.path)
            .parent()
            .map_or("", |v| v.to_str().unwrap_or_default())
            .to_string(),
        owner: file.owner.clone(),
        group: file.group.clone(),
        isdir: file.isdir.clone(),
        permission: file.permission.clone(),
        modification_time: file.modification_time.clone(),
        access_time: file.access_time.clone(),
        length: file.length.clone(),
        file_count: None,
        directory_count: None,
        quota: None,
        space_consumed: None,
        space_quota: None,
    };
    //log::info!("get_hdfs_file_list:hdfsFiles:{:?}", &hdfs_files);
    Ok(hdfs_file)
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
            //log::info!("nbytes_read:{}", nbytes_read);
            // 如果没有字节可读，跳出循环
            if nbytes_read == 0 {
                break;
            }
            let s = &buf[0..nbytes_read];
            let b = bytes::Bytes::copy_from_slice(s);
            // 从buffer构造字符串
            let _write_size = hdfs_file_writer.write(b).await.map_err(|e| e.to_string())?;
            //log::info!("writeSize:{}", writeSize);
        } else {
            break;
        }
    }
    hdfs_file_writer.close().await.map_err(|e| e.to_string())?;

    Ok(true)
}

//写入文本
#[tauri::command]
pub async fn write_text_hdfs_file(
    id: i64,
    file_path: String,
    content: String,
) -> Result<bool, String> {
    let client = get_hdfs_client(id).await.map_err(|e| e.to_string())?;

    let mut hdfs_file_writer = client
        .create(&file_path, WriteOptions::default().overwrite(true))
        .await
        .map_err(|e| e.to_string())?;

    hdfs_file_writer
        .write(bytes::Bytes::copy_from_slice(content.as_bytes()))
        .await
        .map_err(|e| e.to_string())?;
    hdfs_file_writer.close().await.map_err(|e| e.to_string())?;

    Ok(true)
}

//删除文件
#[tauri::command]
pub async fn delete_hdfs_files(id: i64, file_path_list: Vec<String>) -> Result<bool, String> {
    let client = get_hdfs_client(id).await.map_err(|e| e.to_string())?;
    let username = get_hdfs_username(id).await.map_err(|e| e.to_string())?;

    let trash_path = format!("/user/{}/.Trash/Current", &username);

    for file_path in file_path_list {
        let trash_target_path = format!("{}{}", &trash_path, &file_path);

        match std::path::Path::new(&trash_target_path).parent() {
            Some(trash_target_parent_path) => {
                client
                    .mkdirs(
                        trash_target_parent_path.to_str().unwrap_or_default(),
                        0o755,
                        true,
                    )
                    .await
                    .map_err(|e| e.to_string())?;
            }
            None => {}
        };
        client
            .rename(&file_path, &trash_target_path, true)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(true)
}

//删除文件(跳过垃圾箱)
#[tauri::command]
pub async fn delete_hdfs_files_force(id: i64, file_path_list: Vec<String>) -> Result<bool, String> {
    let client = get_hdfs_client(id).await.map_err(|e| e.to_string())?;
    for file_path in file_path_list {
        client
            .delete(&file_path, true)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(true)
}

//改名
#[tauri::command]
pub async fn rename_hdfs_file(
    id: i64,
    old_file_path: String,
    new_file_name: String,
    overwrite: bool,
) -> Result<bool, String> {
    let client = get_hdfs_client(id).await.map_err(|e| e.to_string())?;
    let old_file_parent_path = std::path::Path::new(&old_file_path)
        .parent()
        .map_or("", |v| v.to_str().unwrap_or_default())
        .to_string();

    let new_file_path = format!("{}/{}", &old_file_parent_path, &new_file_name);
    client
        .rename(&old_file_path, &new_file_path, overwrite)
        .await
        .map_err(|e| e.to_string())?;

    Ok(true)
}

//新建目录
#[tauri::command]
pub async fn create_hdfs_dir(
    id: i64,
    parent_path: String,
    dir_name: String,
) -> Result<bool, String> {
    let client = get_hdfs_client(id).await.map_err(|e| e.to_string())?;
    client
        .mkdirs(&format!("{}/{}", &parent_path, &dir_name), 0o755, false)
        .await
        .map_err(|e| e.to_string())?;
    Ok(true)
}

//新建空白文件
#[tauri::command]
pub async fn create_hdfs_empty_file(
    id: i64,
    parent_path: String,
    file_name: String,
) -> Result<bool, String> {
    let client = get_hdfs_client(id).await.map_err(|e| e.to_string())?;
    client
        .create(
            &format!("{}/{}", &parent_path, &file_name),
            WriteOptions::default(),
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok(true)
}

//获取文件预览二进制内容
#[derive(Debug, Default, Deserialize, Serialize, sqlx::FromRow, Clone)]
pub struct HdfsFileContentPreview {
    //文件大小
    pub length: usize,
    //预览内容
    pub content: String,
    //是否orc类型文件
    pub isorc: bool,
    //是否parquet类型文件
    pub isparquet: bool,
    //是否avro类型文件
    pub isavro: bool,
}
#[tauri::command]
pub async fn get_hdfs_file_content_preview(
    id: i64,
    file_path: String,
) -> Result<HdfsFileContentPreview, String> {
    let client = get_hdfs_client(id).await.map_err(|e| e.to_string())?;
    let file_status = client
        .get_file_info(&file_path)
        .await
        .map_err(|e| e.to_string())?;
    let mut hdfs_file_reader = client.read(&file_path).await.map_err(|e| e.to_string())?;

    let buf: Bytes = hdfs_file_reader
        .read(1 * 1024 * 1024)
        .await
        .map_err(|e| e.to_string())?;
    //判断文件是否ORC
    let content = String::from_utf8_lossy(buf.to_vec().as_slice()).to_string();
    let isorc = content.starts_with("ORC");
    let isparquet = content.starts_with("PAR1");
    let isavro: bool =file_path.ends_with(".avro");
    Ok(HdfsFileContentPreview {
        content: content,
        length: file_status.length as usize,
        isorc: isorc,
        isparquet: isparquet,
        isavro: isavro,
    })
}

#[derive(Debug, Default, Deserialize, Serialize, sqlx::FromRow, Clone)]
pub struct HdfsFileContent {
    //文件大小
    pub length: usize,
    //预览内容
    pub content: String,
}
#[tauri::command]
pub async fn get_hdfs_file_content(id: i64, file_path: String) -> Result<HdfsFileContent, String> {
    let client = get_hdfs_client(id).await.map_err(|e| e.to_string())?;
    let file_status = client
        .get_file_info(&file_path)
        .await
        .map_err(|e| e.to_string())?;
    let mut hdfs_file_reader = client.read(&file_path).await.map_err(|e| e.to_string())?;

    let buf: Bytes = hdfs_file_reader
        .read(file_status.length as usize)
        .await
        .map_err(|e| e.to_string())?;
    Ok(HdfsFileContent {
        content: String::from_utf8_lossy(buf.to_vec().as_slice()).to_string(),
        length: file_status.length as usize,
    })
}

//下载文件到目标目录
#[tauri::command]
pub async fn download_file(
    id: i64,
    source_file_path: String,
    target_file_parent_path: String,
) -> Result<bool, String> {
    let client = get_hdfs_client(id).await.map_err(|e| e.to_string())?;
    let mut hdfs_file_reader = client
        .read(&source_file_path)
        .await
        .map_err(|e| e.to_string())?;

    let source_file_name = std::path::Path::new(&source_file_path)
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_string();

    let mut target_file = File::create(&format!(
        "{}/{}",
        &target_file_parent_path, &source_file_name
    ))
    .map_err(|e| e.to_string())?;

    loop {
        if let Ok(b) = hdfs_file_reader.read(102400).await {
            // 如果没有字节可读，跳出循环
            if b.len() == 0 {
                break;
            }
            // 从buffer构造字符串
            let _write_size = target_file.write(&b).map_err(|e| e.to_string())?;
            //log::info!("writeSize:{}", writeSize);
        } else {
            break;
        }
    }

    Ok(true)
}

//下载hdfs目录到目标本地目录
#[tauri::command]
pub async fn download_folder(
    id: i64,
    source_file_path: String,
    target_file_parent_path: String,
) -> Result<bool, String> {
    let client = get_hdfs_client(id).await.map_err(|e| e.to_string())?;

    let dir = client.list_status_iter(&source_file_path, true);
    while let Some(entry) = dir.next().await {
        let entry = entry.map_err(|e| format!("entry file status :{}", &e.to_string()))?;
        let entry_path = entry.path.replace("\\", "/");

        std::fs::create_dir_all(&target_file_parent_path.replace("\\", "/"))
            .map_err(|e| e.to_string())?;

        if entry.isdir {
            //目录创建目录
            let target_dir_path = format!(
                "{}{}",
                &target_file_parent_path.replace("\\", "/"),
                &entry_path
            );
            std::fs::create_dir_all(&target_dir_path.replace("\\", "/"))
                .map_err(|e| format!("create folder error :{}", &e.to_string()))?;
            continue;
        } else {
            let target_file_path = format!(
                "{}{}",
                &target_file_parent_path.replace("\\", "/"),
                &entry_path
            );
            let target_file_dir_path = Path::new(&target_file_path).parent();
            if !target_file_dir_path.is_none() {
                std::fs::create_dir_all(&target_file_dir_path.unwrap())
                    .map_err(|e| format!("create folder error :{}", &e.to_string()))?;
            }

            let mut target_file = File::create(&target_file_path).map_err(|e| {
                format!(
                    "create file {}{} error :{}",
                    &target_file_parent_path.replace("\\", "/"),
                    &entry_path,
                    &e.to_string()
                )
            })?;
            let mut hdfs_file_reader = client
                .read(&entry_path)
                .await
                .map_err(|e| format!("read file error :{}", &e.to_string()))?;

            loop {
                if let Ok(b) = hdfs_file_reader.read(102400).await {
                    // 如果没有字节可读，跳出循环
                    if b.len() == 0 {
                        break;
                    }
                    // 从buffer构造字符串
                    let _write_size = target_file.write(&b).map_err(|e| e.to_string())?;
                    //log::info!("writeSize:{}", writeSize);
                } else {
                    break;
                }
            }
        }
    }
    Ok(true)
}

//设置权限
#[tauri::command]
pub async fn set_hdfs_files_permissions(
    id: i64,
    file_path_list: Vec<String>,
    permission: u32,
    recursive: bool,
) -> Result<bool, String> {
    let client = get_hdfs_client(id).await.map_err(|e| e.to_string())?;

    return set_files_permission_impl(&client, file_path_list, permission, recursive).await;
}

pub fn set_files_permission_impl(
    client: &Client,
    file_path_list: Vec<String>,
    permission: u32,
    recursive: bool,
) -> BoxFuture<Result<bool, String>> {
    async move {
        for file_path in file_path_list {
            client
                .set_permission(&file_path, permission)
                .await
                .map_err(|e| e.to_string())?;
            if recursive {
                let dir = client.list_status_iter(&file_path, recursive);
                while let Some(entry) = dir.next().await {
                    let entry = entry.map_err(|e| e.to_string())?;
                    let entry_path = entry.path.replace("\\", "/");
                    //log::info!("entry_path:{}", &entry_path);
                    set_files_permission_impl(client, vec![entry_path], permission, false).await?;
                }
            }
        }
        Ok(true)
    }
    .boxed()
}
