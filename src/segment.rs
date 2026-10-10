//! Structured, forward-compatible OneBot/NapCat message segments.
//!
//! Unknown segment types and data are retained verbatim so newer NapCat extensions
//! do not require an SDK release just to inspect or forward their payloads.
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageSegment {
    #[serde(rename = "type")]
    pub segment_type: String,
    #[serde(default)]
    pub data: Map<String, Value>,
}

impl MessageSegment {
    pub fn new(segment_type: impl Into<String>, data: Map<String, Value>) -> Self {
        Self {
            segment_type: segment_type.into(),
            data,
        }
    }

    pub fn text(text: impl Into<String>) -> Self {
        Self::new(
            "text",
            Map::from_iter([("text".to_string(), Value::String(text.into()))]),
        )
    }

    pub fn at(user_id: impl ToString) -> Self {
        Self::new(
            "at",
            Map::from_iter([(
                "qq".to_string(),
                Value::String(user_id.to_string()),
            )]),
        )
    }

    pub fn image(file: impl Into<String>) -> Self {
        Self::new(
            "image",
            Map::from_iter([("file".to_string(), Value::String(file.into()))]),
        )
    }

    pub fn reply(message_id: impl ToString) -> Self {
        Self::new(
            "reply",
            Map::from_iter([(
                "id".to_string(),
                Value::String(message_id.to_string()),
            )]),
        )
    }

    pub fn raw_data(&self) -> &Map<String, Value> {
        &self.data
    }
}

/// Message builder for the host's current CQ-code string transport.
/// Arbitrary segment types are supported; NapCat validates actual availability.
#[derive(Debug, Clone, Default)]
pub struct MessageBuilder {
    segments: Vec<MessageSegment>,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(mut self, segment: MessageSegment) -> Self {
        self.segments.push(segment);
        self
    }

    pub fn text(self, text: impl Into<String>) -> Self {
        self.push(MessageSegment::text(text))
    }

    pub fn at(self, user_id: impl ToString) -> Self {
        self.push(MessageSegment::at(user_id))
    }

    pub fn image(self, file: impl Into<String>) -> Self {
        self.push(MessageSegment::image(file))
    }

    pub fn reply(self, message_id: impl ToString) -> Self {
        self.push(MessageSegment::reply(message_id))
    }

    pub fn segments(&self) -> &[MessageSegment] {
        &self.segments
    }

    /// Encode known segment data as CQ codes. Unknown segment types are encoded too,
    /// allowing forward-compatible pass-through where the receiving NapCat supports them.
    pub fn build_cq(self) -> String {
        let mut output = String::new();
        for segment in self.segments {
            if segment.segment_type == "text" {
                if let Some(text) = segment.data.get("text").and_then(Value::as_str) {
                    output.push_str(&escape_text(text));
                }
                continue;
            }
            output.push_str("[CQ:");
            output.push_str(&segment.segment_type);
            for (key, value) in segment.data {
                output.push(',');
                output.push_str(&key);
                output.push('=');
                let value = match value {
                    Value::String(s) => s,
                    other => other.to_string(),
                };
                output.push_str(&escape_parameter(&value));
            }
            output.push(']');
        }
        output
    }
}

fn escape_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('[', "&#91;")
        .replace(']', "&#93;")
}

fn escape_parameter(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('[', "&#91;")
        .replace(']', "&#93;")
        .replace(',', "&#44;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_preserves_unknown_segment() {
        let input = r#"[{"type":"text","data":{"text":"hi"}},{"type":"future_napcat_segment","data":{"opaque":{"x":1}}}]"#;
        let segments: Vec<MessageSegment> = serde_json::from_str(input).unwrap();
        assert_eq!(segments[1].segment_type, "future_napcat_segment");
        assert_eq!(segments[1].data["opaque"]["x"], 1);
        assert_eq!(
            serde_json::to_value(&segments).unwrap()[1]["data"]["opaque"]["x"],
            1
        );
    }

    #[test]
    fn cq_builder_escapes_text_and_supports_arbitrary_types() {
        let data = Map::from_iter([(
            "file".into(),
            Value::String("v.mp4".into()),
        )]);
        let output = MessageBuilder::new()
            .text("a[b]&c")
            .push(MessageSegment::new("video", data))
            .build_cq();
        assert_eq!(output, "a&#91;b&#93;&amp;c[CQ:video,file=v.mp4]");
    }
}
