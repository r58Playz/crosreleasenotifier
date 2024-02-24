use html2text::render::text_renderer::TextDecorator;

#[derive(Clone)]
pub struct MdDecorator {
    currentlink: String,
}

impl MdDecorator {
    pub fn new() -> MdDecorator {
        MdDecorator {
            currentlink: "".into(),
        }
    }
}

impl TextDecorator for MdDecorator {
    type Annotation = ();

    fn decorate_link_start(&mut self, _url: &str) -> (String, Self::Annotation) {
        self.currentlink = _url.into();
        ("[".into(), ())
    }

    fn decorate_link_end(&mut self) -> String {
        format!("]({})", self.currentlink)
    }

    fn decorate_em_start(&self) -> (String, Self::Annotation) {
        ("*".into(), ())
    }

    fn decorate_em_end(&self) -> String {
        "*".into()
    }

    fn decorate_strong_start(&self) -> (String, Self::Annotation) {
        ("**".into(), ())
    }

    fn decorate_strong_end(&self) -> String {
        "**".into()
    }

    fn decorate_strikeout_start(&self) -> (String, Self::Annotation) {
        ("~~".into(), ())
    }

    fn decorate_strikeout_end(&self) -> String {
        "~~".into()
    }

    fn decorate_code_start(&self) -> (String, Self::Annotation) {
        ("`".into(), ())
    }

    fn decorate_code_end(&self) -> String {
        "`".into()
    }

    fn decorate_preformat_first(&self) -> Self::Annotation {}
    fn decorate_preformat_cont(&self) -> Self::Annotation {}

    fn decorate_image(&mut self, src: &str, title: &str) -> (String, Self::Annotation) {
        (format!("[{}]({})", title, src), ())
    }

    fn header_prefix(&self, level: usize) -> String {
        "#".repeat(level) + " "
    }

    fn quote_prefix(&self) -> String {
        "> ".into()
    }

    fn unordered_item_prefix(&self) -> String {
        "* ".into()
    }

    fn ordered_item_prefix(&self, i: i64) -> String {
        format!("{}. ", i)
    }

    fn finalise(
        &mut self,
        _links: Vec<String>,
    ) -> Vec<html2text::render::text_renderer::TaggedLine<()>> {
        Vec::new()
    }

    fn make_subblock_decorator(&self) -> Self {
        self.clone()
    }
}

#[derive(Clone)]
pub struct PlainDecorator {
    currentlink: String,
}

impl PlainDecorator {
    pub fn new() -> PlainDecorator {
        PlainDecorator {
            currentlink: "".into(),
        }
    }
}

impl TextDecorator for PlainDecorator {
    type Annotation = ();

    fn decorate_link_start(&mut self, _url: &str) -> (String, Self::Annotation) {
        self.currentlink = _url.into();
        ("".into(), ())
    }

    fn decorate_link_end(&mut self) -> String {
        format!(" ({})", self.currentlink)
    }

    fn decorate_em_start(&self) -> (String, Self::Annotation) {
        ("*".into(), ())
    }

    fn decorate_em_end(&self) -> String {
        "*".into()
    }

    fn decorate_strong_start(&self) -> (String, Self::Annotation) {
        ("**".into(), ())
    }

    fn decorate_strong_end(&self) -> String {
        "**".into()
    }

    fn decorate_strikeout_start(&self) -> (String, Self::Annotation) {
        ("~~".into(), ())
    }

    fn decorate_strikeout_end(&self) -> String {
        "~~".into()
    }

    fn decorate_code_start(&self) -> (String, Self::Annotation) {
        ("`".into(), ())
    }

    fn decorate_code_end(&self) -> String {
        "`".into()
    }

    fn decorate_preformat_first(&self) -> Self::Annotation {}
    fn decorate_preformat_cont(&self) -> Self::Annotation {}

    fn decorate_image(&mut self, src: &str, title: &str) -> (String, Self::Annotation) {
        (format!(" {} ({})", title, src), ())
    }

    fn header_prefix(&self, level: usize) -> String {
        "#".repeat(level) + " "
    }

    fn quote_prefix(&self) -> String {
        "> ".into()
    }

    fn unordered_item_prefix(&self) -> String {
        "* ".into()
    }

    fn ordered_item_prefix(&self, i: i64) -> String {
        format!("{}. ", i)
    }

    fn finalise(
        &mut self,
        _links: Vec<String>,
    ) -> Vec<html2text::render::text_renderer::TaggedLine<()>> {
        Vec::new()
    }

    fn make_subblock_decorator(&self) -> Self {
        self.clone()
    }
}
