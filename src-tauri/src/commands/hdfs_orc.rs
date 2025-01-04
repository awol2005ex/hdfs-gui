use std::io::ErrorKind;

use bytes::Bytes;
use chrono::{DateTime, Utc};
use futures::TryFutureExt;
use hdfs_native::file::FileReader;
use orc_rust::{reader::{metadata::{read_metadata_async, FileMetadata}, AsyncChunkReader}, ArrowReaderBuilder, ArrowStreamReader};
use serde::{Deserialize, Serialize};
use futures::StreamExt;
use std::{collections::HashMap, fs::File, sync::Arc};
use arrow::{
    array::{downcast_array, Array, StringArray},
    datatypes::*,
};
use crate::get_hdfs_client;
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
    let arrow_reader :ArrowStreamReader<HdfsOrcFileReader>= get_orc_reader(id, file_path, 1).await?;
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
pub async fn get_hdfs_orc_file_rows_count(
    id: i64,
    file_path: String,
) -> Result<u64, String> {
    let hdfs_client = get_hdfs_client(id).await?;
    let hdfs_file_reader = hdfs_client
        .read(&file_path)
        .await
        .map_err(|e| e.to_string())?;
    let mut orc_reader=HdfsOrcFileReader(hdfs_file_reader);
    
    let file_meta:FileMetadata =read_metadata_async::<HdfsOrcFileReader>(&mut orc_reader).await.map_err(|e| e.to_string())?;
    
    let total= file_meta.number_of_rows();
    Ok(total)
}


pub fn i64_to_timestamp_format(timestamp: i64) -> String {
    if timestamp > 0 {
        let naive = DateTime::from_timestamp(timestamp, 0);
        let datetime: DateTime<Utc> =
            DateTime::from_naive_utc_and_offset(naive.unwrap_or_default().naive_utc(), Utc);
        datetime.to_rfc3339()
    } else {
        "".to_owned()
    }
}

