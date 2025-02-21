use serde::{Deserialize, Serialize};

use crate::db::db_init::DB_POOL;
use std::process::Command;
//hdfs配置
#[derive(Debug, Default, Deserialize, Serialize, sqlx::FromRow, Clone)]
pub struct HdfsConfig {
    pub id: i64,
    pub name: String,
    pub hdfs_config: String, //json其他配置
    pub hdfs_url: String,
    pub del_flag: i64, //0正常 1删除
}
//获取hdfs配置列表
#[tauri::command]
pub async fn get_hdfs_config_list() -> Result<Vec<HdfsConfig>, String> {
    crate::db::db_init::init_db()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(pool) = DB_POOL.get() {
        let hdfs_config_list: Vec<HdfsConfig> =
            sqlx::query_as::<_, HdfsConfig>("select * from hdfs_config where del_flag = 0")
                .fetch_all(pool)
                .await
                .unwrap_or(vec![]);
        return Ok(hdfs_config_list);
    }
    Ok(vec![])
}

//保存hdfs配置
#[tauri::command]
pub async fn save_hdfs_config(hdfs_config: HdfsConfig) -> Result<(), String> {
    crate::db::db_init::init_db()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(pool) = DB_POOL.get() {
        if hdfs_config.id > 0 {
            sqlx::query(
                "update hdfs_config set name = ?, hdfs_config = ?, hdfs_url = ? where id = ?",
            )
            .bind(hdfs_config.name)
            .bind(hdfs_config.hdfs_config)
            .bind(hdfs_config.hdfs_url)
            .bind(hdfs_config.id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        } else {
            sqlx::query("insert into hdfs_config (name, hdfs_config, hdfs_url,del_flag) values (?, ?, ? ,0)")
        .bind(hdfs_config.name)
        .bind(hdfs_config.hdfs_config)
        .bind(hdfs_config.hdfs_url)
        .execute(pool).await.map_err(|e| e.to_string())?;
        }
    } else {
        return Err("Database connection pool is not initialized".to_owned());
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_hdfs_config(id: i64) -> Result<(), String> {
    crate::db::db_init::init_db()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(pool) = DB_POOL.get() {
        sqlx::query("update hdfs_config set del_flag=1 where id = ?")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    } else {
        return Err("Database connection pool is not initialized".to_owned());
    }
    Ok(())
}

//初始化kerberos
#[tauri::command]
pub async fn init_connection(id: i64) -> Result<(), String> {
    crate::db::db_init::init_db()
        .await
        .map_err(|e| e.to_string())?;

    let hc = get_one_hdfs_config(id).await.map_err(|e| e.to_string())?;
    log::info!(" Hdfs Config: {:?}", &hc);
    if let Ok(config_json) = serde_json::from_str::<serde_json::Value>(&hc.hdfs_config) {
        if let Some(principal) = config_json.get("dfs.namenode.kerberos.principal") {
            if let Some(keytab) = config_json.get("dfs.namenode.keytab.file") {
                let principal_s = principal.as_str().unwrap_or_default();
                let keytab_s = keytab.as_str().unwrap_or_default();
                if cfg!(target_os = "windows") {
                    let current_dir = std::env::current_dir().map_err(|e| e.to_string())?;
                    let cache_file_path = format!(
                        "{}/hdfs_gui_ccache",
                        current_dir.to_str().unwrap_or_default()
                    );
                    std::env::set_var("KRB5CCNAME", cache_file_path.clone());
                    let s = format!("kinit -kt {} {}", keytab_s, principal_s);
                    log::info!("kinit command: {}", &s);
                    let o = Command::new("cmd")
                        .arg("/c")
                        .arg(&s)
                        .output()
                        .map_err(|e| e.to_string())?;
                    log::info!("kinit output: {:?}", &o);

                    if !(std::fs::exists(cache_file_path).map_err(|e| e.to_string())?) {
                        return Err("Kerberos authentication failed".to_owned());
                    }
                } else {
                    std::env::set_var("KRB5CCNAME", "/tmp/hdfs_gui_ccache");
                    let s = format!("kinit -kt {} {}", keytab_s, principal_s);
                    log::info!("kinit command: {}", &s);
                    let o = Command::new("sh")
                        .arg("-c")
                        .arg(&s)
                        .output()
                        .map_err(|e| e.to_string())?;
                    log::info!("kinit output: {:?}", &o);
                }
            }
        }
    }

    Ok(())
}

//获取单个hdfs配置
pub async fn get_one_hdfs_config(id: i64) -> Result<HdfsConfig, String> {
    if let Some(pool) = DB_POOL.get() {
        let hdfs_config_list: Vec<HdfsConfig> =
            sqlx::query_as::<_, HdfsConfig>("select * from hdfs_config where id=?")
                .bind(&id)
                .fetch_all(pool)
                .await
                .unwrap_or(vec![]);
        if hdfs_config_list.len() > 0 {
            return Ok(hdfs_config_list[0].clone());
        } else {
            return Err("no config found".to_owned());
        }
    } else {
        return Err("Database connection pool is not initialized".to_owned());
    }
}
//获取单个hdfs配置
#[tauri::command]
pub async fn get_hdfs_config(id: i64) -> Result<HdfsConfig, String> {
    crate::db::db_init::init_db()
        .await
        .map_err(|e| e.to_string())?;

    return get_one_hdfs_config(id).await.map_err(|e| e.to_string());
}
//获取当前用户名
pub async fn get_hdfs_username(id: i64) -> Result<String, String> {
    let hdfs_config = get_one_hdfs_config(id).await.map_err(|e| e.to_string());

    if let Ok(hc) = hdfs_config {
        if let Ok(config_json) = serde_json::from_str::<serde_json::Value>(&hc.hdfs_config) {
            if let Some(username) = config_json.get("dfs.namenode.kerberos.principal") {
                return Ok(username
                    .as_str()
                    .unwrap_or_default()
                    .to_string()
                    .split("@")
                    .next()
                    .unwrap_or_default()
                    .to_owned());
            }
        }
    }

    if std::env::var("HDFS_USERNAME").is_ok() {
        return Ok(std::env::var("HDFS_USERNAME").unwrap_or_default());
    }

    return Err("no hdfs username found".to_owned());
}
