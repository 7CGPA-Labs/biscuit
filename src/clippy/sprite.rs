use gtk::cairo;
use gtk::prelude::*;
use gtk::DrawingArea;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, PartialEq)]
pub enum ClippyState {
    Idle,
    Thinking,
    Alert,
    Writing,
    Confused,
}

pub struct ClippySprite {
    pub widget: DrawingArea,
    pub state: Rc<RefCell<ClippyState>>,
    start_time: f64,
}

impl ClippySprite {
    pub fn new() -> Self {
        let widget = DrawingArea::new();
        widget.set_size_request(64, 64);
        widget.set_halign(gtk::Align::End);
        widget.set_valign(gtk::Align::End);
        widget.set_margin_end(20);
        widget.set_margin_bottom(20);

        let state = Rc::new(RefCell::new(ClippyState::Idle));
        let state_clone = state.clone();

        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        widget.set_draw_func(move |_, cr, width, height| {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            let elapsed = current_time - start_time;

            let st = *state_clone.borrow();
            draw_clippy(cr, width as f64, height as f64, st, elapsed);
        });

        widget.add_tick_callback(|widget, _| {
            widget.queue_draw();
            glib::ControlFlow::Continue
        });

        Self {
            widget,
            state,
            start_time,
        }
    }

    pub fn set_state(&self, new_state: ClippyState) {
        *self.state.borrow_mut() = new_state;
    }
}

fn draw_clippy(cr: &cairo::Context, width: f64, height: f64, state: ClippyState, time: f64) {
    let cx = width / 2.0;
    let cy = height / 2.0;

    // Animate bouncing
    let bounce = (time * 5.0).sin() * 5.0;
    let y = cy
        + if state == ClippyState::Idle {
            bounce
        } else {
            0.0
        };

    cr.set_source_rgb(0.8, 0.8, 0.8);
    cr.arc(cx, y, 20.0, 0.0, 2.0 * std::f64::consts::PI);
    cr.fill().unwrap();

    // Draw eyes
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.arc(cx - 8.0, y - 5.0, 3.0, 0.0, 2.0 * std::f64::consts::PI);
    cr.arc(cx + 8.0, y - 5.0, 3.0, 0.0, 2.0 * std::f64::consts::PI);
    cr.fill().unwrap();

    // State-specific accessories
    match state {
        ClippyState::Thinking => {
            cr.set_source_rgb(1.0, 0.8, 0.0);
            cr.arc(cx, y - 25.0, 5.0, 0.0, 2.0 * std::f64::consts::PI); // Lightbulb
            cr.fill().unwrap();
        }
        ClippyState::Alert => {
            cr.set_source_rgb(1.0, 0.0, 0.0);
            cr.move_to(cx - 10.0, y - 25.0);
            cr.line_to(cx + 10.0, y - 25.0);
            cr.line_to(cx, y - 35.0); // Exclamation triangle
            cr.fill().unwrap();
        }
        ClippyState::Writing => {
            cr.set_source_rgb(0.0, 0.5, 1.0);
            let pen_x = cx + 15.0 + (time * 10.0).sin() * 5.0;
            cr.rectangle(pen_x, y + 10.0, 4.0, 15.0);
            cr.fill().unwrap();
        }
        ClippyState::Confused => {
            cr.set_source_rgb(0.5, 0.0, 0.5);
            cr.move_to(cx, y - 30.0);
            cr.show_text("?").unwrap();
        }
        _ => {}
    }
}
