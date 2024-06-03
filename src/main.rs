use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;
use std::f64::consts::PI;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    stream: audio::Stream<Audio>,
    is_mouse_pressed: bool,
    rects: Vec<Rectangle>,
    is_updating: bool,
    grid_slots: Vec<Point2>,
    selected_card: Option<usize>, // Index of the selected rectangle
    hand: Vec<Rectangle>,
    chain: Vec<Rectangle>,
}

struct Audio {
    phase: f64,
    hz: f64,
}

#[derive(Clone, Debug)]
struct Rectangle {
    x: f32,
    x_last: f32,
    y: f32,
    y_last: f32,
    w: f32,
    h: f32,
    dragging: bool,
    rotation: f32,
}

fn model(app: &App) -> Model {
    // Create a window to receive key pressed events.
    app.new_window()
        .key_pressed(key_pressed)
        .mouse_pressed(mouse_pressed)
        .mouse_released(mouse_released)
        .view(view)
        .build()
        .unwrap();

    // Initialise the audio API so we can spawn an audio stream.
    let audio_host = audio::Host::new();

    // Initialise the state that we want to live on the audio thread.
    let audio_model = Audio {
        phase: 0.0,
        hz: 440.0,
    };

    let stream = audio_host
        .new_output_stream(audio_model)
        .render(audio)
        .build()
        .unwrap();

    stream.play().unwrap();

    // Define the grid slots
    let mut grid_slots = vec![];
    let grid_size = 100.0;
    let num_slots = 5;
    let win = app.window_rect();

    // Bottom row
    let bottom_y = win.bottom() + grid_size;
    for i in 0..num_slots {
        let x = win.left() + grid_size + i as f32 * grid_size;
        grid_slots.push(pt2(x, bottom_y));
    }

    // Middle row
    let middle_y = win.bottom() + win.h() / 2.0;
    for i in 0..num_slots {
        let x = win.left() + grid_size + i as f32 * grid_size;
        grid_slots.push(pt2(x, middle_y));
    }

    Model {
        stream,
        is_mouse_pressed: false,
        rects: vec![
            Rectangle {
                x: 0.0,
                x_last: 0.0,
                y: 0.0,
                y_last: 0.0,
                w: 100.0,
                h: 100.0,
                dragging: false,
                rotation: 0.0,
            },
            Rectangle {
                x: 100.0,
                x_last: 100.0,
                y: 100.0,
                y_last: 100.0,
                w: 100.0,
                h: 100.0,
                dragging: false,
                rotation: 0.0,
            },
        ],
        is_updating: false,
        grid_slots,
        selected_card: None,
        hand: vec![],
        chain: vec![],
    }
}

