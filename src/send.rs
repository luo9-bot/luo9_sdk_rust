use crate::bus::Bus;
use serde::Serialize;

const TOPIC_SEND: &str = "luo9_send";

/// 发送回执主题：核心发送成功后把 NapCat 返回的 message_id 发布到此 topic
pub const TOPIC_SENT: &str = "luo9_sent";

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
    /// 撤回消息（群聊/私聊通用，NapCat: delete_msg）
    #[serde(rename = "delete_msg")]
    DeleteMsg { message_id: u64 },
    /// 设置消息表情回应（NapCat: set_msg_emoji_like）
    #[serde(rename = "set_msg_emoji_like")]
    SetMsgEmojiLike { message_id: u64, emoji_id: u64 },
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

/// 发送群消息并引用指定消息（CQ:reply 组合）
pub fn send_group_msg_reply(group_id: u64, reply_to_message_id: u64, message: &str) -> Option<()> {
    let composed = format!("[CQ:reply,id={reply_to_message_id}]{message}");
    send_group_msg(group_id, &composed)
}

/// 发送私聊消息并引用指定消息（CQ:reply 组合）
pub fn send_private_msg_reply(user_id: u64, reply_to_message_id: u64, message: &str) -> Option<()> {
    let composed = format!("[CQ:reply,id={reply_to_message_id}]{message}");
    send_private_msg(user_id, &composed)
}

/// 撤回消息（自己的或可管理范围内的一条）
pub fn delete_msg(message_id: u64) -> Option<()> {
    let req = SendRequest {
        action: SendAction::DeleteMsg { message_id },
    };
    let json = serde_json::to_string(&req).ok()?;
    Bus::topic(TOPIC_SEND).publish(&json).ok()
}

/// 对一条消息设置表情回应
pub fn set_msg_emoji_like(message_id: u64, emoji_id: u64) -> Option<()> {
    let req = SendRequest {
        action: SendAction::SetMsgEmojiLike {
            message_id,
            emoji_id,
        },
    };
    let json = serde_json::to_string(&req).ok()?;
    Bus::topic(TOPIC_SEND).publish(&json).ok()
}
