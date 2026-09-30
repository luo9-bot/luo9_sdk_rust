# 更新日志

## [0.8.0-beta.2]

### 新增
- 完善消息发送者身份访问接口：支持稳定 QQ 身份、昵称与群名片展示。
- 为 AI 场景提供统一的 `[QQ:|昵称]` 身份标签。


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