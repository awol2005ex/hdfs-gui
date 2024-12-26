<template>
  <div class="common-layout">
    <el-container>
      <el-header>
        <table width="100%">
          <tr>
            <td>
              <el-button-group style="float: left">
                <el-button
                  type="primary"
                  :icon="HomeFilled"
                  circle
                  @click="backToHome"
                  title="Back To Home" />
                <el-button
                  type="primary"
                  :icon="Back"
                  circle
                  @click="backToLastPage"
                  title="Back To Last Page" />
                <el-button
                  type="primary"
                  :icon="Refresh"
                  circle
                  @click="refreshData"
                  title="Refresh"
              /></el-button-group>
              <el-input
                v-model="search_words"
                style="
                  width: 240px;
                  float: left;
                  margin-left: 10px;
                  margin-top: 5px;
                "
                placeholder="Search File"
                :prefix-icon="Search"
                @change="on_search_words_change"
                clearable
              />

              <el-button-group style="float: right">
                <el-button
                  type="primary"
                  :icon="FolderAdd"
                  circle
                  @click="NewFolder"
                  title="Create New Folder"
                />
                <el-button
                  type="primary"
                  :icon="DocumentAdd"
                  circle
                  @click="NewEmptyFile"
                  title="Create New File"
                />
                <el-button
                  type="primary"
                  :icon="Location"
                  circle
                  @click="goToLocation"
                  title="Go To Input Path"
                />

                <el-button
                  type="primary"
                  :icon="Upload"
                  circle
                  @click="uploadFileToHdfs"
                  title="Upload File To Hdfs"
                />
                <el-button
                  type="primary"
                  :icon="Suitcase"
                  circle
                  @click="SetFilePermission"
                  title="Set File Permission"
                />
              </el-button-group>
            </td>
            <td>
              <el-dropdown
                split-button
                type="warning"
                circle
                @click="deleteFiles"
                title="Delete Files in Hdfs"
                ><el-icon><Delete /></el-icon
                ><template #dropdown>
                  <el-dropdown-menu>
                    <el-dropdown-item @click="deleteFilesForce"
                      >Delete Files in Hdfs Skip Trash</el-dropdown-item
                    >
                  </el-dropdown-menu>
                </template>
              </el-dropdown>
            </td>
          </tr>
          <tr>
            <td colspan="2">
              <el-breadcrumb
                separator="/"
                style="float: left; padding-left: 10px"
              >
                <el-breadcrumb-item
                  v-if="current_parent_paths.length <= 1"
                  :to="{ path: '/' }"
                  >/</el-breadcrumb-item
                >
                <el-breadcrumb-item
                  v-if="current_parent_paths.length > 1"
                  :to="{
                    path: '/HdfsFolderView/' + route.params.id,
                    query: { path: '/' },
                  }"
                  >Root</el-breadcrumb-item
                >
                <template v-if="current_parent_paths.length > 1">
                  <template v-for="item in current_parent_paths">
                    <el-breadcrumb-item
                      :to="{
                        path: '/HdfsFolderView/' + route.params.id,
                        query: { path: item.path },
                      }"
                      >{{ item.name }}</el-breadcrumb-item
                    >
                  </template>
                </template>
              </el-breadcrumb>
            </td>
          </tr>
        </table>
      </el-header>
      <el-main>
        <el-table
          :data="fileListPageData"
          style="width: 100%"
          @sort-change="sortChange"
          @selection-change="handleSelectionChange"
        >
          <el-table-column type="selection" width="55" />
          <el-table-column prop="isdir" label="" width="60">
            <template #default="scope">
              <el-icon v-if="scope.row.isdir" :size="20">
                <Folder />
              </el-icon>
              <el-icon v-else :size="20">
                <Document />
              </el-icon>
            </template>
          </el-table-column>

          <el-table-column
            prop="name"
            label="Name"
            width="auto"
            show-overflow-tooltip
            sortable="custom"
          >
            <template #default="scope">
              <el-link
                v-if="scope.row.isdir"
                :underline="false"
                @click="goToFolder(scope.row)"
                >{{ scope.row.name }}</el-link
              >
              <el-link v-else :underline="false" @click="goToFile(scope.row)">{{
                scope.row.name
              }}</el-link>
            </template>
          </el-table-column>
          <el-table-column
            prop="length"
            label="Size"
            width="120"
            show-overflow-tooltip
            sortable="custom"
          >
            <template #default="scope">
              {{ scope.row.isdir ? "" : formatFileSize(scope.row.length) }}
            </template>
          </el-table-column>
          <el-table-column
            prop="owner"
            label="Owner"
            width="120"
            show-overflow-tooltip
            sortable="custom"
          />
          <el-table-column
            prop="group"
            label="Group"
            width="120"
            show-overflow-tooltip
            sortable="custom"
          />
          <el-table-column
            prop="permission"
            label="Permission"
            width="140"
            show-overflow-tooltip
            sortable="custom"
          >
            <template #default="scope">
              {{ convertPermissionsToSymbolic(scope.row) }}
            </template>
          </el-table-column>
          <el-table-column
            prop="modification_time"
            label="Time"
            width="180"
            show-overflow-tooltip
            sortable="custom"
          >
            <template #default="scope">
              {{ new Date(scope.row.modification_time).toLocaleString() }}
            </template>
          </el-table-column>

          <el-table-column label="Operation" width="180">
            <template #default="scope">
              <el-button-group style="float: left; margin-left: 20px">
                <el-button
                  type="primary"
                  size="small"
                  @click="SetFileAclsPermissions(scope.row)"
                  >acls</el-button
                >
                <el-button
                  type="warning"
                  size="small"
                  @click="RenameFile(scope.row)"
                  >Rename</el-button
                >
              </el-button-group>
            </template>
          </el-table-column>
        </el-table>

        <el-pagination
          @current-change="handleCurrentChange"
          :current-page="currentPage"
          :page-size="pageSize"
          :total="total"
          :page-sizes="[10, 100, 200, 300, 400]"
          layout="prev, pager, next, sizes, jumper,total"
          @size-change="handleSizeChange"
        ></el-pagination>
      </el-main>
    </el-container>
  </div>

  <el-dialog
    v-model="SetPermissionsDialogVisible"
    title="Set Permissions"
    width="500"
  >
    <HdfsPermissiionsEdit
      ref="hdfsPermissiionsEdit"
      :permission="hdfsPermissiionsEditInit"
    />
    <template #footer>
      <div class="dialog-footer">
        <el-button @click="SetPermissionsDialogVisible = false"
          >Cancel</el-button
        >
        <el-button type="primary" @click="SetPermissionsSubmit">
          Confirm
        </el-button>
      </div>
    </template>
  </el-dialog>


  <el-dialog
    v-model="SetAclsDialogVisible"
    title="Set Acls"
    width="1000"
  >
    <HdfsFileAclsEdit
      ref="hdfsFileAclsEdit"
      :filePath="selectAclsfilePath"
      :hdfsConfigId="parseInt(route.params.id as string)"
    />
    <template #footer>
      <div class="dialog-footer">
        <el-button @click="SetAclsDialogVisible = false"
          >Close</el-button
        >
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import { useRouter, useRoute } from "vue-router";
import {
  Back,
  Refresh,
  Folder,
  Document,
  HomeFilled,
  Search,
  Location,
  Upload,
  Delete,
  FolderAdd,
  DocumentAdd,
  Suitcase,
} from "@element-plus/icons-vue";
import {
  getHdfsFileList,
  HdfsFile,
  uploadHdfsFile,
  deleteHdfsFiles,
  createHdfsFolder,
  deleteHdfsFilesForce,
  createHdfsEmptyFile,
  setHdfsFilesPermissions,
  renameHdfsFile,
  getHdfsFile,
} from "../api/hdfs_file.ts";
import { ElMessage, ElMessageBox, ElLoading } from "element-plus";
//选择文件
import { open } from "@tauri-apps/plugin-dialog";

