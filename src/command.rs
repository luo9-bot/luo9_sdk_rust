use libc::{c_char, c_int};
use std::{collections::HashMap, ffi::{CStr, CString}};
use crate::pattern::Pattern;



#[repr(C)]
pub struct CommandHandle {
    _private: [u8; 0],
}

unsafe extern "C" {
    unsafe fn luo9_command_create(
        msg: *const c_char,
        cmd_name: *const c_char,
        mode: c_int,
        prefix_char: c_char,
    ) -> *mut CommandHandle;

    unsafe fn luo9_command_free(handle: *mut CommandHandle);
    unsafe fn luo9_command_get_name(handle: *const CommandHandle) -> *mut c_char;
    unsafe fn luo9_command_get_args_raw(handle: *const CommandHandle) -> *mut c_char;
    unsafe fn luo9_command_has_args(handle: *const CommandHandle) -> c_int;
    unsafe fn luo9_command_args_count(handle: *const CommandHandle) -> c_int;
    unsafe fn luo9_command_get_arg(handle: *const CommandHandle, index: c_int) -> *mut c_char;
    unsafe fn luo9_free_string(ptr: *mut c_char);
}

#[derive(Debug, Clone, Copy)]
pub enum PrefixMode {
    Required(char),
    Optional(char),
    None,
}

#[derive(Debug)]
pub struct Command {
    handle: *mut CommandHandle,
    name: String,
    args: Vec<String>,
}

// TODO: 添加命令处理单参数后, 无后续参数的情况
// 例如: epic提醒开启 -> 此时无后续参数, 判断逻辑成立
// 例如: epic提醒开启 123 -> 此时有后续参数, 判断逻辑不成立

impl Command {
    pub fn parse(msg: &str, cmd_name: &str, mode: PrefixMode) -> Option<Self> {
        let msg_c = CString::new(msg).ok()?;
        let cmd_c = CString::new(cmd_name).ok()?;
        let (mode_val, prefix) = match mode {
            PrefixMode::Required(c) => (0, c as c_char),
            PrefixMode::Optional(c) => (1, c as c_char),
            PrefixMode::None => (2, 0),
        };

        let handle = unsafe {
            luo9_command_create(msg_c.as_ptr(), cmd_c.as_ptr(), mode_val, prefix)
        };
        if handle.is_null() {
            return None;
        }

        // 获取名称和所有参数并缓存
        let name = unsafe {
            let ptr = luo9_command_get_name(handle);
            let s = CStr::from_ptr(ptr).to_string_lossy().to_string();
            luo9_free_string(ptr);
            s
        };
        let count = unsafe { luo9_command_args_count(handle) as usize };
        let mut args = Vec::with_capacity(count);
        for i in 0..count {
            let arg = unsafe {
                let ptr = luo9_command_get_arg(handle, i as c_int);
                let s = CStr::from_ptr(ptr).to_string_lossy().to_string();
                luo9_free_string(ptr);
                s
            };
            args.push(arg);
        }

        Some(Self { handle, name, args })
    }

     pub fn name(&self) -> &str {
        &self.name
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    pub fn arg_at(&self, index: usize) -> Option<&str> {
        self.args.get(index).map(String::as_str)
    }

    /// 从指定索引开始的剩余参数切片（用于传递给闭包）
    pub fn args_from(&self, start: usize) -> &[String] {
        &self.args[start..]
    }

    pub fn args_raw(&self) -> String {
        unsafe {
            let ptr = luo9_command_get_args_raw(self.handle);
            let s = CStr::from_ptr(ptr).to_string_lossy().to_string();
            luo9_free_string(ptr);
            s
        }
    }

    pub fn has_args(&self) -> bool {
        unsafe { luo9_command_has_args(self.handle) == 1 }
    }

    pub fn args_count(&self) -> usize {
        unsafe { luo9_command_args_count(self.handle) as usize }
    }

    /// 如果命令无参数，执行闭包并返回一个已匹配的 Matcher（后续 on 不会触发）
    /// 如果有参数，返回未匹配的 Matcher，可以继续链式调用
    pub fn handle<'a, F>(&'a self, f: F) -> CommandMatcher<'a>
    where
        F: FnOnce() + 'a,
    {
        let matched = !self.has_args();
        if matched {
            f();
        }
        CommandMatcher {
            cmd: self,
            matched,
        }
    }

