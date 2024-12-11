use serde::{Deserialize,Serialize};
use anyhow;


use crate::db::db_init::DB_POOL;


//hdfs配置
#[derive(Debug, Default, Deserialize, Serialize,sqlx::FromRow)]
pub struct HdfsConfig {
    pub id :i64 ,
    pub name: String,
    pub hdfs_config: String,//json其他配置
    pub hdfs_url: String,
    pub del_flag: i64,//0正常 1删除
}
//获取hdfs配置列表
#[tauri::command]
pub async fn get_hdfs_config_list() -> anyhow_tauri::TAResult<Vec<HdfsConfig> > {

  crate::db::db_init::init_db().await.map_err(|e| anyhow::anyhow!(e.to_string()))?;

    if let Some(pool) =DB_POOL.get() {
      let hdfs_config_list :Vec<HdfsConfig>= sqlx::query_as::<_, HdfsConfig>("select * from hdfs_config where del_flag = 0")
        .fetch_all(pool).await.unwrap_or(vec![]);
       return Ok(hdfs_config_list);
    }
    Ok(vec![])

}


//保存hdfs配置
#[tauri::command]
pub async fn save_hdfs_config(hdfs_config :HdfsConfig) -> anyhow_tauri::TAResult<() > {

  crate::db::db_init::init_db().await.map_err(|e| anyhow::anyhow!(e.to_string()))?;

    if let Some(pool) =DB_POOL.get() {
       if hdfs_config.id > 0 {
        sqlx::query("update hdfs_config set name = ?, hdfs_config = ?, hdfs_url = ? where id = ?")
        .bind(hdfs_config.name)
        .bind(hdfs_config.hdfs_config)
        .bind(hdfs_config.hdfs_url)
        .bind(hdfs_config.id)
        .execute(pool).await.map_err(|e| anyhow::anyhow!(e.to_string()))?;
       }else{
        sqlx::query("insert into hdfs_config (name, hdfs_config, hdfs_url,del_flag) values (?, ?, ? ,0)")
        .bind(hdfs_config.name)
        .bind(hdfs_config.hdfs_config)
        .bind(hdfs_config.hdfs_url)
        .execute(pool).await.map_err(|e| anyhow::anyhow!(e.to_string()))?;
       }
    } else {
      return Err(anyhow::anyhow!("Database connection pool is not initialized").into());
    }
    Ok(())

}