//! 账户备份/导入导出与加解密命令

use crate::log_async_command;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::time::SystemTime;
use tauri::State;

/// 备份数据收集结构
#[derive(Serialize, Deserialize, Debug)]
pub struct AccountExportedData {
    filename: String,
    #[serde(rename = "content")]
    content: Value,
    #[serde(rename = "timestamp")]
    timestamp: u64,
}

/// 恢复结果
#[derive(Serialize, Deserialize, Debug)]
pub struct RestoreResult {
    #[serde(rename = "restoredCount")]
    restored_count: u32,
    failed: Vec<FailedAccountExportedData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FailedAccountExportedData {
    filename: String,
    error: String,
}

const CONFIG_ENCRYPTION_VERSION: u8 = 2;
const PBKDF2_ITERATIONS: u32 = 210_000;
const PBKDF2_SALT_LEN: usize = 16;
const AES_GCM_NONCE_LEN: usize = 12;

#[derive(Serialize, Deserialize, Debug)]
struct EncryptedConfigEnvelopeV2 {
    v: u8,
    kdf: String,
    iter: u32,
    #[serde(rename = "salt")]
    salt_b64: String,
    #[serde(rename = "nonce")]
    nonce_b64: String,
    #[serde(rename = "ciphertext")]
    ciphertext_b64: String,
}

/// 收集所有账户文件的完整内容, 用于导出
#[tauri::command]
pub async fn collect_account_contents(
    state: State<'_, crate::AppState>,
) -> Result<Vec<AccountExportedData>, String> {
    let mut backups_with_content = Vec::new();

    // 读取Antigravity账户目录中的JSON文件
    let antigravity_dir = state.config_dir.join("antigravity-accounts");

    if !antigravity_dir.exists() {
        return Ok(backups_with_content);
    }

    for entry in fs::read_dir(&antigravity_dir).map_err(|e| format!("读取用户目录失败: {}", e))?
    {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "json") {
            let filename = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
                .unwrap_or_default();

            if filename.is_empty() {
                continue;
            }

            match fs::read_to_string(&path).map_err(|e| format!("读取文件失败 {}: {}", filename, e))
            {
                Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(json_value) => {
                        backups_with_content.push(AccountExportedData {
                            filename,
                            content: json_value,
                            timestamp: SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                        });
                    }
                    Err(e) => {
                        tracing::warn!(target: "backup::scan", filename = %filename, error = %e, "跳过损坏的备份文件");
                    }
                },
                Err(_) => {
                    tracing::warn!(target: "backup::scan", filename = %filename, "跳过无法读取的文件");
                }
            }
        }
    }

    Ok(backups_with_content)
}

/// 恢复备份文件到本地
#[tauri::command]
pub async fn restore_backup_files(
    account_file_data: Vec<AccountExportedData>,
    state: State<'_, crate::AppState>,
) -> Result<RestoreResult, String> {
    let mut results = RestoreResult {
        restored_count: 0,
        failed: Vec::new(),
    };

    // 获取目标目录
    let antigravity_dir = state.config_dir.join("antigravity-accounts");

    // 确保目录存在
    if let Err(e) = fs::create_dir_all(&antigravity_dir) {
        return Err(format!("创建目录失败: {}", e));
    }

    // 遍历每个备份
    for account_file in account_file_data {
        let file_path = antigravity_dir.join(&account_file.filename);

        match fs::write(
            &file_path,
            serde_json::to_string_pretty(&account_file.content).unwrap_or_default(),
        )
        .map_err(|e| format!("写入文件失败: {}", e))
        {
            Ok(_) => {
                results.restored_count += 1;
            }
            Err(e) => {
                results.failed.push(FailedAccountExportedData {
                    filename: account_file.filename,
                    error: e,
                });
            }
        }
    }

    Ok(results)
}

/// 删除指定备份
#[tauri::command]
pub async fn delete_backup(
    name: String,
    state: State<'_, crate::AppState>,
) -> Result<String, String> {
    // 只删除Antigravity账户JSON文件
    let antigravity_dir = state.config_dir.join("antigravity-accounts");
    let antigravity_file = antigravity_dir.join(format!("{}.json", name));

    if antigravity_file.exists() {
        fs::remove_file(&antigravity_file).map_err(|e| format!("删除用户文件失败: {}", e))?;
        Ok(format!("删除用户成功: {}", name))
    } else {
        Err("用户文件不存在".to_string())
    }
}

