use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Pattern {
    parts: Vec<PatternPart>,
}

#[derive(Debug, Clone)]
enum PatternPart {
    Literal(String),
    Capture(String),
}

impl Pattern {
    pub fn new(template: &str) -> Self {
        let mut parts = Vec::new();
        let mut rest = template;
        while let Some(start) = rest.find('{') {
            if start > 0 {
                parts.push(PatternPart::Literal(rest[..start].to_string()));
            }
            rest = &rest[start + 1..];
            if let Some(end) = rest.find('}') {
                let capture_name = rest[..end].to_string();
                parts.push(PatternPart::Capture(capture_name));
                rest = &rest[end + 1..];
            } else {
                // 没有闭合的括号，当作字面量处理
                parts.push(PatternPart::Literal("{".to_string()));
            }
        }
        if !rest.is_empty() {
            parts.push(PatternPart::Literal(rest.to_string()));
        }
        Self { parts }
    }

    /// 尝试匹配字符串 s，返回捕获的变量
    pub fn match_str(&self, s: &str) -> Option<HashMap<String, String>> {
        let mut captures = HashMap::new();
        let mut s_remaining = s;
        for part in &self.parts {
            match part {
                PatternPart::Literal(lit) => {
                    if let Some(rest) = s_remaining.strip_prefix(lit) {
                        s_remaining = rest;
                    } else {
                        return None;
                    }
                }
                PatternPart::Capture(name) => {
                    // 贪婪匹配直到下一个字面量出现，或到字符串末尾
                    let next_lit = self.next_literal_after(part);
                    let end_pos = if let Some(next) = next_lit {
                        s_remaining.find(next)?
                    } else {
                        s_remaining.len()
                    };
                    let captured = &s_remaining[..end_pos];
                    captures.insert(name.clone(), captured.to_string());
                    s_remaining = &s_remaining[end_pos..];
                }
            }
        }
        if s_remaining.is_empty() {
            Some(captures)
        } else {
            None
        }
    }

    fn next_literal_after(&self, current: &PatternPart) -> Option<&str> {
        let mut found = false;
        for part in &self.parts {
            if found {
                if let PatternPart::Literal(lit) = part {
                    return Some(lit);
                }
            }
            if std::ptr::eq(part, current) {
                found = true;
            }
        }
        None
    }
}
