use crate::bus::Bus;
use crate::segment::Segment;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const TOPIC_SEND: &str = "luo9_send";

/// 发送回执主题：核心发送成功后把 NapCat 返回的 message_id 发布到此 topic
pub const TOPIC_SENT: &str = "luo9_sent";

// ── 发送请求 ────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct SendRequest {
    pub action: SendAction,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SendAction {
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
    #[serde(rename = "send_group_segments")]
    SendGroupSegments {
        group_id: u64,
        message: Vec<Segment>,
    },
    #[serde(rename = "send_private_segments")]
    SendPrivateSegments { user_id: u64, message: Vec<Segment> },
    #[serde(rename = "call")]
    Call { api: String, params: Value },
    #[serde(rename = "query")]
    Query {
        req_id: String,
        api: String,
        params: Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        response_subscriber: Option<usize>,
    },
}

impl SendAction {
    /// SDK 与核心共用的协议映射，避免 API 名称和消息格式两端漂移。
    pub fn api_call(&self) -> (&str, Value) {
        match self {
            Self::SendGroupMsg { group_id, message } => (
                "send_group_msg",
                json!({"group_id": group_id, "message": message}),
            ),
            Self::SendPrivateMsg { user_id, message } => (
                "send_private_msg",
                json!({"user_id": user_id, "message": message}),
            ),
            Self::DeleteMsg { message_id } => ("delete_msg", json!({"message_id": message_id})),
            Self::SetMsgEmojiLike {
                message_id,
                emoji_id,
            } => (
                "set_msg_emoji_like",
                json!({"message_id": message_id, "emoji_id": emoji_id.to_string()}),
            ),
            Self::SendGroupSegments { group_id, message } => (
                "send_group_msg",
                json!({"group_id": group_id, "message": message}),
            ),
            Self::SendPrivateSegments { user_id, message } => (
                "send_private_msg",
                json!({"user_id": user_id, "message": message}),
            ),
            Self::Call { api, params } | Self::Query { api, params, .. } => (api, params.clone()),
        }
    }
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

/// 发送结构化群消息，支持 @、回复、图片、语音等消息段。
pub fn send_group_segments(group_id: u64, message: Vec<Segment>) -> Option<()> {
    publish_action(SendAction::SendGroupSegments { group_id, message })
}
pub fn send_private_segments(user_id: u64, message: Vec<Segment>) -> Option<()> {
    publish_action(SendAction::SendPrivateSegments { user_id, message })
}
/// 无需返回值的通用调用；需要确认成功或读取 data 时使用 napcat::call。
pub fn call_api(api: &str, params: Value) -> Option<()> {
    publish_action(SendAction::Call {
        api: api.to_owned(),
        params,
    })
}
fn publish_action(action: SendAction) -> Option<()> {
    let json = serde_json::to_string(&SendRequest { action }).ok()?;
    Bus::topic(TOPIC_SEND).publish(&json).ok()
}

#[cfg(test)]
mod protocol_tests {
    use super::*;
    #[test]
    fn old_and_structured_requests_share_the_wire_contract() {
        let old: SendRequest =
            serde_json::from_str(r#"{"action":{"send_group_msg":{"group_id":1,"message":"hi"}}}"#)
                .unwrap();
        assert_eq!(
            old.action.api_call(),
            ("send_group_msg", json!({"group_id":1,"message":"hi"}))
        );
        let request = SendRequest {
            action: SendAction::SendPrivateSegments {
                user_id: 2,
                message: vec![
                    crate::segment::reply(-7),
                    crate::segment::image("base64://abc"),
                ],
            },
        };
        let roundtrip: SendRequest =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        let (api, params) = roundtrip.action.api_call();
        assert_eq!(api, "send_private_msg");
        assert_eq!(params["message"][0]["data"]["id"], "-7");
    }
}