// A function that renders the given `Audio` to the given `Buffer`.
// In this case we play a simple sine wave at the audio's current frequency in `hz`.
fn audio(audio: &mut Audio, buffer: &mut Buffer) {
    let sample_rate = buffer.sample_rate() as f64;
    let volume = 0.5;
    for frame in buffer.frames_mut() {
        let sine_amp = (2.0 * PI * audio.phase).sin() as f32;
        audio.phase += audio.hz / sample_rate;
        if audio.phase >= 1.0 {
            audio.phase -= 1.0;
        }
        for channel in frame {
            *channel = sine_amp * volume;
        }
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        // Pause or unpause the audio when Space is pressed.
        Key::Space => {
            if model.stream.is_playing() {
                model.stream.pause().unwrap();
            } else {
                model.stream.play().unwrap();
            }
        }
        // Raise the frequency when the up key is pressed.
        Key::Up => {
            model
                .stream
                .send(|audio| {
                    audio.hz += 10.0;
                })
                .unwrap();
        }
        // Lower the frequency when the down key is pressed.
        Key::Down => {
            model
                .stream
                .send(|audio| {
                    audio.hz -= 10.0;
                })
                .unwrap();
        }
        _ => {}
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(DARKSLATEGRAY);

    let win = app.window_rect();

    let t = app.time;

    // Draw grid slots
    for slot in &model.grid_slots {
        draw.rect()
            .x_y(slot.x, slot.y)
            .w_h(50.0, 50.0)
            .color(GREEN)
            .stroke_weight(1.0)
            .stroke(BLACK);
    }

    for rect in model.rects.iter() {
        draw.rect()
            .x_y(rect.x, rect.y)
            .w_h(rect.w, rect.h)
            .rotate(rect.rotation)
            .color(BLUE);
    }

    // Draw a line!
    draw.line()
        .weight(10.0 + (t.sin() * 0.5 + 0.5) * 90.0)
        .caps_round()
        .color(PALEGOLDENROD)
        .points(win.top_left() * t.sin(), win.bottom_right() * t.cos());

    draw.to_frame(app, &frame).unwrap();
}

fn mouse_pressed(app: &App, model: &mut Model, _button: MouseButton) {
    if model.selected_card.is_none() {
        let x = app.mouse.x;
        let y = app.mouse.y;
        println!("Mouse pressed at x: {}, y: {}", x, y);
        model.is_mouse_pressed = true;
        for (i, rect) in model.rects.iter_mut().enumerate() {
            if x >= rect.x - rect.w / 2.0
                && x <= rect.x + rect.w / 2.0
                && y >= rect.y - rect.h / 2.0
                && y <= rect.y + rect.h / 2.0
            {
                rect.dragging = true;
                model.selected_card = Some(i);
                break;
            }
        }
    }
}

fn mouse_released(app: &App, model: &mut Model, _button: MouseButton) {
    model.is_mouse_pressed = false;
    if let Some(selected) = model.selected_card {
        let rect = &mut model.rects[selected];
        if rect.dragging {
            let (new_x, new_y) = snap_to_grid(rect.x, rect.y, &model.grid_slots);
            rect.x = new_x;
            rect.y = new_y;
            rect.dragging = false;
            model.is_updating = true;
            println!("is_updating: {}", model.is_updating)
        }
        model.selected_card = None;
    }
}

fn handle_drag(app: &App, model: &mut Model) {
    if let Some(selected) = model.selected_card {
        let rect = &mut model.rects[selected];
        let x = app.mouse.x;
        let y = app.mouse.y;
        rect.x_last = rect.x;
        rect.y_last = rect.y;
        if model.is_mouse_pressed && rect.dragging {
            rect.x = x;
            rect.y = y;
        } else {
            rect.x = rect.x_last;
            rect.y = rect.y_last;
        }
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    handle_drag(app, model);
    update_cards(app, model);
    animations(app, model);
}

// Function to snap coordinates to the nearest grid slot
fn snap_to_grid(x: f32, y: f32, grid_slots: &Vec<Point2>) -> (f32, f32) {
    let mut nearest_slot = grid_slots[0];
    let mut min_distance = distance(x, y, nearest_slot.x, nearest_slot.y);

    for &slot in grid_slots.iter() {
        let dist = distance(x, y, slot.x, slot.y);
        if dist < min_distance {
            nearest_slot = slot;
            min_distance = dist;
        }
    }

    (nearest_slot.x, nearest_slot.y)
}

// Function to calculate the distance between two points
fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
}

fn animations(_app: &App, model: &mut Model) {
    for rect in model.rects.iter_mut() {
        rect.rotation += 0.01;
    }
}

fn update_cards(app: &App, model: &mut Model) {
    let win = app.window_rect();
    if model.is_updating {
        model.hand.clear();
        model.chain.clear();
        for rect in model.rects.iter_mut() {
            if rect.y >= win.bottom() + win.h() / 3.0 {
                model.chain.push(rect.clone());
                println!("Chain: {:?}", model.chain);
            } else if rect.y <= win.bottom() + win.h() / 3.0 {
                model.hand.push(rect.clone());
                println!("Hand: {:?}", model.hand);
            }
        }
        model.is_updating = false;
        println!("is_updating: {}", model.is_updating)
    }
}
