# Luo9 SDK (Rust)

[![Crates.io](https://img.shields.io/crates/v/luo9_sdk.svg)](https://crates.io/crates/luo9_sdk)
[![文档](https://img.shields.io/badge/docs-drluo.top|luo9_sdk-blue.svg)](https://www.drluo.top/posts/luo9_sdk)
[![许可证: GPL-3.0](https://img.shields.io/badge/License-GPL%203.0-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

一个用于开发洛玖 (Luo9) 机器人插件的 Rust SDK。

## 功能特性

- **命令解析**：灵活的命令解析系统，支持前缀模式和链式匹配
- **消息构建**：流式 API 构建复杂消息（文本、@用户、图片）
- **消息总线**：发布/订阅模式的消息系统，支持多订阅者
- **事件处理**：完整的 OneBot 标准事件解析（消息、通知、元事件）
- **消息发送**：基于总线的异步消息发送

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
luo9_sdk = "0.5"
```

## 快速开始

### 初始化

```rust
use luo9_sdk::bus::Bus;

// 初始化消息总线
Bus::init().expect("Failed to init bus");
```

### 命令解析

```rust
use luo9_sdk::command::{Command, PrefixMode};

fn handle_message(msg: &str) {
    let cmd = Command::parse(msg, "help", PrefixMode::Required('/'));

    if let Some(cmd) = cmd {
        cmd.on("start", |args| {
            println!("启动帮助系统");
        })
        .on("about", |args| {
            println!("关于信息");
        })
        .otherwise(|| {
            println!("未知子命令");
        });
    }
}
```

### 模式匹配

```rust
use luo9_sdk::command::{Command, PrefixMode};

let cmd = Command::parse("epic[CQ:at,qq=123456]状态", "epic", PrefixMode::None).unwrap();

cmd.on_pattern("[CQ:at,qq={qq}]{content}", |caps, args| {
    let qq = caps.get("qq").unwrap();
    let content = caps.get("content").unwrap();
    println!("QQ: {}, 内容: {}", qq, content);
});
```

### 消息构建

```rust
use luo9_sdk::message::Msg;

// 构建包含多种元素的消息
let msg = Msg::txt("Hello ")
    .at(123456)  // @用户
    .endl()      // 换行
    .txt("这是一条消息")
    .image("https://example.com/image.png")  // 网络图片
    .build();

// 发送消息
use luo9_sdk::Bot;
Bot::send_group_msg(987654, msg);
```

### 消息总线

```rust
use luo9_sdk::bus::Bus;

// 订阅消息
let sub_id = Bus::topic("luo9_message").subscribe().unwrap();

// 非阻塞获取
if let Some(msg) = Bus::topic("luo9_message").pop(sub_id) {
    println!("收到消息: {}", msg);
}

// 阻塞等待
match Bus::topic("luo9_message").wait_pop(sub_id) {
    Ok(msg) => println!("收到消息: {}", msg),
    Err(e) => eprintln!("错误: {:?}", e),
}
```

### 事件处理

```rust
use luo9_sdk::payload::{BusPayload, MsgType};

fn handle_event(json: &str) {
    if let Some(payload) = BusPayload::parse(json) {
        match payload {
            BusPayload::Message(msg) => {
                match msg.message_type {
                    MsgType::Private => println!("私聊消息: {}", msg.message),
                    MsgType::Group => println!("群消息: {}", msg.message),
                    _ => {}
                }
            }
            BusPayload::Notice(notice) => {
                println!("通知: {:?}", notice.notice_type);
            }
            BusPayload::MetaEvent(meta) => {
                println!("元事件: {:?}", meta.meta_event_type);
            }
        }
    }
}
```

## 模块结构

| 模块 | 说明 |
|------|------|
| `command` | 命令解析，支持前缀模式和链式匹配 |
| `message` | 消息构建器，流式 API |
| `pattern` | 模式匹配，支持变量捕获 |
| `bus` | 消息总线，发布/订阅系统 |
| `payload` | 事件载荷解析 |
| `send` | 消息发送 API |

## 事件类型

### 消息事件
- `MsgType::Private` - 私聊消息
- `MsgType::Group` - 群消息

### 通知事件
- `NoticeType::FriendAdd` - 好友添加
- `NoticeType::FriendRecall` - 好友消息撤回
- `NoticeType::GroupAdmin` - 群管理员变更
- `NoticeType::GroupBan` - 群禁言
- `NoticeType::GroupIncrease` - 群成员增加
- `NoticeType::GroupDecrease` - 群成员减少
- `NoticeType::GroupRecall` - 群消息撤回
- `NoticeType::GroupUpload` - 群文件上传
- 更多...

### 元事件
- `MetaEventType::Lifecycle` - 生命周期事件
- `MetaEventType::Heartbeat` - 心跳事件

## 依赖

- `libc` - C FFI 支持
- `libloading` - 动态库加载
- `serde` / `serde_json` - JSON 序列化

## 许可证

本项目采用 [GPL-3.0](LICENSE) 许可证。

## 相关链接

- [文档](https://www.drluo.top/posts/luo9_sdk)
- [GitHub](https://github.com/luo9-bot/luo9_sdk_rust)
- [Crate](https://crates.io/crates/luo9_sdk)
