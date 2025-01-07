use std::io::ErrorKind;

use crate::get_hdfs_client;
use arrow::util::display::{ArrayFormatter, FormatOptions};
use bytes::Bytes;
use futures::StreamExt;
use futures::TryFutureExt;
use hdfs_native::file::FileReader;
use orc_rust::{
    reader::{
        metadata::{read_metadata_async, FileMetadata},
        AsyncChunkReader,
    },
    ArrowReaderBuilder, ArrowStreamReader,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use arrow::csv;
use std::fs::File;

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct OrcField {
    pub name: String,
    pub type_name: String,
}

pub struct HdfsOrcFileReader(FileReader);
impl AsyncChunkReader for HdfsOrcFileReader {
    fn len(&mut self) -> futures::future::BoxFuture<'_, std::io::Result<u64>> {
        Box::pin(futures::future::ready(Ok(self.0.file_length() as u64)))
    }

    fn get_bytes(
        &mut self,
        offset_from_start: u64,
        length: u64,
    ) -> futures::future::BoxFuture<'_, std::io::Result<Bytes>> {
        //println!("get_bytes: offset_from_start:{},length:{}--start", &offset_from_start,&length);
        let mut buf_len = length;
        if offset_from_start + length > self.0.file_length() as u64 {
            buf_len = self.0.file_length() as u64 - offset_from_start;
        }
        return Box::pin(
            self.0
                .read_range(offset_from_start as usize, buf_len as usize)
                .map_err(|e| std::io::Error::new(ErrorKind::Other, e)),
        );
    }
}

///获取orc文件reader
pub async fn get_orc_reader(
    id: i64,
    file_path: String,
    batch_size: usize,
) -> Result<ArrowStreamReader<HdfsOrcFileReader>, String> {
    let hdfs_client = get_hdfs_client(id).await?;
    let hdfs_file_reader = hdfs_client
        .read(&file_path)
        .await
        .map_err(|e| e.to_string())?;
    let arrow_reader: ArrowStreamReader<HdfsOrcFileReader> =
        ArrowReaderBuilder::try_new_async(HdfsOrcFileReader(hdfs_file_reader))
            .await
            .map_err(|e| e.to_string())?
            .with_batch_size(batch_size)
            .build_async();
    return Ok(arrow_reader);
}

////获取orc文件字段列表
#[tauri::command]
pub async fn get_hdfs_orc_file_field_list(
    id: i64,
    file_path: String,
) -> Result<Vec<OrcField>, String> {
    let arrow_reader: ArrowStreamReader<HdfsOrcFileReader> =
        get_orc_reader(id, file_path, 1).await?;
    let schema = arrow_reader.schema();
    println!("schema:{:?}", schema);
    let mut field_list = vec![];
    for field in schema.fields() {
        let orc_field = OrcField {
            name: field.name().to_string(),
            type_name: field.data_type().to_string(),
        };
        field_list.push(orc_field);
    }
    Ok(field_list)
}

////获取orc文件总行数
#[tauri::command]
pub async fn get_hdfs_orc_file_rows_count(id: i64, file_path: String) -> Result<u64, String> {
    let hdfs_client = get_hdfs_client(id).await?;
    let hdfs_file_reader = hdfs_client
        .read(&file_path)
        .await
        .map_err(|e| e.to_string())?;
    let mut orc_reader = HdfsOrcFileReader(hdfs_file_reader);

    let file_meta: FileMetadata = read_metadata_async::<HdfsOrcFileReader>(&mut orc_reader)
        .await
        .map_err(|e| e.to_string())?;

    let total = file_meta.number_of_rows();
    Ok(total)
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ReadOrcResultColumn {
    pub name: String,
    pub data_type: String,
}
//分页读取orc文件数据
#[tauri::command]
pub async fn read_orc_file_data_by_page(
    id: i64,
    file_path: String,
    page_size: usize,
    page_number: usize,
) -> Result<Vec<HashMap<String, String>>, String> {
    let arrow_reader: ArrowStreamReader<HdfsOrcFileReader> =
        get_orc_reader(id, file_path, page_size).await?;

    let mut result_data: Vec<HashMap<String, String>> = vec![];
    let mut result_columns: Vec<ReadOrcResultColumn> = vec![];

    let mut it = arrow_reader.skip(page_number - 1);
    match it.next().await {
        Some(Ok(batch)) => {
            let options = FormatOptions::default().with_display_error(true);

            if result_columns.is_empty() {
                for field in batch.schema().fields() {
                    result_columns.push(ReadOrcResultColumn {
                        name: field.name().to_owned(),
                        data_type: field.data_type().to_string(),
                    });
                }
            }

            let mut batch_result_data: Vec<HashMap<String, String>> = vec![];
            let batch_size = batch.num_rows();
            for (columnindex, column) in batch.columns().into_iter().enumerate() {
                let formatter_result = ArrayFormatter::try_new(column.as_ref(), &options);
                if formatter_result.is_ok() {
                    let formatter = formatter_result.unwrap();
                    for rowindex in 0..batch_size {
                        
                        let value = formatter.value(rowindex);
                        if rowindex >= batch_result_data.len() {
                            let mut row = HashMap::new();
                            row.insert(result_columns[columnindex].name.clone(), value.to_string());
                            batch_result_data.push(row);
                        } else {
                            batch_result_data[rowindex].insert(
                                result_columns[columnindex].name.clone(),
                                value.to_string(),
                            );
                        }
                    }
                }
            }
            result_data.append(&mut batch_result_data);
        }
        Some(Err(e)) => {
            return Err(e.to_string());
        }
        None => {}
    }
    return Ok(result_data);
}

//导出orc文件数据到csv文件
#[tauri::command]
pub async fn export_orc_file_date_to_csv(
    id: i64,
    file_path: String,
    target_csv_file_path: &str,
) -> Result<(), String> {
    let mut arrow_reader: ArrowStreamReader<HdfsOrcFileReader> =
        get_orc_reader(id, file_path, 10000).await?;

    loop {
        if let Some(Ok(batch)) = arrow_reader.next().await {
            let csv_file = File::create(target_csv_file_path)
                .map_err(|e| format!("Failed to create file: {}", e))?;
            let mut writer =  csv::WriterBuilder::new().with_delimiter(b',').with_show_nested(true).build(csv_file);

            let _ = writer
                .write(&batch)
                .map_err(|e| format!("Failed to write file: {}", e))?;
        } else {
            break;
        }
    }
    Ok(())
}
