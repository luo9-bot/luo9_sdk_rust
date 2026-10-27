//! 带返回值的 NapCat API。查询使用独立订阅和 req_id，可并发调用。
use crate::bus::{Bus, BusError};
use crate::send::{SendAction, SendRequest, TOPIC_SEND};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub const TOPIC_RESPONSE: &str = "luo9_api_response";
static SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub req_id: String,
    pub api: String,
    pub ok: bool,
    #[serde(default)]
    pub data: Value,
    #[serde(default)]
    pub retcode: Option<i64>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug)]
pub enum ApiError {
    Bus(BusError),
    Json(serde_json::Error),
    Timeout,
    Remote {
        api: String,
        retcode: Option<i64>,
        message: String,
    },
}
impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bus(e) => write!(f, "总线错误: {e:?}"),
            Self::Json(e) => write!(f, "JSON 错误: {e}"),
            Self::Timeout => write!(f, "NapCat API 请求超时"),
            Self::Remote {
                api,
                retcode,
                message,
            } => write!(f, "NapCat {api} 失败 ({retcode:?}): {message}"),
        }
    }
}
impl std::error::Error for ApiError {}
impl From<BusError> for ApiError {
    fn from(error: BusError) -> Self {
        Self::Bus(error)
    }
}
impl From<serde_json::Error> for ApiError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
pub type Result<T> = std::result::Result<T, ApiError>;

#[derive(Debug, Clone)]
pub struct ApiClient {
    timeout: Duration,
}
impl Default for ApiClient {
    fn default() -> Self {
        Self::new(Duration::from_secs(30))
    }
}

struct ResponseSubscription(usize);
impl Drop for ResponseSubscription {
    fn drop(&mut self) {
        let _ = Bus::topic(TOPIC_RESPONSE).unsubscribe(self.0);
    }
}

impl ApiResponse {
    pub fn into_result(self) -> Result<Value> {
        if self.ok {
            Ok(self.data)
        } else {
            Err(ApiError::Remote {
                api: self.api,
                retcode: self.retcode,
                message: self.error.unwrap_or_default(),
            })
        }
    }
}

impl ApiClient {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// 同步等待，仅用于插件线程；异步宿主应使用 connection::Sender::call。
    /// 成功返回 NapCat 的 data，失败保留 retcode 和错误信息。
    pub fn call(&self, api: &str, params: Value) -> Result<Value> {
        let subscription =
            ResponseSubscription(Bus::topic(TOPIC_RESPONSE).subscribe_independent()?);
        let req_id = format!(
            "{}-{}-{}",
            std::process::id(),
            subscription.0,
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        );
        let request = SendRequest {
            action: SendAction::Query {
                req_id: req_id.clone(),
                api: api.to_owned(),
                params,
                response_subscriber: Some(subscription.0),
            },
        };
        let started = Instant::now();
        Bus::topic(TOPIC_SEND).publish(&serde_json::to_string(&request)?)?;
        loop {
            if started.elapsed() >= self.timeout {
                return Err(ApiError::Timeout);
            }
            if let Some(raw) = Bus::topic(TOPIC_RESPONSE).pop(subscription.0) {
                if let Some(response) = matching_response(&raw, &req_id, api) {
                    return response.into_result();
                }
                // 非本请求（含 latch 旧消息）不会延长超时，也不会被当作结果。
                continue;
            }
            std::thread::sleep(
                Duration::from_millis(5).min(self.timeout.saturating_sub(started.elapsed())),
            );
        }
    }
}

fn matching_response(raw: &str, req_id: &str, api: &str) -> Option<ApiResponse> {
    let value: Value = serde_json::from_str(raw).ok()?;
    let response: ApiResponse = serde_json::from_value(value.get("ApiResponse")?.clone()).ok()?;
    (response.req_id == req_id && response.api == api).then_some(response)
}

/// 使用默认 30 秒等待上限调用任意 NapCat API。
pub fn call(api: &str, params: Value) -> Result<Value> {
    ApiClient::default().call(api, params)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn responses_are_correlated_and_preserve_remote_failures() {
        let raw = r#"{"ApiResponse":{"req_id":"a","api":"get_msg","ok":false,"retcode":1200,"error":"not found"}}"#;
        assert!(matching_response(raw, "b", "get_msg").is_none());
        assert!(matching_response(raw, "a", "get_image").is_none());
        assert!(matches!(
            matching_response(raw, "a", "get_msg")
                .unwrap()
                .into_result(),
            Err(ApiError::Remote {
                retcode: Some(1200),
                ..
            })
        ));
    }
}

