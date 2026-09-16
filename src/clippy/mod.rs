pub mod sprite;
pub mod bubble;

use gtk::prelude::*;
use gtk::Overlay;
use std::rc::Rc;
use crate::clippy::sprite::{ClippySprite, ClippyState};
use crate::clippy::bubble::ClippyBubble;

pub struct ClippyOverlay {
    pub sprite: Rc<ClippySprite>,
    pub bubble: Rc<ClippyBubble>,
}

impl ClippyOverlay {
    pub fn new(overlay: &Overlay) -> Self {
        let sprite = Rc::new(ClippySprite::new());
        overlay.add_overlay(&sprite.widget);
        
        let bubble = Rc::new(ClippyBubble::new(&sprite.widget));
        
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