import HdfsPermissiionsEdit from "../components/HdfsPermissionsEdit.vue";
import HdfsFileAclsEdit from "../components/HdfsFileAclsEdit.vue";
const router = useRouter();
const route = useRoute();

//console.log(route.params);
//console.log(route.query);
//返回首页
const backToHome = () => {
  router.push("/");
};
//返回历史上一页
const backToLastPage = () => {
  router.go(-1);
};
//当前路径
const current_parent_path = ref(
  route.query.path ? (route.query.path as string) : "/"
);
//当前路径分解点击
const get_file_path_separator = (path: string) => {
  if (path == "/") {
    return [{ path: "/", name: "/" }];
  }
  const paths = path.split("/");

  return paths.map((item, index) => {
    return {
      path: index == 0 ? "/" : paths.slice(0, index + 1).join("/"),
      name: item,
    };
  });
};

const current_parent_paths = ref(
  get_file_path_separator(route.query.path ? (route.query.path as string) : "/")
);
//搜索过滤排序
const fileListData = ref<HdfsFile[]>([]);

const fileListPageData = ref<HdfsFile[]>([]);

const filert_by_search_words = () => {
  if (search_words.value == "") {
    fileListPageData.value = fileListData.value
      //排序
      .sort((a: HdfsFile, b: HdfsFile) => {
        if (a[sortProp.value.toString()] < b[sortProp.value.toString()])
          return sortOrder.value == "descending" ? -1 : 1;
        if (a[sortProp.value.toString()] > b[sortProp.value.toString()])
          return sortOrder.value == "descending" ? 1 : -1;
        return 0;
      })
      //分页
      .slice(
        (currentPage.value - 1) * pageSize.value,
        currentPage.value * pageSize.value
      );
    total.value = (fileListData.value || []).length;
    return;
  }
  const filterData = fileListData.value.filter((item) => {
    return (item.name || "").includes(search_words.value);
  });
  fileListPageData.value = filterData
    //排序
    .sort((a: HdfsFile, b: HdfsFile) => {
      if (a[sortProp.value.toString()] < b[sortProp.value.toString()])
        return sortOrder.value == "descending" ? -1 : 1;
      if (a[sortProp.value.toString()] > b[sortProp.value.toString()])
        return sortOrder.value == "descending" ? 1 : -1;
      return 0;
    })
    //分页
    .slice(
      (currentPage.value - 1) * pageSize.value,
      currentPage.value * pageSize.value
    );
  total.value = filterData.length;
  return;
};
//刷新表格
const refreshData = () => {
  const loadingInstance1 = ElLoading.service({ fullscreen: true });
  getHdfsFileList(
    parseInt(route.params.id as string),
    current_parent_path.value
  )
    .then((res) => {
      fileListData.value = res;
      filert_by_search_words();
      loadingInstance1.close();
    })
    .catch((err) => {
      ElMessage({
        showClose: true,
        message: err.toString(),
        type: "error",
      });
      loadingInstance1.close();
      backToLastPage();
    });
};
//刷新数据
refreshData();
//显示文件大小
const formatFileSize = (size: number) => {
  if (size < 1024) {
    return size + " B";
  } else if (size < 1024 * 1024) {
    return (size / 1024).toFixed(2) + " KB";
  } else if (size < 1024 * 1024 * 1024) {
    return (size / 1024 / 1024).toFixed(2) + " MB";
  } else {
    return (size / 1024 / 1024 / 1024).toFixed(2) + " GB";
  }
};
//打开文件
const goToFile = (row: HdfsFile) => {
  router.push({
    path: "/HdfsFileView/" + route.params.id,
    query: {
      path: row.path,
    },
  });
};
//打开目录
const goToFolder = (row: HdfsFile) => {
  router.push({
    path: "/HdfsFolderView/" + route.params.id,
    query: {
      path: row.path,
    },
  });
};
//路由跳转
watch(route, (newRoute) => {
  current_parent_path.value = newRoute.query.path as string;
  current_parent_paths.value = get_file_path_separator(
    newRoute.query.path as string
  );
  refreshData();
});

