use std::io::ErrorKind;
use std::sync::Arc;

use crate::get_hdfs_client;
use bytes::Bytes;
use futures::StreamExt;
use futures::TryFutureExt;
use hdfs_native::file::FileReader;
use parquet::arrow::async_reader::AsyncFileReader;
use parquet::arrow::async_reader::MetadataFetch;
use parquet::arrow::async_reader::ParquetRecordBatchStream;
use parquet::errors::ParquetError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::fs::File;
use std::io::Write;

use parquet::arrow::ParquetRecordBatchStreamBuilder;
use parquet::errors::Result;
use parquet::file::metadata::ParquetMetaDataReader;

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct ParquetField {
    pub name: String,
    pub type_name: String,
}

pub struct HdfsParquetFileReader(FileReader);

impl MetadataFetch for HdfsParquetFileReader {
    fn fetch(
        &mut self,
        range: std::ops::Range<usize>,
    ) -> futures::future::BoxFuture<'_, Result<Bytes>> {
        Box::pin(async move {
            return self
                .0
                .read_range(range.start as usize, (range.end - range.start) as usize)
                .await
                .map_err(|e| ParquetError::ArrowError(e.to_string()));
        })
    }
}

impl AsyncFileReader for HdfsParquetFileReader {
    fn get_bytes(
        &mut self,
        range: std::ops::Range<usize>,
    ) -> futures::future::BoxFuture<'_, Result<Bytes>> {
        Box::pin(async move {
            return self
                .0
                .read_range(range.start as usize, (range.end - range.start) as usize)
                .await
                .map_err(|e| ParquetError::ArrowError(e.to_string()));
        })
    }

    fn get_metadata(
        &mut self,
    ) -> futures::future::BoxFuture<
        '_,
        Result<std::sync::Arc<parquet::file::metadata::ParquetMetaData>>,
    > {
        let file_size = self.0.file_length();

        Box::pin(async move {
            let reader = ParquetMetaDataReader::new();
            match reader.load_and_finish(self, file_size).await {
                Ok(meta_data) => Ok(Arc::new(meta_data)),
                Err(e) => Err(ParquetError::ArrowError(e.to_string())),
            }
        })
    }
}

///获取parquet文件reader
pub async fn get_parquet_reader(
    id: i64,
    file_path: String,
    batch_size: usize,
) -> Result<ParquetRecordBatchStream<HdfsParquetFileReader>, String> {
    let hdfs_client = get_hdfs_client(id).await?;
    let hdfs_file_reader = hdfs_client
        .read(&file_path)
        .await
        .map_err(|e| e.to_string())?;

    ParquetRecordBatchStreamBuilder::new(HdfsParquetFileReader(hdfs_file_reader))
        .await
        .map_err(|e| e.to_string())?
        .with_batch_size(batch_size)
        .build()
        .map_err(|e| e.to_string())
}

////获取parquet文件字段列表
#[tauri::command]
pub async fn get_hdfs_parquet_file_field_list(
    id: i64,
    file_path: String,
) -> Result<Vec<ParquetField>, String> {
    let arrow_reader: ParquetRecordBatchStream<HdfsParquetFileReader> =
        get_parquet_reader(id, file_path, 1).await?;
    let schema = arrow_reader.schema();
    println!("schema:{:?}", schema);
    let mut field_list = vec![];
    for field in schema.fields() {
        let parquet_field = ParquetField {
            name: field.name().to_string(),
            type_name: field.data_type().to_string(),
        };
        field_list.push(parquet_field);
    }
    Ok(field_list)
}

////获取parquet文件总行数
#[tauri::command]
pub async fn get_hdfs_parquet_file_rows_count(id: i64, file_path: String) -> Result<i64, String> {
    let hdfs_client = get_hdfs_client(id).await?;
    let hdfs_file_reader = hdfs_client
        .read(&file_path)
        .await
        .map_err(|e| e.to_string())?;
    let mut parquet_reader = HdfsParquetFileReader(hdfs_file_reader);

    let meta = parquet_reader
        .get_metadata()
        .await
        .map_err(|e| e.to_string())?;
    let file_meta = meta.file_metadata();
    let total = file_meta.num_rows();
    Ok(total)
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct ParquetMeta {
    pub total: i64,
    pub compression_type: Option<String>,
}

////获取parquet文件meta
#[tauri::command]
pub async fn get_hdfs_parquet_file_meta(id: i64, file_path: String) -> Result<ParquetMeta, String> {
    let hdfs_client = get_hdfs_client(id).await?;
    let hdfs_file_reader = hdfs_client
        .read(&file_path)
        .await
        .map_err(|e| e.to_string())?;
    let mut parquet_reader = HdfsParquetFileReader(hdfs_file_reader);

    let meta = parquet_reader
        .get_metadata()
        .await
        .map_err(|e| e.to_string())?;
    let file_meta = meta.file_metadata();
    let total = file_meta.num_rows();
    let mut compression_type = "Uncompressed".to_owned();
    if !meta.row_groups().is_empty() {
        if let Some(row_group) = meta.row_groups().get(0) {
            if !row_group.columns().is_empty() {
                if let Some(column_chunk) = row_group.columns().get(0) {
                    compression_type = column_chunk.compression().to_string();
                }
            }
        }
    }
    Ok(ParquetMeta {
        total: total as i64,
        compression_type: Some(compression_type),
    })
}




#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ReadParquetResultColumn {
    pub name: String,
    pub data_type: String,
}
//分页读取parquet文件数据
#[tauri::command]
pub async fn read_parquet_file_data_by_page(
    id: i64,
    file_path: String,
    page_size: usize,
    page_number: usize,
) -> Result<Vec<HashMap<String, String>>, String> {
    let arrow_reader: ParquetRecordBatchStream<HdfsParquetFileReader> =
    get_parquet_reader(id, file_path, page_size).await?;

    let mut result_data: Vec<HashMap<String, String>> = vec![];
    let mut result_columns: Vec<ReadParquetResultColumn> = vec![];

    let mut it = arrow_reader.skip(page_number - 1);
    match it.next().await {
        Some(Ok(batch)) => {

            if result_columns.is_empty() {
                for field in batch.schema().fields() {
                    result_columns.push(ReadParquetResultColumn {
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



//导出parquet文件数据到csv文件
#[tauri::command]
pub async fn export_parquet_file_data_to_csv(
    id: i64,
    file_path: String,
    target_csv_file_path: &str,
) -> Result<(), String> {
    let mut arrow_reader: ParquetRecordBatchStream<HdfsParquetFileReader> =
    get_parquet_reader(id, file_path, 10000).await?;
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
