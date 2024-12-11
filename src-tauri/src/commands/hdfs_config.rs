use serde::{Deserialize,Serialize};
use anyhow;


//hdfs配置
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct HdfsConfig {
    pub id :i64 ,
    pub name: String,
    pub hdfs_config: String,//json其他配置
    pub hdfs_url: String,
    pub del_flag: i64,//0正常 1删除
}
//获取hdfs配置列表
#[tauri::command]
pub fn get_hdfs_config_list() -> anyhow_tauri::TAResult<Vec<HdfsConfig> > {


    // let hdfs_config_list = HdfsConfig::get_hdfs_config_list();
    // Ok(hdfs_config_list)
    Ok(vec![])

}