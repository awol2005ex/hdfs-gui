<template>
  <div class="common-layout">
    <el-container>
      <el-header>
        <el-button
          type="primary"
          :icon="HomeFilled"
          circle
          @click="backToHome"
          title="Back To Home"
          style="float: left"
        />
        <el-button
          type="primary"
          :icon="Back"
          circle
          @click="backToLastPage"
          title="Back To Last Page"
          style="float: left"
        />
        <el-button
          type="primary"
          :icon="Refresh"
          circle
          @click="refreshData"
          title="Refresh"
          style="float: left"
        />

        <el-breadcrumb separator="/" style="float: left; padding-left: 10px">
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

        <el-input
          v-model="search_words"
          style="width: 240px; float: left; padding-left: 10px"
          placeholder="Search File"
          :prefix-icon="Search"
          @change="on_search_words_change"
          clearable
        />
      </el-header>
      <el-main>
        <el-table
          :data="fileListPageData"
          style="width: 100%"
          @sort-change="sortChange"
        >
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
            width="400"
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
            width="120"
            show-overflow-tooltip
            sortable="custom"
          />
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
</template>

<script setup lang="ts">
import { reactive, Ref, ref, watch } from "vue";
import { useRouter, useRoute } from "vue-router";
import {
  Back,
  Refresh,
  Folder,
  Document,
  HomeFilled,
  Search,
} from "@element-plus/icons-vue";
import { getHdfsFileList, HdfsFile } from "../api/hdfs_file.ts";
import { ElMessage } from "element-plus";
const router = useRouter();
const route = useRoute();

//console.log(route.params);
//console.log(route.query);
//返回首页
const backToHome = () => {
  router.push("/");
};
const backToLastPage = () => {
  router.go(-1);
};

const current_parent_path = ref(
  route.query.path ? (route.query.path as string) : "/"
);

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

const refreshData = () => {
  getHdfsFileList(
    parseInt(route.params.id as string),
    current_parent_path.value
  )
    .then((res) => {
      fileListData.value = res;
      filert_by_search_words();
    })
    .catch((err) => {
      ElMessage({
        showClose: true,
        message: err.toString(),
        type: "error",
      });
    });
};

refreshData();

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
const goToFile = (row: HdfsFile) => {};
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
  const { column, prop, order } = row;
  console.log(column, prop, order);
  console.log("column", column);
  console.log("prop", prop);
  console.log("order", order);

  sortProp.value = prop;
  sortOrder.value = order;
  filert_by_search_words();
};
</script>

<style scoped></style>
