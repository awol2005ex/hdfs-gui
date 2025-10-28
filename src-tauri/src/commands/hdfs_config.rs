use serde::{Deserialize, Serialize};

use crate::db::db_init::DB_POOL;
use ascii::AsciiString;
use awol2005ex_kerbeiros::{TgtRequester, Credential};
use awol2005ex_kerberos_keytab::Keytab;
use awol2005ex_kerberos_crypto::Key;
use std::net::{IpAddr, ToSocketAddrs};
use std::fs;
use std::path::Path;
use std::env;
// Helper functions for Kerberos authentication using awol2005ex_kerbeiros

/// Read KDC IP from Kerberos configuration file (krb5.conf)
fn get_kdc_ip_from_config(realm: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Get KRB5_CONFIG environment variable, or use default locations
    let config_paths = if let Ok(krb5_config) = env::var("KRB5_CONFIG") {
        vec![krb5_config]
    } else {
        // Default locations for krb5.conf
        vec![
            "/etc/krb5.conf".to_string(),
            "/etc/krb5/krb5.conf".to_string(),
            "C:\\Windows\\krb5.ini".to_string(),
            format!("{}\\krb5.ini", env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string())),
        ]
    };
    
    for config_path in config_paths {
        if let Ok(config_content) = fs::read_to_string(&config_path) {
            if let Some(kdc_ip) = parse_krb5_conf_for_kdc(&config_content, realm) {
                return Ok(kdc_ip);
            }
        }
    }
    
    Err(format!("Could not find KDC for realm {} in any krb5.conf file", realm).into())
}

/// Parse krb5.conf content to find KDC for the specified realm
fn parse_krb5_conf_for_kdc(config_content: &str, realm: &str) -> Option<String> {
    let mut in_realms_section = false;
    let mut current_realm = String::new();
    
    for line in config_content.lines() {
        let line = line.trim();
        
        if line.starts_with('[') && line.ends_with(']') {
            let section = line[1..line.len()-1].trim();
            in_realms_section = section == "realms" || section.ends_with("realms");
            continue;
        }
        
        if !in_realms_section {
            continue;
        }
        
        if line.starts_with(&format!("{} =", realm)) || line.starts_with(&format!("{}=", realm)) {
            current_realm = realm.to_string();
            continue;
        }
        
        if !current_realm.is_empty() {
            if line.contains('{') {
                continue;
            }
            if line.contains('}') {
                current_realm.clear();
                continue;
            }
            
            // Look for kdc = <hostname_or_ip>
            if line.starts_with("kdc") || line.contains("kdc =") {
                if let Some(kdc_part) = line.split('=').nth(1) {
                    let kdc = kdc_part.trim().trim_matches('"').trim_matches('\'');
                    if !kdc.is_empty() {
                        return Some(kdc.to_string());
                    }
                }
            }
        }
    }
    
    None
}

/// Load a keytab file and find the entry for the specified principal
fn load_key_from_keytab(keytab_path: &str, principal: &str) -> Result<Key, Box<dyn std::error::Error>> {
    let keytab_data = fs::read(keytab_path)?;
    let keytab = Keytab::parse(&keytab_data)
        .map_err(|e| format!("Failed to parse keytab: {:?}", e))?
        .1;

    // Find the entry matching the principal
    for entry in keytab.entries {
        let realm_str: String = entry.realm.try_into()?;
        let components: Vec<String> = entry.components
            .into_iter()
            .map(|c| String::from_utf8(c.data))
            .collect::<Result<Vec<_>, _>>()?;
        
        let principal_str = format!("{}@{}", components.join("/"), realm_str);
        
        if principal_str == principal {
            // Convert keyvalue to hex string
            let key_hex = hex::encode(&entry.key.keyvalue);
            
            // Create key based on keytype
            match entry.key.keytype {
                23 => return Ok(Key::from_rc4_key_string(&key_hex)?), // RC4-HMAC
                17 => return Ok(Key::from_aes_128_key_string(&key_hex)?), // AES128
                18 => return Ok(Key::from_aes_256_key_string(&key_hex)?), // AES256
                _ => return Err(format!("Unsupported key type: {}", entry.key.keytype).into()),
            }
        }
    }
    
    Err(format!("Principal {} not found in keytab", principal).into())
}

