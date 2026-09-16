use biscuit::ai;
use biscuit::clippy;
use biscuit::editor;
use biscuit::export;
use biscuit::linter;

use biscuit::ui;

use gtk::prelude::*;
use libadwaita::prelude::*;
use gtk::Application;

fn build_ui(app: &libadwaita::Application) {
    let window = libadwaita::ApplicationWindow::builder()
        .application(app)
        .title("Biscuit")
        .default_width(1024)
        .default_height(768)
        .build();

    // App state
    let state = std::rc::Rc::new(std::cell::RefCell::new(ui::actions::AppState {
        open_tabs: std::collections::HashMap::new(),
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
    
    let create_tab = std::rc::Rc::new(move |file_path: Option<std::path::PathBuf>, content: &str| {
        let editor = biscuit::editor::Editor::new();
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
        let preview_label = gtk::Label::new(Some("Markdown / LaTeX Preview Area"));
        preview_scrolled.set_child(Some(&preview_label));
        
        let stack = gtk::Stack::new();
        stack.set_hexpand(true);
        stack.set_vexpand(true);
        stack.add_named(&overlay, Some("editor"));
        stack.add_named(&preview_scrolled, Some("preview"));
        
        let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        toolbar.set_margin_top(4);
        toolbar.set_margin_bottom(4);
        toolbar.set_margin_end(6);
        let preview_toggle = gtk::ToggleButton::builder().label("Preview").halign(gtk::Align::End).hexpand(true).build();
        toolbar.append(&preview_toggle);
        
        let stack_clone = stack.clone();
        preview_toggle.connect_toggled(move |toggle| {
            if toggle.is_active() {
                stack_clone.set_visible_child_name("preview");
            } else {
                stack_clone.set_visible_child_name("editor");
            }
        });
        
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.append(&toolbar);
        vbox.append(&stack);
        
        let page = tab_view_clone.append(&vbox);
        let title = if let Some(path) = &file_path {
            path.file_name().unwrap_or_default().to_string_lossy().into_owned()
        } else {
            "Untitled".to_string()
        };
        page.set_title(&title);
        
        let base_title = std::rc::Rc::new(std::cell::RefCell::new(title));
        
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
        
        state_clone.borrow_mut().open_tabs.insert(page.clone(), ui::actions::TabState {
            file_path,
            buffer: editor.buffer.clone(),
            base_title,
        });
        
        tab_view_clone.set_selected_page(&page);
    });

    // Create the initial empty tab
    create_tab(None, "");
    
    // Wire up status bar location updates when switching tabs
    let state_clone_for_switch = state.clone();
    let location_label_for_switch = status_bar.location_label.clone();
    tabs.tab_view.connect_selected_page_notify(move |tv| {
        if let Some(page) = tv.selected_page() {
            if let Some(ts) = state_clone_for_switch.borrow().open_tabs.get(&page) {
                if let Some(mark) = ts.buffer.mark("insert") {
                    let iter = ts.buffer.iter_at_mark(&mark);
                    let line = iter.line() + 1;
                    let col = iter.line_offset() + 1;
                    location_label_for_switch.set_label(&format!("Ln {}, Col {}", line, col));
                }
            }
        }
    });

    main_vbox.append(&status_bar.widget);
    
    toolbar_view.set_content(Some(&main_vbox));
    window.set_content(Some(&toolbar_view));
    
    // Actions setup
    ui::actions::setup_actions(&app, &window, state.clone(), &tabs, create_tab.clone());
    
    window.present();
}

fn main() {
    // Check and download models on first run
    std::thread::spawn(|| {
        ai::downloader::check_and_download_models();
    });

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
        provider.load_from_data(css);
        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });
    
    app.connect_activate(build_ui);
    app.run();
}