    /// 启动链式匹配
    pub fn on<'a, F>(&'a self, expected: &'a str, f: F) -> CommandMatcher<'a>
    where
        F: FnOnce(&[String]) + 'a
    {
        let matched = self.arg_at(0) == Some(expected);
        if matched {
            f(self.args_from(1));
        }
        CommandMatcher {
            cmd: self,
            matched
        }
    }

    pub fn on_pattern<'a, F>(&'a self, pattern: &'a str, f: F) -> CommandMatcher<'a>
    where
        F: FnOnce(HashMap<String, String>, &[String]) + 'a
    {
        let mut matched = false;
        if let Some(first_arg) = self.arg_at(0) {
            let pat = Pattern::new(pattern);
            if let Some(captures) = pat.match_str(first_arg) {
                f(captures, self.args_from(1));
                matched = true;
            }
        }
        CommandMatcher {
            cmd: self,
            matched
        }
    }
}

impl Drop for Command {
    fn drop(&mut self) {
        unsafe {
            luo9_command_free(self.handle);
        }
    }
}

pub struct CommandMatcher<'a> {
    cmd: &'a Command,
    matched: bool,
}

impl<'a> CommandMatcher<'a> {
    pub fn on<F>(mut self, expected: &str, f: F) -> Self
    where
        F: FnOnce(&[String]),
    {
        if !self.matched {
            if self.cmd.arg_at(0) == Some(expected) {
                f(self.cmd.args_from(1));
                self.matched = true;
            }
        }
        self
    }

    pub fn on_pattern<F>(mut self, pattern: &str, f: F) -> Self
    where
        F: FnOnce(HashMap<String, String>, &[String]),
    {
        if !self.matched {
            if let Some(first_arg) = self.cmd.arg_at(0) {
                let pat = Pattern::new(pattern);
                if let Some(captures) = pat.match_str(first_arg) {
                    f(captures, self.cmd.args_from(1));
                    self.matched = true;
                }
            }
        }
        self
    }

    pub fn otherwise<F>(&mut self, f: F)
    where
        F: FnOnce(),
    {
        if !self.matched {
            f();
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;
   
    // #[test]
    fn test_optional_prefix_echo() {
        let cmd = Command::parse("/echo hello world", "echo", PrefixMode::Optional('/')).unwrap();
        assert_eq!(cmd.name(), "echo");
        assert_eq!(cmd.args_raw(), " hello world");
        assert_eq!(cmd.has_args(), true);
        assert_eq!(cmd.args_count(), 2);
        assert_eq!(cmd.arg_at(0), Some("hello"));
        assert_eq!(cmd.arg_at(1), Some("world"));
    }

    // #[test]
    fn test_required_prefix_echo() {
        let cmd = Command::parse("epic提醒关闭", "epic", PrefixMode::None).unwrap();

        cmd.on("提醒开启", |_| {
            println!("提醒开启");
        })
        .on("提醒关闭", |_| {
            println!("提醒关闭");
        })
        .on("状态", |_| {
            println!("状态");
        });

    }

    #[test]
    fn test_status() {
        let cmd = Command::parse("epic[CQ:at,qq=123123321321]状态", "epic", PrefixMode::None).unwrap();
        cmd.on_pattern("[CQ:at,qq={qq}]{content}", |caps, args| {
            let qq = caps.get("qq").unwrap();
            println!("提取到 QQ: {}", qq);  // 现在会输出
            let content = caps.get("content").unwrap();
            println!("提取到 内容: {}", content);  // 现在会输出
            if let Some(subcmd) = args.first() {
                match subcmd.as_str() {
                    "状态" => println!("执行状态查询"),
                    _ => {}
                }
            }
        });
    }

    #[test]
    fn test_only_command() {
        let cmd = Command::parse("epic", "epic", PrefixMode::None).unwrap();
        // 只有命令文本的情况下,命令：epic 文本：epic
        cmd.on("epic", |_| {
            println!("epic");
        });

    }


}