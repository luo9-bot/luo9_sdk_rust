use serde::Serialize;
use crate::bus::Bus;

const TOPIC_SEND: &str = "luo9_send";

// ── 发送请求 ────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct SendRequest {
    action: SendAction,
}

#[derive(Debug, Serialize)]
enum SendAction {
    #[serde(rename = "send_group_msg")]
    SendGroupMsg { group_id: u64, message: String },
    #[serde(rename = "send_private_msg")]
    SendPrivateMsg { user_id: u64, message: String },
}

// ── 发送 API ────────────────────────────────────────────────────

/// 发送群消息（fire-and-forget）
pub fn send_group_msg(group_id: u64, message: &str) -> Option<()> {
    let req = SendRequest {
        action: SendAction::SendGroupMsg {
            group_id,
            message: message.to_string(),
        },
    };
    let json = serde_json::to_string(&req).ok()?;
    Bus::topic(TOPIC_SEND).publish(&json).ok()
}

/// 发送私聊消息（fire-and-forget）
pub fn send_private_msg(user_id: u64, message: &str) -> Option<()> {
    let req = SendRequest {
        action: SendAction::SendPrivateMsg {
            user_id,
            message: message.to_string(),
        },
    };
    let json = serde_json::to_string(&req).ok()?;
    Bus::topic(TOPIC_SEND).publish(&json).ok()
}
