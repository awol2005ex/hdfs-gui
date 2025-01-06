<template>
  <el-container>
    <el-header height="40">
      <el-button-group style="float: left; margin-left: 20px">
        <el-button @click="router.go(-1)">Back</el-button>
        <el-button @click="openStruct">show struct</el-button>
        <el-button @click="exportCsv">export to csv</el-button>
        
      </el-button-group>
    </el-header>
    <el-main>
      <el-table
        style="width: 100%"
        min-height="240"
        border
        fit
        :data="data"
        :header-cell-style="{ background: '#f5f7fa', color: '#606266' }"
      >
        <el-table-column label="__rowindex" width="150px">
          <template #default="scope">
            {{ scope.$index + 1 + pageSize * (currentPage - 1) }}
          </template>
        </el-table-column>
        <el-table-column
          v-for="(field, _index) in fields"
          :key="field.name"
          :prop="field.name"
          :label="field.name"
          width="auto"
        >
          <template #header>
            <span :title="field.type_name.toString()"> {{ field.name }}</span>
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

  <el-drawer v-model="strunctdrawer" title="Struct">
    {{ orc_struct }}
  </el-drawer>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { useRouter } from "vue-router";
import {
  get_hdfs_orc_file_field_list,
  get_hdfs_orc_file_rows_count,
  OrcField,
  read_orc_file_data_by_page,
  DataRow,
  export_orc_file_date_to_csv,
} from "../api/hdfs_orc";
import { ElLoading, ElMessage } from "element-plus";
import { save } from "@tauri-apps/plugin-dialog";
const router = useRouter();

interface Props {
  hdfsConfigId?: number;
  filePath?: string;
}
const props = withDefaults(defineProps<Props>(), {
  hdfsConfigId: 0,
  filePath: "",
});
//字段列表
const fields = ref<OrcField[]>([]);
//数据
const data = ref<DataRow[]>([]);
//每页条数
const pageSize = ref(10);
//总条数
const total = ref(0);
//当前页
const currentPage = ref(1);
const orc_struct = ref("");
const strunctdrawer = ref(false);
const openStruct = async () => {
  strunctdrawer.value = !strunctdrawer.value;
};
const handleCurrentChange = async (val: number) => {
  currentPage.value = val;

  await read_orc_file_data_by_page_func(
    props.hdfsConfigId as number,
    props.filePath as string,
    pageSize.value,
    currentPage.value
  );
};
const handleSizeChange = async (val: number) => {
  pageSize.value = val;
  await read_orc_file_data_by_page_func(
    props.hdfsConfigId as number,
    props.filePath as string,
    pageSize.value,
    currentPage.value
  );
};

//重新加载文件
const reloadFile = async () => {
  //字段列表
  fields.value = await get_hdfs_orc_file_field_list(
    props.hdfsConfigId as number,
    props.filePath as string
  );
  orc_struct.value= JSON.stringify(fields.value);
  //数据行数

  total.value = await get_hdfs_orc_file_rows_count(
    props.hdfsConfigId as number,
    props.filePath as string
  );
  //按页读取数据
  await read_orc_file_data_by_page_func(
    props.hdfsConfigId as number,
    props.filePath as string,
    pageSize.value,
    currentPage.value
  );
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
//按页读取数据
const read_orc_file_data_by_page_func = async (
  currentHdfsConfigId: number,
  readFilePath: string,
  readPageSize: number,
  readCurrentPage: number
) => {
  const loadingInstance1 = ElLoading.service({ fullscreen: true });
  try {
    data.value = await read_orc_file_data_by_page(
      currentHdfsConfigId,
      readFilePath,
      readPageSize,
      readCurrentPage
    );
  } catch (error: any) {
    ElMessage({
      showClose: true,
      message: error.toString(),
      type: "error",
    });
  }
  loadingInstance1.close();
};



const exportCsv = async () => {
  const selected = await save({
    filters: [
      {
        name: "Csv Files",
        extensions: ["csv"],
      },
    ],
    defaultPath: props.filePath.replace("\\","/").split("/").pop()+".csv",
  });
  if (selected) {
    const loadingInstance1 = ElLoading.service({ fullscreen: true });
    await export_orc_file_date_to_csv(
      props.hdfsConfigId as number,
      props.filePath as string,
      selected,
    );
    loadingInstance1.close();
  }
};
</script>
<style lang="less" scoped></style>
