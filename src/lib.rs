use libc::c_char;
use std::ffi::CString;


pub mod command;
pub mod message;
pub mod pattern;
pub mod bus;
pub mod payload;
pub mod send;


#[cfg(test)]
pub mod tests;

pub struct Bot;
pub struct Msg;

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

    /// 发送私聊消息（基于 bus 总线，fire-and-forget）
    pub fn send_private_msg(user_id: u64, message: CString) -> Option<()> {
        let msg = message.to_str().ok()?;
        send::send_private_msg(user_id, msg)
    }
}

unsafe extern "C" {
    /// 获取核心版本信息
    pub fn luo9_version() -> *const c_char;
}
