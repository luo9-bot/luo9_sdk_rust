// src/version.rs
// 插件版本响应协议：SDK 维护格式，插件提供内容

use crate::bus::Bus;

pub const TOPIC_VERSION: &str = "luo9_version";
pub const TOPIC_VERSION_REPLY: &str = "luo9_version_reply";

/// 判断收到的消息是否为版本查询请求
pub fn is_version_query(json: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(json)
        .ok()
        .and_then(|v| v["action"].as_str().map(|s| s == "query"))
        .unwrap_or(false)
}

/// 回复版本信息，name 和 version 由插件提供
pub fn reply_version(name: &str, version: &str) {
    let resp = serde_json::json!({
        "action": "response",
        "name": name,
        "version": version,
    });
    let _ = Bus::topic(TOPIC_VERSION_REPLY).publish(&resp.to_string());
}
