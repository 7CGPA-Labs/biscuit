use glib::translate::*;
use gtk::prelude::*;
use pulldown_cmark::{html, Options, Parser};
use std::ffi::CString;

#[link(name = "webkitgtk-6.0")]
extern "C" {
    pub fn webkit_web_view_new() -> *mut gtk::ffi::GtkWidget;
    pub fn webkit_web_view_load_html(
        web_view: *mut std::ffi::c_void,
        content: *const std::ffi::c_char,
        base_uri: *const std::ffi::c_char,
    );
    pub fn webkit_web_view_set_zoom_level(
        web_view: *mut std::ffi::c_void,
        zoom_level: f64,
    );
}

pub struct WebView {
    widget: gtk::Widget,
}

impl WebView {
    pub fn new() -> Self {
        let widget = unsafe {
            let ptr = webkit_web_view_new();
            let widget: gtk::Widget = from_glib_none(ptr);
            
            // Add GTK gesture to intercept and block right-clicks at the widget level
            let gesture = gtk::GestureClick::new();
            gesture.set_button(3); // Right click
            gesture.connect_pressed(|gesture, _, _, _| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
            });
            widget.add_controller(gesture);
            
            widget
        };

        Self { widget }
    }

    pub fn get_widget(&self) -> &gtk::Widget {
        &self.widget
    }

    pub fn load_html(&self, html: &str, base_uri: Option<&str>) {
        let html_c = CString::new(html).unwrap();
        let base_uri_c = base_uri.map(|s| CString::new(s).unwrap());
        
        unsafe {
            webkit_web_view_load_html(
                self.widget.as_ptr() as *mut _,
                html_c.as_ptr(),
                base_uri_c.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
            );
        }
    }

    pub fn set_zoom_level(&self, zoom_level: f64) {
        unsafe {
            webkit_web_view_set_zoom_level(
                self.widget.as_ptr() as *mut _,
                zoom_level,
            );
        }
    }
}

pub const KATEX_AUTO_RENDER_JS: &str =
    include_str!(concat!(env!("OUT_DIR"), "/katex/katex/contrib/auto-render.min.js"));

pub const LATEX_RENDERER_TGZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/latex_renderer.tar.gz"));

pub fn strip_yaml_frontmatter(content: &str) -> &str {
    if let Some(frontmatter_end) = content.find("\n---\n") {
        if content.starts_with("---\n") {
            return &content[frontmatter_end + 5..];
        }
    }
    content
}

pub fn render_preview(content: &str, is_latex: bool) -> String {
    let stripped_content = strip_yaml_frontmatter(content);

    if is_latex {
        let escaped = stripped_content
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
            
        let cache_dir = dirs::cache_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
        let latex_dir = cache_dir.join("biscuit").join("latex_renderer");
        let base_url = format!("file://{}/", latex_dir.to_str().unwrap());

        return format!(r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="utf-8">
                <style>
                    body {{ margin: 0; padding: 2rem; background: #fff; }}
                    @media (prefers-color-scheme: dark) {{
                        body {{ background: #0d1117; color: #c9d1d9; }}
                    }}
                </style>
                <script>
                    window.onerror = function(msg, url, lineNo, columnNo, error) {{
                        document.body.innerHTML += '<div style=\"color:red; background:white; position:fixed; top:0; left:0; z-index:9999; width:100%; height:100%; overflow:auto;\">Error: ' + msg + '<br>Line: ' + lineNo + '</div>';
                        return false;
                    }};
                </script>
            </head>
            <body>
                <script src="{}latex.js"></script>
                <script src="{}index.js"></script>
                <script>
                    window.renderLatex(`{}`);
                </script>
            </body>
            </html>
        "#, base_url, base_url, escaped.replace('`', "\\`").replace('\\', "\\\\").replace('$', "\\$"));
    }

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(stripped_content, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    let katex_css = include_str!(concat!(env!("OUT_DIR"), "/katex/katex/katex.min.css"));
    let katex_js = include_str!(concat!(env!("OUT_DIR"), "/katex/katex/katex.min.js"));

    // Base HTML template with KaTeX
    format!(
        r#"
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset="utf-8">
        <style>
            {}
        </style>
        <script>
            {}
        </script>
        <script>
            {}
        </script>
        <script>
            document.addEventListener("DOMContentLoaded", function() {{
                renderMathInElement(document.body, {{
                    delimiters: [
                        {{left: '$$', right: '$$', display: true}},
                        {{left: '$', right: '$', display: false}},
                        {{left: '\\(', right: '\\)', display: false}},
                        {{left: '\\[', right: '\\]', display: true}}
                    ]
                }});
            }});
        </script>
        <style>
            body {{
                font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
                line-height: 1.6;
                padding: 2rem;
                color: #333;
                background-color: #fff;
            }}
            pre {{
                background-color: #f6f8fa;
                padding: 16px;
                border-radius: 6px;
                overflow: auto;
            }}
            code {{
                font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Liberation Mono", monospace;
                background-color: rgba(175,184,193,0.2);
                padding: 0.2em 0.4em;
                border-radius: 6px;
            }}
            blockquote {{
                border-left: .25em solid #d0d7de;
                padding: 0 1em;
                color: #656d76;
            }}
            @media (prefers-color-scheme: dark) {{
                body {{
                    color: #c9d1d9;
                    background-color: #0d1117;
                }}
                pre, code {{ background-color: rgba(110,118,129,0.4); }}
            }}
        </style>
    </head>
    <body>
        {}
    </body>
    </html>
    "#,
        katex_css, katex_js, KATEX_AUTO_RENDER_JS, html_output
    )
}
