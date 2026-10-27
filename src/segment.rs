//! OneBot 消息段。保留未知类型和扩展字段，避免 NapCat 升级时丢失信息。
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Segment {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub data: Value,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

impl Segment {
    pub fn new(kind: impl Into<String>, data: Value) -> Self {
        Self {
            kind: kind.into(),
            data,
            extra: Default::default(),
        }
    }

    /// 转为 CQ 码；参数先转义，文本中的 CQ 标记不会变成消息段。
    pub fn to_cq(&self) -> String {
        if self.kind == "text" {
            return escape(
                self.data.get("text").and_then(Value::as_str).unwrap_or(""),
                false,
            );
        }
        let mut code = format!("[CQ:{}", self.kind);
        if let Some(data) = self.data.as_object() {
            for (key, value) in data {
                let value = value
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| value.to_string());
                code.push_str(&format!(",{key}={}", escape(&value, true)));
            }
        }
        code.push(']');
        code
    }
}

pub fn escape(value: &str, parameter: bool) -> String {
    let value = value
        .replace('&', "&amp;")
        .replace('[', "&#91;")
        .replace(']', "&#93;");
    if parameter {
        value.replace(',', "&#44;")
    } else {
        value
    }
}

pub fn unescape(value: &str) -> String {
    value
        .replace("&#91;", "[")
        .replace("&#93;", "]")
        .replace("&#44;", ",")
        .replace("&amp;", "&")
}

/// 解析 CQ 字符串；先分段再解码，避免把转义后的文字误当作消息段。
pub fn from_cq(message: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut remaining = message;
    while let Some(start) = remaining.find("[CQ:") {
        if start > 0 {
            segments.push(text(unescape(&remaining[..start])));
        }
        let after = &remaining[start + 4..];
        let Some(end) = after.find(']') else {
            segments.push(text(unescape(&remaining[start..])));
            return segments;
        };
        let mut fields = after[..end].split(',');
        let kind = fields.next().unwrap_or("");
        let fields: Option<Vec<_>> = fields.map(|field| field.split_once('=')).collect();
        if !kind.is_empty()
            && kind
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            if let Some(fields) = fields {
                let data = fields
                    .into_iter()
                    .map(|(key, value)| (key.to_owned(), Value::String(unescape(value))))
                    .collect::<serde_json::Map<_, _>>();
                segments.push(Segment::new(kind, Value::Object(data)));
            } else {
                segments.push(text(unescape(&remaining[start..start + 4 + end + 1])));
            }
        } else {
            segments.push(text(unescape(&remaining[start..start + 4 + end + 1])));
        }
        remaining = &after[end + 1..];
    }
    if !remaining.is_empty() {
        segments.push(text(unescape(remaining)));
    }
    segments
}

pub fn text(value: impl Into<String>) -> Segment {
    Segment::new("text", json!({"text": value.into()}))
}
pub fn at(qq: u64) -> Segment {
    Segment::new("at", json!({"qq": qq.to_string()}))
}
pub fn at_all() -> Segment {
    Segment::new("at", json!({"qq": "all"}))
}
pub fn reply(message_id: i64) -> Segment {
    Segment::new("reply", json!({"id": message_id.to_string()}))
}
pub fn face(id: u32) -> Segment {
    Segment::new("face", json!({"id": id.to_string()}))
}
/// URL、base64:// 或 NapCat 主机上的文件路径。
pub fn image_bytes(bytes: &[u8]) -> Segment {
    use base64::{Engine, engine::general_purpose::STANDARD};
    image(format!("base64://{}", STANDARD.encode(bytes)))
}
pub fn image(file: impl Into<String>) -> Segment {
    Segment::new("image", json!({"file": file.into()}))
}
pub fn record(file: impl Into<String>) -> Segment {
    Segment::new("record", json!({"file": file.into()}))
}
pub fn video(file: impl Into<String>) -> Segment {
    Segment::new("video", json!({"file": file.into()}))
}
pub fn file(file: impl Into<String>, name: impl Into<String>) -> Segment {
    Segment::new("file", json!({"file": file.into(), "name": name.into()}))
}
pub fn json_card(data: impl Into<String>) -> Segment {
    Segment::new("json", json!({"data": data.into()}))
}
pub fn forward(id: impl Into<String>) -> Segment {
    Segment::new("forward", json!({"id": id.into()}))
}
pub fn node(user_id: u64, nickname: impl Into<String>, content: Vec<Segment>) -> Segment {
    Segment::new(
        "node",
        json!({"user_id": user_id, "nickname": nickname.into(), "content": content}),
    )
}
pub fn music(platform: &str, id: &str) -> Segment {
    Segment::new("music", json!({"type": platform, "id": id}))
}
pub fn dice() -> Segment {
    Segment::new("dice", json!({}))
}
pub fn rps() -> Segment {
    Segment::new("rps", json!({}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cq_parameters_cannot_inject_segments() {
        assert_eq!(
            image("https://example.test/a,b?x=[CQ:at,qq=all]&y=1").to_cq(),
            "[CQ:image,file=https://example.test/a&#44;b?x=&#91;CQ:at&#44;qq=all&#93;&amp;y=1]"
        );
        assert_eq!(text("[CQ:at,qq=all]").to_cq(), "&#91;CQ:at,qq=all&#93;");
        assert_eq!(unescape("&amp;#91;"), "&#91;");
    }
    #[test]
    fn extensions_and_signed_replies_survive_round_trip() {
        let raw = json!({"type":"mface","data":{"emoji_id":"abc"},"vendor":true});
        let segment: Segment = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(serde_json::to_value(segment).unwrap(), raw);
        assert_eq!(reply(-12).data["id"], "-12");
    }

    #[test]
    fn cq_strings_preserve_literal_text_and_image_parameters() {
        let parts = from_cq(
            "&#91;CQ:at,qq=all&#93;[CQ:image,file=https://example.test/a&#44;b?x=1&amp;y=2][CQ:reply,id=-7]",
        );
        assert_eq!(parts[0].kind, "text");
        assert_eq!(parts[0].data["text"], "[CQ:at,qq=all]");
        assert_eq!(parts[1].data["file"], "https://example.test/a,b?x=1&y=2");
        assert_eq!(parts[2].data["id"], "-7");
        assert_eq!(
            from_cq("hello[CQ:image,file=x")[1].data["text"],
            "[CQ:image,file=x"
        );
    }
}
