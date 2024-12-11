<template>
   <div class="common-layout">
    <el-container>
        <el-main>
        <div class="flex flex-wrap gap-4">
          <el-card style="width:480px" shadow="always" v-for="item in hdfsConfigList" :key="item.id">
          {{item.name}}
          </el-card>
        </div>
      </el-main>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { reactive, Ref, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

//HDFS配置列表
const hdfsConfigList = ref([]);
</script>

<style scoped>

</style>
