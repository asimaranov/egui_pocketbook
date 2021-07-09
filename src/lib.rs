use crate::backend::{PocketbookBackend, AppRunner};
use inkview_sys::c_api;
use std::sync::{Mutex, Arc};
use egui::Response;

mod app;
mod backend;
mod texture_allocator;
mod storage;

pub fn start(mut app: Box<dyn epi::App>, native_options: epi::NativeOptions) -> () {
    let backend = PocketbookBackend::new();
    let mut runner = AppRunner::new(backend, app);
    unsafe { c_api::OpenScreen() }

    //let c = pb_ui::UiComponent{ pos: (), size: (), data: ()};
    let h: Arc<Mutex<dyn inkview_sys::EventHandler>> = Arc::new(Mutex::new(runner));


    inkview_sys::main(&h);
}

pub fn handle_component_update(response: Response) -> Response{
    if response.changed(){

        inkview_sys::partial_update((response.rect.min.x * response.ctx.pixels_per_point()) as i32,
                                    (response.rect.min.y * response.ctx.pixels_per_point()) as i32,
                                    (response.rect.width() * response.ctx.pixels_per_point()) as i32,
                                    (response.rect.height() * response.ctx.pixels_per_point()) as i32,

        );
    }
    return response
}