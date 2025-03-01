pub struct Op {
    pub w: Option<i16>,
    pub h: Option<i16>,
    pub background_color: Option<String>,
    // pub char_prompt: Option<String>, // 未使用
    pub font: Option<String>,
    pub font_size: Option<String>,
    pub font_color: Option<String>,
    pub border_color: Option<String>,
    pub word: Option<String>,
}

impl Default for Op {
    fn default() -> Self {
        let current_dir = std::env::current_dir().unwrap();
        let font_filepath = current_dir
            .join("resource")
            .join("fonts")
            .join("static")
            .join("NotoSansJP-Bold.ttf")
            .to_string_lossy()
            .into_owned();

        Op {
            w: Some(1920),
            h: Some(1080),
            background_color: Some("white".to_string()),
            // char_prompt: Some("".to_string()),
            font: Some(font_filepath),
            font_size: Some("36".to_string()),
            font_color: Some("white".to_string()),
            border_color: Some("0xBBDEFB".to_string()),
            word: Some("".to_string()),
        }
    }
}

impl Op {
    pub fn set_word(&mut self, word: &str) {
        self.word = Some(word.to_string());
    }

    pub fn create_filter_complex(&self) -> String {
        let width = self.w.unwrap_or_default();
        let height = self.h.unwrap_or_default();
        let binding = self.background_color.as_deref().unwrap_or_default();
        let background_color = binding;
        let font = self.font.as_deref().unwrap_or_default();
        let font_size = self.font_size.as_deref().unwrap_or_default();
        let font_color = self.font_color.as_deref().unwrap_or_default();
        let border_color = self.border_color.as_deref().unwrap_or_default();

        let mut args = Vec::new();
        let basic_filer = format!(
            "[0]scale=w='min({width},iw)':h='min({height},ih)':
        force_original_aspect_ratio=decrease,\
        pad={width}:{height}:({width}-iw)/
        2:({height}-ih)/
        2:{background_color}[bg];",
        );
        args.push(basic_filer.as_str());

        let word = self.word.as_deref().unwrap_or("");
        let prompt = format!(
            "[bg]drawtext=fontfile='{font}':\
        fontsize={font_size}:\
        fontcolor={font_color}@0.9:\
        borderw=10:\
        bordercolor={border_color}:\
        text='{word}':\
        x=(W-text_w)/2:\
        y=(H-text_h-50):\
        wrap_unicode[out2]"
        );

        args.push(prompt.as_str());

        args.join("")
    }
}
