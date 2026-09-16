use gtk::prelude::*;
fn main() {
    let mb = gtk::MenuButton::new();
    mb.set_menu_model(Some(&gio::Menu::new()));
    if let Some(pop) = mb.popover() {
        if let Ok(p) = pop.downcast::<gtk::Popover>() {
            p.set_has_arrow(false);
        }
    }
}
