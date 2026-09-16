use gtk::gio;
use gtk::prelude::*;

pub fn create_menu_model() -> gio::Menu {
    let menu = gio::Menu::new();

    // File section
    let file_section = gio::Menu::new();
    file_section.append(Some("New File"), Some("app.new"));
    file_section.append(Some("Open..."), Some("app.open"));
    file_section.append(Some("Save"), Some("app.save"));
    file_section.append(Some("Save As..."), Some("app.save_as"));
    menu.append_section(None, &file_section);

    // Export section
    let export_section = gio::Menu::new();
    let export_submenu = gio::Menu::new();
    export_submenu.append(Some("Export to PDF"), Some("app.export_pdf"));
    export_submenu.append(Some("Export to Docx"), Some("app.export_docx"));
    export_section.append_submenu(Some("Export"), &export_submenu);
    menu.append_section(None, &export_section);

    // Preferences & About section
    let prefs_section = gio::Menu::new();
    prefs_section.append(Some("Preferences"), Some("app.preferences"));
    prefs_section.append(Some("Keyboard Shortcuts"), Some("app.shortcuts"));
    prefs_section.append(Some("About Biscuit"), Some("app.about"));
    menu.append_section(None, &prefs_section);

    // Quit section
    let quit_section = gio::Menu::new();
    quit_section.append(Some("Quit"), Some("app.quit"));
    menu.append_section(None, &quit_section);

    menu
}
