use libc::c_char;
use std::ffi::CString;

pub mod bus;
pub mod command;
pub mod message;
pub mod pattern;
pub mod payload;
pub mod send;
pub mod version;

#[cfg(test)]
pub mod tests;

pub struct Bot;
pub struct Msg;

/// 宿主传递给插件的预创建 subscriber ID 集合
///
/// 宿主在加载插件时为每个 topic 创建 subscriber，然后通过此结构传递给插件。
/// 插件调用 `luo9_init_subscribers` 后，SDK 的 `Bus::topic(...).subscribe()`
/// 将直接返回预分配的 ID，不再自行创建 subscriber。
#[repr(C)]
pub struct PluginSubscribers {
    pub message_sub_id: libc::c_int,
    pub meta_event_sub_id: libc::c_int,
    pub notice_sub_id: libc::c_int,
    pub request_sub_id: libc::c_int,
    pub task_sub_id: libc::c_int,
    pub send_sub_id: libc::c_int,
}

/// 插件 DLL 导出的初始化函数签名
///
/// 宿主通过 `libloading` 调用此函数，传入预创建的 subscriber ID 集合。
/// 插件的 SDK 会将这些 ID 存储到全局 HashMap 中，
/// 后续 `Bus::topic("luo9_message").subscribe()` 等调用将直接返回预分配的 ID。
///
/// # Safety
/// `subscribers` 指针必须非空且指向有效的 `PluginSubscribers` 结构。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn luo9_init_subscribers(subscribers: *const PluginSubscribers) {
    if subscribers.is_null() {
        return;
    }
    let subs = unsafe { &*subscribers };

    let mapping = [
        ("luo9_message", subs.message_sub_id),
        ("luo9_meta_event", subs.meta_event_sub_id),
        ("luo9_notice", subs.notice_sub_id),
        ("luo9_request", subs.request_sub_id),
        ("luo9_task", subs.task_sub_id),
        ("luo9_send", subs.send_sub_id),
    ];

    for (topic, sub_id) in mapping {
        if sub_id >= 0 {
            let _ = bus::PRECREATED_SUBSCRIBERS
                .get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
            if let Some(map) = bus::PRECREATED_SUBSCRIBERS.get() {
                map.lock()
                    .unwrap()
                    .insert(topic.to_string(), sub_id as usize);
            }
        }
    }
}

impl Bot {
    pub fn get_version() -> String {
        unsafe {
            let ptr = luo9_version();
            if !ptr.is_null() {
                let c_string = CString::from_raw(ptr as *mut i8);
                c_string.into_string().unwrap_or_default()
            } else {
                String::new()
            }
        }
    }

    /// 发送群消息（基于 bus 总线，fire-and-forget）
    pub fn send_group_msg(group_id: u64, message: CString) -> Option<()> {
        let msg = message.to_str().ok()?;
        send::send_group_msg(group_id, msg)
    }

    /// 发送群消息并引用指定消息（CQ:reply 组合）
    pub fn send_group_msg_reply(
        group_id: u64,
        reply_to_message_id: u64,
        message: CString,
    ) -> Option<()> {
        let msg = message.to_str().ok()?;
        send::send_group_msg_reply(group_id, reply_to_message_id, msg)
    }

    /// 发送私聊消息（基于 bus 总线，fire-and-forget）
    pub fn send_private_msg(user_id: u64, message: CString) -> Option<()> {
        let msg = message.to_str().ok()?;
        send::send_private_msg(user_id, msg)
    }

    /// 发送私聊消息并引用指定消息（CQ:reply 组合）
    pub fn send_private_msg_reply(
        user_id: u64,
        reply_to_message_id: u64,
        message: CString,
    ) -> Option<()> {
        let msg = message.to_str().ok()?;
        send::send_private_msg_reply(user_id, reply_to_message_id, msg)
    }

    /// 撤回消息（需要先通过 luo9_sent 回执拿到自己消息的 message_id）
    pub fn delete_msg(message_id: u64) -> Option<()> {
        send::delete_msg(message_id)
    }

    /// 对一条消息设置表情回应（emoji_id 为 QQ 表情数字 ID）
    pub fn set_msg_emoji_like(message_id: u64, emoji_id: u64) -> Option<()> {
        send::set_msg_emoji_like(message_id, emoji_id)
    }
}

#[link(name = "luo9_core")]
unsafe extern "C" {
    /// 获取核心版本信息
    pub fn luo9_version() -> *const c_char;
}
