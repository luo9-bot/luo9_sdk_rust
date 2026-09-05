use serde::Deserialize;

// ── 消息类型 ────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum MsgType {
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "group")]
    Group,
    #[serde(other)]
    Other,
}

/// 消息发送者信息
#[derive(Debug, Clone, Deserialize)]
pub struct Sender {
    #[serde(default)]
    pub user_id: u64,
    #[serde(default)]
    pub nickname: String,
    #[serde(default)]
    pub card: String,
    #[serde(default)]
    pub sex: String,
    #[serde(default)]
    pub age: u32,
    #[serde(default)]
    pub area: String,
    #[serde(default)]
    pub level: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub title: String,
}

/// 匿名信息
#[derive(Debug, Clone, Deserialize)]
pub struct Anonymous {
    pub id: u64,
    pub name: String,
    pub flag: String,
}

/// 消息载荷
///
/// 所有新字段都使用 `#[serde(default)]` 以保持向后兼容：
/// - 新版宿主 + 旧版 SDK：新字段被忽略
/// - 旧版宿主 + 新版 SDK：新字段使用默认值
#[derive(Debug, Clone, Deserialize)]
pub struct MessagePayload {
    pub message_type: MsgType,
    pub user_id: u64,
    pub group_id: Option<u64>,
    pub message: String,
    // ── 新增字段（向后兼容）──
    #[serde(default)]
    pub time: u64,
    #[serde(default)]
    pub self_id: u64,
    #[serde(default)]
    pub message_id: u64,
    #[serde(default)]
    pub message_seq: Option<u64>,
    #[serde(default)]
    pub real_id: Option<u64>,
    #[serde(default)]
    pub real_seq: Option<String>,
    #[serde(default)]
    pub sub_type: SubType,
    #[serde(default)]
    pub font: u32,
    #[serde(default)]
    pub sender: Option<Sender>,
    #[serde(default)]
    pub anonymous: Option<Anonymous>,
    #[serde(default)]
    pub message_format: String,
}

// ── 元事件类型 ──────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub enum MetaEventType {
    #[serde(rename = "lifecycle")]
    Lifecycle,
    #[serde(rename = "heartbeat")]
    Heartbeat,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Status {
    pub good: bool,
    pub online: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetaEventPayload {
    pub interval: Option<u64>,
    pub meta_event_type: MetaEventType,
    pub sub_type: SubType,
    pub self_id: u64,
    pub status: Option<Status>,
    pub time: u64,
}

// ── 通知类型 ────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub enum NoticeType {
    #[serde(rename = "friend_add")]
    FriendAdd,
    #[serde(rename = "friend_recall")]
    FriendRecall,
    #[serde(rename = "group_admin")]
    GroupAdmin,
    #[serde(rename = "group_ban")]
    GroupBan,
    #[serde(rename = "group_increase")]
    GroupIncrease,
    #[serde(rename = "group_decrease")]
    GroupDecrease,
    #[serde(rename = "group_card")]
    GroupCard,
    #[serde(rename = "group_recall")]
    GroupRecall,
    #[serde(rename = "group_upload")]
    GroupUpload,
    #[serde(rename = "group_title")]
    GroupTitle,
    #[serde(rename = "honor")]
    Honor,
    #[serde(rename = "essence")]
    Essence,
    #[serde(rename = "poke")]
    Poke,
    #[serde(rename = "lucky_king")]
    LuckyKing,
    #[serde(rename = "group_msg_emoji_like")]
    GroupMsgEmojiLike,
    #[serde(rename = "notify")]
    Notify,
    #[serde(other)]
    Unknown,
}

/// 群文件信息
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub busid: i64,
}

/// 群荣誉类型
#[derive(Debug, Clone, Deserialize)]
pub enum HonorType {
    #[serde(rename = "talkative")]
    Talkative,
    #[serde(rename = "performer")]
    Performer,
    #[serde(rename = "emotion")]
    Emotion,
}

/// 通知载荷
///
/// 所有新字段都使用 `#[serde(default)]` 以保持向后兼容。
#[derive(Debug, Clone, Deserialize)]
pub struct NoticePayload {
    pub notice_type: NoticeType,
    #[serde(default)]
    pub sub_type: SubType,
    #[serde(default)]
    pub user_id: u64,
    #[serde(default)]
    pub group_id: Option<u64>,
    #[serde(default)]
    pub time: u64,
    // ── 新增字段（向后兼容）──
    #[serde(default)]
    pub operator_id: Option<u64>,
    #[serde(default)]
    pub target_id: Option<u64>,
    #[serde(default)]
    pub message_id: Option<u64>,
    #[serde(default)]
    pub file: Option<FileInfo>,
    #[serde(default)]
    pub duration: Option<u64>,
    #[serde(default)]
    pub card_new: Option<String>,
    #[serde(default)]
    pub card_old: Option<String>,
    #[serde(default)]
    pub honor_type: Option<HonorType>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub flag: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
}

