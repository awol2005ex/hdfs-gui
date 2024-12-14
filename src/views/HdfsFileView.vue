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
            title="Back To Last Page"
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
          <p>Preview File Bytes Content</p>
          <HdfsByteFileView  :hdfsConfigId="route.params.id" :filePath="route.query.path" />
        </div>
      </el-main>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useRouter, useRoute } from "vue-router";
import {
  Back,
  HomeFilled,
} from "@element-plus/icons-vue";




import {ElMessage  } from "element-plus";
import HdfsByteFileView from "../components/HdfsByteFileView.vue";


const router = useRouter();
const route = useRoute();

const mode=ref("byte_preview")
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

const current_parent_paths = ref(
  get_file_path_separator(route.query.path ? (route.query.path as string) : "/")
);

</script>

<style scoped>

</style>
