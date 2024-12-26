<template>
  <div>
    <el-descriptions title="File Acls" :column="1" border>
      <el-descriptions-item label="Owner">{{
        acls.owner
      }}</el-descriptions-item>
      <el-descriptions-item label="Group">{{
        acls.group
      }}</el-descriptions-item>
      <el-descriptions-item label="Sticky">{{
        acls.sticky
      }}</el-descriptions-item>
      <el-descriptions-item label="Permission">{{
        convertPermissionsToSymbolic(acls.permission, acls.filestatus)
      }}</el-descriptions-item>
      <el-descriptions-item label="Entries">
        <el-table :data="acls.entries" border>
          <el-table-column prop="rtype" label="Type" width="180" />
          <el-table-column prop="scope" label="scope" width="180" />
          <el-table-column prop="name" label="Name" width="180" />
          <el-table-column prop="permissions" label="Permission" width="180" />
        </el-table>
      </el-descriptions-item>
    </el-descriptions>
  </div>
</template>

<script setup lang="ts">
import { reactive, Reactive, ref, watch } from "vue";
import { getHdfsFileAclList, HdfsAcl } from "../api/hdfs_acls";
import { HdfsFile } from "../api/hdfs_file";

interface Props {
  filePath?: string;
  hdfsConfigId?: number;
}
const props = withDefaults(defineProps<Props>(), {
  hdfsConfigId: 0,
  filePath: "",
});

const acls: Reactive<HdfsAcl> = reactive({
  owner: "",
  group: "",
  sticky: false,
  permission: 0,
  entries: [],
  filestatus: {
    name: "",
    path: "",
    parent_path: "",
    owner: "",
    isdir: false,
    group: "",
    permission: 0,
    modification_time: 0,
    access_time: 0,
    length: 0,
  },
});

const reloadFileAcls = async () => {
  console.log("reloadFileAcls:", props.filePath);
  if (props.filePath && props.filePath != "") {
    const acls_now = await getHdfsFileAclList(
      props.hdfsConfigId,
      props.filePath
    );
    //console.log("acls_now:", acls_now);

    acls.owner = acls_now.owner;
    acls.group = acls_now.group;
    acls.sticky = acls_now.sticky;
    acls.permission = acls_now.permission;
    acls.entries = acls_now.entries;
    acls.filestatus = acls_now.filestatus;
  }
};

//显示文件目录权限
function convertPermissionsToSymbolic(permissionNumber: number, row: HdfsFile) {
  // 将十进制数转换为八进制字符串，并确保它是3位长。
  let octal = ("000" + permissionNumber.toString(8)).slice(-3);

  // 定义权限映射。
  const permissionMap = [
    "---",
    "--x",
    "-w-",
    "-wx",
    "r--",
    "r-x",
    "rw-",
    "rwx",
  ];

  // 将八进制每一位转换为对应的权限字符串。
  let permissions = octal
    .split("")
    .map((digit) => permissionMap[parseInt(digit, 8)])
    .join("");

  // 假设我们处理的是普通文件，所以添加 '-' 到最前面。
  if (!row.isdir) return "-" + permissions;
  else return "d" + permissions;
}

watch(
  () => props.filePath,
  () => {
    reloadFileAcls();
  }
);
</script>

<style scoped></style>
