use gtk::prelude::*;
use glib::translate::*;

#[link(name = "webkitgtk-6.0")]
extern "C" {
    pub fn webkit_web_view_new() -> *mut gtk::ffi::GtkWidget;
    pub fn webkit_web_view_load_html(
        web_view: *mut std::ffi::c_void,
        content: *const std::ffi::c_char,
        base_uri: *const std::ffi::c_char,
    );
}

fn main() {
    gtk::init().unwrap();
    let web_view: gtk::Widget = unsafe {
        let ptr = webkit_web_view_new();
        from_glib_none(ptr)
    };
    println!("Webview created: {:?}", web_view);
}
