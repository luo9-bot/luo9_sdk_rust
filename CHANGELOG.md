# 更新日志

本文档记录了 luo9_sdk 的所有重要更改。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [0.5.1] - 2026-05-01

更新核心动态库文件

## [0.5.0] - 2026-05-01

### 新增
- **消息总线 (bus) 扩展**：支持多订阅者和阻塞等待
  - `Bus::init()` 初始化总线
  - `Topic::subscribe()` 订阅主题，支持多个订阅者
  - `Topic::publish()` / `publish_fmt()` 发布消息
  - `Topic::pop()` 非阻塞获取消息
  - `Topic::wait_pop()` 阻塞等待消息
- **事件载荷解析模块 (payload)**：完整支持 OneBot 标准事件类型
  - `MessagePayload`：私聊/群消息解析
  - `MetaEventPayload`：生命周期和心跳事件
  - `NoticePayload`：通知事件（好友添加、群管理、撤回等）
  - `BusPayload`：统一的总线载荷枚举
- **消息发送模块 (send)**：基于总线的 fire-and-forget 发送
  - `send_group_msg()` 发送群消息
  - `send_private_msg()` 发送私聊消息

### 变更
- 更新 luo9_core 至 v1.1.0
- 消息发送接口改用总线模式，解耦发送逻辑

## [0.4.0] - 2026-04-20

### 新增
- **消息总线模块 (bus)**：基础发布/订阅消息系统
- **模式匹配模块 (pattern)**：支持模板变量捕获
  - `Pattern::new()` 创建模式
  - `Pattern::match_str()` 匹配字符串并提取变量

### 变更
- 命令解析模块集成模式匹配支持
- 添加 `on_pattern()` 方法支持模式匹配命令

## [0.3.0] - 2026-04-15

### 新增
- **消息构建器模块 (message)**：链式构建消息
  - `Msg::new()` / `Msg::txt()` 创建文本消息
  - `Msg::at()` @用户
  - `Msg::image()` 发送图片（支持本地和网络图片）
  - `MsgBuilder::endl()` 换行
  - `MsgBuilder::build()` 构建最终消息

### 修复
- 移除 CQ 码中 image 标签参数后的多余空格
- 将 @用户标记替换为 CQ 码格式

### 变更
- 为消息构建器方法添加泛型参数支持（`&str` 和 `String`）

## [0.2.0] - 2026-04-13

### 变更
- 调整命令解析函数为封装库，方便多语言管理
- 优化消息发送函数

## [0.1.0] - 2026-04-12

### 新增
- 初始版本发布
- **核心功能**：
  - `Bot::get_version()` 获取版本信息
  - `Bot::send_group_msg()` 发送群消息
  - `Bot::send_private_msg()` 发送私聊消息
- **命令解析模块 (command)**：
  - `Command::parse()` 解析命令
  - `PrefixMode` 前缀模式（Required/Optional/None）
  - `CommandMatcher` 链式匹配
  - `handle()` / `on()` / `otherwise()` 匹配方法

[0.5.0]: https://github.com/luo9-bot/luo9_sdk_rust/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/luo9-bot/luo9_sdk_rust/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/luo9-bot/luo9_sdk_rust/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/luo9-bot/luo9_sdk_rust/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/luo9-bot/luo9_sdk_rust/releases/tag/v0.1.0
