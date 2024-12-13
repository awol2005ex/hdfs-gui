import { invoke } from "@tauri-apps/api/core";

export interface HdfsFile {
  name: string;
  path: string;
  parent_path: string;
  owner: string;
  isdir: Boolean;
  group: string;
  permission: number;
  modification_time: number;
  access_time: number;
  length: number;
  [key: string]: string|number|Boolean;
}

//获取HDFS文件列表
export const getHdfsFileList = async (id: number, parent_path: string) => {
  const result: Array<HdfsFile> = await invoke("get_hdfs_file_list", {
    id: id,
    parentPath: parent_path,
  });
  return result;
};
