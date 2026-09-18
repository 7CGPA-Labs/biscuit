#![allow(deprecated)]
use biscuit::ai;

use biscuit::ui;

use gtk::prelude::*;
use libadwaita::prelude::*;
use sourceview5::prelude::*;

fn build_ui(app: &libadwaita::Application) {
    let window = libadwaita::ApplicationWindow::builder()
        .application(app)
        .title("Biscuit")
        .default_width(1024)
        .default_height(768)
        .build();

    let zoom_provider = gtk::CssProvider::new();
    zoom_provider.load_from_string("textview { font-size: 1.0em; }");
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not connect to a display."),
        &zoom_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // App state
    let state = std::rc::Rc::new(std::cell::RefCell::new(ui::actions::AppState {
        open_tabs: std::collections::HashMap::new(),
        css_provider: zoom_provider,
        zoom_level: 1.0,
    }));

    let toolbar_view = libadwaita::ToolbarView::new();

    // Header bar with Hamburger menu
    let header_bar = libadwaita::HeaderBar::new();
    let menu_button = gtk::MenuButton::new();
    menu_button.set_icon_name("open-menu-symbolic");
    let menu_model = ui::menu::create_menu_model();
    menu_button.set_menu_model(Some(&menu_model));
    if let Some(popover) = menu_button.popover() {
        if let Ok(pop) = popover.downcast::<gtk::Popover>() {
            pop.set_has_arrow(false);
            pop.set_offset(-18, 4); // Shift left by half the button width, down by a few pixels
        }
    }
    header_bar.pack_end(&menu_button);
    toolbar_view.add_top_bar(&header_bar);

    // Main container
    let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

    // Tab bar
    let tabs = biscuit::ui::tab_bar::TabBar::new();
    main_vbox.append(&tabs.tab_bar);
    main_vbox.append(&tabs.tab_view);

    // Status bar
    let status_bar = biscuit::ui::status_bar::StatusBar::new();
    let location_label_clone = status_bar.location_label.clone();

    let tab_view_clone = tabs.tab_view.clone();
    let state_clone = state.clone();
    let window_clone_for_tab = window.clone();
    let type_label_clone = status_bar.type_label.clone();

    let create_tab = std::rc::Rc::new(
        move |file_path: Option<std::path::PathBuf>, content: &str| {
            let editor = biscuit::editor::Editor::new();
            let style_manager = sourceview5::StyleSchemeManager::default();
            let libadwaita_manager = libadwaita::StyleManager::default();
            let scheme = if libadwaita_manager.is_dark() {
                style_manager.scheme("Adwaita-dark").or_else(|| style_manager.scheme("oblivion"))
            } else {
                style_manager.scheme("Adwaita").or_else(|| style_manager.scheme("classic"))
            };
            if let Some(scheme) = scheme {
                editor.buffer.set_style_scheme(Some(&scheme));
            }
            editor.buffer.set_text(content);
            editor.buffer.set_modified(false); // Reset modified flag after initial text load

            let overlay = gtk::Overlay::new();
            let scrolled_window = gtk::ScrolledWindow::builder()
                .hexpand(true)
                .vexpand(true)
                .build();
            scrolled_window.set_child(Some(editor.widget()));
            overlay.set_child(Some(&scrolled_window));
            let _clippy = biscuit::clippy::ClippyOverlay::new(&overlay);

            let preview_scrolled = gtk::ScrolledWindow::builder()
                .hexpand(true)
                .vexpand(true)
                .build();

            let preview_rc = std::rc::Rc::new(biscuit::preview::PdfPreview::new());
            preview_scrolled.set_child(Some(preview_rc.get_widget()));

            let _text = editor.buffer.text(
                &editor.buffer.start_iter(),
                &editor.buffer.end_iter(),
                false,
            );
            let is_latex = std::rc::Rc::new(std::cell::RefCell::new(
                file_path.as_ref().map_or(false, |p| p.extension().map_or(false, |ext| ext == "tex"))
            ));
            
            if *is_latex.borrow() {
                type_label_clone.set_label("LaTeX");
            } else {
                type_label_clone.set_label("Markdown");
            }
            // Initial render removed in favor of reload_preview
            
            let preview_container = gtk::Overlay::new();
            let preview = biscuit::preview::PdfPreview::new();
            
            let spinner = gtk::Spinner::builder()
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Center)
                .width_request(48)
                .height_request(48)
                .build();
            let warning_bar = gtk::InfoBar::builder()
                .message_type(gtk::MessageType::Warning)
                .show_close_button(true)
                .valign(gtk::Align::Start)
                .build();
            let warning_label = gtk::Label::new(Some("File changes detected. Click Reload to see the latest changes."));
            warning_bar.add_child(&warning_label);
            warning_bar.set_revealed(false);
            
            preview_container.set_child(Some(preview.get_widget()));
            preview_container.add_overlay(&spinner);
            preview_container.add_overlay(&warning_bar);

            let stack = gtk::Stack::new();
            stack.set_hexpand(true);
            stack.set_vexpand(true);
            stack.add_named(&overlay, Some("editor"));
            stack.add_named(&preview_container, Some("preview"));

            let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 4);
            toolbar.set_margin_top(4);
            toolbar.set_margin_bottom(4);
            toolbar.set_margin_end(6);

            let reload_preview_btn = gtk::Button::builder()
                .icon_name("view-refresh-symbolic")
                .tooltip_text("Reload Preview")
                .valign(gtk::Align::Center)
                .visible(false) // Only visible when preview is active
                .build();
            toolbar.append(&reload_preview_btn);

            let preview_toggle = gtk::ToggleButton::builder()
                .label("Preview")
                .halign(gtk::Align::End)
                .hexpand(true)
                .build();
            toolbar.append(&preview_toggle);

            let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
            vbox.append(&toolbar);
            vbox.append(&stack);

            let page = tab_view_clone.append(&vbox);
            let title = if let Some(path) = &file_path {
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()
            } else {
                "Untitled".to_string()
            };
            page.set_title(&title);

            let base_title = std::rc::Rc::new(std::cell::RefCell::new(title));

            let stack_clone = stack.clone();
            let reload_btn_clone_for_toggle = reload_preview_btn.clone();
            let window_for_dialog = window_clone_for_tab.clone();
            let state_clone_for_toggle = state_clone.clone();
            let page_clone_for_toggle = page.clone();
            let buffer_for_toggle = editor.buffer.clone();
            
            preview_toggle.connect_toggled(move |toggle| {
                if toggle.is_active() {
                    if buffer_for_toggle.is_modified() {
                        let dialog = gtk::AlertDialog::builder()
                            .message("You have unsaved changes. Do you want to save before previewing?")
                            .buttons(["Cancel", "Preview Without Saving", "Save and Preview"])
                            .default_button(2)
                            .cancel_button(0)
                            .build();

                        let toggle_clone_for_dialog = toggle.clone();
                        let stack_clone_for_dialog = stack_clone.clone();
                        let reload_btn_clone_for_dialog = reload_btn_clone_for_toggle.clone();
                        let state_clone_for_dialog = state_clone_for_toggle.clone();
                        let page_clone_for_dialog = page_clone_for_toggle.clone();
                        let window_for_action = window_for_dialog.clone();
                        
                        dialog.choose(Some(&window_for_dialog), gtk::gio::Cancellable::NONE, move |res| {
                            if let Ok(response) = res {
                                if response == 0 {
                                    toggle_clone_for_dialog.set_active(false);
                                } else {
                                    stack_clone_for_dialog.set_visible_child_name("preview");
                                    reload_btn_clone_for_dialog.set_visible(true);
                                    if response == 2 {
                                        let action = window_for_action.application().unwrap().lookup_action("save").unwrap();
                                        action.activate(None);
                                    } else if response == 1 {
                                        let state = state_clone_for_dialog.borrow();
                                        if let Some(ts) = state.open_tabs.get(&page_clone_for_dialog) {
                                            ui::actions::reload_preview(ts, state.zoom_level, false);
                                        }
                                    }
                                }
                            } else {
                                toggle_clone_for_dialog.set_active(false);
                            }
                        });
                    } else {
                        stack_clone.set_visible_child_name("preview");
                        reload_btn_clone_for_toggle.set_visible(true);
                        let state = state_clone_for_toggle.borrow();
                        if let Some(ts) = state.open_tabs.get(&page_clone_for_toggle) {
                            ui::actions::reload_preview(ts, state.zoom_level, false);
                        }
                    }
                } else {
                    stack_clone.set_visible_child_name("editor");
                    reload_btn_clone_for_toggle.set_visible(false);
                }
            });


            // Wire up modified state for asterisk in title
            let page_clone_for_mod = page.clone();
            let base_title_clone = base_title.clone();
            editor.buffer.connect_modified_changed(move |buffer| {
                let t = base_title_clone.borrow();
                if buffer.is_modified() {
                    page_clone_for_mod.set_title(&format!("{} *", t));
                } else {
                    page_clone_for_mod.set_title(&t);
                }
            });

            // Wire up status bar
            let loc_label = location_label_clone.clone();
            editor.buffer.connect_cursor_position_notify(move |buffer| {
                if let Some(mark) = buffer.mark("insert") {
                    let iter = buffer.iter_at_mark(&mark);
                    let line = iter.line() + 1;
                    let col = iter.line_offset() + 1;
                    loc_label.set_label(&format!("Ln {}, Col {}", line, col));
                }
            });

            // Wire up warning bar for stale preview on change
            let warning_bar_clone = warning_bar.clone();
            let preview_toggle_clone_for_warning = preview_toggle.clone();
            let preview_stale = std::rc::Rc::new(std::cell::RefCell::new(false));
            let preview_stale_clone = preview_stale.clone();
            editor.buffer.connect_changed(move |_| {
                *preview_stale_clone.borrow_mut() = true;
                if preview_toggle_clone_for_warning.is_active() {
                    warning_bar_clone.set_revealed(true);
                }
            });

            // Wire up reload button
            let state_clone_for_reload = state_clone.clone();
            let page_clone_for_reload = page.clone();
            reload_preview_btn.connect_clicked(move |_| {
                let state = state_clone_for_reload.borrow();
                if let Some(ts) = state.open_tabs.get(&page_clone_for_reload) {
                    ui::actions::reload_preview(ts, state.zoom_level, true);
                }
            });

            let current_preview = std::rc::Rc::new(std::cell::RefCell::new(None));
            
            // Wire up warning bar close button
            let warning_bar_close_clone = warning_bar.clone();
            warning_bar.connect_response(move |_, response| {
                if response == gtk::ResponseType::Close {
                    warning_bar_close_clone.set_revealed(false);
                }
            });

            state_clone.borrow_mut().open_tabs.insert(
                page.clone(),
                ui::actions::TabState {
                    file_path,
                    buffer: editor.buffer.clone(),
                    base_title,
                    preview_container,
                    current_preview,
                    is_latex,
                    preview_stale,
                    spinner,
                    warning_bar,
                },
            );

            tab_view_clone.set_selected_page(&page);
        },
    );

    // Create the initial empty tab
    create_tab(None, "");

    // Wire up status bar location updates when switching tabs
    let state_clone_for_switch = state.clone();
    let location_label_for_switch = status_bar.location_label.clone();
    let type_label_for_switch = status_bar.type_label.clone();
    tabs.tab_view.connect_selected_page_notify(move |tv| {
        if let Some(page) = tv.selected_page() {
            if let Some(ts) = state_clone_for_switch.borrow().open_tabs.get(&page) {
                if let Some(mark) = ts.buffer.mark("insert") {
                    let iter = ts.buffer.iter_at_mark(&mark);
                    let line = iter.line() + 1;
                    let col = iter.line_offset() + 1;
                    location_label_for_switch.set_label(&format!("Ln {}, Col {}", line, col));
                }
                
                if *ts.is_latex.borrow() {
                    type_label_for_switch.set_label("LaTeX");
                } else {
                    type_label_for_switch.set_label("Markdown");
                }
            }
        }
    });

    main_vbox.append(&status_bar.widget);

    toolbar_view.set_content(Some(&main_vbox));
    window.set_content(Some(&toolbar_view));

    // Actions setup
    ui::actions::setup_actions(&app, &window, state.clone(), &tabs, create_tab.clone(), status_bar.type_label.clone());

    let style_manager = libadwaita::StyleManager::default();
    let state_for_theme = state.clone();
    style_manager.connect_dark_notify(move |manager| {
        let is_dark = manager.is_dark();
        let sv_manager = sourceview5::StyleSchemeManager::default();
        let scheme = if is_dark {
            sv_manager.scheme("Adwaita-dark").or_else(|| sv_manager.scheme("oblivion"))
        } else {
            sv_manager.scheme("Adwaita").or_else(|| sv_manager.scheme("classic"))
        };

        let s = state_for_theme.borrow();
        for ts in s.open_tabs.values() {
            if let Some(scheme) = &scheme {
                ts.buffer.set_style_scheme(Some(scheme));
            }
            if let Some(wv) = ts.current_preview.borrow().as_ref() {
                wv.set_theme(is_dark);
            }
        }
    });

    window.present();

    if ai::downloader::needs_download() {
        let (progress_tx, progress_rx) = std::sync::mpsc::channel();
        let (text_tx, text_rx) = std::sync::mpsc::channel();
        
        std::thread::spawn(move || {
            ai::downloader::check_and_download_models(progress_tx, text_tx);
        });

        let progress_window = gtk::Window::builder()
            .title("Downloading AI Models")
            .transient_for(&window)
            .modal(true)
            .hide_on_close(true)
            .default_width(350)
            .build();
            
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 8);
        vbox.set_margin_top(16);
        vbox.set_margin_bottom(16);
        vbox.set_margin_start(16);
        vbox.set_margin_end(16);
        
        let label = gtk::Label::new(Some("Initializing download..."));
        label.set_wrap(true);
        let progress_bar = gtk::ProgressBar::new();
        progress_bar.set_fraction(0.0);
        
        vbox.append(&label);
        vbox.append(&progress_bar);
        progress_window.set_child(Some(&vbox));
        
        progress_window.present();
        
        let pw_clone = progress_window.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            while let Ok(text) = text_rx.try_recv() {
                label.set_label(&text);
            }
            if let Ok(progress) = progress_rx.try_recv() {
                progress_bar.set_fraction(progress);
                if progress >= 1.0 {
                    pw_clone.close();
                    return glib::ControlFlow::Break;
                }
            }
            glib::ControlFlow::Continue
        });
    }
}

