use std::io::ErrorKind;

use crate::get_hdfs_client;
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

use std::fs::File;
use std::io::Write;

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

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct OrcMeta {
    pub total: i64,
    pub compression_type: Option<String>,
}
//读取orc文件meta
#[tauri::command]
pub async fn get_hdfs_orc_file_meta(id: i64, file_path: String) -> Result<OrcMeta, String> {
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
    let mut compression_type ="NONE".to_owned();
    if let Some(compression) = file_meta.compression(){
        compression_type=compression.compression_type().to_string();
    }
    Ok(OrcMeta{
        total:total as i64,
        compression_type:Some(compression_type),
    })
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

            if result_columns.is_empty() {
                for field in batch.schema().fields() {
                    result_columns.push(ReadOrcResultColumn {
                        name: field.name().to_owned(),
                        data_type: field.data_type().to_string(),
                    });
                }
            }

            let mut batch_result_data: Vec<HashMap<String, String>> = vec![];
            let buf = Vec::new();
            let mut writer = arrow::json::ArrayWriter::new(buf);
            writer.write_batches(&vec![&batch]).unwrap_or_default();
            writer.finish().unwrap_or_default();
            let buf = writer.into_inner();
            let json_str = String::from_utf8(buf).unwrap();
            let json: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap_or_default();
            for item in json {
                let mut row = HashMap::new();
                if let Some(object) = item.as_object() {
                    object.iter().for_each(|(k, v)| {
                        if v.is_string() {
                            row.insert(k.to_string(), v.as_str().unwrap_or_default().to_string());
                        } else {
                            row.insert(k.to_string(), v.to_string());
                        }
                    });
                }
                batch_result_data.push(row);
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
pub async fn export_orc_file_data_to_csv(
    id: i64,
    file_path: String,
    target_csv_file_path: &str,
) -> Result<(), String> {
    let mut arrow_reader: ArrowStreamReader<HdfsOrcFileReader> =
        get_orc_reader(id, file_path, 10000).await?;
    let mut csv_file =
        File::create(target_csv_file_path).map_err(|e| format!("Failed to create file: {}", e))?;
    let mut result_columns: Vec<String> = vec![];
    loop {
        if let Some(Ok(batch)) = arrow_reader.next().await {
            if result_columns.is_empty() {
                for field in batch.schema().fields() {
                    result_columns.push(field.name().to_owned());
                }
                csv_file
                    .write(result_columns.join(",").as_bytes())
                    .map_err(|e| format!("Failed to write file: {}", e))?;
                csv_file
                    .write("\n".as_bytes())
                    .map_err(|e| format!("Failed to write file: {}", e))?;
            }
            let buf = Vec::new();
            let mut writer = arrow::json::ArrayWriter::new(buf);
            writer
                .write_batches(&vec![&batch])
                .map_err(|e| format!("Failed to write batch: {}", e))?;
            writer
                .finish()
                .map_err(|e| format!("Failed to write batch: {}", e))?;
            let buf = writer.into_inner();
            let json_str =
                String::from_utf8(buf).map_err(|e| format!("Failed to parse json: {}", e))?;
            let json: Vec<serde_json::Value> = serde_json::from_str(&json_str)
                .map_err(|e| format!("Failed to parse json: {}", e))?;
            for item in json {
                if let Some(object) = item.as_object() {
                    let row: Vec<String> = result_columns
                        .iter()
                        .map(|c| {
                            let s = object.get(c).unwrap_or(&serde_json::Value::Null);
                            let mut ss = "".to_owned();
                            if s.is_string() {
                                ss = s.as_str().unwrap_or_default().to_owned().replace("\"", "\"\"");
                            } else {
                                ss = s.to_string().replace("\"", "\"\"");
                            }
                            if ss.contains(",") {
                                format!("\"{}\"", &ss)
                            } else {
                                ss
                            }
                        })
                        .collect();
                    csv_file
                        .write(row.join(",").as_bytes())
                        .map_err(|e| format!("Failed to write file: {}", e))?;
                    csv_file
                        .write("\n".as_bytes())
                        .map_err(|e| format!("Failed to write file: {}", e))?;
                }
            }
        } else {
            break;
        }
    }
    csv_file
        .flush()
        .map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}
