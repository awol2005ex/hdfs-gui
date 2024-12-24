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


//删除文件跳过垃圾箱
export const deleteHdfsFilesForce = async (id: number, file_path_list: Array<string>) => {
  const result: Boolean = await invoke("delete_hdfs_files_force", {
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

//查看文件预览内容
export const get_file_preview_content = async (id: number, file_path : string) => {
  const result: string = await invoke("get_hdfs_file_content_preview", {
    id: id,
    filePath: file_path,
  });
  return result;
}


//下载文件
export const download_file = async (id: number, source_file_path : string, target_file_parent_path : string) => {
  const result: string = await invoke("download_file", {
    id: id,
    sourceFilePath: source_file_path,
    targetFileParentPath: target_file_parent_path,
  });
  return result;
}