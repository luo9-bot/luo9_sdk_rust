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

#[derive(Debug, Clone, Deserialize)]
pub struct MessagePayload {
    pub message_type: MsgType,
    pub user_id: u64,
    pub group_id: Option<u64>,
    pub message: String,
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
    #[serde(rename = "essence")]
    Essence,
    #[serde(rename = "notify")]
    Notify,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NoticePayload {
    pub notice_type: NoticeType,
    pub sub_type: SubType,
    pub status: String,
    pub user_id: u64,
    pub group_id: Option<u64>,
    pub time: u64,
}

// ── 子类型 ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub enum SubType {
    #[serde(rename = "enable")]
    Enable,
    #[serde(rename = "disable")]
    Disable,
    #[serde(rename = "connect")]
    Connect,
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
    #[serde(rename = "notice")]
    Notice,
    #[serde(rename = "set")]
    Set,
    #[serde(rename = "unset")]
    Unset,
    #[serde(rename = "ban")]
    Ban,
    #[serde(rename = "lift_ban")]
    LiftBan,
    #[serde(rename = "leave")]
    Leave,
    #[serde(rename = "kick")]
    Kick,
    #[serde(rename = "kick_me")]
    KickMe,
    #[serde(rename = "approve")]
    Approve,
    #[serde(rename = "poke")]
    Poke,
    #[serde(rename = "input_status")]
    InputStatus,
    #[serde(rename = "title")]
    Title,
    #[serde(rename = "profile_like")]
    ProfileLike,
    #[serde(rename = "add")]
    Add,
    #[serde(rename = "invite")]
    Invite,
    #[serde(rename = "none")]
    None,
}

// ── 总线载荷 ────────────────────────────────────────────────────

/// 从总线 topic pop 出的 JSON 解析为此枚举
///
/// JSON 格式与 topic 对应关系：
/// - `luo9_message`     → `{"Message": { ... }}`
/// - `luo9_meta_event`  → `{"MetaEvent": { ... }}`
/// - `luo9_notice`      → `{"Notice": { ... }}`
#[derive(Debug, Deserialize)]
pub enum BusPayload {
    Message(MessagePayload),
    MetaEvent(MetaEventPayload),
    Notice(NoticePayload),
}

impl BusPayload {
    /// 从 JSON 字符串解析总线载荷
    pub fn parse(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}
