use egui::{CtxRef, TouchDeviceId, TouchId, TouchPhase, Pos2, PointerButton, Vec2, Shape};
use crate::texture_allocator::PocketbookTextureAllocator;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;
use epi::{IntegrationInfo, Storage};
use egui::epaint::ClippedShape;
use inkview_sys::{c_api, Color};
use egui::Event;
use crate::storage::ResourceStorage;

pub struct PocketbookBackend{
    egui_ctx: CtxRef,
    previous_frame_time: Option<f32>,
    frame_start: Option<f64>,
}
impl PocketbookBackend{
    pub(crate) fn new() -> Self{
        let ctx = CtxRef::default();

        Self{
            egui_ctx: ctx,
            previous_frame_time: None,
            frame_start: None
        }
    }
    pub fn begin_frame(&mut self, raw_input: egui::RawInput) {
        self.frame_start = Some(1f64); // TODO
        self.egui_ctx.begin_frame(raw_input)
    }

    pub fn end_frame(&mut self) -> (egui::Output, Vec<ClippedShape>) {
        let frame_start = self
            .frame_start
            .take()
            .expect("unmatched calls to begin_frame/end_frame");

        let (output, shapes) = self.egui_ctx.end_frame();


        //let clipped_meshes = self.egui_ctx.tessellate(shapes);

        let now = 1f64; // TODO
        self.previous_frame_time = Some((now - frame_start) as f32);

        (output, shapes)
    }



}

pub struct AppRunner {
    pocketbook_backend: PocketbookBackend,
    app: Box<dyn epi::App>,
    pub(crate) needs_repaint: std::sync::Arc<NeedRepaint>,
    resource_storage: ResourceStorage



}

pub struct NeedRepaint(std::sync::atomic::AtomicBool);

impl Default for NeedRepaint {
    fn default() -> Self {
        Self(true.into())
    }
}

impl NeedRepaint {
    pub fn fetch_and_clear(&self) -> bool {
        self.0.swap(false, SeqCst)
    }

    pub fn set_true(&self) {
        self.0.store(true, SeqCst);
    }
}

impl epi::RepaintSignal for NeedRepaint {
    fn request_repaint(&self) {
        self.0.store(true, SeqCst);
    }
}


impl AppRunner {
    pub(crate) fn new(pocketbook_backend: PocketbookBackend, app: Box<dyn epi::App>) -> Self{
        pocketbook_backend.egui_ctx.set_visuals(egui::Visuals::light());
        let mut runner = Self{
            pocketbook_backend: PocketbookBackend::new(),

            app,
            needs_repaint: Arc::from(NeedRepaint::default()),

            resource_storage: ResourceStorage::new(),
        };

        let mut app_output = epi::backend::AppOutput::default();
        let mut texture_allocator = PocketbookTextureAllocator{};
        let mut frame = epi::backend::FrameBuilder {
            info: IntegrationInfo{
                web_info: None,
                prefer_dark_mode: None,
                cpu_usage: None,
                seconds_since_midnight: None,
                native_pixels_per_point: None
            },
            tex_allocator: &mut texture_allocator,
            #[cfg(feature = "http")]
            http: runner.http.clone(),
            output: &mut app_output,
            repaint_signal: runner.needs_repaint.clone(),
        }
            .build();

        runner.app.setup(
            &runner.pocketbook_backend.egui_ctx,
            &mut frame,
            None,
        );

        runner


    }
    fn draw_shapes(&mut self, clipped_shapes: Vec<ClippedShape>) {
        let ppp = self.pocketbook_backend.egui_ctx.pixels_per_point();
        for ClippedShape(clip_rect, shape) in clipped_shapes {
            if !clip_rect.is_positive() {
                continue;
            }

            match shape {
                Shape::Noop => {}
                Shape::Vec(_) => {}
                Shape::Circle { center, radius, fill, stroke } => {
                    inkview_sys::draw_circle((center.x * ppp) as i32, (center.y * ppp) as i32, (radius * ppp) as i32, inkview_sys::Color::rgb(fill.r(), fill.g(), fill.b()));
                }
                Shape::LineSegment { .. } => {}
                Shape::Path { .. } => {}
                Shape::Rect { rect, corner_radius, fill, stroke } => {
                    inkview_sys::fill_area((rect.min.x * ppp) as i32,
                                           (rect.min.y * ppp) as i32,
                                           (rect.width() * ppp) as i32,
                                           (rect.height() * ppp) as i32,
                                           inkview_sys::Color::rgb(fill.r(), fill.g(), fill.b()),
                    );
                    /*inkview_sys::draw_rect_round(rect.min.x as i32,
                                                 rect.min.y as i32,
                                                 rect.width() as i32,
                                                 rect.height() as i32,
                        inkview_sys::Color::rgb(fill.r(), fill.g(), fill.b()), corner_radius as i32
                    );*/
                }
                Shape::Text { pos, galley, color, fake_italics, } => {
                    inkview_sys::set_font(self.resource_storage.static_fonts.regular_text_font, Color::rgb(color.r(), color.g(), color.b()));
                    inkview_sys::draw_text_rect((pos.x * ppp) as i32, (pos.y * ppp) as i32, (galley.size.x * ppp) as i32, (galley.size.y * ppp) as i32, &*galley.text, inkview_sys::TextAlignFlag::VALIGN_BOTTOM as i32 | inkview_sys::TextAlignFlag::ALIGN_LEFT as i32);
                }
                Shape::Mesh(_) => {}
            }
        }
    }
}