fn main() {
    // Suppress noisy libEGL warnings in containers without hardware acceleration
    std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
    std::env::set_var("GDK_DEBUG", "gl-disable");
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");

    // Disable the WebKitGTK sandbox to prevent bwrap permission errors in restricted environments.
    std::env::set_var("WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS", "1");

    let app = libadwaita::Application::builder()
        .application_id("com.biscuit.App")
        .build();

    app.connect_startup(|_| {
        let _ = libadwaita::init();
        let manager = libadwaita::StyleManager::default();
        manager.set_color_scheme(libadwaita::ColorScheme::PreferDark);

        let css = "
            window.popup {
                background-color: transparent;
                border-radius: 0px;
            }
            popover > arrow {
                background: none;
                border: none;
                min-width: 0;
                min-height: 0;
            }
            popover {
                margin: 0px;
            }
            popover > contents {
                margin: 0px;
                padding: 0px;
                box-shadow: none;
                border-radius: 0px;
                border: 1px solid alpha(currentColor, 0.05);
            }
            headerbar {
                min-height: 20px;
                padding: 0px;
                margin: 0px;
            }
            headerbar windowcontrols {
                min-height: 20px;
            }
            headerbar button {
                min-height: 16px;
                padding: 2px;
                margin: 0px;
            }
            tabbar, tabbar tab {
                min-height: 20px;
                padding-top: 0px;
                padding-bottom: 0px;
                margin-top: 0px;
                margin-bottom: 0px;
            }
            tabbar button {
                min-height: 16px;
                padding: 0px;
                margin: 0px;
            }
        ";
        let provider = gtk::CssProvider::new();
        provider.load_from_string(css);
        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });

    app.connect_activate(build_ui);
    app.run();
}
