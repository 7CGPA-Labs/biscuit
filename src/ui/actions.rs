#![allow(deprecated)]
use crate::ui::tab_bar::TabBar;
use gtk::gio;
use gtk::prelude::*;
use gtk::FileDialog;
use libadwaita::ApplicationWindow;
use sourceview5::prelude::*;
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use libadwaita::TabPage;
use std::collections::HashMap;

pub struct TabState {
    pub file_path: Option<PathBuf>,
    pub buffer: sourceview5::Buffer,
    pub base_title: Rc<RefCell<String>>,
    pub web_view_container: gtk::ScrolledWindow,
    pub current_web_view: Rc<RefCell<Option<crate::preview::WebView>>>,
    pub is_latex: Rc<RefCell<bool>>,
    pub preview_stale: Rc<RefCell<bool>>,
    pub spinner: gtk::Spinner,
    pub warning_bar: gtk::InfoBar,
}

pub struct AppState {
    pub open_tabs: HashMap<TabPage, TabState>,
    pub css_provider: gtk::CssProvider,
    pub zoom_level: f64,
}

pub fn reload_preview(ts: &TabState, zoom_level: f64) {
    let start = ts.buffer.start_iter();
    let end = ts.buffer.end_iter();
    let text = ts.buffer.text(&start, &end, false);
    
    let is_ltx = *ts.is_latex.borrow();
    let html = crate::preview::render_preview(text.as_str(), is_ltx);
    let base_uri = if is_ltx {
        let cache_dir = dirs::cache_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
        let latexjs_dir = cache_dir.join("biscuit").join("latexjs-0.12.6");
        Some(format!("file://{}/", latexjs_dir.to_str().unwrap()))
    } else {
        None
    };

    if let Some(old_wv) = ts.current_web_view.borrow_mut().take() {
        if let Some(_parent) = old_wv.get_widget().parent() {
            // Need to use ScrolledWindow's set_child to replace instead of parent.remove since parent might be a viewport
            ts.web_view_container.set_child(gtk::Widget::NONE);
        }
    }

    let new_wv = crate::preview::WebView::new();
    new_wv.set_zoom_level(zoom_level);
    new_wv.load_html(&html, base_uri.as_deref());
    
    ts.web_view_container.set_child(Some(new_wv.get_widget()));
    *ts.current_web_view.borrow_mut() = Some(new_wv);
    
    ts.spinner.start();
    let spinner_clone = ts.spinner.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(1500), move || {
        spinner_clone.stop();
        glib::ControlFlow::Break
    });

    *ts.preview_stale.borrow_mut() = false;
    ts.warning_bar.set_revealed(false);
}