impl ApiClient {
    pub fn get_login_info(&self) -> Result<Value> {
        self.call("get_login_info", serde_json::json!({}))
    }
    /// `get_status`：返回成功响应的 data。
    pub fn get_status(&self) -> Result<Value> {
        self.call("get_status", serde_json::json!({}))
    }
    /// `get_version_info`：返回成功响应的 data。
    pub fn get_version_info(&self) -> Result<Value> {
        self.call("get_version_info", serde_json::json!({}))
    }
    /// `get_friend_list`：返回成功响应的 data。
    pub fn get_friend_list(&self, no_cache: bool) -> Result<Value> {
        self.call("get_friend_list", serde_json::json!({"no_cache": no_cache}))
    }
    /// `get_stranger_info`：返回成功响应的 data。
    pub fn get_stranger_info(&self, user_id: u64) -> Result<Value> {
        self.call("get_stranger_info", serde_json::json!({"user_id": user_id}))
    }
    /// `get_group_list`：返回成功响应的 data。
    pub fn get_group_list(&self) -> Result<Value> {
        self.call("get_group_list", serde_json::json!({}))
    }
    /// `get_group_info`：返回成功响应的 data。
    pub fn get_group_info(&self, group_id: u64, no_cache: bool) -> Result<Value> {
        self.call(
            "get_group_info",
            serde_json::json!({"group_id": group_id, "no_cache": no_cache}),
        )
    }
    /// `get_group_member_info`：返回成功响应的 data。
    pub fn get_group_member_info(
        &self,
        group_id: u64,
        user_id: u64,
        no_cache: bool,
    ) -> Result<Value> {
        self.call(
            "get_group_member_info",
            serde_json::json!({"group_id": group_id, "user_id": user_id, "no_cache": no_cache}),
        )
    }
    /// `get_group_member_list`：返回成功响应的 data。
    pub fn get_group_member_list(&self, group_id: u64) -> Result<Value> {
        self.call(
            "get_group_member_list",
            serde_json::json!({"group_id": group_id}),
        )
    }
    /// `get_group_honor_info`：返回成功响应的 data。
    pub fn get_group_honor_info(&self, group_id: u64, kind: &str) -> Result<Value> {
        self.call(
            "get_group_honor_info",
            serde_json::json!({"group_id": group_id, "type": kind}),
        )
    }
    /// `get_msg`：返回成功响应的 data。
    pub fn get_msg(&self, message_id: i64) -> Result<Value> {
        self.call("get_msg", serde_json::json!({"message_id": message_id}))
    }
    /// `get_forward_msg`：返回成功响应的 data。
    pub fn get_forward_msg(&self, message_id: &str) -> Result<Value> {
        self.call(
            "get_forward_msg",
            serde_json::json!({"message_id": message_id}),
        )
    }
    /// `get_group_msg_history`：返回成功响应的 data。
    pub fn get_group_msg_history(
        &self,
        group_id: u64,
        message_seq: Option<&str>,
        count: u32,
        reverse_order: bool,
    ) -> Result<Value> {
        let mut params = serde_json::json!({"group_id": group_id, "message_seq": message_seq, "count": count, "reverse_order": reverse_order});
        if message_seq.is_none() {
            params.as_object_mut().unwrap().remove("message_seq");
        }
        self.call("get_group_msg_history", params)
    }
    /// `get_friend_msg_history`：返回成功响应的 data。
    pub fn get_friend_msg_history(
        &self,
        user_id: u64,
        message_seq: Option<&str>,
        count: u32,
        reverse_order: bool,
    ) -> Result<Value> {
        let mut params = serde_json::json!({"user_id": user_id, "message_seq": message_seq, "count": count, "reverse_order": reverse_order});
        if message_seq.is_none() {
            params.as_object_mut().unwrap().remove("message_seq");
        }
        self.call("get_friend_msg_history", params)
    }
    /// `get_image`：返回成功响应的 data。
    pub fn get_image(&self, file: &str) -> Result<Value> {
        self.call("get_image", serde_json::json!({"file": file}))
    }
    /// `get_file`：返回成功响应的 data。
    pub fn get_file(&self, file: &str) -> Result<Value> {
        self.call("get_file", serde_json::json!({"file": file}))
    }
    /// `get_record`：返回成功响应的 data。
    pub fn get_record(&self, file: &str, out_format: &str) -> Result<Value> {
        self.call(
            "get_record",
            serde_json::json!({"file": file, "out_format": out_format}),
        )
    }
    /// `ocr_image`：返回成功响应的 data。
    pub fn ocr_image(&self, image: &str) -> Result<Value> {
        self.call("ocr_image", serde_json::json!({"image": image}))
    }
    /// `can_send_image`：返回成功响应的 data。
    pub fn can_send_image(&self) -> Result<Value> {
        self.call("can_send_image", serde_json::json!({}))
    }
    /// `can_send_record`：返回成功响应的 data。
    pub fn can_send_record(&self) -> Result<Value> {
        self.call("can_send_record", serde_json::json!({}))
    }
    /// `send_like`：返回成功响应的 data。
    pub fn send_like(&self, user_id: u64, times: u32) -> Result<Value> {
        self.call(
            "send_like",
            serde_json::json!({"user_id": user_id, "times": times}),
        )
    }
    /// `set_friend_add_request`：返回成功响应的 data。
    pub fn set_friend_add_request(&self, flag: &str, approve: bool, remark: &str) -> Result<Value> {
        self.call(
            "set_friend_add_request",
            serde_json::json!({"flag": flag, "approve": approve, "remark": remark}),
        )
    }
    /// `set_group_add_request`：返回成功响应的 data。
    pub fn set_group_add_request(&self, flag: &str, approve: bool, reason: &str) -> Result<Value> {
        self.call(
            "set_group_add_request",
            serde_json::json!({"flag": flag, "approve": approve, "reason": reason}),
        )
    }
    /// `set_friend_remark`：返回成功响应的 data。
    pub fn set_friend_remark(&self, user_id: u64, remark: &str) -> Result<Value> {
        self.call(
            "set_friend_remark",
            serde_json::json!({"user_id": user_id, "remark": remark}),
        )
    }
    /// `set_group_ban`：返回成功响应的 data。
    pub fn set_group_ban(&self, group_id: u64, user_id: u64, duration: u32) -> Result<Value> {
        self.call(
            "set_group_ban",
            serde_json::json!({"group_id": group_id, "user_id": user_id, "duration": duration}),
        )
    }
    /// `set_group_kick`：返回成功响应的 data。
    pub fn set_group_kick(
        &self,
        group_id: u64,
        user_id: u64,
        reject_add_request: bool,
    ) -> Result<Value> {
        self.call("set_group_kick", serde_json::json!({"group_id": group_id, "user_id": user_id, "reject_add_request": reject_add_request}))
    }
    /// `set_group_whole_ban`：返回成功响应的 data。
    pub fn set_group_whole_ban(&self, group_id: u64, enable: bool) -> Result<Value> {
        self.call(
            "set_group_whole_ban",
            serde_json::json!({"group_id": group_id, "enable": enable}),
        )
    }
    /// `set_group_admin`：返回成功响应的 data。
    pub fn set_group_admin(&self, group_id: u64, user_id: u64, enable: bool) -> Result<Value> {
        self.call(
            "set_group_admin",
            serde_json::json!({"group_id": group_id, "user_id": user_id, "enable": enable}),
        )
    }
    /// `set_group_card`：返回成功响应的 data。
    pub fn set_group_card(&self, group_id: u64, user_id: u64, card: &str) -> Result<Value> {
        self.call(
            "set_group_card",
            serde_json::json!({"group_id": group_id, "user_id": user_id, "card": card}),
        )
    }
    /// `set_group_name`：返回成功响应的 data。
    pub fn set_group_name(&self, group_id: u64, group_name: &str) -> Result<Value> {
        self.call(
            "set_group_name",
            serde_json::json!({"group_id": group_id, "group_name": group_name}),
        )
    }
    /// `set_group_leave`：返回成功响应的 data。
    pub fn set_group_leave(&self, group_id: u64, is_dismiss: bool) -> Result<Value> {
        self.call(
            "set_group_leave",
            serde_json::json!({"group_id": group_id, "is_dismiss": is_dismiss}),
        )
    }
    /// `set_group_special_title`：返回成功响应的 data。
    pub fn set_group_special_title(
        &self,
        group_id: u64,
        user_id: u64,
        special_title: &str,
    ) -> Result<Value> {
        self.call("set_group_special_title", serde_json::json!({"group_id": group_id, "user_id": user_id, "special_title": special_title}))
    }
    /// `group_poke`：返回成功响应的 data。
    pub fn group_poke(&self, group_id: u64, user_id: u64) -> Result<Value> {
        self.call(
            "group_poke",
            serde_json::json!({"group_id": group_id, "user_id": user_id}),
        )
    }
    /// `friend_poke`：返回成功响应的 data。
    pub fn friend_poke(&self, user_id: u64) -> Result<Value> {
        self.call("friend_poke", serde_json::json!({"user_id": user_id}))
    }
    /// `set_self_longnick`：返回成功响应的 data。
    pub fn set_self_longnick(&self, long_nick: &str) -> Result<Value> {
        self.call(
            "set_self_longnick",
            serde_json::json!({"longNick": long_nick}),
        )
    }
    /// `set_input_status`：返回成功响应的 data。
    pub fn set_input_status(&self, user_id: u64, event_type: u32) -> Result<Value> {
        self.call(
            "set_input_status",
            serde_json::json!({"user_id": user_id, "event_type": event_type}),
        )
    }
    /// `get_essence_msg_list`：返回成功响应的 data。
    pub fn get_essence_msg_list(&self, group_id: u64) -> Result<Value> {
        self.call(
            "get_essence_msg_list",
            serde_json::json!({"group_id": group_id}),
        )
    }
    /// `set_essence_msg`：返回成功响应的 data。
    pub fn set_essence_msg(&self, message_id: i64) -> Result<Value> {
        self.call(
            "set_essence_msg",
            serde_json::json!({"message_id": message_id}),
        )
    }
    /// `delete_essence_msg`：返回成功响应的 data。
    pub fn delete_essence_msg(&self, message_id: i64) -> Result<Value> {
        self.call(
            "delete_essence_msg",
            serde_json::json!({"message_id": message_id}),
        )
    }
    /// `upload_group_file`：返回成功响应的 data。
    pub fn upload_group_file(
        &self,
        group_id: u64,
        file: &str,
        name: &str,
        folder: Option<&str>,
    ) -> Result<Value> {
        let mut params =
            serde_json::json!({"group_id": group_id, "file": file, "name": name, "folder": folder});
        if folder.is_none() {
            params.as_object_mut().unwrap().remove("folder");
        }
        self.call("upload_group_file", params)
    }
    /// `upload_private_file`：返回成功响应的 data。
    pub fn upload_private_file(&self, user_id: u64, file: &str, name: &str) -> Result<Value> {
        self.call(
            "upload_private_file",
            serde_json::json!({"user_id": user_id, "file": file, "name": name}),
        )
    }
    /// `get_group_root_files`：返回成功响应的 data。
    pub fn get_group_root_files(&self, group_id: u64) -> Result<Value> {
        self.call(
            "get_group_root_files",
            serde_json::json!({"group_id": group_id}),
        )
    }
    /// `get_group_files_by_folder`：返回成功响应的 data。
    pub fn get_group_files_by_folder(&self, group_id: u64, folder_id: &str) -> Result<Value> {
        self.call(
            "get_group_files_by_folder",
            serde_json::json!({"group_id": group_id, "folder_id": folder_id}),
        )
    }
    /// `get_group_file_url`：返回成功响应的 data。
    pub fn get_group_file_url(&self, group_id: u64, file_id: &str) -> Result<Value> {
        self.call(
            "get_group_file_url",
            serde_json::json!({"group_id": group_id, "file_id": file_id}),
        )
    }
    /// `get_group_file_system_info`：返回成功响应的 data。
    pub fn get_group_file_system_info(&self, group_id: u64) -> Result<Value> {
        self.call(
            "get_group_file_system_info",
            serde_json::json!({"group_id": group_id}),
        )
    }
    /// `create_group_file_folder`：返回成功响应的 data。
    pub fn create_group_file_folder(&self, group_id: u64, name: &str) -> Result<Value> {
        self.call(
            "create_group_file_folder",
            serde_json::json!({"group_id": group_id, "name": name}),
        )
    }
    /// `delete_group_folder`：返回成功响应的 data。
    pub fn delete_group_folder(&self, group_id: u64, folder_id: &str) -> Result<Value> {
        self.call(
            "delete_group_folder",
            serde_json::json!({"group_id": group_id, "folder_id": folder_id}),
        )
    }
    /// `delete_group_file`：返回成功响应的 data。
    pub fn delete_group_file(&self, group_id: u64, file_id: &str) -> Result<Value> {
        self.call(
            "delete_group_file",
            serde_json::json!({"group_id": group_id, "file_id": file_id}),
        )
    }
    /// `_send_group_notice`：返回成功响应的 data。
    pub fn send_group_notice(&self, group_id: u64, content: &str) -> Result<Value> {
        self.call(
            "_send_group_notice",
            serde_json::json!({"group_id": group_id, "content": content}),
        )
    }
    /// `_get_group_notice`：返回成功响应的 data。
    pub fn get_group_notice(&self, group_id: u64) -> Result<Value> {
        self.call(
            "_get_group_notice",
            serde_json::json!({"group_id": group_id}),
        )
    }
    /// `_del_group_notice`：返回成功响应的 data。
    pub fn delete_group_notice(&self, group_id: u64, notice_id: &str) -> Result<Value> {
        self.call(
            "_del_group_notice",
            serde_json::json!({"group_id": group_id, "notice_id": notice_id}),
        )
    }
    /// `get_group_at_all_remain`：返回成功响应的 data。
    pub fn get_group_at_all_remain(&self, group_id: u64) -> Result<Value> {
        self.call(
            "get_group_at_all_remain",
            serde_json::json!({"group_id": group_id}),
        )
    }
    /// `get_credentials`：返回成功响应的 data。
    pub fn get_credentials(&self, domain: &str) -> Result<Value> {
        self.call("get_credentials", serde_json::json!({"domain": domain}))
    }
    /// `send_group_msg`：返回成功响应的 data。
    pub fn send_group_segments(
        &self,
        group_id: u64,
        message: Vec<crate::segment::Segment>,
    ) -> Result<Value> {
        self.call(
            "send_group_msg",
            serde_json::json!({"group_id": group_id, "message": message}),
        )
    }
    /// `send_private_msg`：返回成功响应的 data。
    pub fn send_private_segments(
        &self,
        user_id: u64,
        message: Vec<crate::segment::Segment>,
    ) -> Result<Value> {
        self.call(
            "send_private_msg",
            serde_json::json!({"user_id": user_id, "message": message}),
        )
    }
    /// `send_group_forward_msg`：返回成功响应的 data。
    pub fn send_group_forward_msg(
        &self,
        group_id: u64,
        messages: Vec<crate::segment::Segment>,
    ) -> Result<Value> {
        self.call(
            "send_group_forward_msg",
            serde_json::json!({"group_id": group_id, "messages": messages}),
        )
    }
    /// `send_private_forward_msg`：返回成功响应的 data。
    pub fn send_private_forward_msg(
        &self,
        user_id: u64,
        messages: Vec<crate::segment::Segment>,
    ) -> Result<Value> {
        self.call(
            "send_private_forward_msg",
            serde_json::json!({"user_id": user_id, "messages": messages}),
        )
    }
    /// `delete_msg`：返回成功响应的 data。
    pub fn delete_msg_signed(&self, message_id: i64) -> Result<Value> {
        self.call("delete_msg", serde_json::json!({"message_id": message_id}))
    }
    /// `set_msg_emoji_like`：返回成功响应的 data。
    pub fn set_msg_emoji_like_signed(&self, message_id: i64, emoji_id: &str) -> Result<Value> {
        self.call(
            "set_msg_emoji_like",
            serde_json::json!({"message_id": message_id, "emoji_id": emoji_id}),
        )
    }
}
