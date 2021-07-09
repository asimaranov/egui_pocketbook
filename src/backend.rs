use egui::{CtxRef, TouchDeviceId, TouchId, TouchPhase, Pos2, PointerButton, Vec2};
use crate::texture_allocator::PocketbookTextureAllocator;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;
use epi::IntegrationInfo;
use inkview_sys::c_api::Event;
use inkview_sys::c_api;

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

    pub fn end_frame(&mut self) -> (egui::Output, Vec<egui::ClippedMesh>) {
        let frame_start = self
            .frame_start
            .take()
            .expect("unmatched calls to begin_frame/end_frame");

        let (output, shapes) = self.egui_ctx.end_frame();
        let clipped_meshes = self.egui_ctx.tessellate(shapes);

        let now = 1f64; // TODO
        self.previous_frame_time = Some((now - frame_start) as f32);

        (output, clipped_meshes)
    }



}

pub struct AppRunner {
    pocketbook_backend: PocketbookBackend,
    app: Box<dyn epi::App>,
    pub(crate) needs_repaint: std::sync::Arc<NeedRepaint>,



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
            needs_repaint: Arc::from(NeedRepaint::default())

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
}

impl inkview_sys::EventHandler for AppRunner{
    fn handle_event(&mut self, event: Event, par1: i32, par2: i32) -> i32 {
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
                ctx.begin_frame(raw_input);

                self.app.draw(&mut ctx);

                let (output, shapes) = ctx.end_frame();
                self.draw_shapes(shapes);
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

