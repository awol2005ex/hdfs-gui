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
        <div>
          <p>Preview File Bytes Content</p>
          <Codemirror
            v-model="codeValue"
            :style="codeStyle"
            :extensions="extensions"
            v-bind="$attrs"
            :disabled="true"
            @ready="handleReady"
            @change="onChange"
            @focus="onFocus"
            @blur="onBlur"
          />
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

import type { CSSProperties } from "vue";
import { Codemirror } from "vue-codemirror";
import { vue } from "@codemirror/lang-vue";
import { oneDark } from "@codemirror/theme-one-dark";


import { get_file_preview_content } from "../api/hdfs_file.ts";
import {ElMessage  } from "element-plus";

interface Props {
  codeStyle?: CSSProperties; // 代码样式
  dark?: boolean; // 是否暗黑主题
  code?: string; // 代码字符串
  // placeholder?: string // 占位文本
  // autofocus?: boolean // 自动聚焦
  // disabled?: boolean // 禁用输入行为和更改状态
  // indentWithTab?: boolean // 启用tab按键
  // tabSize?: number // tab按键缩进空格数
}
const props = withDefaults(defineProps<Props>(), {
  // placeholder: 'Code goes here...',
  codeStyle: () => {
    return {};
  },
  dark: false,
  // autofocus: false,
  // disabled: false,
  // indentWithTab: true,
  // tabSize: 2
});
const extensions = props.dark ? [vue(), oneDark] : [vue()];
const codeValue = ref("");
const emits = defineEmits(["update:code", "ready", "change", "focus", "blur"]);
function handleReady(payload: any) {
  // console.log('ready')
  emits("ready", payload);
}
function onChange(value: string, viewUpdate: any) {
  emits("change", value, viewUpdate);
  emits("update:code", value);
}
function onFocus(viewUpdate: any) {
  emits("focus", viewUpdate);
}
function onBlur(viewUpdate: any) {
  emits("blur", viewUpdate);
}

const router = useRouter();
const route = useRoute();

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

const reloadFile= async () => {
    try{
         const content = await get_file_preview_content( parseInt(route.params.id as string) ,(route.query.path as string));
   
         codeValue.value = content;
     }catch (error:any) {
        ElMessage({
          showClose: true,
          message: error.toString(),
          type: "error",
        });

    }
}

reloadFile();
</script>

<style lang="less" scoped>
:deep( .cm-editor) {
  border-radius: 8px;
  outline: none;
  border: 1px solid transparent;
  .cm-scroller {
    border-radius: 8px;
  }
}
:deep( .cm-focused ){
  border: 1px solid fade(#000, 48%);
}
</style>