//分页
const pageSize = ref(10);
const total = ref(0);
const currentPage = ref(1);
const handleCurrentChange = async (val: number) => {
  currentPage.value = val;
  filert_by_search_words();
};
const handleSizeChange = async (val: number) => {
  pageSize.value = val;
  filert_by_search_words();
};

const on_search_words_change = () => {
  filert_by_search_words();
};
//搜索框
const search_words = ref("");

const sortProp = ref("");
const sortOrder = ref("");
//排序
const sortChange = (row: { column: any; prop: any; order: any }) => {
  const { prop, order } = row;

  sortProp.value = prop;
  sortOrder.value = order;
  filert_by_search_words();
};
//跳转地址
const goToLocation = async () => {
  const newLocation = await ElMessageBox.prompt("Please input Path", "Tip", {
    confirmButtonText: "OK",
    cancelButtonText: "Cancel",
  });
  if (newLocation.action == "confirm") {
    router.push({
      path: "/HdfsFolderView/" + route.params.id,
      query: {
        path: newLocation.value,
      },
    });
  }
};
//多选文件
const multipleSelection = ref<HdfsFile[]>([]);
const handleSelectionChange = (val: HdfsFile[]) => {
  multipleSelection.value = val;
};
//上传文件
const uploadFileToHdfs = async () => {
  const selected = await open({
    multiple: false,
    directory: false,
  });
  if (selected) {
    const loadingInstance1 = ElLoading.service({ fullscreen: true });
    try {
      const result = await uploadHdfsFile(
        parseInt(route.params.id as string),
        current_parent_path.value,
        selected
      );
      if (result) {
        ElMessage({
          showClose: true,
          message: "Upload success",
          type: "success",
        });
        refreshData();
        loadingInstance1.close();
      } else {
        ElMessage({
          showClose: true,
          message: "Upload failed",
          type: "error",
        });
        loadingInstance1.close();
      }
    } catch (err: any) {
      ElMessage({
        showClose: true,
        message: err.toString(),
        type: "error",
      });
      loadingInstance1.close();
    }
  }
};

