use gtk::prelude::*;
fn main() {
    let tab_view = libadwaita::TabView::new();
    let tab_bar = libadwaita::TabBar::new();
    tab_bar.set_view(Some(&tab_view));
    
    let label = gtk::Label::new(Some("Content"));
    let page = tab_view.append(&label);
    page.set_title("draft.md");
    
    let toggle = gtk::ToggleButton::with_label("Preview");
    tab_bar.add_end_action_widget(&toggle);
}
