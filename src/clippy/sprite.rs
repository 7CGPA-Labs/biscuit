#![allow(deprecated)]
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
    Playful,
}

pub struct ClippySprite {
    pub widget: DrawingArea,
    pub state: Rc<RefCell<ClippyState>>,
    pub cursor_pos: Rc<RefCell<(f64, f64)>>,
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
        
        let cursor_pos = Rc::new(RefCell::new((0.0, 0.0)));
        let cursor_pos_clone = cursor_pos.clone();

        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        widget.set_draw_func(move |area, cr, width, height| {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            let elapsed = current_time - start_time;

            let st = *state_clone.borrow();
            
            // Get clippy's actual allocation coordinates to calculate relative angle to cursor
            let alloc = area.allocation();
            let clippy_x = alloc.x() as f64 + alloc.width() as f64 / 2.0;
            let clippy_y = alloc.y() as f64 + alloc.height() as f64 / 2.0;
            
            let pos = *cursor_pos_clone.borrow();
            draw_clippy(cr, width as f64, height as f64, st, elapsed, pos, clippy_x, clippy_y);
        });

        widget.add_tick_callback(|widget, _| {
            widget.queue_draw();
            glib::ControlFlow::Continue
        });
        
        // Add click gesture for playful animation
        let click_gesture = gtk::GestureClick::new();
        let state_for_click = state.clone();
        click_gesture.connect_pressed(move |_, _, _, _| {
            *state_for_click.borrow_mut() = ClippyState::Playful;
            let st_clone = state_for_click.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(1500), move || {
                // Return to idle after animation if still playful
                if *st_clone.borrow() == ClippyState::Playful {
                    *st_clone.borrow_mut() = ClippyState::Idle;
                }
                glib::ControlFlow::Break
            });
        });
        widget.add_controller(click_gesture);

        Self {
            widget,
            state,
            cursor_pos,
        }
    }

    pub fn set_state(&self, new_state: ClippyState) {
        *self.state.borrow_mut() = new_state;
    }
}