//删除文件
const deleteFiles = async () => {
  //console.log(multipleSelection.value.map((item) => item.path).join(","))
  if (multipleSelection.value.length == 0) {
    ElMessage({
      showClose: true,
      message: "Please select files",
      type: "error",
    });
    return;
  }
  const s = await ElMessageBox.confirm(
    "Delete files " +
      multipleSelection.value.map((item) => item.path).join(",") +
      " . Continue?",
    "Warning",
    {
      confirmButtonText: "OK",
      cancelButtonText: "Cancel",
      type: "warning",
      draggable: true,
    }
  );
  if (s != "confirm") {
    return;
  }
  const loadingInstance1 = ElLoading.service({ fullscreen: true });
  try {
    const result = await deleteHdfsFiles(
      parseInt(route.params.id as string),
      multipleSelection.value.map((item) => item.path)
    );
    if (result) {
      ElMessage({
        showClose: true,
        message: "Delete success",
        type: "success",
      });
      refreshData();
      loadingInstance1.close();
    } else {
      ElMessage({
        showClose: true,
        message: "Delete failed",
        type: "error",
      });
      loadingInstance1.close();
    }
  } catch (err: any) {
    ElMessage({
      showClose: true,
      message: err.toString(),
      type: "error",
    });
    loadingInstance1.close();
  }
};

