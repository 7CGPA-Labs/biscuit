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
}

pub struct WebView {
    widget: gtk::Widget,
}

impl WebView {
    pub fn new() -> Self {
        let widget = unsafe {
            let ptr = webkit_web_view_new();
            from_glib_none(ptr)
        };

        Self { widget }
    }

    pub fn get_widget(&self) -> &gtk::Widget {
        &self.widget
    }

    pub fn load_html(&self, html: &str) {
        let html_c = CString::new(html).unwrap();
        unsafe {
            webkit_web_view_load_html(
                self.widget.as_ptr() as *mut _,
                html_c.as_ptr(),
                std::ptr::null(),
            );
        }
    }
}

pub fn render_markdown(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(markdown, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    let katex_css = include_str!(concat!(env!("OUT_DIR"), "/katex/katex/katex.min.css"));
    let katex_js = include_str!(concat!(env!("OUT_DIR"), "/katex/katex/katex.min.js"));
    let auto_render_js = include_str!(concat!(
        env!("OUT_DIR"),
        "/katex/katex/contrib/auto-render.min.js"
    ));

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
        katex_css, katex_js, auto_render_js, html_output
    )
}