/// Perform kinit using keytab file
fn kinit_with_keytab(keytab_path: &str, principal: &str, kdc_ip: &str) -> Result<Credential, Box<dyn std::error::Error>> {
    log::info!("Performing kinit with keytab...");
    log::info!("Keytab: {}", keytab_path);
    log::info!("Principal: {}", principal);
    log::info!("KDC IP: {}", kdc_ip);
    
    // Parse the principal to extract realm
    let principal_parts: Vec<&str> = principal.split('@').collect();
    if principal_parts.len() != 2 {
        return Err("Principal must be in format username@REALM".into());
    }
    let realm = principal_parts[1];
    
    // Load the key from keytab
    let user_key = load_key_from_keytab(keytab_path, principal)?;
    
    // Parse KDC address - can be IP address or hostname
    let kdc_address: IpAddr = match kdc_ip.parse() {
        Ok(ip) => ip,
        Err(_) => {
            // Try to resolve hostname to IP address
            let kdc_socket = format!("{}:88", kdc_ip);
            match kdc_socket.to_socket_addrs() {
                Ok(mut addrs) => {
                    match addrs.next() {
                        Some(addr) => addr.ip(),
                        None => return Err(format!("Could not resolve KDC hostname: {}", kdc_ip).into()),
                    }
                }
                Err(e) => return Err(format!("Could not resolve KDC hostname '{}': {}", kdc_ip, e).into()),
            }
        }
    };
    
    // Prepare the arguments
    let realm_ascii = AsciiString::from_ascii(realm)
        .map_err(|e| format!("Invalid realm: {}", e))?;
    let username_ascii = AsciiString::from_ascii(principal_parts[0])
        .map_err(|e| format!("Invalid username: {}", e))?;
    
    // Create TGT requester
    let tgt_requester = TgtRequester::new(realm_ascii, kdc_address);
    
    // Request the TGT
    let credential = tgt_requester.request(&username_ascii, Some(&user_key))
        .map_err(|e| format!("TGT request failed: {}", e))?;
    
    log::info!("Successfully obtained TGT for {}", principal);
    Ok(credential)
}

//hdfs配置
#[derive(Debug, Default, Deserialize, Serialize, sqlx::FromRow, Clone)]
pub struct HdfsConfig {
    pub id: i64,
    pub name: String,
    pub hdfs_config: String, //json其他配置
    pub hdfs_url: String,
    pub del_flag: i64, //0正常 1删除
}
//获取hdfs配置列表
#[tauri::command]
pub async fn get_hdfs_config_list() -> Result<Vec<HdfsConfig>, String> {
    crate::db::db_init::init_db()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(pool) = DB_POOL.get() {
        let hdfs_config_list: Vec<HdfsConfig> =
            sqlx::query_as::<_, HdfsConfig>("select * from hdfs_config where del_flag = 0")
                .fetch_all(pool)
                .await
                .unwrap_or(vec![]);
        return Ok(hdfs_config_list);
    }
    Ok(vec![])
}