const deleteFilesForce = async () => {
  //console.log(multipleSelection.value.map((item) => item.path).join(","))
  const s = await ElMessageBox.confirm(
    "Delete files " +
      multipleSelection.value.map((item) => item.path).join(",") +
      " . Continue?",
    "Warning",
    {
      confirmButtonText: "OK",
      cancelButtonText: "Cancel",
      type: "warning",
      draggable: true,
    }
  );
  if (s != "confirm") {
    return;
  }
  const loadingInstance1 = ElLoading.service({ fullscreen: true });
  try {
    const result = await deleteHdfsFilesForce(
      parseInt(route.params.id as string),
      multipleSelection.value.map((item) => item.path)
    );
    if (result) {
      ElMessage({
        showClose: true,
        message: "Delete success",
        type: "success",
      });
      refreshData();
      loadingInstance1.close();
    } else {
      ElMessage({
        showClose: true,
        message: "Delete failed",
        type: "error",
      });
      loadingInstance1.close();
    }
  } catch (err: any) {
    ElMessage({
      showClose: true,
      message: err.toString(),
      type: "error",
    });
    loadingInstance1.close();
  }
};

//创建目录
const NewFolder = async () => {
  const folderName = await ElMessageBox.prompt(
    "Please input folder name",
    "Prompt",
    {
      confirmButtonText: "OK",
      cancelButtonText: "Cancel",
      inputPattern: /^[a-zA-Z0-9_-]{1,64}$/,
      inputErrorMessage: "Name is invalid",
    }
  );
  if (folderName.action == "confirm" && folderName.value) {
    const loadingInstance1 = ElLoading.service({ fullscreen: true });
    try {
      const result = await createHdfsFolder(
        parseInt(route.params.id as string),
        current_parent_path.value,
        folderName.value
      );
      if (result) {
        ElMessage({
          showClose: true,
          message: "Created successfully",
          type: "success",
        });
        refreshData();
        loadingInstance1.close();
      } else {
        ElMessage({
          showClose: true,
          message: "Create failed",
          type: "error",
        });
        loadingInstance1.close();
      }
    } catch (err: any) {
      ElMessage({
        showClose: true,
        message: err.toString(),
        type: "error",
      });
      loadingInstance1.close();
    }
  }
};

//创建空白文件
const NewEmptyFile = async () => {
  const fileName = await ElMessageBox.prompt(
    "Please input file name",
    "Prompt",
    {
      confirmButtonText: "OK",
      cancelButtonText: "Cancel",
      inputPattern: /^[a-zA-Z0-9_-]{1,64}$/,
      inputErrorMessage: "Name is invalid",
    }
  );
  if (fileName.action == "confirm" && fileName.value) {
    const loadingInstance1 = ElLoading.service({ fullscreen: true });
    try {
      const result = await createHdfsEmptyFile(
        parseInt(route.params.id as string),
        current_parent_path.value,
        fileName.value
      );
      if (result) {
        ElMessage({
          showClose: true,
          message: "Created successfully",
          type: "success",
        });
        refreshData();
        loadingInstance1.close();
      } else {
        ElMessage({
          showClose: true,
          message: "Create failed",
          type: "error",
        });
        loadingInstance1.close();
      }
    } catch (err: any) {
      ElMessage({
        showClose: true,
        message: err.toString(),
        type: "error",
      });
      loadingInstance1.close();
    }
  }
};

