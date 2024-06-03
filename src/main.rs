use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;
use std::f64::consts::PI;

fn main() {
    nannou::app(model).run();
}

struct Model {
    stream: audio::Stream<Audio>,
}

struct Audio {
    phase: f64,
    hz: f64,
}

fn model(app: &App) -> Model {
    // Create a window to receive key pressed events.
    app.new_window()
        .key_pressed(key_pressed)
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

    Model { stream }
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
    let draw = app.draw();

    draw.background().color(DARKSLATEGRAY);

    let win = app.window_rect();

    // Draw an ellipse to follow the mouse.
    let t = app.time;
    draw.ellipse()
        .x_y(app.mouse.x, app.mouse.y)
        .radius(50.0)
        .color(RED);

    // Draw a line!
    draw.line()
        .weight(10.0 + (t.sin() * 0.5 + 0.5) * 90.0)
        .caps_round()
        .color(PALEGOLDENROD)
        .points(win.top_left() * t.sin(), win.bottom_right() * t.cos());

    for r in win.subdivisions_iter() {
        for r in r.subdivisions_iter() {
            for r in r.subdivisions_iter() {
                let side = r.w().min(r.h());
                let start = r.xy();
                let start_to_mouse = app.mouse.position() - start;
                let target_mag = start_to_mouse.length().min(side * 0.5);
                let end = start + start_to_mouse.normalize_or_zero() * target_mag;
                draw.arrow().weight(5.0).points(start, end);
            }
        }
    }

    draw.to_frame(app, &frame).unwrap();
}
