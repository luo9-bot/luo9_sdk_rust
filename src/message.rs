use std::ffi::CString;
use crate::Msg;

#[derive(Default)]
pub struct MsgBuilder {
    parts: Vec<String>,
}

impl MsgBuilder {
    pub fn endl(mut self) -> Self {
        self.parts.push("\n".to_string());
        self
    }

    pub fn at(mut self, user_id: u64) -> Self {
        self.parts.push(format!("[CQ:at,qq={}]", user_id));
        self
    }

    pub fn txt(mut self, text: &str) -> Self {
        self.parts.push(text.to_string());
        self
    }

    pub fn image(mut self, path: &str) -> Self {
        if path.contains("http") {
            self.parts.push(format!("[CQ:image, url={}]", path));
            self
        } else {
            self.parts.push(format!("[CQ:image, file={}]", path));
            self
        }
    }

    pub fn build(self) -> CString {
        let combined = self.parts.join("");
        CString::new(combined).unwrap()
    }
}

impl Msg {
    pub fn new(text: &str) -> MsgBuilder {
        MsgBuilder::default().txt(text)
    }

    pub fn at(user_id: u64) -> MsgBuilder {
        MsgBuilder::default().at(user_id)
    }

    pub fn txt(text: &str) -> MsgBuilder {
        MsgBuilder::default().txt(text)
    }

    pub fn image(path: &str) -> MsgBuilder {
        MsgBuilder::default().image(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_msg() {
        let msg = Msg::txt("hello world")
                                .at(123321).endl()
                                .image("https://example.com/image.jpg").build();
        
        println!(">>>> msg : {:?}", msg);
    }
}