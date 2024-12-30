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
        <el-button-group style="float: left; margin-left: 20px">
        <el-button type="primary" @click="AddAclDialogVisible = true"
          >Add Acl</el-button
        >
        </el-button-group>
        <el-table :data="acls.entries" border>
          <el-table-column prop="rtype" label="Type" width="180" />
          <el-table-column prop="scope" label="scope" width="180" />
          <el-table-column prop="name" label="Name" width="180" />
          <el-table-column prop="permissions" label="Permission" width="180" />
        </el-table>
      </el-descriptions-item>
    </el-descriptions>
  </div>

  <el-dialog v-model="AddAclDialogVisible" title="Add Acl" width="500">
    <el-form :model="addacls" label-width="120px">
      <el-form-item label="Type">
        <el-select v-model="addacls.rtype" placeholder="Select">
          <el-option label="user" value="user" />
          <el-option label="group" value="group" />
          <el-option label="mask" value="mask" />
          <el-option label="other" value="other" />
        </el-select>
      </el-form-item>
      <el-form-item label="Scope">
        <el-select v-model="addacls.scope" placeholder="Select">
          <el-option label="default" value="default" />
          <el-option label="access" value="access" />
        </el-select>
      </el-form-item>
      <el-form-item
        label="Name"
        v-if="addacls.rtype != 'mask' && addacls.rtype != 'other'"
      >
        <el-input v-model="addacls.name" />
      </el-form-item>
      <el-form-item
        label="Permission"
        v-if="addacls.rtype != 'mask' && addacls.rtype != 'other'"
      >

        <el-select v-model="addacls.permissions" placeholder="Select">
          <el-option label="---" value="---" />
          <el-option label="--x" value="--x" />
          <el-option label="-w-" value="-w-" />
          <el-option label="-wx" value="-wx" />
          <el-option label="r--" value="r--" />
          <el-option label="r-x" value="r-x" />
          <el-option label="rw-" value="rw-" />
          <el-option label="rwx" value="rwx" />
        </el-select>
      </el-form-item>
    </el-form>
    <template #footer>
      <span class="dialog-footer">
        <el-button @click="AddAclDialogVisible = false">Cancel</el-button>
        <el-button type="primary" @click="addAclFunc">Confirm</el-button>
      </span>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { reactive, Reactive, ref, watch } from "vue";
import { addHdfsFileAcl, getHdfsFileAclList, HdfsAcl } from "../api/hdfs_acls";
import { HdfsFile } from "../api/hdfs_file";
import { ElMessage, ElLoading } from "element-plus";

interface Props {
  filePath?: string;
  hdfsConfigId?: number;
}
const props = withDefaults(defineProps<Props>(), {
  hdfsConfigId: 0,
  filePath: "",
});

interface AddAcl {
  rtype: string;
  scope: string;
  name?: string|null|undefined;
  permissions: string;
}
const addacls: Reactive<AddAcl> = reactive({
  rtype: "",
  scope: "",
  name: "",
  permissions: "",
});
const AddAclDialogVisible = ref(false);

const addAclFunc = async () => {
  if (props.filePath && props.filePath != "") {
    const loadingInstance1 = ElLoading.service({ fullscreen: true });
    try {
      const b = await addHdfsFileAcl(
        props.hdfsConfigId,
        props.filePath,
        addacls.rtype,
        addacls.scope,
        addacls.permissions,
        addacls.name==""?null:addacls.name,
      );
      if (b) {
        ElMessage({
          showClose: true,
          message: "Download Success",
          type: "success",
        });
      } else {
        ElMessage({
          showClose: true,
          message: "Download Failed",
          type: "error",
        });
      }

      loadingInstance1.close();
      AddAclDialogVisible.value = false
      reloadFileAcls()
    } catch (error: any) {
      ElMessage({
        showClose: true,
        message: error.toString(),
        type: "error",
      });
      loadingInstance1.close();
    }
  }
};

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
