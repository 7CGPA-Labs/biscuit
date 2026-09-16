use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, Orientation};

pub struct TabBar {
    pub tab_bar: libadwaita::TabBar,
    pub tab_view: libadwaita::TabView,
}

impl TabBar {
    pub fn new() -> Self {
        let tab_view = libadwaita::TabView::new();
        tab_view.set_hexpand(true);
        tab_view.set_vexpand(true);

        let tab_bar = libadwaita::TabBar::new();
        tab_bar.set_view(Some(&tab_view));
        tab_bar.set_autohide(false); // Ensure the tab bar is always visible even with one tab

        let nav_box = GtkBox::new(Orientation::Horizontal, 4);
        nav_box.set_margin_top(0);
        nav_box.set_margin_bottom(0);
        nav_box.set_margin_end(4);

        let btn_prev = Button::from_icon_name("go-previous-symbolic");
        btn_prev.add_css_class("flat");
        let btn_next = Button::from_icon_name("go-next-symbolic");
        btn_next.add_css_class("flat");

        nav_box.append(&btn_prev);
        nav_box.append(&btn_next);

        let tv_prev = tab_view.clone();
        btn_prev.connect_clicked(move |_| {
            tv_prev.select_previous_page();
        });

        let tv_next = tab_view.clone();
        btn_next.connect_clicked(move |_| {
            tv_next.select_next_page();
        });

        tab_bar.set_end_action_widget(Some(&nav_box));

        Self { tab_bar, tab_view }
    }
}
