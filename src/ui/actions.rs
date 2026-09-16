use crate::ui::tab_bar::TabBar;
use gtk::gio;
use gtk::prelude::*;
use gtk::{FileDialog, Window};
use libadwaita::ApplicationWindow;
use sourceview5::Buffer;
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
    pub web_view: Rc<crate::preview::WebView>,
}

pub struct AppState {
    pub open_tabs: HashMap<TabPage, TabState>,
}

pub fn setup_actions(
    app: &libadwaita::Application,
    window: &ApplicationWindow,
    state: Rc<RefCell<AppState>>,
    tab_bar: &TabBar,
    create_tab: Rc<dyn Fn(Option<PathBuf>, &str)>,
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
    let window_clone = window.clone();
    action_about.connect_activate(move |_, _| {
        let about = libadwaita::AboutWindow::builder()
            .application_name("Biscuit")
            .developer_name("Developer")
            .version("0.1.0")
            .build();
        about.present();
    });
    app.add_action(&action_about);

    // Quit Action
    let action_quit = gio::SimpleAction::new("quit", None);
    let app_clone = app.clone();
    action_quit.connect_activate(move |_, _| {
        app_clone.quit();
    });
    app.add_action(&action_quit);
    app.set_accels_for_action("app.quit", &["<Ctrl>q"]);
}
