use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

use super::utils::{find_latest_antigravity_log, parse_ports_from_log, find_csrf_token_from_memory};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Root {
    #[serde(default)]
    user_status: Option<UserStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserStatus {
    #[serde(default)]
    disable_telemetry: bool,
    #[serde(default)]
    name: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    plan_status: Option<PlanStatus>,
    #[serde(default)]
    cascade_model_config_data: Option<CascadeModelConfigData>,
    #[serde(default)]
    accepted_latest_terms_of_service: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlanStatus {
    #[serde(default)]
    plan_info: Option<PlanInfo>,
    #[serde(default)]
    available_prompt_credits: i64,
    #[serde(default)]
    available_flow_credits: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlanInfo {
    #[serde(default)]
    teams_tier: String,
    #[serde(default)]
    plan_name: String,
    #[serde(default)]
    has_autocomplete_fast_mode: bool,
    #[serde(default)]
    allow_sticky_premium_models: bool,
    #[serde(default)]
    allow_premium_command_models: bool,
    #[serde(default)]
    has_tab_to_jump: bool,
    #[serde(default)]
    max_num_premium_chat_messages: String,
    #[serde(default)]
    max_num_chat_input_tokens: String,
    #[serde(default)]
    max_custom_chat_instruction_characters: String,
    #[serde(default)]
    max_num_pinned_context_items: String,
    #[serde(default)]
    max_local_index_size: String,
    #[serde(default)]
    monthly_prompt_credits: i64,
    #[serde(default)]
    monthly_flow_credits: i64,
    #[serde(default)]
    monthly_flex_credit_purchase_amount: i64,
    #[serde(default)]
    can_buy_more_credits: bool,
    #[serde(default)]
    cascade_web_search_enabled: bool,
    #[serde(default)]
    can_customize_app_icon: bool,
    #[serde(default)]
    cascade_can_auto_run_commands: bool,
    #[serde(default)]
    can_generate_commit_messages: bool,
    #[serde(default)]
    knowledge_base_enabled: bool,
    #[serde(default)]
    default_team_config: Option<DefaultTeamConfig>,
    #[serde(default)]
    can_allow_cascade_in_background: bool,
    #[serde(default)]
    browser_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DefaultTeamConfig {
    #[serde(default)]
    allow_mcp_servers: bool,
    #[serde(default)]
    allow_auto_run_commands: bool,
    #[serde(default)]
    allow_browser_experimental_features: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CascadeModelConfigData {
    #[serde(default)]
    client_model_configs: Vec<ClientModelConfig>,
    #[serde(default)]
    client_model_sorts: Vec<ClientModelSort>,
    #[serde(default)]
    default_override_model_config: Option<DefaultOverrideModelConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClientModelConfig {
    #[serde(default)]
    label: String,
    #[serde(default)]
    model_or_alias: Option<ModelOrAlias>,
    #[serde(default)]
    supports_images: Option<bool>,
    #[serde(default)]
    is_recommended: bool,
    #[serde(default)]
    allowed_tiers: Vec<String>,
    #[serde(default)]
    quota_info: Option<QuotaInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModelOrAlias {
    #[serde(default)]
    model: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct QuotaInfo {
    #[serde(default)]
    remaining_fraction: f64,
    #[serde(default)]
    reset_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClientModelSort {
    #[serde(default)]
    name: String,
    #[serde(default)]
    groups: Vec<Group>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Group {
    #[serde(default)]
    model_labels: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DefaultOverrideModelConfig {
    #[serde(default)]
    model_or_alias: Option<ModelOrAlias>,
}

/// 前端调用 GetUserStatus 的公开命令
#[tauri::command]
pub async fn language_server_get_user_status(
    api_key: String,
) -> Result<serde_json::Value, String> {
    if api_key.trim().is_empty() {
        return Err("apiKey 不能为空".to_string());
    }

    // 1) 解析日志拿 HTTPS 端口
    let log_path = find_latest_antigravity_log()
        .ok_or_else(|| "未找到 Antigravity.log，无法确定端口".to_string())?;
    let content = std::fs::read_to_string(&log_path)
        .map_err(|e| format!("读取日志失败: {e}"))?;
    let (https_port, _, _) = parse_ports_from_log(&content);
    let port = https_port.ok_or_else(|| "日志中未找到 HTTPS 端口".to_string())?;

    // 2) 构造固定 URL/路径/请求体
    let target_url = format!(
        "https://127.0.0.1:{}/exa.language_server_pb.LanguageServerService/GetUserStatus",
        port
    );

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_millis(4000))
        .build()
        .map_err(|e| format!("构建 HTTP 客户端失败: {e}"))?;

    let body = json!({
        "metadata": {
            "ideName": "antigravity",
            "apiKey": api_key,
            "locale": "en",
            "ideVersion": "1.11.5",
            "extensionName": "antigravity"
        }
    });
    let body_bytes = serde_json::to_vec(&body)
        .map_err(|e| format!("序列化请求体失败: {e}"))?;

    // CSRF token：从运行中的进程内存直接提取
    let csrf = find_csrf_token_from_memory()
        .map_err(|e| format!("提取 csrf_token 失败: {e}"))?;
    let mut req = client.post(&target_url);

  println!("csrf token: {csrf}");

    // 模拟前端请求头
    req = req
        .header("accept", "*/*")
        .header("accept-language", "en-US")
        .header("connect-protocol-version", "1")
        .header("content-type", "application/json")
        .header("priority", "u=1, i")
        .header("sec-ch-ua", "\"Not)A;Brand\";v=\"8\", \"Chromium\";v=\"138\"")
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", "\"Windows\"")
        .header("sec-fetch-dest", "empty")
        .header("sec-fetch-mode", "cors")
        .header("sec-fetch-site", "cross-site")
        .header("x-codeium-csrf-token", csrf.clone());

    // 打印请求信息（脱敏 api_key）
    tracing::info!(
        target_url = %target_url,
        https_port = port,
        method = "POST",
        headers = %format!(
            "accept=*/*; accept-language=en-US; connect-protocol-version=1; content-type=application/json; priority=u=1,i; sec-ch-ua=\"Not)A;Brand\";v=\"8\", \"Chromium\";v=\"138\"; sec-ch-ua-mobile=?0; sec-ch-ua-platform=\"Windows\"; sec-fetch-dest=empty; sec-fetch-mode=cors; sec-fetch-site=cross-site; x-codeium-csrf-token={}",
            csrf
        ),
        body = %String::from_utf8_lossy(&body_bytes),
        "language_server_get_user_status request"
    );

    let resp = req
        .body(body_bytes)
        .send()
        .await
        .map_err(|e| format!("请求失败: {e}"))?;

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("读取响应失败: {e}"))?;

    let parsed: Root = serde_json::from_slice(&bytes)
        .map_err(|e| format!("解析响应失败: {e}; body={}", String::from_utf8_lossy(&bytes)))?;

    Ok(serde_json::to_value(parsed).map_err(|e| format!("序列化响应失败: {e}"))?)
}
