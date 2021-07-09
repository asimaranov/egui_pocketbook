use crate::backend::{PocketbookBackend, AppRunner};
use inkview_sys::c_api;
use std::sync::{Mutex, Arc};

mod app;
mod backend;
mod texture_allocator;

pub fn start(mut app: Box<dyn epi::App>, native_options: epi::NativeOptions) -> () {
    let backend = PocketbookBackend::new();
    let mut runner = AppRunner::new(backend, app);
    unsafe { c_api::OpenScreen() }

    //let c = pb_ui::UiComponent{ pos: (), size: (), data: ()};
    let h: Arc<Mutex<dyn inkview_sys::EventHandler>> = Arc::new(Mutex::new(MyHandler::new()));


    inkview_sys::main(&h);
}
