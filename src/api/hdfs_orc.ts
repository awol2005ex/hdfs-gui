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

interface OrcMeta{
  total: number;
  compression_type: string;
}
//获取orc文件meta
export const get_hdfs_orc_file_meta = async (
  id: number,
  filePath: string
) => {
  const result: OrcMeta = await invoke("get_hdfs_orc_file_meta", {
    id: id,
    filePath: filePath,
  });
  return result;
};
// 定义数据项的类型
export interface DataRow {
  [key: string]: any;
}

//获取orc文件数据
export const read_orc_file_data_by_page = async (
  id: number,
  filePath: string,
  pageSize: number,
  page: number
) => {
  const result: Array<DataRow> = await invoke("read_orc_file_data_by_page", {
    id: id,
    filePath: filePath,
    pageNumber: page,
    pageSize: pageSize,
  });
  return result;
};

export const export_orc_file_data_to_csv = async (
  id: number,
  filePath: string,
  target_csv_file_path: string
) => {
  const result: Array<DataRow> = await invoke("export_orc_file_data_to_csv", {
    id: id,
    filePath: filePath,
    targetCsvFilePath: target_csv_file_path,
  });
  return result;
};