impl inkview_sys::EventHandler for AppRunner{
    fn handle_event(&mut self, event: c_api::Event, par1: i32, par2: i32) -> i32 {
        let mut ctx = self.pocketbook_backend.egui_ctx.clone();
        match event{
            c_api::Event::KEYPRESS => {

                unsafe { c_api::CloseApp(); }
            }

            c_api::Event::SHOW | c_api::Event::POINTERDOWN | c_api::Event::POINTERUP | c_api::Event::POINTERDRAG => {
                if event == c_api::Event::SHOW{
                    inkview_sys::set_panel_type(c_api::PanelType(0));
                }
                let events = match event{
                    c_api::Event::POINTERDOWN => {
                        vec![Event::Touch{
                            device_id: TouchDeviceId(0),
                            id: TouchId(0),
                            phase: TouchPhase::Start,
                            pos: Pos2{ x: par1 as f32/ctx.pixels_per_point(), y: par2 as f32/ctx.pixels_per_point() },
                            force: 0.0
                        },
                             Event::PointerButton {
                                 pos: Pos2{ x: par1 as f32/ctx.pixels_per_point(), y: par2 as f32/ctx.pixels_per_point() },
                                 button: PointerButton::Primary,
                                 pressed: true,
                                 modifiers: Default::default()
                             }

                        ]
                    }


                    c_api::Event::POINTERUP => {
                        vec![
                            Event::Touch{
                                device_id: TouchDeviceId(0),
                                id: TouchId(0),
                                phase: TouchPhase::End,
                                pos: Pos2{ x: par1 as f32/ctx.pixels_per_point(), y: par2 as f32/ctx.pixels_per_point() },
                                force: 0.0
                            },
                            Event::PointerButton {
                                pos: Pos2{ x: par1 as f32/ctx.pixels_per_point(), y: par2 as f32/ctx.pixels_per_point() },
                                button: PointerButton::Primary,
                                pressed: false,
                                modifiers: Default::default()
                            }

                        ]
                    }
                    c_api::Event::POINTERDRAG => {
                        vec![Event::PointerMoved(Pos2{ x: par1 as f32/ctx.pixels_per_point(), y: par2 as f32/ctx.pixels_per_point() })]}

                    _=>vec![]
                };
                let raw_input: egui::RawInput = egui::RawInput {
                    scroll_delta: Vec2 { x: 0.0, y: 0.0 },
                    zoom_delta: 0.0,
                    screen_size: Vec2 { x: 1404f32 / 2f32, y: 1872f32 / 2f32 },
                    screen_rect: Some(egui::Rect { min: Default::default(), max: egui::Pos2 { x: 1404f32, y: 1872f32 } }),
                    pixels_per_point: Some(3f32),
                    time: None,
                    predicted_dt: 0.0,
                    modifiers: Default::default(),
                    events: events,
                };
                self.pocketbook_backend.begin_frame(raw_input);

                let mut texture_allocator = PocketbookTextureAllocator{};



                let mut app_output = epi::backend::AppOutput::default();

                let mut frame = epi::backend::FrameBuilder {
                    info: IntegrationInfo{
                        web_info: None,
                        prefer_dark_mode: None,
                        cpu_usage: None,
                        seconds_since_midnight: None,
                        native_pixels_per_point: None
                    },
                    tex_allocator: &mut texture_allocator,
                    #[cfg(feature = "http")]
                    http: runner.http.clone(),
                    output: &mut app_output,
                    repaint_signal: self.needs_repaint.clone(),
                }
                    .build();

                self.app.update(&ctx, &mut frame);

                let (output, shapes) = ctx.end_frame();
                self.draw_shapes(shapes);
                self.pocketbook_backend.end_frame();
                //
                if event == c_api::Event::SHOW{
                    inkview_sys::full_update();
                }

            }
            _ => {}
        }

        0

    }
}

