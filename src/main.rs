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
    rect: Rectangle,
}

struct Audio {
    phase: f64,
    hz: f64,
}

struct Rectangle {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
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
    let model = Audio {
        phase: 0.0,
        hz: 440.0,
    };

    let stream = audio_host
        .new_output_stream(model)
        .render(audio)
        .build()
        .unwrap();

    stream.play().unwrap();

    Model {
        stream,
        is_mouse_pressed: false,
        rect: Rectangle {
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 100.0,
        },
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
        audio.phase %= sample_rate;
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
    let mut rect = &model.rect;

    let draw = app.draw();

    draw.background().color(DARKSLATEGRAY);

    let win = app.window_rect();

    // Draw an ellipse to follow the mouse.
    let t = app.time;

    draw.rect()
        .x_y(rect.x, rect.y)
        .w_h(rect.w, rect.h)
        .color(BLUE);

    // Draw a line!
    draw.line()
        .weight(10.0 + (t.sin() * 0.5 + 0.5) * 90.0)
        .caps_round()
        .color(PALEGOLDENROD)
        .points(win.top_left() * t.sin(), win.bottom_right() * t.cos());

    draw.to_frame(app, &frame).unwrap();
}

fn mouse_pressed(app: &App, model: &mut Model, _button: MouseButton) {
    let x = app.mouse.x;
    let y = app.mouse.y;
    println!("Mouse pressed at x: {}, y: {}", x, y);
    if x >= model.rect.x - model.rect.w / 2.0
        && x <= model.rect.x + model.rect.w / 2.0
        && y >= model.rect.y - model.rect.h / 2.0
        && y <= model.rect.y + model.rect.h / 2.0
    {
        model.is_mouse_pressed = true;
    }
}

fn mouse_released(_app: &App, model: &mut Model, _button: MouseButton) {
    model.is_mouse_pressed = false;
}

fn handle_drag(app: &App, model: &mut Model) {
    let x = app.mouse.x;
    let y = app.mouse.y;
    let x_last = model.rect.x;
    let y_last = model.rect.y;
    if model.is_mouse_pressed {
        model.rect.x = x;
        model.rect.y = y;
    } else {
        model.rect.x = x_last;
        model.rect.y = y_last;
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    handle_drag(_app, model);
}
