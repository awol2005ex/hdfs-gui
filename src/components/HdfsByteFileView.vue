<template>
  <el-container>
    <el-header height="40">
      <p>
        <span style="float: left">Preview File Bytes Content</span>

        <el-button-group class="ml-4" style="float: left; margin-left: 20px">
          <el-button
            v-if="fileSize < 5 * 1024 * 1024"
            type="primary"
            :icon="Edit"
            circle
            @click="EditTextFile"
            title="Edit By Text Editor"
          />
          <el-button
            v-if="isOrc"
            type="primary"
            @click="OrcView"
            title="Orc View"
            >Orc View</el-button
          >
        </el-button-group>
      </p>
    </el-header>
    <el-main>
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
    </el-main>
  </el-container>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import type { CSSProperties, Ref } from "vue";
import { Codemirror } from "vue-codemirror";
import { vue } from "@codemirror/lang-vue";
import { oneDark } from "@codemirror/theme-one-dark";
import { get_file_preview_content } from "../api/hdfs_file.ts";
import { ElMessage } from "element-plus";
import { Edit } from "@element-plus/icons-vue";
import { useRouter, useRoute } from "vue-router";
const router = useRouter();
const route = useRoute();

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
//是否orc
const isOrc: Ref<Boolean, Boolean> = ref(false);

const reloadFile = async () => {
  try {
    if (props.hdfsConfigId > 0 && props.filePath != "") {
      const previewResult = await get_file_preview_content(
        props.hdfsConfigId as number,
        props.filePath as string
      );
      //自动跳转到orc查看模式
      if (previewResult.isorc) {
        router.replace({
          path: "/HdfsFileView/" + route.params.id,
          query: {
            path: props.filePath,
            mode: "orc",
          },
        });
        return;
      }
      //自动跳转到parquet查看模式
      if (previewResult.isparquet) {
        router.replace({
          path: "/HdfsFileView/" + route.params.id,
          query: {
            path: props.filePath,
            mode: "parquet",
          },
        });
        return;
      }

      //自动跳转到avro查看模式
      if (previewResult.isavro) {
        router.replace({
          path: "/HdfsFileView/" + route.params.id,
          query: {
            path: props.filePath,
            mode: "avro",
          },
        });
        return;
      }

      codeValue.value = previewResult.content;
      fileSize.value = previewResult.length;
      isOrc.value = previewResult.isorc;
    }
  } catch (error: any) {
    ElMessage({
      showClose: true,
      message: error.toString(),
      type: "error",
    });
  }
};
watch(
  () => props.hdfsConfigId,
  (_newId) => {
    reloadFile();
  }
);
reloadFile();
//文本编辑
const EditTextFile = () => {
  router.push({
    path: "/HdfsFileView/" + route.params.id,
    query: {
      path: props.filePath,
      mode: "text_edit",
    },
  });
};
//orc文件查看
const OrcView = () => {
  router.push({
    path: "/HdfsFileView/" + route.params.id,
    query: {
      path: props.filePath,
      mode: "orc",
    },
  });
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
