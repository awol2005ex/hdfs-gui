import { invoke } from "@tauri-apps/api/core";
import { HdfsFile } from "./hdfs_file";

export interface HdfsAcl {
  owner: string;
  group: string;
  sticky: Boolean;
  entries: Array<HdfsAclEntry>;
  permission: number;
  filestatus: HdfsFile;
}
export interface HdfsAclEntry {
  rtype: string;
  scope: string;
  permissions: string;
  name: string;
}

//获取HDFS文件acls列表
export const getHdfsFileAclList = async (id: number, file_path: string) => {
  const result: HdfsAcl = await invoke("get_hdfs_file_acl_list", {
    id: id,
    filePath: file_path,
  });
  return result;
};

//添加HDFS文件acl
export const addHdfsFileAcl = async (
  id: number,
  file_path: string,
  rtype: string,
  scope: string,
  permissions: string,
  name: string
) => {
  const result: HdfsAcl = await invoke("add_acl", {
    id: id,
    filePath: file_path,
    rtype: rtype,
    scope: scope,
    permissions: permissions,
    name: name,
  });
  return result;
};