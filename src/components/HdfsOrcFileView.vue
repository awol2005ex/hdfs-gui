<template>
  <el-container>
    <el-header height="40">
      <el-button-group style="float: left; margin-left: 20px">
        <el-button @click="router.go(-1)">Back</el-button>
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
          v-for="(field, index) in fields"
          :key="field.name"
          :prop="field.name"
          :label="field.name"
          width="auto"
        ></el-table-column>
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
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { useRouter } from "vue-router";
import { get_hdfs_orc_file_field_list, get_hdfs_orc_file_rows_count, OrcField } from "../api/hdfs_orc";
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
const data = ref([]);
//每页条数
const pageSize = ref(10);
//总条数
const total = ref(0);
//当前页
const currentPage = ref(1);

const handleCurrentChange = async (val: number) => {
  currentPage.value = val;

  await read_orc_file_data_by_page(
    props.hdfsConfigId as number,
    props.filePath as string,
    pageSize.value,
    currentPage.value
  );
};
const handleSizeChange = async (val: number) => {
  pageSize.value = val;
  await read_orc_file_data_by_page(
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
  //数据行数

  total.value = await get_hdfs_orc_file_rows_count(
    props.hdfsConfigId as number,
    props.filePath as string
  );
  //按页读取数据
  await read_orc_file_data_by_page(
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
const read_orc_file_data_by_page = async (
  currentHdfsConfigId: number,
  readFilePath: string,
  readPageSize: number,
  readCurrentPage: number
) => {};
</script>
<style lang="less" scoped></style>
