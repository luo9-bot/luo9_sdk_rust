// src/bus.rs

use libc::c_char;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::sync::{Mutex, OnceLock};

/// 取消订阅时返回的哨兵消息
pub const SENTINEL: &str = "__luo9_unsubscribed__";

/// 宿主预分配的 subscriber_id 映射（topic -> subscriber_id）
///
/// 由 `luo9_init_subscribers` 填充，`Topic::subscribe()` 优先从此映射中查找。
pub(crate) static PRECREATED_SUBSCRIBERS: OnceLock<Mutex<HashMap<String, usize>>> = OnceLock::new();

#[link(name = "luo9_core")]
unsafe extern "C" {
    unsafe fn luo9_bus_init() -> libc::c_int;
    unsafe fn luo9_bus_subscribe(topic: *const c_char) -> libc::c_int;
    unsafe fn luo9_bus_unsubscribe(topic: *const c_char, subscriber_id: libc::c_int) -> libc::c_int;
    unsafe fn luo9_bus_publish(topic: *const c_char, payload: *const c_char) -> libc::c_int;
    unsafe fn luo9_bus_publish_to(
        topic: *const c_char,
        payload: *const c_char,
        subscriber_ids_ptr: *const libc::c_int,
        subscriber_ids_len: libc::c_int,
    ) -> libc::c_int;
    unsafe fn luo9_bus_pop(topic: *const c_char, subscriber_id: libc::c_int) -> *mut c_char;
    unsafe fn luo9_bus_wait_pop(topic: *const c_char, subscriber_id: libc::c_int) -> *mut c_char;
    unsafe fn luo9_bus_free_string(ptr: *mut c_char);
}

pub struct Bus;

impl Bus {
    /// 初始化总线
    pub fn init() -> Result<(), BusError> {
        let ret = unsafe { luo9_bus_init() };
        // bus_init 可能返回 -1（已初始化），这是可接受的
        if ret != 0 && ret != -1 {
            return Err(BusError::InitFailed);
        }
        Ok(())
    }

    pub fn topic<'a>(name: &'a str) -> Topic<'a> {
        Topic { name }
    }
}

pub struct Topic<'a> {
    name: &'a str,
}

impl<'a> Topic<'a> {
    /// 订阅该 topic，返回 subscriber_id
    ///
    /// 若宿主已通过 `luo9_init_subscribers` 预分配了 subscriber_id，
    /// 则直接返回预分配的 ID，不实际调用 FFI subscribe。
    ///
    /// 如果该 topic 已有 latch 消息（即之前曾 publish 过），
    /// 新订阅者会自动在队列中收到最后一条消息。
    pub fn subscribe(&self) -> Result<usize, BusError> {
        // 检查是否有预分配的 subscriber_id
        if let Some(map) = PRECREATED_SUBSCRIBERS.get() {
            if let Some(&id) = map.lock().unwrap().get(self.name) {
                return Ok(id);
            }
        }

        let topic = CString::new(self.name).map_err(|_| BusError::InvalidString)?;
        let ret = unsafe { luo9_bus_subscribe(topic.as_ptr()) };
        match ret {
            id if id >= 0 => Ok(id as usize),
            _ => Err(BusError::SubscribeFailed),
        }
    }

    /// 取消订阅：标记为 dead，唤醒阻塞的 wait_pop 返回哨兵
    pub fn unsubscribe(&self, subscriber_id: usize) -> Result<(), BusError> {
        let topic = CString::new(self.name).map_err(|_| BusError::InvalidString)?;
        let ret = unsafe { luo9_bus_unsubscribe(topic.as_ptr(), subscriber_id as libc::c_int) };
        match ret {
            0 => Ok(()),
            _ => Err(BusError::UnsubscribeFailed),
        }
    }

    pub fn publish(&self, payload: &str) -> Result<(), BusError> {
        let topic = CString::new(self.name).map_err(|_| BusError::InvalidString)?;
        let payload = CString::new(payload).map_err(|_| BusError::InvalidString)?;

        let ret = unsafe { luo9_bus_publish(topic.as_ptr(), payload.as_ptr()) };
        match ret {
            0 => Ok(()),
            -2 => Err(BusError::NotInitialized),
            _ => Err(BusError::PublishFailed),
        }
    }

    /// 定向发布：只向指定的 subscriber 推送消息
    pub fn publish_to(&self, payload: &str, subscriber_ids: &[usize]) -> Result<(), BusError> {
        let topic = CString::new(self.name).map_err(|_| BusError::InvalidString)?;
        let payload = CString::new(payload).map_err(|_| BusError::InvalidString)?;

        let ids_cint: Vec<libc::c_int> = subscriber_ids.iter().map(|&id| id as libc::c_int).collect();

        let ret = unsafe {
            luo9_bus_publish_to(
                topic.as_ptr(),
                payload.as_ptr(),
                ids_cint.as_ptr(),
                ids_cint.len() as libc::c_int,
            )
        };
        match ret {
            0 => Ok(()),
            -2 => Err(BusError::NotInitialized),
            _ => Err(BusError::PublishFailed),
        }
    }

    /// 非阻塞 pop：指定订阅者队列为空返回 None
    /// 若 subscriber 已被取消订阅，返回 `Err(BusError::Unsubscribed)`
    pub fn pop(&self, subscriber_id: usize) -> Option<String> {
        let topic = CString::new(self.name).ok()?;
        let ptr = unsafe { luo9_bus_pop(topic.as_ptr(), subscriber_id as libc::c_int) };
        if ptr.is_null() {
            return None;
        }
        let msg = unsafe {
            let s = CStr::from_ptr(ptr).to_string_lossy().to_string();
            luo9_bus_free_string(ptr);
            s
        };
        if msg == SENTINEL {
            return None; // sentinel 视为无消息
        }
        Some(msg)
    }

    /// 阻塞 pop：指定订阅者队列为空时挂起线程，有消息时立即返回
    /// 若 subscriber 已被取消订阅，返回 `Err(BusError::Unsubscribed)`
    pub fn wait_pop(&self, subscriber_id: usize) -> Result<String, BusError> {
        let topic = CString::new(self.name).map_err(|_| BusError::InvalidString)?;
        let ptr = unsafe { luo9_bus_wait_pop(topic.as_ptr(), subscriber_id as libc::c_int) };
        if ptr.is_null() {
            return Err(BusError::WaitPopFailed);
        }
        let msg = unsafe {
            let s = CStr::from_ptr(ptr).to_string_lossy().to_string();
            luo9_bus_free_string(ptr);
            s
        };
        if msg == SENTINEL {
            return Err(BusError::Unsubscribed);
        }
        Ok(msg)
    }

    pub fn publish_fmt(&self, payload: impl ToString) -> Result<(), BusError> {
        self.publish(&payload.to_string())
    }
}

#[derive(Debug)]
pub enum BusError {
    InitFailed,
    PublishFailed,
    SubscribeFailed,
    WaitPopFailed,
    NotInitialized,
    InvalidString,
    UnsubscribeFailed,
    /// subscriber 已被取消订阅（收到哨兵消息）
    Unsubscribed,
}
