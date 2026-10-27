# Luo9 SDK (Rust)

[![Crates.io](https://img.shields.io/crates/v/luo9_sdk.svg)](https://crates.io/crates/luo9_sdk)
[![Documentation](https://img.shields.io/badge/docs-drluo.top|luo9_sdk-blue.svg)](https://www.drluo.top/posts/luo9_sdk)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL%203.0-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

一个用于开发洛玖 (Luo9) 机器人插件的 Rust SDK。

[English](README.md) | [中文](README-CN.md)

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
luo9_sdk = "0.8.0-beta.3"
```

源码联调时可使用 `luo9_sdk = { version = "0.8.0-beta.3", path = "../luo9_sdk_rust" }`。
该版本应先发布 SDK，再发布依赖它的插件；本次源码更新不自动发布到 crates.io。

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
use luo9_sdk::Msg;

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


## NapCat API（0.8.0-beta.3）

需配合 `luo9_bot 0.1.0-beta.13`。新增 `napcat::ApiClient`、`segment::Segment`
以及 58 个常用 API 方法，覆盖消息/历史、图片/语音/文件、账号/好友、群管理、
申请处理、群文件、公告、精华和合并转发。所有未单独封装的 NapCat 接口可用 `call`。

```rust
use luo9_sdk::{Bot, napcat::ApiClient, segment as seg};
use serde_json::json;
use std::time::Duration;

// 消息段发送（fire-and-forget），图片 file 接受 URL、base64://、NapCat 主机路径。
Bot::send_group_segments(123456, vec![
    seg::reply(789), seg::at(10001), seg::text("看图"),
    seg::image("https://example.com/image.png"),
]);

// 需要返回值时，在插件工作线程内调用；返回 NapCat 成功响应的 data。
let api = ApiClient::new(Duration::from_secs(10));
let members = api.get_group_member_list(123456)?;
let image = api.get_image("NapCat 图片标识")?;
api.set_group_ban(123456, 10001, 60)?;
api.set_group_add_request("事件中的 flag", true, "")?;
```

API 查询先创建独立 `luo9_api_response` 订阅，再向 `luo9_send` 发布带 `req_id`
及 `response_subscriber` 的请求。核心将 `ApiResponse` 定向发布给调用者，SDK 同时
校验请求 ID 和 API 名称；并发查询不会互相抢结果。成功、失败、超时均取消订阅。
失败返回 `ApiError`，远端错误保留 `retcode`；断线也会得到明确错误。
同步查询应运行在插件工作线程，不要直接阻塞异步运行时的 worker。SDK 公共入口只
提供强类型方法；未封装接口由核心内部协议处理，不要求插件作者传递 action 字符串。

旧文本发送、CQ 消息和 `PluginSubscribers` 的 FFI 布局保持兼容。
`MessagePayload.segments` 保留图片 URL 和未知消息段数据；`NoticePayload.raw_event`
保留完整通知事件。新子类型可回退解析。`post_type`、`target_id` 可区分
`message_sent`；消息段发送回执包含 `message_format` 与 `segments`。
`signed_message_id` / `onebot_message_id()` 保留有符号消息 ID，旧 `u64` 字段继续提供
正整数 ID。撤回有符号 ID 用 `delete_msg_signed`。

参考 [NapCat 官方 API](https://napneko.github.io/onebot/api) 和
[消息段文档](https://napneko.github.io/onebot/segment)。接口是否可用及权限由 NapCat 决定。