//显示文件目录权限
function convertPermissionsToSymbolic(row: HdfsFile) {
  // 将十进制数转换为八进制字符串，并确保它是3位长。
  const permissionNumber = row.permission;
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

//显示权限控制对话框
const SetPermissionsDialogVisible = ref(false);
//权限控制对话框
const hdfsPermissiionsEdit = ref<InstanceType<typeof HdfsPermissiionsEdit>>();

const hdfsPermissiionsEditInit = ref(509);
//打开窗口
const SetFilePermission = async () => {
  if (multipleSelection.value.length == 0) {
    ElMessage({
      showClose: true,
      message: "Please select files",
      type: "error",
    });
    return;
  }
  SetPermissionsDialogVisible.value = true;
  await nextTick();
  //使用选中的文件权限为初始值
  hdfsPermissiionsEditInit.value = multipleSelection.value[0].permission;
  if (hdfsPermissiionsEdit.value) {
    hdfsPermissiionsEdit.value.setPermissionsValue(
      multipleSelection.value[0].permission
    );
  }
};
//提交新的权限
const SetPermissionsSubmit = async () => {
  if (hdfsPermissiionsEdit.value) {
    const p = hdfsPermissiionsEdit.value.getNewPermission();
    const s = await ElMessageBox.confirm(
      "Change files Permissions " +
        multipleSelection.value.map((item) => item.path).join(",") +
        " . Continue?",
      "Warning",
      {
        confirmButtonText: "OK",
        cancelButtonText: "Cancel",
        type: "warning",
        draggable: true,
      }
    );
    if (s != "confirm") {
      return;
    }
    const loadingInstance1 = ElLoading.service({ fullscreen: true });
    try {
      const result = await setHdfsFilesPermissions(
        parseInt(route.params.id as string),
        multipleSelection.value.map((item) => item.path),
        p,
        hdfsPermissiionsEdit.value.recursive
      );
      if (result) {
        ElMessage({
          showClose: true,
          message: "Change success",
          type: "success",
        });
        SetPermissionsDialogVisible.value = false;
        refreshData();
        loadingInstance1.close();
      } else {
        ElMessage({
          showClose: true,
          message: "Change failed",
          type: "error",
        });
        loadingInstance1.close();
      }
    } catch (err: any) {
      ElMessage({
        showClose: true,
        message: err.toString(),
        type: "error",
      });
      loadingInstance1.close();
    }
  }
};

//设置ACLS（todo）
const SetFileAclsPermissions = async (file: HdfsFile) => {
  //打开窗口
  SetAclsDialogVisible.value = true;
  await nextTick();

  selectAclsfilePath.value = file.path;
};
//文件改名
const RenameFile = async (file: HdfsFile) => {
  const s = await ElMessageBox.prompt(
    "Rename " + file.path + " . Continue?",
    "Warning",
    {
      confirmButtonText: "OK",
      cancelButtonText: "Cancel",
      type: "warning",
      draggable: true,
    }
  );
  if (s.action != "confirm") {
    return;
  }

  const file_parent_path = file.path.substring(0, file.path.lastIndexOf("/"));
  const new_file_path = file_parent_path + "/" + s.value;
  let new_file = null;

  try {
    new_file = await getHdfsFile(
      parseInt(route.params.id as string),
      new_file_path
    );
  } catch (err: any) {}
  let overwrite = false;
  if (new_file) {
    const s2 = await ElMessageBox.confirm(
      "File " + new_file_path + " exists. Overwrite?",
      "Warning",
      {
        confirmButtonText: "OK",
        cancelButtonText: "Cancel",
        type: "warning",
        draggable: true,
      }
    );
    if (s2.action != "confirm") {
      overwrite = true;
    } else {
      return;
    }
  }

  const loadingInstance1 = ElLoading.service({ fullscreen: true });
  try {
    const result = await renameHdfsFile(
      parseInt(route.params.id as string),
      file.path,
      s.value,
      overwrite
    );
    if (result) {
      ElMessage({
        showClose: true,
        message: "Rename success",
        type: "success",
      });
      refreshData();
      loadingInstance1.close();
    } else {
      ElMessage({
        showClose: true,
        message: "Rename failed",
        type: "error",
      });
      loadingInstance1.close();
    }

    loadingInstance1.close();
  } catch (err: any) {
    ElMessage({
      showClose: true,
      message: err.toString(),
      type: "error",
    });
    loadingInstance1.close();
  }
};

//选中的acl file
const selectAclsfilePath = ref("");
//打开Acls编辑框
const SetAclsDialogVisible = ref(false);
//Acls编辑框
const hdfsFileAclsEdit=ref<InstanceType<typeof HdfsFileAclsEdit>>();
</script>

<style scoped></style>
