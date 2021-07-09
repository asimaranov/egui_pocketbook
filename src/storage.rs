pub struct StaticFonts {
    public_name_font: inkview_sys::Font,
    post_date_font: inkview_sys::Font,
    pub(crate) regular_text_font: inkview_sys::Font,
}

impl StaticFonts {
    fn new() -> Self {
        let public_name_font = inkview_sys::open_font("Roboto-Bold", 60, 0);
        let post_date_font = inkview_sys::open_font("Roboto-BoldItalic", 30, 0);
        let regular_text_font = inkview_sys::open_font("Roboto", 40, 0);
        Self {
            public_name_font,
            post_date_font,
            regular_text_font,
        }
    }
}

pub struct ResourceStorage {

    pub(crate) static_fonts: StaticFonts,
}

impl ResourceStorage {
    pub fn new() -> Self {
        return Self {

            static_fonts: StaticFonts::new(),
        };
    }
}
