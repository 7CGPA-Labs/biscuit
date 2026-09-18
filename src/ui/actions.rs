#![allow(deprecated)]
use crate::ui::tab_bar::TabBar;
use gtk::gio;
use gtk::prelude::*;
use gtk::FileDialog;
use libadwaita::ApplicationWindow;

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
    pub preview_container: gtk::Overlay,
    pub current_preview: Rc<RefCell<Option<crate::preview::PdfPreview>>>,
    pub is_latex: Rc<RefCell<bool>>,
    pub preview_stale: Rc<RefCell<bool>>,
    pub spinner: gtk::Spinner,
    pub warning_bar: gtk::InfoBar,
    pub clippy: Rc<crate::clippy::ClippyOverlay>,
}

pub struct AppState {
    pub open_tabs: HashMap<TabPage, TabState>,
    pub css_provider: gtk::CssProvider,
    pub zoom_level: f64,
}

pub fn reload_preview(ts: &TabState, zoom_level: f64, hard_reload: bool) {
    let start = ts.buffer.start_iter();
    let end = ts.buffer.end_iter();
    let text = ts.buffer.text(&start, &end, false);
    
    let is_ltx = *ts.is_latex.borrow();

    let wv = if let Some(old_wv) = ts.current_preview.borrow_mut().take() {
        old_wv
    } else {
        let new_wv = crate::preview::PdfPreview::new();
        ts.preview_container.set_child(Some(new_wv.get_widget()));
        new_wv
    };
    
    wv.set_zoom_level(zoom_level);
    let is_dark = libadwaita::StyleManager::default().is_dark();

    if hard_reload {
        wv.hard_reload(&text, is_ltx, is_dark);
    } else {
        wv.load_content(&text, is_ltx);
        wv.set_theme(is_dark);
    }
    
    *ts.current_preview.borrow_mut() = Some(wv);
    


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
    // Shared AiWorker
    let ai_worker = std::sync::Arc::new(crate::ai::worker::AiWorker::new());
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
                                reload_preview(ts, zoom_level, false);
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
                        reload_preview(ts, zoom_level, false);
                    }
                }
            });
        }
    });
    app.add_action(&action_save_as);
    app.set_accels_for_action("app.save_as", &["<Ctrl><Shift>s"]);

    // Export actions
    let action_export_pdf = gio::SimpleAction::new("export_pdf", None);
    let state_clone = state.clone();
    let window_clone = window.clone();
    let tab_view = tab_bar.tab_view.clone();
    action_export_pdf.connect_activate(move |_, _| {
        if let Some(page) = tab_view.selected_page() {
            let s = state_clone.borrow();
            if let Some(ts) = s.open_tabs.get(&page) {
                let is_ltx = *ts.is_latex.borrow();
                let start = ts.buffer.start_iter();
                let end = ts.buffer.end_iter();
                let text = ts.buffer.text(&start, &end, false).to_string();
                
                let dialog = FileDialog::new();
                let filter = gtk::FileFilter::new();
                filter.add_pattern("*.pdf");
                filter.set_name(Some("PDF Documents"));
                let filters = gio::ListStore::new::<gtk::FileFilter>();
                filters.append(&filter);
                dialog.set_filters(Some(&filters));

                dialog.save(Some(&window_clone), gio::Cancellable::NONE, move |result| {
                    if let Ok(file) = result {
                        let path = file.path().expect("Expected a path");
                        crate::preview::export_document(&text, is_ltx, &path, "pdf");
                    }
                });
            }
        }
    });
    app.add_action(&action_export_pdf);

    let action_export_docx = gio::SimpleAction::new("export_docx", None);
    let state_clone = state.clone();
    let window_clone = window.clone();
    let tab_view = tab_bar.tab_view.clone();
    action_export_docx.connect_activate(move |_, _| {
        if let Some(page) = tab_view.selected_page() {
            let s = state_clone.borrow();
            if let Some(ts) = s.open_tabs.get(&page) {
                let is_ltx = *ts.is_latex.borrow();
                let start = ts.buffer.start_iter();
                let end = ts.buffer.end_iter();
                let text = ts.buffer.text(&start, &end, false).to_string();
                
                let dialog = FileDialog::new();
                let filter = gtk::FileFilter::new();
                filter.add_pattern("*.docx");
                filter.set_name(Some("Word Documents"));
                let filters = gio::ListStore::new::<gtk::FileFilter>();
                filters.append(&filter);
                dialog.set_filters(Some(&filters));

                dialog.save(Some(&window_clone), gio::Cancellable::NONE, move |result| {
                    if let Ok(file) = result {
                        let path = file.path().expect("Expected a path");
                        crate::preview::export_document(&text, is_ltx, &path, "docx");
                    }
                });
            }
        }
    });
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

    // Helper for Clippy actions
    fn trigger_clippy_action(
        intent: crate::ai::models::Intent,
        tab_view: &libadwaita::TabView,
        state: &std::rc::Rc<std::cell::RefCell<AppState>>,
        worker: &std::sync::Arc<crate::ai::worker::AiWorker>,
    ) {
        if let Some(page) = tab_view.selected_page() {
            let s = state.borrow();
            if let Some(ts) = s.open_tabs.get(&page) {
                let clippy = ts.clippy.clone();
                let buffer = ts.buffer.clone();
                let worker_inner = worker.clone();

                clippy.think();
                
                let (start, end) = if buffer.has_selection() {
                    buffer.selection_bounds().unwrap_or((buffer.start_iter(), buffer.end_iter()))
                } else {
                    let mut start = buffer.start_iter();
                    let mut end = buffer.end_iter();
                    if let Some(mark) = buffer.mark("insert") {
                        let iter = buffer.iter_at_mark(&mark);
                        start = iter.clone();
                        start.backward_line();
                        end = iter.clone();
                        end.forward_line();
                    }
                    (start, end)
                };

                let mark_start = buffer.create_mark(None, &start, true);
                let mark_end = buffer.create_mark(None, &end, false);

                let selected_text = buffer.text(&start, &end, false).to_string();

                let (tx, rx) = std::sync::mpsc::channel();
                
                std::thread::spawn(move || {
                    let result = match intent {
                        crate::ai::models::Intent::FixGrammar => {
                            crate::ai::models::fix_grammar(&worker_inner, &selected_text)
                        }
                        crate::ai::models::Intent::Ghostwrite => {
                            crate::ai::models::ghostwrite(&worker_inner, &selected_text)
                        }
                        crate::ai::models::Intent::Unknown => {
                            crate::ai::models::fix_grammar(&worker_inner, &selected_text) // Fallback
                        }
                    };
                    let _ = tx.send((intent, selected_text, result));
                });
                
                let buffer_clone = buffer.clone();
                glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                    match rx.try_recv() {
                        Ok((intent, original, result)) => {
                            match result {
                                Ok(new_text) => {
                                    clippy.set_state(crate::clippy::sprite::ClippyState::Alert);
                                    
                                    let diffs = crate::ai::models::compute_diff(&original, &new_text);
                                    let mut display_text = format!("{:?}:\n", intent);
                            for (tag, val) in diffs {
                                match tag {
                                    similar::ChangeTag::Delete => display_text.push_str(&format!("[-{}-]", val.trim())),
                                    similar::ChangeTag::Insert => display_text.push_str(&format!("[+{}+]", val.trim())),
                                    similar::ChangeTag::Equal => display_text.push_str(&val),
                                }
                            }
        
                            let clippy_inner_accept = clippy.clone();
                            let clippy_inner_reject = clippy.clone();
                            let buffer_inner = buffer_clone.clone();
                            
                            let mark_start_accept = mark_start.clone();
                            let mark_end_accept = mark_end.clone();
                            
                            clippy.bubble.show_actions(
                                &display_text,
                                vec![
                                    ("Accept", Box::new(move || {
                                        let mut iter_start = buffer_inner.iter_at_mark(&mark_start_accept);
                                        let mut iter_end = buffer_inner.iter_at_mark(&mark_end_accept);
                                        buffer_inner.delete(&mut iter_start, &mut iter_end);
                                        buffer_inner.insert(&mut iter_start, &new_text);
                                        
                                        clippy_inner_accept.set_state(crate::clippy::sprite::ClippyState::Idle);
                                        clippy_inner_accept.bubble.widget.popdown();
                                    })),
                                    ("Reject", Box::new(move || {
                                        clippy_inner_reject.set_state(crate::clippy::sprite::ClippyState::Idle);
                                        clippy_inner_reject.bubble.widget.popdown();
                                    })),
                                ],
                            );
                            
                            return glib::ControlFlow::Break;
                                }
                                Err(error_msg) => {
                                    clippy.set_state(crate::clippy::sprite::ClippyState::Confused);
                                    let clippy_inner_dismiss = clippy.clone();
                                    clippy.bubble.show_actions(
                                        &error_msg,
                                        vec![
                                            ("Dismiss", Box::new(move || {
                                                clippy_inner_dismiss.set_state(crate::clippy::sprite::ClippyState::Idle);
                                                clippy_inner_dismiss.bubble.widget.popdown();
                                            })),
                                        ]
                                    );
                                    return glib::ControlFlow::Break;
                                }
                            }
                        }
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            clippy.set_state(crate::clippy::sprite::ClippyState::Confused);
                            
                            let clippy_inner_dismiss = clippy.clone();
                            clippy.bubble.show_actions(
                                "Failed to generate text. The AI worker might have crashed.",
                                vec![
                                    ("Dismiss", Box::new(move || {
                                        clippy_inner_dismiss.set_state(crate::clippy::sprite::ClippyState::Idle);
                                        clippy_inner_dismiss.bubble.widget.popdown();
                                    })),
                                ]
                            );
                            return glib::ControlFlow::Break;
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => {
                            return glib::ControlFlow::Continue;
                        }
                    }
                });
            }
        }
    }

    // Fix Grammar Action
    let action_fix_grammar = gio::SimpleAction::new("clippy_fix_grammar", None);
    let tab_view_fg = tab_bar.tab_view.clone();
    let state_fg = state.clone();
    let worker_fg = ai_worker.clone();
    action_fix_grammar.connect_activate(move |_, _| {
        trigger_clippy_action(crate::ai::models::Intent::FixGrammar, &tab_view_fg, &state_fg, &worker_fg);
    });
    window.add_action(&action_fix_grammar);
    app.set_accels_for_action("win.clippy_fix_grammar", &["<Ctrl>g"]);

    // Autocomplete Action
    let action_autocomplete = gio::SimpleAction::new("clippy_autocomplete", None);
    let tab_view_ac = tab_bar.tab_view.clone();
    let state_ac = state.clone();
    let worker_ac = ai_worker.clone();
    action_autocomplete.connect_activate(move |_, _| {
        trigger_clippy_action(crate::ai::models::Intent::Ghostwrite, &tab_view_ac, &state_ac, &worker_ac);
    });
    window.add_action(&action_autocomplete);
    app.set_accels_for_action("win.clippy_autocomplete", &["<Ctrl>space"]);

    // Zoom In
    let action_zoom_in = gio::SimpleAction::new("zoom_in", None);
    let state_clone = state.clone();
    action_zoom_in.connect_activate(move |_, _| {
        let mut s = state_clone.borrow_mut();
        s.zoom_level += 0.1;
        
        let css = format!("textview {{ font-size: {}em; }}", s.zoom_level);
        s.css_provider.load_from_string(&css);
        
        for ts in s.open_tabs.values() {
            if let Some(wv) = ts.current_preview.borrow().as_ref() {
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
            if let Some(wv) = ts.current_preview.borrow().as_ref() {
                wv.set_zoom_level(s.zoom_level);
            }
        }
    });
    app.add_action(&action_zoom_out);
    app.set_accels_for_action("app.zoom_out", &["<Ctrl>minus"]);

    // Toggle Theme
    let action_toggle_theme = gio::SimpleAction::new("toggle_theme", None);
    action_toggle_theme.connect_activate(move |_, _| {
        let manager = libadwaita::StyleManager::default();
        if manager.is_dark() {
            manager.set_color_scheme(libadwaita::ColorScheme::ForceLight);
        } else {
            manager.set_color_scheme(libadwaita::ColorScheme::ForceDark);
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

