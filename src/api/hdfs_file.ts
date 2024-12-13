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


//上传文件
export const uploadHdfsFile = async (id: number, parent_path: string ,local_file_path: string) => {
  const result: Boolean = await invoke("upload_hdfs_file", {
    id: id,
    parentPath: parent_path,
    localFilePath: local_file_path,
  });
  return result;
};
//删除文件
export const deleteHdfsFiles = async (id: number, file_path_list: Array<string>) => {
  const result: Boolean = await invoke("delete_hdfs_files", {
    id: id,
    filePathList: file_path_list,
  });
  return result;
};
//创建目录
export const createHdfsFolder = async (id: number, parent_path: string, folder_name: string) => {
  const result: Boolean = await invoke("create_hdfs_dir", {
    id: id,
    parentPath: parent_path,
    dirName: folder_name,
  });
  return result;
}