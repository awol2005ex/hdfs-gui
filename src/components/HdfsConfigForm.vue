<template>
 <el-form :model="hdfsConfigForm" label-width="150px" size="small" @submit.native.prevent>
    <el-form-item label="Name:">
      <input  
        style="width: 300px"
        clearable
        v-model="hdfsConfigForm.name"
      />
    </el-form-item>
    <el-form-item label="Hdfs Url:">
      <input  
        style="width: 300px"
        clearable
        v-model="hdfsConfigForm.hdfs_url"
      />
    </el-form-item>
    <el-form-item label="Hdfs Config:">
      <textarea  
        style="width: 300px"
        clearable
        v-model="hdfsConfigForm.hdfs_config"
      />
    </el-form-item>
  </el-form>
</template>

<script setup lang="ts">
import { Reactive, reactive, defineExpose } from "vue";
import { HdfsConfig } from "../api/hdfs_config.ts";

const hdfsConfigForm :Reactive<HdfsConfig> = reactive({ 
  id: 0,  
  name: '',
  hdfs_config: '{}',
  hdfs_url: '',
  del_flag:0,
 });


const setHdfsConfigForm = (hdfsConfig: HdfsConfig) => {
  hdfsConfigForm.id = hdfsConfig.id;
  hdfsConfigForm.name = hdfsConfig.name;
  hdfsConfigForm.hdfs_config = hdfsConfig.hdfs_config;
  hdfsConfigForm.hdfs_url = hdfsConfig.hdfs_url;
}
defineExpose({
  hdfsConfigForm,setHdfsConfigForm
})

</script>

<style scoped></style>