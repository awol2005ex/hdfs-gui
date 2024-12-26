
use serde::{Deserialize, Serialize};

use crate::HdfsFile;

use super::hdfs_file::get_hdfs_client;

//hdfs file acl
#[derive(Debug, Default, Deserialize, Serialize, sqlx::FromRow, Clone)]
pub struct HdfsAcl {
    pub owner: String,
    pub group: String,
    pub sticky: bool,
    pub entries: Vec<HdfsAclEntry>,
    pub permission: u16,
    pub filestatus: HdfsFile,
}
#[derive(Debug, Default, Deserialize, Serialize, sqlx::FromRow, Clone)]
pub struct HdfsAclEntry {
    pub rtype: String,
    pub scope: String,
    pub permissions: String,
    pub name: Option<String>,
}
//获取hdfs文件acl列表
#[tauri::command]
pub async fn get_hdfs_file_acl_list(id: i64, file_path: String) -> Result<HdfsAcl, String> {
    //println!("get_hdfs_file_list:parent_path:{}", &parent_path);
    let client = get_hdfs_client(id).await?;
    let file_status =client.get_file_info(&file_path).await.map_err(|e| e.to_string())?;
    let acl_status = client
        .get_acl_status(&file_path)
        .await
        .map_err(|e| e.to_string())?;
    let hdfs_acls = HdfsAcl{
        owner: acl_status.owner,
        group: acl_status.group,
        sticky: acl_status.sticky,
        entries: acl_status.entries.into_iter().map(|entry|{
             HdfsAclEntry{
                rtype: entry.r#type.to_string(),
                scope: entry.scope.to_string(),
                permissions: entry.permissions.to_string(),
                name: entry.name,
            }
        }).collect(),
        permission: acl_status.permission,
        filestatus:HdfsFile {
            //根据路径获取文件名
            name: std::path::Path::new(&file_status.path)
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
                .to_string(),
            path: file_status.path.clone().replace("\\", "/"),
            parent_path: std::path::Path::new(&file_status.path)
                .parent()
                .map_or("", |v| v.to_str().unwrap_or_default())
                .to_string(),
            owner: file_status.owner.clone(),
            group: file_status.group.clone(),
            isdir: file_status.isdir.clone(),
            permission: file_status.permission.clone(),
            modification_time: file_status.modification_time.clone(),
            access_time: file_status.access_time.clone(),
            length: file_status.length.clone(),
        }
    };


    Ok(hdfs_acls)
}