/// 清空所有备份
#[tauri::command]
pub async fn clear_all_backups(state: State<'_, crate::AppState>) -> Result<String, String> {
    let antigravity_dir = state.config_dir.join("antigravity-accounts");

    if antigravity_dir.exists() {
        // 读取目录中的所有文件
        let mut deleted_count = 0;
        for entry in
            fs::read_dir(&antigravity_dir).map_err(|e| format!("读取用户目录失败: {}", e))?
        {
            let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
            let path = entry.path();

            // 只删除 JSON 文件
            if path.extension().is_some_and(|ext| ext == "json") {
                fs::remove_file(&path)
                    .map_err(|e| format!("删除文件 {} 失败: {}", path.display(), e))?;
                deleted_count += 1;
            }
        }

        Ok(format!(
            "已清空所有用户备份，共删除 {} 个文件",
            deleted_count
        ))
    } else {
        Ok("用户目录不存在，无需清空".to_string())
    }
}

fn derive_config_key_pbkdf2(password: &[u8], salt: &[u8], iterations: u32) -> [u8; 32] {
    use pbkdf2::pbkdf2_hmac;
    use sha2::Sha256;

    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password, salt, iterations, &mut key);
    key
}

fn encrypt_config_data_v2(json_data: &str, password: &str) -> Result<String, String> {
    use aes_gcm::aead::Aead;
    use aes_gcm::KeyInit;
    use aes_gcm::{Aes256Gcm, Nonce};
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
    use rand::RngCore;

    let mut salt = [0u8; PBKDF2_SALT_LEN];
    rand::rngs::OsRng.fill_bytes(&mut salt);

    let mut nonce_bytes = [0u8; AES_GCM_NONCE_LEN];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);

    let key = derive_config_key_pbkdf2(password.as_bytes(), &salt, PBKDF2_ITERATIONS);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| "加密失败".to_string())?;

    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), json_data.as_bytes())
        .map_err(|_| "加密失败".to_string())?;

    let envelope = EncryptedConfigEnvelopeV2 {
        v: CONFIG_ENCRYPTION_VERSION,
        kdf: "pbkdf2-sha256".to_string(),
        iter: PBKDF2_ITERATIONS,
        salt_b64: BASE64.encode(salt),
        nonce_b64: BASE64.encode(nonce_bytes),
        ciphertext_b64: BASE64.encode(ciphertext),
    };

    serde_json::to_string(&envelope).map_err(|_| "加密失败".to_string())
}

fn decrypt_config_data_v2(encrypted_data: &str, password: &str) -> Result<String, String> {
    use aes_gcm::aead::Aead;
    use aes_gcm::KeyInit;
    use aes_gcm::{Aes256Gcm, Nonce};
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

    let envelope: EncryptedConfigEnvelopeV2 =
        serde_json::from_str(encrypted_data).map_err(|_| "解密失败，数据格式无效".to_string())?;

    if envelope.v != CONFIG_ENCRYPTION_VERSION {
        return Err("解密失败，不支持的加密版本".to_string());
    }

    if envelope.kdf != "pbkdf2-sha256" {
        return Err("解密失败，不支持的 KDF".to_string());
    }

    // 防止被构造的极端参数拖慢解密
    if envelope.iter < 10_000 || envelope.iter > 10_000_000 {
        return Err("解密失败，不支持的 KDF 参数".to_string());
    }

    let salt = BASE64
        .decode(envelope.salt_b64)
        .map_err(|_| "解密失败，salt 无效".to_string())?;
    let nonce_bytes = BASE64
        .decode(envelope.nonce_b64)
        .map_err(|_| "解密失败，nonce 无效".to_string())?;
    let ciphertext = BASE64
        .decode(envelope.ciphertext_b64)
        .map_err(|_| "解密失败，密文无效".to_string())?;

    if salt.len() != PBKDF2_SALT_LEN || nonce_bytes.len() != AES_GCM_NONCE_LEN {
        return Err("解密失败，数据格式无效".to_string());
    }

    let key = derive_config_key_pbkdf2(password.as_bytes(), &salt, envelope.iter);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| "解密失败".to_string())?;

    let plaintext = cipher
        .decrypt(Nonce::from_slice(&nonce_bytes), ciphertext.as_ref())
        .map_err(|_| "解密失败，密码错误或数据已损坏".to_string())?;

    String::from_utf8(plaintext).map_err(|_| "解密失败，数据可能已损坏".to_string())
}