pub fn i64_to_nanosecond_format(timestamp: i64) -> String {
    if timestamp > 0 {
        let naive = DateTime::from_timestamp(timestamp / 1000000000, 0);
        let datetime: DateTime<Utc> =
            DateTime::from_naive_utc_and_offset(naive.unwrap_or_default().naive_utc(), Utc);
        datetime.to_rfc3339()
    } else {
        "".to_owned()
    }
}
pub fn get_column_value(column: &Arc<dyn Array>, rowindex: usize) -> String {
    match column.data_type() {
        arrow::datatypes::DataType::Utf8 => {
            return downcast_array::<StringArray>(column.as_ref())
                .value(rowindex)
                .to_string();
        }
        arrow::datatypes::DataType::Null => {
            return "NULL".to_owned();
        }
        arrow::datatypes::DataType::Boolean => {
            return downcast_array::<arrow::array::BooleanArray>(column.as_ref())
                .value(rowindex)
                .to_string();
        }
        arrow::datatypes::DataType::Int8 => {
            return downcast_array::<arrow::array::Int8Array>(column.as_ref())
                .value(rowindex)
                .to_string();
        }
        arrow::datatypes::DataType::Int16 => {
            return downcast_array::<arrow::array::Int16Array>(column.as_ref())
                .value(rowindex)
                .to_string();
        }
        arrow::datatypes::DataType::Int32 => {
            return downcast_array::<arrow::array::Int32Array>(column.as_ref())
                .value(rowindex)
                .to_string();
        }
        arrow::datatypes::DataType::Int64 => {
            return downcast_array::<arrow::array::Int64Array>(column.as_ref())
                .value(rowindex)
                .to_string();
        }
        arrow::datatypes::DataType::UInt8 => {
            return downcast_array::<arrow::array::UInt8Array>(column.as_ref())
                .value(rowindex)
                .to_string();
        }
        arrow::datatypes::DataType::UInt16 => {
            return downcast_array::<arrow::array::UInt16Array>(column.as_ref())
                .value(rowindex)
                .to_string();
        }
        arrow::datatypes::DataType::UInt32 => {
            return downcast_array::<arrow::array::UInt32Array>(column.as_ref())
                .value(rowindex)
                .to_string();
        }
        arrow::datatypes::DataType::UInt64 => {
            return downcast_array::<arrow::array::UInt64Array>(column.as_ref())
                .value(rowindex)
                .to_string();
        }
        arrow::datatypes::DataType::Float16 => {
            return downcast_array::<arrow::array::Float16Array>(column.as_ref())
                .value(rowindex)
                .to_string();
        }
        arrow::datatypes::DataType::Float32 => {
            return downcast_array::<arrow::array::Float32Array>(column.as_ref())
                .value(rowindex)
                .to_string();
        }
        arrow::datatypes::DataType::Float64 => {
            return downcast_array::<arrow::array::Float64Array>(column.as_ref())
                .value(rowindex)
                .to_string();
        }
        arrow::datatypes::DataType::Timestamp(_time_unit, _arc) => {
            return i64_to_nanosecond_format(
                downcast_array::<arrow::array::TimestampNanosecondArray>(column.as_ref())
                    .value(rowindex),
            );
        }
        arrow::datatypes::DataType::Date32 => {
            return i64_to_timestamp_format(
                downcast_array::<arrow::array::Date32Array>(column.as_ref())
                    .value(rowindex)
                    .into(),
            );
        }
        arrow::datatypes::DataType::Date64 => {
            return i64_to_timestamp_format(
                downcast_array::<arrow::array::Date64Array>(column.as_ref()).value(rowindex),
            );
        }
        arrow::datatypes::DataType::Time32(_time_unit) => {
            return downcast_array::<arrow::array::Time32SecondArray>(column.as_ref())
                .value(rowindex)
                .to_string();
        }
        arrow::datatypes::DataType::Time64(_time_unit) => {
            return downcast_array::<arrow::array::Time64NanosecondArray>(column.as_ref())
                .value(rowindex)
                .to_string();
        }
        arrow::datatypes::DataType::Duration(_time_unit) => {
            return downcast_array::<arrow::array::DurationNanosecondArray>(column.as_ref())
                .value(rowindex)
                .to_string();
        }
        arrow::datatypes::DataType::Interval(_interval_unit) => {
            return downcast_array::<arrow::array::IntervalDayTimeArray>(column.as_ref())
                .value(rowindex)
                .milliseconds
                .to_string();
        }
        arrow::datatypes::DataType::Binary => {
            return String::from_utf8_lossy(
                &downcast_array::<arrow::array::BinaryArray>(column.as_ref()).value(rowindex),
            )
            .to_string();
        }
        arrow::datatypes::DataType::FixedSizeBinary(_) => {
            return String::from_utf8_lossy(
                &downcast_array::<arrow::array::FixedSizeBinaryArray>(column.as_ref())
                    .value(rowindex),
            )
            .to_string();
        }
        arrow::datatypes::DataType::LargeBinary => {
            return String::from_utf8_lossy(
                &downcast_array::<arrow::array::LargeBinaryArray>(column.as_ref()).value(rowindex),
            )
            .to_string();
        }
        arrow::datatypes::DataType::BinaryView => {
            return String::from_utf8_lossy(
                &downcast_array::<arrow::array::BinaryArray>(column.as_ref()).value(rowindex),
            )
            .to_string();
        }
        arrow::datatypes::DataType::LargeUtf8 => {
            return downcast_array::<arrow::array::LargeStringArray>(column.as_ref())
                .value(rowindex)
                .to_string();
        }
        arrow::datatypes::DataType::Utf8View => {
            return downcast_array::<arrow::array::StringArray>(column.as_ref())
                .value(rowindex)
                .to_string();
        }
        arrow::datatypes::DataType::List(_arc) => {
            return format!(
                "{:?}",
                downcast_array::<arrow::array::ListArray>(column.as_ref()).value(rowindex)
            );
        }
        arrow::datatypes::DataType::ListView(_arc) => {
            return format!(
                "{:?}",
                downcast_array::<arrow::array::ListArray>(column.as_ref()).value(rowindex)
            );
        }
        arrow::datatypes::DataType::FixedSizeList(_arc, _) => {
            return format!(
                "{:?}",
                downcast_array::<arrow::array::FixedSizeListArray>(column.as_ref()).value(rowindex)
            );
        }
        arrow::datatypes::DataType::LargeList(_arc) => {
            return format!(
                "{:?}",
                downcast_array::<arrow::array::LargeListArray>(column.as_ref()).value(rowindex)
            );
        }
        arrow::datatypes::DataType::LargeListView(_arc) => {
            return format!(
                "{:?}",
                downcast_array::<arrow::array::LargeListArray>(column.as_ref()).value(rowindex)
            );
        }
        arrow::datatypes::DataType::Struct(_fields) => {
            return format!(
                "{:?}",
                downcast_array::<arrow::array::StructArray>(column.as_ref()).slice(rowindex, 1)
            );
        }
        arrow::datatypes::DataType::Union(_union_fields, _union_mode) => {
            return format!(
                "{:?}",
                downcast_array::<arrow::array::UnionArray>(column.as_ref()).value(rowindex)
            );
        }
        arrow::datatypes::DataType::Dictionary(data_type, _) => {
            if data_type.equals_datatype(&arrow::datatypes::DataType::Int8) {
                return format!(
                    "{:?}",
                    downcast_array::<arrow::array::DictionaryArray<Int8Type>>(column.as_ref())
                        .slice(rowindex, 1)
                );
            }
            if data_type.equals_datatype(&arrow::datatypes::DataType::Int16) {
                return format!(
                    "{:?}",
                    downcast_array::<arrow::array::DictionaryArray<Int16Type>>(column.as_ref())
                        .slice(rowindex, 1)
                );
            }
            if data_type.equals_datatype(&arrow::datatypes::DataType::Int32) {
                return format!(
                    "{:?}",
                    downcast_array::<arrow::array::DictionaryArray<Int32Type>>(column.as_ref())
                        .slice(rowindex, 1)
                );
            }
            if data_type.equals_datatype(&arrow::datatypes::DataType::Int64) {
                return format!(
                    "{:?}",
                    downcast_array::<arrow::array::DictionaryArray<Int64Type>>(column.as_ref())
                        .slice(rowindex, 1)
                );
            }
            if data_type.equals_datatype(&arrow::datatypes::DataType::UInt8) {
                return format!(
                    "{:?}",
                    downcast_array::<arrow::array::DictionaryArray<UInt8Type>>(column.as_ref())
                        .slice(rowindex, 1)
                );
            }
            if data_type.equals_datatype(&arrow::datatypes::DataType::UInt16) {
                return format!(
                    "{:?}",
                    downcast_array::<arrow::array::DictionaryArray<UInt16Type>>(column.as_ref())
                        .slice(rowindex, 1)
                );
            }
            if data_type.equals_datatype(&arrow::datatypes::DataType::UInt32) {
                return format!(
                    "{:?}",
                    downcast_array::<arrow::array::DictionaryArray<UInt32Type>>(column.as_ref())
                        .slice(rowindex, 1)
                );
            }
            if data_type.equals_datatype(&arrow::datatypes::DataType::UInt64) {
                return format!(
                    "{:?}",
                    downcast_array::<arrow::array::DictionaryArray<UInt64Type>>(column.as_ref())
                        .slice(rowindex, 1)
                );
            }

            return "NULL".to_owned();
        }
        arrow::datatypes::DataType::Decimal128(_, _) => {
            return downcast_array::<arrow::array::Decimal128Array>(column.as_ref())
                .value(rowindex)
                .to_string();
        }
        arrow::datatypes::DataType::Decimal256(_, _) => {
            return downcast_array::<arrow::array::Decimal256Array>(column.as_ref())
                .value(rowindex)
                .to_string();
        }
        arrow::datatypes::DataType::Map(_arc, _) => {
            return format!(
                "{:?}",
                downcast_array::<arrow::array::MapArray>(column.as_ref()).value(rowindex)
            );
        }
        arrow::datatypes::DataType::RunEndEncoded(arc, _arc1) => {
            if arc
                .data_type()
                .equals_datatype(&arrow::datatypes::DataType::Int16)
            {
                return format!(
                    "{:?}",
                    downcast_array::<arrow::array::RunArray<Int16Type>>(column.as_ref())
                        .slice(rowindex, 1)
                );
            }
            if arc
                .data_type()
                .equals_datatype(&arrow::datatypes::DataType::Int32)
            {
                return format!(
                    "{:?}",
                    downcast_array::<arrow::array::RunArray<Int32Type>>(column.as_ref())
                        .slice(rowindex, 1)
                );
            }
            if arc
                .data_type()
                .equals_datatype(&arrow::datatypes::DataType::Int64)
            {
                return format!(
                    "{:?}",
                    downcast_array::<arrow::array::RunArray<Int64Type>>(column.as_ref())
                        .slice(rowindex, 1)
                );
            }

            return "NULL".to_owned();
        }
    }
}