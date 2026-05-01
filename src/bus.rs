// src/bus.rs

use libc::c_char;
use std::ffi::{CStr, CString};

#[link(name = "luo9_core")]
unsafe extern "C" {
    unsafe fn luo9_bus_init() -> libc::c_int;
    unsafe fn luo9_bus_subscribe(topic: *const c_char) -> libc::c_int;
    unsafe fn luo9_bus_publish(topic: *const c_char, payload: *const c_char) -> libc::c_int;
    unsafe fn luo9_bus_pop(topic: *const c_char, subscriber_id: libc::c_int) -> *mut c_char;
    unsafe fn luo9_bus_wait_pop(topic: *const c_char, subscriber_id: libc::c_int) -> *mut c_char;
    unsafe fn luo9_bus_free_string(ptr: *mut c_char);
}

pub struct Bus;

impl Bus {
    pub fn init() -> Result<(), BusError> {
        let ret = unsafe { luo9_bus_init() };
        match ret {
            0 => Ok(()),
            _ => Err(BusError::InitFailed),
        }
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
    /// 如果该 topic 已有 latch 消息（即之前曾 publish 过），
    /// 新订阅者会自动在队列中收到最后一条消息。
    pub fn subscribe(&self) -> Result<usize, BusError> {
        let topic = CString::new(self.name).map_err(|_| BusError::InvalidString)?;
        let ret = unsafe { luo9_bus_subscribe(topic.as_ptr()) };
        match ret {
            id if id >= 0 => Ok(id as usize),
            _ => Err(BusError::SubscribeFailed),
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

    /// 非阻塞 pop：指定订阅者队列为空返回 None
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
        Some(msg)
    }

    /// 阻塞 pop：指定订阅者队列为空时挂起线程，有消息时立即返回
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
}
