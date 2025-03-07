<template>
  <el-container>
    <el-header height="40">
      <el-button-group style="float: left; margin-left: 20px">
        <el-button type="primary" @click="saveFile">Save</el-button>
        <el-button @click="router.go(-1)">Back</el-button>
      </el-button-group>
    </el-header>
    <el-main>
      <Codemirror
        v-model="codeValue"
        :style="codeStyle"
        :extensions="extensions"
        v-bind="$attrs"
        :disabled="false"
        @ready="handleReady"
        @change="onChange"
        @focus="onFocus"
        @blur="onBlur"
      />
    </el-main>
  </el-container>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import type { CSSProperties } from "vue";
import { Codemirror } from "vue-codemirror";
import { vue } from "@codemirror/lang-vue";
import { oneDark } from "@codemirror/theme-one-dark";
import { get_file_content, writeTextToHdfsFile } from "../api/hdfs_file.ts";
import { ElMessage, ElLoading } from "element-plus";

import { useRouter } from "vue-router";
const router = useRouter();

interface Props {
  codeStyle?: CSSProperties; // 代码样式
  dark?: boolean; // 是否暗黑主题
  code?: string; // 代码字符串
  hdfsConfigId?: number;
  filePath?: string;
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
  code: "",
  hdfsConfigId: 0,
  filePath: "",
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
//文件大小
const fileSize = ref(0);

const reloadFile = async () => {
  const loadingInstance1 = ElLoading.service({ fullscreen: true });
  try {
    if (props.hdfsConfigId > 0 && props.filePath != "") {
      const result = await get_file_content(
        props.hdfsConfigId as number,
        props.filePath as string
      );

      codeValue.value = result.content;
      fileSize.value = result.length;
    }
  } catch (error: any) {
    ElMessage({
      showClose: true,
      message: error.toString(),
      type: "error",
    });
  }
  loadingInstance1.close();
};
watch(
  () => props.hdfsConfigId,
  (_newId) => {
    reloadFile();
  }
);
reloadFile();

const saveFile = async () => {
  const loadingInstance1 = ElLoading.service({ fullscreen: true });
  try {
    if (props.hdfsConfigId > 0 && props.filePath != "") {
      const result = await writeTextToHdfsFile(
        props.hdfsConfigId as number,
        props.filePath as string,
        codeValue.value
      );
      if (result) {
        ElMessage({
          showClose: true,
          message: "Save file success",
          type: "success",
        });
      } else {
        ElMessage({
          showClose: true,
          message: "Save file failed",
          type: "error",
        });
      }
    }
  } catch (error: any) {
    ElMessage({
      showClose: true,
      message: error.toString(),
      type: "error",
    });
  }
  loadingInstance1.close();
};
</script>
<style lang="less" scoped>
:deep(.cm-editor) {
  border-radius: 8px;
  outline: none;
  border: 1px solid transparent;
  .cm-scroller {
    border-radius: 8px;
  }
}
:deep(.cm-focused) {
  border: 1px solid fade(#000, 48%);
}
</style>
