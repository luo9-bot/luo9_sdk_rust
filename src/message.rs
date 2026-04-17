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

    pub fn txt(mut self, text: impl Into<String>) -> Self {
        self.parts.push(text.into());
        self
    }

    pub fn image(mut self, path: impl Into<String>) -> Self {
        let path_str: String = path.into();
        if path_str.contains("http") {
            self.parts.push(format!("[CQ:image, url={}]", path_str));
        } else {
            self.parts.push(format!("[CQ:image, file={}]", path_str));
        }
        self
    }

    pub fn build(self) -> CString {
        let combined = self.parts.join("");
        CString::new(combined).unwrap()
    }
}

impl Msg {
    pub fn new(text: impl Into<String>) -> MsgBuilder {
        MsgBuilder::default().txt(text)
    }

    pub fn at(user_id: u64) -> MsgBuilder {
        MsgBuilder::default().at(user_id)
    }

    pub fn txt(text: impl Into<String>) -> MsgBuilder {
        MsgBuilder::default().txt(text)
    }

    pub fn image(path: impl Into<String>) -> MsgBuilder {
        MsgBuilder::default().image(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_msg() {
        // 仍然可以使用 &str
        let msg1 = Msg::txt("hello world")
                            .at(123321).endl()
                            .image("https://example.com/image.jpg").build();
        
        println!(">>>> msg1 : {:?}", msg1);

        // 现在也可以使用 String
        let text = String::from("hello from String");
        let img_path = String::from("local_image.png");
        
        let msg2 = Msg::txt(text)
                            .at(456654).endl()
                            .image(img_path).build();
        
        println!(">>>> msg2 : {:?}", msg2);
    }
}