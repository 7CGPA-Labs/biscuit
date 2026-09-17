use gtk::prelude::*;
use glib::clone;
use std::path::PathBuf;
use std::process::Command;
use webkit6::prelude::*;
use webkit6::{WebView, LoadEvent};
use std::rc::Rc;
use std::cell::RefCell;

pub struct PdfPreview {
    webview: WebView,
    is_loaded: Rc<RefCell<bool>>,
    pending_script: Rc<RefCell<Option<String>>>,
}

impl PdfPreview {
    pub fn new() -> Self {
        let webview = WebView::new();
        if let Some(settings) = webkit6::prelude::WebViewExt::settings(&webview) {
            settings.set_enable_developer_extras(true);
            settings.set_enable_write_console_messages_to_stdout(true);
        }
        
        let is_loaded = Rc::new(RefCell::new(false));
        let pending_script: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
        
        let html_content = include_str!(concat!(env!("OUT_DIR"), "/webview_index.html"));
        webview.load_html(html_content, Some("http://localhost/"));
        
        let loaded_clone = is_loaded.clone();
        let pending_clone = pending_script.clone();
        
        webview.connect_load_changed(move |wv, event| {
            if event == LoadEvent::Finished {
                *loaded_clone.borrow_mut() = true;
                if let Some(script) = pending_clone.borrow_mut().take() {
                    wv.evaluate_javascript(&script, None, None, gtk::gio::Cancellable::NONE, |result| {
                        if let Err(e) = result {
                            println!("JS Evaluation Error (pending): {}", e);
                        }
                    });
                }
            }
        });
        
        Self { webview, is_loaded, pending_script }
    }

    pub fn get_widget(&self) -> &gtk::Widget {
        self.webview.upcast_ref()
    }

    pub fn set_zoom_level(&self, zoom_level: f64) {
        self.webview.set_zoom_level(zoom_level);
    }

    pub fn load_content(&self, text: &str, is_latex: bool) {
        let escaped_text = text.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$");
        let script = format!("if (window.renderContent) {{ window.renderContent(`{}`, {}); }}", escaped_text, is_latex);
        
        if *self.is_loaded.borrow() {
            self.webview.evaluate_javascript(&script, None, None, gtk::gio::Cancellable::NONE, |result| {
                if let Err(e) = result {
                    println!("JS Evaluation Error (direct): {}", e);
                }
            });
        } else {
            *self.pending_script.borrow_mut() = Some(script);
        }
    }
}

pub fn export_document(text: &str, is_latex: bool, output_path: &std::path::Path, format: &str) {
    let out_dir = PathBuf::from(env!("OUT_DIR"));
    let temp_dir = std::env::temp_dir();
    
    let text = text.to_string();
    let is_latex = is_latex;
    let output_path = output_path.to_path_buf();
    let format = format.to_string();
    
    // Create a progress window
    let window = gtk::Window::builder()
        .title("Exporting Document")
        .default_width(300)
        .default_height(100)
        .modal(true)
        .deletable(false)
        .build();
        
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
    vbox.set_margin_top(20);
    vbox.set_margin_bottom(20);
    vbox.set_margin_start(20);
    vbox.set_margin_end(20);
    
    let label = gtk::Label::new(Some("Exporting..."));
    let progress_bar = gtk::ProgressBar::new();
    
    vbox.append(&label);
    vbox.append(&progress_bar);
    window.set_child(Some(&vbox));
    
    window.present();

    let (sender, receiver) = std::sync::mpsc::channel();
    
    std::thread::spawn(move || {
        let success = if format == "pdf" {
            if is_latex {
                let tectonic = out_dir.join("tectonic");
                let p = Command::new(&tectonic)
                    .arg("-")
                    .arg("-o")
                    .arg(&temp_dir)
                    .stdin(std::process::Stdio::piped())
                    .spawn();
                
                if let Ok(mut child) = p {
                    if let Some(mut stdin) = child.stdin.take() {
                        use std::io::Write;
                        let _ = stdin.write_all(text.as_bytes());
                    }
                    let res = child.wait().unwrap();
                    if res.success() {
                        let texput = temp_dir.join("texput.pdf");
                        if texput.exists() {
                            let _ = std::fs::rename(texput, &output_path);
                            true
                        } else { false }
                    } else { false }
                } else { false }
            } else {
                let pandoc = out_dir.join("pandoc");
                let typst = out_dir.join("typst");
                let p = Command::new(&pandoc)
                    .arg("-")
                    .arg("-o")
                    .arg(&output_path)
                    .arg(format!("--pdf-engine={}", typst.display()))
                    .stdin(std::process::Stdio::piped())
                    .spawn();
                
                if let Ok(mut child) = p {
                    if let Some(mut stdin) = child.stdin.take() {
                        use std::io::Write;
                        let _ = stdin.write_all(text.as_bytes());
                    }
                    child.wait().unwrap().success()
                } else { false }
            }
        } else if format == "docx" {
            let pandoc = out_dir.join("pandoc");
            let mut cmd = Command::new(&pandoc);
            cmd.arg("-").arg("-o").arg(&output_path);
            if is_latex { cmd.arg("-f").arg("latex"); }
            cmd.stdin(std::process::Stdio::piped());
            
            if let Ok(mut child) = cmd.spawn() {
                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    let _ = stdin.write_all(text.as_bytes());
                }
                child.wait().unwrap().success()
            } else { false }
        } else { false };
        
        let _ = sender.send(success);
    });

    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        progress_bar.pulse();
        match receiver.try_recv() {
            Ok(success) => {
                if success {
                    label.set_text("Export completed successfully!");
                } else {
                    label.set_text("Export failed.");
                }
                glib::timeout_add_local(std::time::Duration::from_secs(2), clone!(@weak window => @default-return glib::ControlFlow::Break, move || {
                    window.close();
                    glib::ControlFlow::Break
                }));
                glib::ControlFlow::Break
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                window.close();
                glib::ControlFlow::Break
            }
        }
    });
}