fn decrypt_config_data_legacy_xor_base64(
    encrypted_data: String,
    password: String,
) -> Result<String, String> {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

    let decoded = BASE64
        .decode(encrypted_data)
        .map_err(|_| "Base64 解码失败".to_string())?;

    let password_bytes = password.as_bytes();
    let mut result = Vec::with_capacity(decoded.len());

    for (i, byte) in decoded.iter().enumerate() {
        let key_byte = password_bytes[i % password_bytes.len()];
        result.push(byte ^ key_byte);
    }

    String::from_utf8(result).map_err(|_| "解密失败，数据可能已损坏".to_string())
}

/// 加密配置数据（用于账户导出）
#[tauri::command]
pub async fn encrypt_config_data(json_data: String, password: String) -> Result<String, String> {
    log_async_command!("encrypt_config_data", async {
        if password.is_empty() {
            return Err("密码不能为空".to_string());
        }

        encrypt_config_data_v2(&json_data, &password)
    })
}

/// 解密配置数据（用于账户导入）
#[tauri::command]
pub async fn decrypt_config_data(
    encrypted_data: String,
    password: String,
) -> Result<String, String> {
    log_async_command!("decrypt_config_data", async {
        if password.is_empty() {
            return Err("密码不能为空".to_string());
        }

        let trimmed = encrypted_data.trim();

        if trimmed.starts_with('{') {
            // v2 加密格式：JSON envelope
            if serde_json::from_str::<EncryptedConfigEnvelopeV2>(trimmed).is_ok() {
                return decrypt_config_data_v2(trimmed, &password);
            }
        }

        decrypt_config_data_legacy_xor_base64(encrypted_data, password)
    })
}

/// 备份并重启 Antigravity（迁移自 process_commands）
#[tauri::command]
pub async fn sign_in_new_antigravity_account() -> Result<String, String> {
    println!("🔄 开始执行 sign_in_new_antigravity_account 命令");

    // 1. 关闭进程 (如果存在)
    println!("🛑 步骤1: 检查并关闭 Antigravity 进程");
    let kill_result = match crate::platform::kill_antigravity_processes() {
        Ok(result) => {
            if result.contains("not found") || result.contains("未找到") {
                println!("ℹ️ Antigravity 进程未运行，跳过关闭步骤");
                "Antigravity 进程未运行".to_string()
            } else {
                println!("✅ 进程关闭结果: {}", result);
                result
            }
        }
        Err(e) => {
            if e.contains("not found") || e.contains("未找到") {
                println!("ℹ️ Antigravity 进程未运行，跳过关闭步骤");
                "Antigravity 进程未运行".to_string()
            } else {
                return Err(format!("关闭进程时发生错误: {}", e));
            }
        }
    };

    // 等待500ms确保进程完全关闭（缩短等待时间避免前端超时）
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 2. 备份当前账户信息（直接调用 save_antigravity_current_account）
    println!("💾 步骤2: 调用 save_antigravity_current_account 备份当前账户信息");
    let backup_info = match crate::commands::save_antigravity_current_account().await {
        Ok(msg) => {
            println!("✅ 备份完成: {}", msg);
            Some(msg)
        }
        Err(e) => {
            println!("⚠️ 备份失败: {}", e);
            None
        }
    };

    // 3. 清除 Antigravity 所有数据 (彻底注销)
    println!("🗑️ 步骤3: 清除所有 Antigravity 数据 (彻底注销)");
    match crate::antigravity::cleanup::clear_all_antigravity_data().await {
        Ok(result) => {
            println!("✅ 清除完成: {}", result);
        }
        Err(e) => {
            // 清除失败可能是因为数据库本来就是空的，这是正常情况
            println!("ℹ️ 清除数据时出现: {}（可能数据库本来就是空的）", e);
        }
    }

    // 等待300ms确保操作完成（缩短等待时间避免前端超时）
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    // 4. 重新启动进程
    println!("🚀 步骤4: 重新启动 Antigravity");
    let start_result = crate::antigravity::starter::start_antigravity();
    let start_message = match start_result {
        Ok(result) => {
            println!("✅ 启动结果: {}", result);
            result
        }
        Err(e) => {
            println!("⚠️ 启动失败: {}", e);
            format!("启动失败: {}", e)
        }
    };

    let final_message = if let Some(backup_message) = backup_info {
        format!(
            "{} -> 已备份: {} -> 已清除账户数据 -> {}",
            kill_result, backup_message, start_message
        )
    } else {
        format!(
            "{} -> 未检测到登录用户（跳过备份） -> 已清除账户数据 -> {}",
            kill_result, start_message
        )
    };
    println!("🎉 所有操作完成: {}", final_message);

    Ok(final_message)
}