//保存hdfs配置
#[tauri::command]
pub async fn save_hdfs_config(hdfs_config: HdfsConfig) -> Result<(), String> {
    crate::db::db_init::init_db()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(pool) = DB_POOL.get() {
        if hdfs_config.id > 0 {
            sqlx::query(
                "update hdfs_config set name = ?, hdfs_config = ?, hdfs_url = ? where id = ?",
            )
            .bind(hdfs_config.name)
            .bind(hdfs_config.hdfs_config)
            .bind(hdfs_config.hdfs_url)
            .bind(hdfs_config.id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        } else {
            sqlx::query("insert into hdfs_config (name, hdfs_config, hdfs_url,del_flag) values (?, ?, ? ,0)")
        .bind(hdfs_config.name)
        .bind(hdfs_config.hdfs_config)
        .bind(hdfs_config.hdfs_url)
        .execute(pool).await.map_err(|e| e.to_string())?;
        }
    } else {
        return Err("Database connection pool is not initialized".to_owned());
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_hdfs_config(id: i64) -> Result<(), String> {
    crate::db::db_init::init_db()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(pool) = DB_POOL.get() {
        sqlx::query("update hdfs_config set del_flag=1 where id = ?")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    } else {
        return Err("Database connection pool is not initialized".to_owned());
    }
    Ok(())
}

//初始化kerberos
#[tauri::command]
pub async fn init_connection(id: i64) -> Result<(), String> {
    crate::db::db_init::init_db()
        .await
        .map_err(|e| e.to_string())?;

    let hc = get_one_hdfs_config(id).await.map_err(|e| e.to_string())?;
    log::info!(" Hdfs Config: {:?}", &hc);
    if let Ok(config_json) = serde_json::from_str::<serde_json::Value>(&hc.hdfs_config) {
        if let Some(principal) = config_json.get("dfs.namenode.kerberos.principal") {
            if let Some(keytab) = config_json.get("dfs.namenode.keytab.file") {
                let principal_s = principal.as_str().unwrap_or_default();
                let keytab_s = keytab.as_str().unwrap_or_default();
                
                // Check if keytab file exists
                if !Path::new(&keytab_s).exists() {
                    return Err(format!("Keytab file not found: {}", keytab_s));
                }
                
                // Parse the principal to extract realm for KDC lookup
                let principal_parts: Vec<&str> = principal_s.split('@').collect();
                if principal_parts.len() != 2 {
                    return Err("Principal must be in format username@REALM".to_owned());
                }
                let realm = principal_parts[1];
                
                // Get KDC IP from Kerberos configuration file
                let kdc_ip = match get_kdc_ip_from_config(realm) {
                    Ok(ip) => ip,
                    Err(e) => {
                        log::warn!("Could not get KDC IP from config: {}", e);
                        log::warn!("Falling back to default KDC IP: 192.168.1.100");
                        "192.168.1.100".to_string()
                    }
                };
                
                // Perform kinit using awol2005ex_kerbeiros library
                match kinit_with_keytab(&keytab_s, &principal_s, &kdc_ip) {
                    Ok(credential) => {
                        log::info!("Successfully obtained TGT for {}", principal_s);
                        
                        // Set up credential cache path
                        let ccache_file = if cfg!(target_os = "windows") {
                            let current_dir = std::env::current_dir().map_err(|e| e.to_string())?;
                            format!("{}/hdfs_gui_ccache", current_dir.to_str().unwrap_or_default())
                        } else {
                            "/tmp/hdfs_gui_ccache".to_string()
                        };
                        
                        // Save the credential to the cache file
                        credential.save_into_ccache_file(&ccache_file)
                            .map_err(|e| format!("Failed to save credential cache: {}", e))?;
                        
                        // Set KRB5CCNAME environment variable
                        unsafe {
                            std::env::set_var("KRB5CCNAME", ccache_file.clone());
                        }
                        
                        log::info!("Saved credential cache to: {}", ccache_file);
                        Ok(())
                    }
                    Err(e) => {
                        log::error!("Kinit failed: {}", e);
                        Err(format!("Kerberos authentication failed: {}", e))
                    }
                }
            } else {
                log::warn!("No keytab file found in configuration");
                Ok(())
            }
        } else {
            log::warn!("No Kerberos principal found in configuration");
            Ok(())
        }
    } else {
        log::warn!("Failed to parse HDFS configuration JSON");
        Ok(())
    }
}

//获取单个hdfs配置
pub async fn get_one_hdfs_config(id: i64) -> Result<HdfsConfig, String> {
    if let Some(pool) = DB_POOL.get() {
        let hdfs_config_list: Vec<HdfsConfig> =
            sqlx::query_as::<_, HdfsConfig>("select * from hdfs_config where id=?")
                .bind(&id)
                .fetch_all(pool)
                .await
                .unwrap_or(vec![]);
        if hdfs_config_list.len() > 0 {
            return Ok(hdfs_config_list[0].clone());
        } else {
            return Err("no config found".to_owned());
        }
    } else {
        return Err("Database connection pool is not initialized".to_owned());
    }
}
//获取单个hdfs配置
#[tauri::command]
pub async fn get_hdfs_config(id: i64) -> Result<HdfsConfig, String> {
    crate::db::db_init::init_db()
        .await
        .map_err(|e| e.to_string())?;

    return get_one_hdfs_config(id).await.map_err(|e| e.to_string());
}
//获取当前用户名
pub async fn get_hdfs_username(id: i64) -> Result<String, String> {
    let hdfs_config = get_one_hdfs_config(id).await.map_err(|e| e.to_string());

    if let Ok(hc) = hdfs_config {
        if let Ok(config_json) = serde_json::from_str::<serde_json::Value>(&hc.hdfs_config) {
            if let Some(username) = config_json.get("dfs.namenode.kerberos.principal") {
                return Ok(username
                    .as_str()
                    .unwrap_or_default()
                    .to_string()
                    .split("@")
                    .next()
                    .unwrap_or_default()
                    .to_owned());
            }
        }
    }

    if std::env::var("HDFS_USERNAME").is_ok() {
        return Ok(std::env::var("HDFS_USERNAME").unwrap_or_default());
    }

    return Err("no hdfs username found".to_owned());
}
