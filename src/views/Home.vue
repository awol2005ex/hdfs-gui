<template>
  <div class="common-layout">
    <el-container>
      <el-main>
        <div class="flex flex-wrap gap-4">
          <el-card
            style="width: 480px;margin-top: 20px"
            shadow="always"
            v-for="item in hdfsConfigList"
            :key="item.id"
          >
           <table>
             <tr>
               <td>{{ item.name }}</td>
               <td><el-button type="primary" :icon="EditPen" circle  @click="editHdfsConfig(item.id||0)"  title="Edit"/></td>
               <td><el-button type="danger" :icon="Delete" circle  @click="deleteHdfsConfig(item.id||0)" title="Delete"/></td>
               <td><el-button type="primary" :icon="Connection" circle  @click="connectToHdfs(item.id||0)"  title="Connect"/></td>
               
               
             </tr>
           </table>
            
          </el-card>
          <el-card style="width: 480px;margin-top: 20px" shadow="always">
            <el-button type="primary" :icon="DocumentAdd" circle  @click="addHdfsConfig"/>
          </el-card>
        </div>
      </el-main>
    </el-container>
  </div>


  <el-dialog
    v-model="AddHdfsConfigDialogVisible"
    title="Add Hdfs Config"
    width="500"
  >
    <HdfsConfigForm ref="hdfsConfigForm"/>
    <template #footer>
      <div class="dialog-footer">
        <el-button @click="AddHdfsConfigDialogVisible = false">Cancel</el-button>
        <el-button type="primary" @click="AddHdfsConfigConfirm">
          Confirm
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { reactive, Ref, ref,nextTick  } from "vue";
import { HdfsConfig, getHdfsConfigList,saveHdfsConfig,getHdfsConfig } from "../api/hdfs_config.ts";
import HdfsConfigForm from "../components/HdfsConfigForm.vue";
import { DocumentAdd, EditPen, Delete,Connection } from "@element-plus/icons-vue";
import { useRouter, useRoute } from 'vue-router'
import { ElMessage } from "element-plus";
const router = useRouter()
const route = useRoute()

//HDFS配置列表
const hdfsConfigList: Ref<Array<HdfsConfig>> = ref([]);
//初始化已保存的HDFS配置列表
getHdfsConfigList().then((res) => {
  hdfsConfigList.value = res;
});
//打开新建窗口
const AddHdfsConfigDialogVisible:Ref<Boolean> = ref(false)

const addHdfsConfig = () => {
  AddHdfsConfigDialogVisible.value = true
}

const hdfsConfigForm = ref<InstanceType<typeof HdfsConfigForm>>()
//保存
const AddHdfsConfigConfirm = () => {
  saveHdfsConfig(hdfsConfigForm.value?.hdfsConfigForm||{}).then((res) => {
    AddHdfsConfigDialogVisible.value = false
  }).catch((err) => {
    ElMessage({
        showClose: true,
        message: err.toString(),
        type: "error",
      });
  })
  
}

const editHdfsConfig = async (id:number) => {
  AddHdfsConfigDialogVisible.value = true
  await nextTick();
  getHdfsConfig(id).then((res) => {
    
   if(hdfsConfigForm.value) {
    hdfsConfigForm.value.setHdfsConfigForm(res)
   }
  }).catch((err) => {
    ElMessage({
        showClose: true,
        message: err.toString(),
        type: "error",
      });
  })
}
const deleteHdfsConfig = (id:number) => {
  console.log(id)
}
const connectToHdfs = (id:number) => {
  console.log(id)
  
  router.push('/HdfsFolderView/'+id)
}
</script>

<style scoped></style>