fn draw_clippy(cr: &cairo::Context, width: f64, height: f64, state: ClippyState, time: f64, cursor_pos: (f64, f64), clippy_x: f64, clippy_y: f64) {
    let cx = width / 2.0;
    let cy = height / 2.0;

    // Animate bouncing and bobbing based on state
    let (bounce, scale, rotate) = match state {
        ClippyState::Idle => ((time * 3.0).sin() * 1.5, 1.0, 0.0),
        ClippyState::Thinking => ((time * 8.0).sin() * 2.0, 1.0 + (time * 4.0).sin() * 0.05, 0.0),
        ClippyState::Alert => (0.0, 1.1, (time * 15.0).sin() * 0.1),
        ClippyState::Writing => ((time * 10.0).sin() * 4.0, 1.0, (time * 5.0).sin() * 0.15),
        ClippyState::Confused => (0.0, 1.0, (time * 2.0).sin() * 0.2),
        ClippyState::Playful => {
            // Knocking on glass animation
            let knock_cycle = time % 0.5;
            let knock_scale = 1.0 + (knock_cycle * std::f64::consts::PI).sin() * 0.4;
            let knock_rotate = (knock_cycle * std::f64::consts::PI * 4.0).sin() * 0.2;
            (0.0, knock_scale, knock_rotate)
        }
    };

    let y = cy + bounce;

    cr.save().unwrap();
    cr.translate(cx, y);
    cr.rotate(rotate);
    cr.scale(scale, scale);

    // Paperclip body (silver/grey)
    cr.set_source_rgb(0.7, 0.75, 0.8);
    cr.set_line_width(4.5);
    cr.set_line_join(cairo::LineJoin::Round);
    cr.set_line_cap(cairo::LineCap::Round);

    // Approximate paperclip shape
    cr.move_to(-8.0, 10.0);
    cr.line_to(-8.0, -10.0);
    cr.arc(0.0, -10.0, 8.0, std::f64::consts::PI, 0.0); // Outer top arc
    cr.line_to(8.0, 15.0);
    cr.arc(2.0, 15.0, 6.0, 0.0, std::f64::consts::PI); // Outer bottom arc
    cr.line_to(-4.0, -6.0);
    cr.arc(0.0, -6.0, 4.0, std::f64::consts::PI, 0.0); // Inner top arc
    cr.line_to(4.0, 10.0);
    cr.stroke().unwrap();

    // Blink animation
    let is_blinking = (time % 4.0) > 3.8;
    
    if is_blinking && state != ClippyState::Playful {
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_line_width(2.0);
        cr.move_to(-10.0, -6.0);
        cr.line_to(-4.0, -6.0);
        cr.move_to(4.0, -6.0);
        cr.line_to(10.0, -6.0);
        cr.stroke().unwrap();
    } else {
        // Eyes (white)
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.arc(-5.0, -8.0, 5.0, 0.0, 2.0 * std::f64::consts::PI);
        cr.arc(5.0, -8.0, 5.0, 0.0, 2.0 * std::f64::consts::PI);
        cr.fill_preserve().unwrap();
        
        // Glasses frames
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_line_width(1.5);
        cr.stroke().unwrap();

        // Glasses bridge
        cr.move_to(-10.0, -8.0);
        cr.line_to(10.0, -8.0);
        cr.stroke().unwrap();

        // Pupils (black)
        let mut pupil_x = 0.0;
        let mut pupil_y = 0.0;

        // Occasional glance logic or playful staring
        let glance_cycle = time % 5.0; // Every 5 seconds
        let is_glancing = glance_cycle > 3.0 && glance_cycle < 4.0;

        if is_glancing || state == ClippyState::Playful {
            let dx = cursor_pos.0 - clippy_x;
            let dy = cursor_pos.1 - clippy_y;
            let distance = (dx*dx + dy*dy).sqrt();
            if distance > 5.0 {
                let max_offset = if state == ClippyState::Playful { 3.0 } else { 2.5 };
                pupil_x = (dx / distance) * max_offset;
                pupil_y = (dy / distance) * max_offset;
            }
        } else {
            // Look straight or idle thinking
            pupil_x = match state {
                ClippyState::Thinking => (time * 2.0).sin() * 2.0,
                _ => 0.0,
            };
            pupil_y = match state {
                ClippyState::Thinking => -2.0,
                _ => 0.0,
            };
        }

        // Draw pupils
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.arc(-5.0 + pupil_x, -8.0 + pupil_y, 2.0, 0.0, 2.0 * std::f64::consts::PI);
        cr.arc(5.0 + pupil_x, -8.0 + pupil_y, 2.0, 0.0, 2.0 * std::f64::consts::PI);
        cr.fill().unwrap();
    }

    cr.restore().unwrap();

    // State-specific accessories
    cr.save().unwrap();
    cr.translate(cx, y);
    match state {
        ClippyState::Thinking => {
            cr.set_source_rgb(1.0, 0.8, 0.0);
            cr.arc(15.0, -25.0, 6.0, 0.0, 2.0 * std::f64::consts::PI); // Lightbulb
            cr.fill().unwrap();
        }
        ClippyState::Alert => {
            cr.set_source_rgb(1.0, 0.2, 0.2);
            cr.move_to(15.0, -35.0);
            cr.line_to(5.0, -20.0);
            cr.line_to(25.0, -20.0);
            cr.close_path();
            cr.fill().unwrap();
            
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.arc(15.0, -23.0, 1.0, 0.0, 2.0 * std::f64::consts::PI);
            cr.fill().unwrap();
            cr.move_to(15.0, -31.0);
            cr.line_to(15.0, -26.0);
            cr.set_line_width(2.0);
            cr.stroke().unwrap();
        }
        ClippyState::Writing => {
            cr.set_source_rgb(0.2, 0.6, 1.0);
            cr.rotate((time * 10.0).sin() * 0.2);
            cr.rectangle(12.0, -10.0, 5.0, 20.0);
            cr.fill().unwrap();
            cr.set_source_rgb(0.8, 0.8, 0.8);
            cr.move_to(12.0, 10.0);
            cr.line_to(17.0, 10.0);
            cr.line_to(14.5, 16.0);
            cr.close_path();
            cr.fill().unwrap();
        }
        ClippyState::Confused => {
            cr.set_source_rgb(0.6, 0.2, 0.8);
            cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
            cr.set_font_size(24.0);
            cr.move_to(15.0, -20.0);
            cr.show_text("?").unwrap();
        }
        _ => {}
    }
    cr.restore().unwrap();
}
