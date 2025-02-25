use bytes::Bytes;
use futures::{StreamExt, TryFutureExt, TryStreamExt};
use hdfs_native::file::FileReader;
use hdfs_native::Client;
use std::{
    collections::HashMap,
    future::IntoFuture,
    io::{BufReader, ErrorKind, Read},
    process::Command,
    time::Duration,
};

use apache_avro::from_value;
use apache_avro::Reader;

use super::hdfs_file::get_hdfs_client;

pub struct HdfsAvroFileReader {
    client: Box<hdfs_native::Client>,
    file_path: String,
    offset: Box<usize>,
}
impl Read for HdfsAvroFileReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut size: usize = 0;
        let mut err: std::io::Error = std::io::Error::new(ErrorKind::Other, "");
        let mut success = false;
        //println!("offset={}", self.offset.clone());

        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let hdfs_file_reader_result = self.client.read(&self.file_path.clone()).await;
                if hdfs_file_reader_result.is_ok() {
                    let hdfs_file_reader = hdfs_file_reader_result.unwrap();
                    let file_length = hdfs_file_reader.file_length();
                    if *self.offset >= file_length {
                        success = true;
                        return;
                    }
                    if *self.offset + buf.len() > file_length {
                        //println!("read_range_buf1");
                        let read_len = file_length - *self.offset;
                        match hdfs_file_reader
                            .read_range_buf(&mut buf[..read_len], *self.offset)
                            .await
                        {
                            Ok(()) => {
                                size = read_len;
                                success = true;
                            }
                            Err(e) => {
                                //println!("read_range_buf error1: {:?}", e);
                                err = std::io::Error::new(ErrorKind::Other, e);
                            }
                        }
                    } else {
                        //println!("read_range_buf2");
                        match hdfs_file_reader.read_range_buf(buf, *self.offset).await {
                            Ok(()) => {
                                size = buf.len();
                                success = true;
                            }
                            Err(e) => {
                                //println!("read_range_buf error2: {:?}", e);
                                err = std::io::Error::new(ErrorKind::Other, e);
                            }
                        }
                    }
                } else {
                    success = false;
                    err = std::io::Error::new(ErrorKind::Other, "Connect Hdfs Failed")
                }
            });
        });
        if success {
            self.offset = Box::new(*self.offset + size);
            //println!("size: {}", &size);
            Ok(size)
        } else {
            Err(err)
        }
    }
}

///获取avro文件reader
pub async fn get_avro_reader(
    id: i64,
    file_path: String,
) -> Result<Reader<'static, HdfsAvroFileReader>, String> {
    let hdfs_client = get_hdfs_client(id).await?;
    let hdfs_avro_file_reader = HdfsAvroFileReader {
        client: Box::new(hdfs_client),
        file_path: file_path.to_string(),
        offset: Box::new(0),
    };
    let avro_reader: Reader<HdfsAvroFileReader> =
        Reader::new(hdfs_avro_file_reader).map_err(|e| e.to_string())?;
    return Ok(avro_reader);
}

////获取avro内容
#[tauri::command]
pub async fn get_avro_content(
    id: i64,
    file_path: String,
) -> Result<Vec<serde_json::Value>, String> {
    let avro_reader: Reader<'static, HdfsAvroFileReader> = get_avro_reader(id, file_path).await?;

    let mut list: Vec<serde_json::Value> = vec![];
    for value in avro_reader {
        let json: serde_json::Value =
            serde_json::Value::try_from(value.map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        list.push(json);
    }
    Ok(list)
}
