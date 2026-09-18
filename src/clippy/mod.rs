pub mod bubble;
pub mod sprite;

use crate::clippy::bubble::ClippyBubble;
use crate::clippy::sprite::{ClippySprite, ClippyState};
use gtk::prelude::*;
use gtk::Overlay;
use std::rc::Rc;

pub struct ClippyOverlay {
    pub sprite: Rc<ClippySprite>,
    pub bubble: Rc<ClippyBubble>,
}

impl ClippyOverlay {
    pub fn new(overlay: &Overlay) -> Self {
        let sprite = Rc::new(ClippySprite::new());
        overlay.add_overlay(&sprite.widget);

        let bubble = Rc::new(ClippyBubble::new(&sprite.widget));

        let motion = gtk::EventControllerMotion::new();
        let cursor_pos = sprite.cursor_pos.clone();
        motion.connect_motion(move |_, x, y| {
            *cursor_pos.borrow_mut() = (x, y);
        });
        overlay.add_controller(motion);

        Self { sprite, bubble }
    }

    pub fn set_state(&self, state: ClippyState) {
        self.sprite.set_state(state);
    }

    pub fn alert(&self, message: &str) {
        self.set_state(ClippyState::Alert);
        self.bubble.show_message(message);
    }

    pub fn think(&self) {
        self.set_state(ClippyState::Thinking);
    }
}