// ── 请求类型 ────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub enum RequestType {
    #[serde(rename = "friend")]
    Friend,
    #[serde(rename = "group")]
    Group,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub enum GroupRequestSubType {
    #[serde(rename = "add")]
    Add,
    #[serde(rename = "invite")]
    Invite,
    #[default]
    #[serde(other)]
    Unknown,
}

/// 请求载荷
#[derive(Debug, Clone, Deserialize)]
pub struct RequestPayload {
    pub request_type: RequestType,
    #[serde(default)]
    pub user_id: u64,
    #[serde(default)]
    pub group_id: Option<u64>,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub flag: String,
    #[serde(default)]
    pub sub_type: GroupRequestSubType,
    #[serde(default)]
    pub time: u64,
    #[serde(default)]
    pub self_id: u64,
}

// ── 子类型 ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Default)]
pub enum SubType {
    // Lifecycle
    #[serde(rename = "enable")]
    Enable,
    #[serde(rename = "disable")]
    Disable,
    #[serde(rename = "connect")]
    Connect,
    // Message
    #[serde(rename = "friend")]
    Friend,
    #[serde(rename = "group")]
    GroupTemp,
    #[serde(rename = "group_self")]
    GroupSelf,
    #[serde(rename = "other")]
    Other,
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "anonymous")]
    Anonymous,
    #[serde(rename = "notice")]
    Notice,
    // Notice - 管理
    #[serde(rename = "set")]
    Set,
    #[serde(rename = "unset")]
    Unset,
    // Notice - 禁言
    #[serde(rename = "ban")]
    Ban,
    #[serde(rename = "lift_ban")]
    LiftBan,
    #[serde(rename = "unban")]
    Unban,
    // Notice - 成员
    #[serde(rename = "leave")]
    Leave,
    #[serde(rename = "kick")]
    Kick,
    #[serde(rename = "kick_me")]
    KickMe,
    #[serde(rename = "approve")]
    Approve,
    #[serde(rename = "invite")]
    Invite,
    #[serde(rename = "add")]
    Add,
    // Notice - 互动
    #[serde(rename = "poke")]
    Poke,
    #[serde(rename = "lucky_king")]
    LuckyKing,
    // Notice - 荣誉
    #[serde(rename = "talkative")]
    Talkative,
    #[serde(rename = "performer")]
    Performer,
    #[serde(rename = "emotion")]
    Emotion,
    #[serde(rename = "honor")]
    Honor,
    // Notice - 其他
    #[serde(rename = "input_status")]
    InputStatus,
    #[serde(rename = "title")]
    Title,
    #[serde(rename = "profile_like")]
    ProfileLike,
    // 默认值
    #[default]
    #[serde(rename = "none")]
    None,
}

// ── 发送回执载荷 ────────────────────────────────────────────────

/// 发送回执：核心调用 NapCat 发送成功后发布到 `luo9_sent`
///
/// JSON 形如 `{"Sent": {"group_id":..., "user_id":..., "message_id":..., "message":...}}`。
/// 插件用它把自己发出的内容与 message_id 关联起来（撤回等操作的前提）。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SentPayload {
    #[serde(default)]
    pub group_id: Option<u64>,
    #[serde(default)]
    pub user_id: u64,
    #[serde(default)]
    pub message_id: u64,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub time: u64,
}

// ── 总线载荷 ────────────────────────────────────────────────────

/// 从总线 topic pop 出的 JSON 解析为此枚举
///
/// JSON 格式与 topic 对应关系：
/// - `luo9_message`     → `{"Message": { ... }}`
/// - `luo9_meta_event`  → `{"MetaEvent": { ... }}`
/// - `luo9_notice`      → `{"Notice": { ... }}`
/// - `luo9_request`     → `{"Request": { ... }}`
/// - `luo9_sent`        → `{"Sent": { ... }}`
#[derive(Debug, Deserialize)]
pub enum BusPayload {
    Message(MessagePayload),
    MetaEvent(MetaEventPayload),
    Notice(NoticePayload),
    Request(RequestPayload),
    Sent(SentPayload),
}

impl BusPayload {
    /// 从 JSON 字符串解析总线载荷
    pub fn parse(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}
