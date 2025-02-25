<template>
  <el-container>
    <el-header height="40">
      <el-button-group style="float: left; margin-left: 20px">
        <el-button @click="router.go(-1)">Back</el-button>
      </el-button-group>
      <span style="float: left; margin-left: 20px">AVRO View</span>
    </el-header>
    <el-main> 
    
      <JsonViewer :value="content" copyable boxed sort theme="jv-dark"/>
    </el-main>
  </el-container>
</template>

<script setup lang="ts">
import { Ref, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { ElLoading, ElMessage } from "element-plus";

import { get_avro_content } from "../api/hdfs_avro";

const router = useRouter();

interface Props {
  hdfsConfigId?: number;
  filePath?: string;
}
const props = withDefaults(defineProps<Props>(), {
  hdfsConfigId: 0,
  filePath: "",
});

const content: Ref<any[], any[]> = ref([]);
//重新加载文件
const reloadFile = async () => {
  const loadingInstance1 = ElLoading.service({ fullscreen: true });
  try{
  const result: any[] = await get_avro_content(
    props.hdfsConfigId as number,
    props.filePath as string
  );
  content.value = result;
  }catch(error:any){
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
    //重新加载文件
    reloadFile();
  }
);
//重新加载文件
reloadFile();
</script>
<style lang="less" scoped></style>