pub fn setup_actions(
    app: &libadwaita::Application,
    window: &ApplicationWindow,
    state: Rc<RefCell<AppState>>,
    tab_bar: &TabBar,
    create_tab: Rc<dyn Fn(Option<PathBuf>, &str)>,
    type_label: gtk::Label,
) {
    // New File Action
    let action_new = gio::SimpleAction::new("new", None);
    let create_tab_clone = create_tab.clone();
    action_new.connect_activate(move |_, _| {
        create_tab_clone(None, "");
    });
    app.add_action(&action_new);
    app.set_accels_for_action("app.new", &["<Ctrl>n"]);

    // Open Action
    let action_open = gio::SimpleAction::new("open", None);
    let window_clone = window.clone();
    let create_tab_clone = create_tab.clone();
    action_open.connect_activate(move |_, _| {
        let dialog = FileDialog::new();
        let filter = gtk::FileFilter::new();
        filter.add_pattern("*.md");
        filter.add_pattern("*.tex");
        filter.set_name(Some("Markdown and LaTeX files"));

        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        dialog.set_filters(Some(&filters));

        let create_tab_inner = create_tab_clone.clone();

        dialog.open(Some(&window_clone), gio::Cancellable::NONE, move |result| {
            if let Ok(file) = result {
                let path = file.path().expect("Expected a path");
                if let Ok(content) = fs::read_to_string(&path) {
                    create_tab_inner(Some(path), &content);
                }
            }
        });
    });
    app.add_action(&action_open);
    app.set_accels_for_action("app.open", &["<Ctrl>o"]);

    // Save Action
    let type_label_for_save = type_label.clone();
    let action_save = gio::SimpleAction::new("save", None);
    let state_clone = state.clone();
    let window_clone = window.clone();
    let tab_view = tab_bar.tab_view.clone();
    action_save.connect_activate(move |_, _| {
        if let Some(page) = tab_view.selected_page() {
            let mut app_state = state_clone.borrow_mut();
            if let Some(tab_state) = app_state.open_tabs.get_mut(&page) {
                if let Some(path) = &tab_state.file_path {
                    let start = tab_state.buffer.start_iter();
                    let end = tab_state.buffer.end_iter();
                    let text = tab_state.buffer.text(&start, &end, false);
                    let _ = fs::write(path, text.as_str());
                    tab_state.buffer.set_modified(false);
                } else {
                    let dialog = FileDialog::new();
                    let filter = gtk::FileFilter::new();
                    filter.add_pattern("*.md");
                    filter.add_pattern("*.tex");
                    filter.set_name(Some("Markdown and LaTeX files"));

                    let filters = gio::ListStore::new::<gtk::FileFilter>();
                    filters.append(&filter);
                    dialog.set_filters(Some(&filters));

                    let state_inner = state_clone.clone();
                    let page_clone = page.clone();
                    let type_label_save = type_label_for_save.clone();

                    dialog.save(Some(&window_clone), gio::Cancellable::NONE, move |result| {
                        if let Ok(file) = result {
                            let path = file.path().expect("Expected a path");

                            if let Some(ts) =
                                state_inner.borrow_mut().open_tabs.get_mut(&page_clone)
                            {
                                let start = ts.buffer.start_iter();
                                let end = ts.buffer.end_iter();
                                let text = ts.buffer.text(&start, &end, false);
                                let _ = fs::write(&path, text.as_str());
                                ts.file_path = Some(path.clone());
                                ts.buffer.set_modified(false);

                                if let Some(name) = path.file_name() {
                                    let new_title = name.to_string_lossy().into_owned();
                                    *ts.base_title.borrow_mut() = new_title.clone();
                                    page_clone.set_title(&new_title);
                                }
                                *ts.is_latex.borrow_mut() = path.extension().map_or(false, |ext| ext == "tex");
                                if *ts.is_latex.borrow() {
                                    type_label_save.set_label("LaTeX");
                                } else {
                                    type_label_save.set_label("Markdown");
                                }
                                // force update preview
                                let zoom_level = state_inner.borrow().zoom_level;
                                reload_preview(ts, zoom_level);
                            }
                        }
                    });
                }
            }
        }
    });
    app.add_action(&action_save);
    app.set_accels_for_action("app.save", &["<Ctrl>s"]);

    // Save As Action
    let action_save_as = gio::SimpleAction::new("save_as", None);
    let state_clone = state.clone();
    let window_clone = window.clone();
    let tab_view = tab_bar.tab_view.clone();
    let type_label_for_save_as = type_label.clone();
    action_save_as.connect_activate(move |_, _| {
        if let Some(page) = tab_view.selected_page() {
            let dialog = FileDialog::new();
            let filter = gtk::FileFilter::new();
            filter.add_pattern("*.md");
            filter.add_pattern("*.tex");
            filter.set_name(Some("Markdown and LaTeX files"));

            let filters = gio::ListStore::new::<gtk::FileFilter>();
            filters.append(&filter);
            dialog.set_filters(Some(&filters));

            let state_inner = state_clone.clone();
            let page_clone = page.clone();
            let type_label_save_as = type_label_for_save_as.clone();

            dialog.save(Some(&window_clone), gio::Cancellable::NONE, move |result| {
                if let Ok(file) = result {
                    let path = file.path().expect("Expected a path");

                    if let Some(ts) = state_inner.borrow_mut().open_tabs.get_mut(&page_clone) {
                        let start = ts.buffer.start_iter();
                        let end = ts.buffer.end_iter();
                        let text = ts.buffer.text(&start, &end, false);
                        let _ = fs::write(&path, text.as_str());
                        ts.file_path = Some(path.clone());
                        ts.buffer.set_modified(false);

                        if let Some(name) = path.file_name() {
                            let new_title = name.to_string_lossy().into_owned();
                            *ts.base_title.borrow_mut() = new_title.clone();
                            page_clone.set_title(&new_title);
                        }
                        *ts.is_latex.borrow_mut() = path.extension().map_or(false, |ext| ext == "tex");
                        if *ts.is_latex.borrow() {
                            type_label_save_as.set_label("LaTeX");
                        } else {
                            type_label_save_as.set_label("Markdown");
                        }
                        // force update preview
                        let zoom_level = state_inner.borrow().zoom_level;
                        reload_preview(ts, zoom_level);
                    }
                }
            });
        }
    });
    app.add_action(&action_save_as);
    app.set_accels_for_action("app.save_as", &["<Ctrl><Shift>s"]);

    // Export actions (dummy for now)
    let action_export_pdf = gio::SimpleAction::new("export_pdf", None);
    action_export_pdf.connect_activate(|_, _| println!("Export to PDF triggered"));
    app.add_action(&action_export_pdf);

    let action_export_docx = gio::SimpleAction::new("export_docx", None);
    action_export_docx.connect_activate(|_, _| println!("Export to DOCX triggered"));
    app.add_action(&action_export_docx);

    let action_prefs = gio::SimpleAction::new("preferences", None);
    action_prefs.connect_activate(|_, _| println!("Preferences triggered"));
    app.add_action(&action_prefs);

    // About action
    let action_about = gio::SimpleAction::new("about", None);
    let _window_clone = window.clone();
    action_about.connect_activate(move |_, _| {
        let about = libadwaita::AboutWindow::builder()
            .application_name("Biscuit")
            .developer_name("Developer")
            .version("0.1.0")
            .build();
        about.present();
    });
    app.add_action(&action_about);

    // Zoom In
    let action_zoom_in = gio::SimpleAction::new("zoom_in", None);
    let state_clone = state.clone();
    action_zoom_in.connect_activate(move |_, _| {
        let mut s = state_clone.borrow_mut();
        s.zoom_level += 0.1;
        
        let css = format!("textview {{ font-size: {}em; }}", s.zoom_level);
        s.css_provider.load_from_string(&css);
        
        for ts in s.open_tabs.values() {
            if let Some(wv) = ts.current_web_view.borrow().as_ref() {
                wv.set_zoom_level(s.zoom_level);
            }
        }
    });
    app.add_action(&action_zoom_in);
    app.set_accels_for_action("app.zoom_in", &["<Ctrl>equal", "<Ctrl>plus"]);

    // Zoom Out
    let action_zoom_out = gio::SimpleAction::new("zoom_out", None);
    let state_clone = state.clone();
    action_zoom_out.connect_activate(move |_, _| {
        let mut s = state_clone.borrow_mut();
        s.zoom_level = (s.zoom_level - 0.1).max(0.5); // minimum zoom level 0.5
        
        let css = format!("textview {{ font-size: {}em; }}", s.zoom_level);
        s.css_provider.load_from_string(&css);
        
        for ts in s.open_tabs.values() {
            if let Some(wv) = ts.current_web_view.borrow().as_ref() {
                wv.set_zoom_level(s.zoom_level);
            }
        }
    });
    app.add_action(&action_zoom_out);
    app.set_accels_for_action("app.zoom_out", &["<Ctrl>minus"]);

    // Toggle Theme
    let action_toggle_theme = gio::SimpleAction::new("toggle_theme", None);
    let state_clone = state.clone();
    action_toggle_theme.connect_activate(move |_, _| {
        let manager = libadwaita::StyleManager::default();
        let is_dark = if manager.is_dark() {
            manager.set_color_scheme(libadwaita::ColorScheme::ForceLight);
            false
        } else {
            manager.set_color_scheme(libadwaita::ColorScheme::ForceDark);
            true
        };

        let style_manager = sourceview5::StyleSchemeManager::default();
        let scheme = if is_dark {
            style_manager.scheme("Adwaita-dark").or_else(|| style_manager.scheme("oblivion"))
        } else {
            style_manager.scheme("Adwaita").or_else(|| style_manager.scheme("classic"))
        };

        if let Some(scheme) = scheme {
            let s = state_clone.borrow();
            for ts in s.open_tabs.values() {
                ts.buffer.set_style_scheme(Some(&scheme));
            }
        }
    });
    app.add_action(&action_toggle_theme);

    // Quit Action
    let action_quit = gio::SimpleAction::new("quit", None);
    let app_clone = app.clone();
    action_quit.connect_activate(move |_, _| {
        app_clone.quit();
    });
    app.add_action(&action_quit);
    app.set_accels_for_action("app.quit", &["<Ctrl>q"]);
}

