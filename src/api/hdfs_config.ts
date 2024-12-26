import { invoke } from "@tauri-apps/api/core";
//HDFS连接配置列表
export interface HdfsConfig {
  id?: number;
  name?: string;
  hdfs_config?: string; //json其他配置
  hdfs_url?: string;
  del_flag?: number; //0正常 1删除
}
//获取HDFS连接配置列表
export const getHdfsConfigList = async () => {
  const result: Array<HdfsConfig> = await invoke("get_hdfs_config_list", {});
  return result;
};

//保证HDFS连接配置
export const saveHdfsConfig = async (hdfs_config: HdfsConfig) => {
  await invoke("save_hdfs_config", { hdfsConfig: hdfs_config });
};

//获取HDFS连接配置列表
export const getHdfsConfig = async (id: number) => {
  const result: HdfsConfig = await invoke("get_hdfs_config", { id: id });
  return result;
};
//删除HDFS连接配置
export const deleteHdfsConfig = async (id: number) => {
  await invoke("delete_hdfs_config", { id: id });
};
