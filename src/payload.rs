
    /// 消息入口提供的发送者展示名称。
    ///
    /// 群名片优先于昵称；不要把返回值作为身份 key。
    pub fn sender_display_name(&self) -> &str {
        self.sender
            .as_ref()
            .map(Sender::display_name)
            .unwrap_or("群友")
    }

    /// 生成可直接用于 AI 上下文的稳定身份标签。
    pub fn sender_identity_label(&self) -> String {
        match &self.sender {
            Some(sender) if sender.user_id != 0 => sender.identity_label(),
            _ => format!("[QQ:{}|{}]", self.user_id, self.sender_display_name()),
        }
    }
}

impl BusPayload {
    /// 从 JSON 字符串解析总线载荷
    pub fn parse(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}

#[cfg(test)]
mod identity_tests {
    use super::*;

    #[test]
    fn sender_prefers_group_card_over_nickname() {
        let sender = Sender {
            user_id: 10001,
            nickname: "小明".into(),
            card: "群里小明".into(),
            sex: String::new(),
            age: 0,
            area: String::new(),
            level: String::new(),
            role: String::new(),
            title: String::new(),
        };
        assert_eq!(sender.display_name(), "群里小明");
        assert_eq!(sender.identity_label(), "[QQ:10001|群里小明]");
    }

    #[test]
    fn same_nickname_keeps_distinct_qq_identity() {
        let a = Sender {
            user_id: 10001,
            nickname: "小明".into(),
            card: String::new(),
            sex: String::new(),
            age: 0,
            area: String::new(),
            level: String::new(),
            role: String::new(),
            title: String::new(),
        };
        let mut b = a.clone();
        b.user_id = 10002;
        assert_eq!(a.display_name(), b.display_name());
        assert_ne!(a.identity_label(), b.identity_label());
        assert_eq!(a.identity_label(), "[QQ:10001|小明]");
        assert_eq!(b.identity_label(), "[QQ:10002|小明]");
    }

    #[test]
    fn message_sender_falls_back_to_top_level_user_id() {
        let payload: MessagePayload = serde_json::from_str(r#"{
            "message_type": "group",
            "user_id": 10003,
            "group_id": 20003,
            "message": "你好"
        }"#).expect("test payload should parse");
        assert_eq!(payload.sender_id(), 10003);
        assert_eq!(payload.sender_display_name(), "群友");
        assert_eq!(payload.sender_identity_label(), "[QQ:10003|群友]");
    }
}
