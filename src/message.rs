use crate::Msg;
use std::ffi::CString;

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
        self.parts.push(crate::segment::image(path_str).to_cq());
        self
    }

    pub fn reply(mut self, message_id: i64) -> Self {
        self.parts.push(crate::segment::reply(message_id).to_cq());
        self
    }

    pub fn build(self) -> CString {
        let combined = self.parts.join("");
        CString::new(combined).unwrap()
    }
}

impl Msg {
    /// builder 入口：返回 MsgBuilder 而非 Self（at/txt/image 同理）
    #[allow(clippy::new_ret_no_self)]
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
    fn image_builder_uses_file_and_escapes_cq_parameters() {
        let message = Msg::image("https://example.test/a,b?x=1&y=2")
            .reply(-7)
            .build();
        assert_eq!(
            message.to_str().unwrap(),
            "[CQ:image,file=https://example.test/a&#44;b?x=1&amp;y=2][CQ:reply,id=-7]"
        );
    }

    #[test]
    fn test_new_msg() {
        // 仍然可以使用 &str
        let msg1 = Msg::txt("hello world")
            .at(123321)
            .endl()
            .image("https://example.com/image.jpg")
            .build();

        println!(">>>> msg1 : {:?}", msg1);

        // 现在也可以使用 String
        let text = String::from("hello from String");
        let img_path = String::from("local_image.png");

        let msg2 = Msg::txt(text).at(456654).endl().image(img_path).build();

        println!(">>>> msg2 : {:?}", msg2);
    }
}
