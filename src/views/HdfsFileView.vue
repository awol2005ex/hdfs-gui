<template>
  <div class="common-layout">
    <el-container>
      <el-header>
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
            title="Back To Last Page" /><el-button
            type="primary"
            :icon="Download"
            circle
            @click="DownloadFile"
            title="Download"
        /></el-button-group>

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
                v-if="!item.last"
                :to="{
                  path: '/HdfsFolderView/' + route.params.id,
                  query: { path: item.path },
                }"
                >{{ item.name }}</el-breadcrumb-item
              >
              <el-breadcrumb-item v-if="item.last">{{
                item.name
              }}</el-breadcrumb-item>
            </template>
          </template>
        </el-breadcrumb>
      </el-header>
      <el-main>
        <div v-if="mode == 'byte_preview'">
          
          <HdfsByteFileView
            :hdfsConfigId="parseInt(route.params.id[0])"
            :filePath="route.query.path?.toString()"
          />
        </div>
        <div v-if="mode == 'text_edit'">
          
          <HdfsTextFileEdit
            :hdfsConfigId="parseInt(route.params.id[0])"
            :filePath="route.query.path?.toString()"
          />
        </div>
      </el-main>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { useRouter, useRoute } from "vue-router";
import { Back, HomeFilled, Download } from "@element-plus/icons-vue";

import HdfsByteFileView from "../components/HdfsByteFileView.vue";
import HdfsTextFileEdit from "../components/HdfsTextFileEdit.vue";
import { download_file } from "../api/hdfs_file";
//选择文件
import { open } from "@tauri-apps/plugin-dialog";
import { ElMessage,ElLoading  } from "element-plus";

const router = useRouter();
const route = useRoute();



const mode = ref("byte_preview");
if(route.query.mode){
     mode.value =route.query.mode.toString() ;
} else{
     mode.value = "byte_preview";
}
//返回首页
const backToHome = () => {
  router.push("/");
};
//返回历史上一页
const backToLastPage = () => {
  router.go(-1);
};

//当前路径分解点击
const get_file_path_separator = (path: string) => {
  if (path == "/") {
    return [{ path: "/", name: "/", last: false }];
  }
  const paths = path.split("/");

  return paths.map((item, index) => {
    return {
      path: index == 0 ? "/" : paths.slice(0, index + 1).join("/"),
      name: item,
      last: index == paths.length - 1,
    };
  });
};
//当前路径分解点击
const current_parent_paths = ref(
  get_file_path_separator(route.query.path ? (route.query.path as string) : "/")
);
//下载文件
const DownloadFile = async () => {
  const selected = await open({
    multiple: false,
    directory: true,
  });

  const loadingInstance1 = ElLoading.service({ fullscreen: true })
  try {
    const b = await download_file(
      parseInt(route.params.id[0]),
      route.query.path as string,
      selected || ""
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
    loadingInstance1.close()
  } catch (error: any) {
    ElMessage({
      showClose: true,
      message: error.toString(),
      type: "error",
    });
    loadingInstance1.close()
  }
};

watch(route, (to, from) => {
  if(route.query.mode){
     mode.value =route.query.mode.toString() ;
  } else{
     mode.value = "byte_preview";
  }
})
</script>

<style scoped></style>
