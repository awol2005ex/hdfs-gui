import { invoke } from "@tauri-apps/api/core";

export interface OrcField {
  name: String;
  type_name: String;
}
//获取orc文件字段列表
export const get_hdfs_orc_file_field_list = async (
  id: number,
  filePath: string
) => {
  const result: Array<OrcField> = await invoke("get_hdfs_orc_file_field_list", {
    id: id,
    filePath: filePath,
  });
  return result;
};
//获取orc文件行数
export const get_hdfs_orc_file_rows_count = async (
    id: number,
    filePath: string
  ) => {
    const result: number = await invoke("get_hdfs_orc_file_rows_count", {
      id: id,
      filePath: filePath,
    });
    return result;
  };
