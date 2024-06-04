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
    cards: Vec<Card>,
    is_updating: bool,
    grid_slots: Vec<Point2>,
    selected_card: Option<usize>, // Index of the selected Card
    hand: Vec<Card>,
    chain: Vec<Card>,
    bpm: f32,
    last_update: f32,
    beat_time: f32,
}

struct Audio {
    phase: f64,
    hz: f64,
    playing: bool,
    envelope: f32,
    beat_time: f32,
}

#[derive(Clone, Debug)]
struct Oscillator {}

#[derive(Clone, Debug)]
struct Sequencer {
    sequence: Vec<f32>,
    step: usize,
}

impl Sequencer {
    fn next_value(&mut self) -> f32 {
        let value = self.sequence[self.step];
        self.step = (self.step + 1) % self.sequence.len();
        value
    }
}

#[derive(Clone, Debug)]
struct Envelope {}

#[derive(Clone, Debug)]
enum CardClass {
    Oscillator(Oscillator),
    Sequencer(Sequencer),
    Envelope(Envelope),
    // Add more variants here as needed
}

#[derive(Clone, Debug)]
struct Card {
    x: f32,
    x_last: f32,
    x_targ: f32,
    y: f32,
    y_last: f32,
    y_targ: f32,
    w: f32,
    h: f32,
    dragging: bool,
    rotation: f32,
    scale: f32,
    start_time: f32,
    class: CardClass,
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
        playing: false,
        envelope: 0.0, // Initialize to 0
        beat_time: 0.0,
    };

    let stream = audio_host
        .new_output_stream(audio_model)
        .render(audio)
        .build()
        .unwrap();

    stream.play().unwrap();

    // Define the grid slots
    let mut grid_slots = vec![];
    let grid_size = 110.0;
    let num_slots = 5;
    let win = app.window_rect();

    // Middle row
    let middle_y = win.bottom() + win.h() / 2.0;
    for i in 0..num_slots {
        let x = win.left() + 2.6 * grid_size + i as f32 * grid_size;
        grid_slots.push(pt2(x, middle_y));
    }

    // Bottom row
    let bottom_y = win.bottom() + grid_size;
    for i in 0..num_slots {
        let x = win.left() + 2.6 * grid_size + i as f32 * grid_size;
        grid_slots.push(pt2(x, bottom_y));
    }

    Model {
        stream,
        is_mouse_pressed: false,
        cards: vec![
            Card {
                x: 0.0,
                x_last: 0.0,
                x_targ: 0.0,
                y: 0.0,
                y_last: 0.0,
                y_targ: 0.0,
                w: 100.0,
                h: 140.0,
                dragging: false,
                rotation: 0.0,
                scale: 1.0,
                start_time: 0.0,
                class: CardClass::Oscillator(Oscillator {}),
            },
            Card {
                x: 100.0,
                x_last: 100.0,
                x_targ: 100.0,
                y: 100.0,
                y_last: 100.0,
                y_targ: 100.0,
                w: 100.0,
                h: 140.0,
                dragging: false,
                rotation: 0.0,
                scale: 1.0,
                start_time: 0.0,
                class: CardClass::Sequencer(Sequencer {
                    sequence: vec![0.8, 1.0, 1.2, 1.0],
                    step: 0,
                }),
            },
            Card {
                x: 100.0,
                x_last: 100.0,
                x_targ: 100.0,
                y: 100.0,
                y_last: 100.0,
                y_targ: 100.0,
                w: 100.0,
                h: 140.0,
                dragging: false,
                rotation: 0.0,
                scale: 1.0,
                start_time: 0.0,
                class: CardClass::Envelope(Envelope {}),
            },
        ],
        is_updating: false,
        grid_slots,
        selected_card: None,
        hand: vec![],
        chain: vec![],
        bpm: 120.0,
        last_update: 0.0,
        beat_time: 0.0,
    }
}

fn audio(audio: &mut Audio, buffer: &mut Buffer) {
    let sample_rate = buffer.sample_rate() as f64;
    let max_volume = 0.5;
    let volume = if audio.playing {
        max_volume * audio.envelope.min(1.0) // Ensure envelope doesn't exceed 1.0
    } else {
        0.0
    };

    for frame in buffer.frames_mut() {
        let sine_amp = (2.0 * PI * audio.phase).sin() as f32;
        audio.phase += audio.hz / sample_rate;
        if audio.phase >= 1.0 {
            audio.phase -= 1.0;
        }
        for channel in frame {
            *channel = sine_amp * volume as f32;
        }
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Space => {
            if model.stream.is_playing() {
                model.stream.pause().unwrap();
            } else {
                model.stream.play().unwrap();
            }
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
            .w_h(110.0, 150.0)
            .color(Rgba::new(1.0, 1.0, 1.0, 0.2))
            .stroke_weight(0.4);
        draw.rect()
            .x_y(slot.x, slot.y)
            .w_h(100.0, 140.0)
            .color(Rgba::new(1.0, 1.0, 1.0, 0.2))
            .stroke_weight(0.2)
            .stroke(BLACK);
    }

    for card in model.cards.iter() {
        draw.rect()
            .x_y(card.x, card.y)
            .w_h(card.w * card.scale, card.h * card.scale)
            .rotate(card.rotation)
            .color(BLUE);

        if let CardClass::Sequencer(_) = card.class {
            draw.text("S")
                .x_y(card.x, card.y)
                .color(WHITE)
                .font_size(32);
        }
        if let CardClass::Oscillator(_) = card.class {
            draw.text("O")
                .x_y(card.x, card.y)
                .color(WHITE)
                .font_size(32);
        }
        if let CardClass::Envelope(_) = card.class {
            draw.text("E")
                .x_y(card.x, card.y)
                .color(WHITE)
                .font_size(32);
        }
    }

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
        for (i, card) in model.cards.iter_mut().enumerate() {
            if x >= card.x - card.w / 2.0
                && x <= card.x + card.w / 2.0
                && y >= card.y - card.h / 2.0
                && y <= card.y + card.h / 2.0
            {
                card.dragging = true;
                model.selected_card = Some(i);
                card.start_time = app.time;
                break;
            }
        }
    }
}

fn mouse_released(_app: &App, model: &mut Model, _button: MouseButton) {
    model.is_mouse_pressed = false;
    if let Some(selected) = model.selected_card {
        let card = &mut model.cards[selected];
        if card.dragging {
            let (new_x, new_y) = snap_to_grid(card.x_targ, card.y_targ, &model.grid_slots);
            card.x_targ = new_x;
            card.y_targ = new_y;
            card.dragging = false;
            model.is_updating = true;
            println!("is_updating: {}", model.is_updating)
        }
        model.selected_card = None;
    }
}

fn handle_drag(app: &App, model: &mut Model) {
    if let Some(selected) = model.selected_card {
        let card = &mut model.cards[selected];
        let x = app.mouse.x;
        let y = app.mouse.y;
        card.x_last = card.x_targ;
        card.y_last = card.y_targ;
        if model.is_mouse_pressed && card.dragging {
            card.x_targ = x;
            card.y_targ = y;
        } else {
            card.x_targ = card.x_last;
            card.y_targ = card.y_last;
        }
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let now = app.time;
    let time_since_last_update = now - model.last_update;
    let beat_duration = 60.0 / model.bpm;

    model.beat_time += time_since_last_update as f32;

    if model.beat_time >= beat_duration {
        model.beat_time = 0.0;
    }

    model.last_update = now;
    handle_drag(app, model);
    update_cards(app, model);
    animations(app, model);
    lerp(model);
    update_sound(app, model)
}

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

fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
}

fn animations(app: &App, model: &mut Model) {
    let decay_rate = 3.0;
    let wobble_amplitude = 0.4;
    let wobble_speed = 1.0;
    let frequency = 20.0;
    let lerp_rate = 0.4;

    for (i, card) in model.cards.iter_mut().enumerate() {
        let t = app.time - card.start_time;
        card.rotation += (t * frequency * wobble_speed).sin()
            * wobble_amplitude
            * (-decay_rate * t * wobble_speed).exp();
        let target_rotation = 0.004 * (card.x_targ - card.x);
        card.rotation = card.rotation * (1.0 - lerp_rate) + target_rotation * lerp_rate;

        if Some(i) == model.selected_card {
            if card.scale < 1.3 {
                card.scale += 0.04;
            }
        } else {
            if card.scale > 1.0 {
                let target_scale = 1.0;
                card.scale = card.scale * (1.0 - lerp_rate) + target_scale * lerp_rate;
            }
        }
    }
}

fn update_cards(app: &App, model: &mut Model) {
    let win = app.window_rect();
    if model.is_updating {
        model.hand.clear();
        model.chain.clear();
        for card in model.cards.iter_mut() {
            if card.y >= win.bottom() + win.h() / 3.0 {
                model.chain.push(card.clone());
                println!("Chain: {:?}", model.chain);
            } else if card.y <= win.bottom() + win.h() / 3.0 {
                model.hand.push(card.clone());
                println!("Hand: {:?}", model.hand);
            }
        }
        model.is_updating = false;
        println!("is_updating: {}", model.is_updating)
    }
}

fn lerp(model: &mut Model) {
    for card in model.cards.iter_mut() {
        card.x += (card.x_targ - card.x) * 0.3;
        card.y += (card.y_targ - card.y) * 0.3;
    }
}

fn update_sound(app: &App, model: &mut Model) {
    let hz_increment = 1.0 * (app.time as f64).sin();

    let bpm = 120.0;
    let beat_duration = 60.0 / bpm; // Duration of one beat in seconds

    let sequencer_index = model
        .chain
        .iter()
        .position(|card| matches!(card.class, CardClass::Sequencer(_)));

    let oscillator_index = model
        .chain
        .iter()
        .position(|card| matches!(card.class, CardClass::Oscillator(_)));

    let envelope_index = model
        .chain
        .iter()
        .position(|card| matches!(card.class, CardClass::Envelope(_)));

    if let Some(index) = oscillator_index {
        if let Some(CardClass::Oscillator(_osc)) =
            model.chain.get_mut(index).map(|card| &mut card.class)
        {
            model
                .stream
                .send(move |audio| {
                    audio.playing = true;
                })
                .unwrap();
        }
    } else {
        model
            .stream
            .send(move |audio| {
                audio.playing = false;
            })
            .unwrap();
    }

    if let Some(index) = sequencer_index {
        if let Some(CardClass::Sequencer(seq)) =
            model.chain.get_mut(index).map(|card| &mut card.class)
        {
            if model.beat_time == 0.0 {
                let next_value = seq.next_value();
                let new_hz = next_value as f64;

                let base_hz = 440.0;

                model
                    .stream
                    .send(move |audio| {
                        audio.hz = base_hz * new_hz;
                    })
                    .unwrap();
            }
        }
    } else {
        model
            .stream
            .send(move |audio| {
                audio.hz += hz_increment;
            })
            .unwrap();
    }

    if let Some(index) = envelope_index {
        if let Some(CardClass::Envelope(_env)) =
            model.chain.get_mut(index).map(|card| &mut card.class)
        {
            let envelope = if model.beat_time < beat_duration {
                model.beat_time / (beat_duration / 2.0) // Increasing part of the triangle
            } else {
                1.0 - ((model.beat_time - beat_duration / 2.0) / (beat_duration / 2.0))
                // Decreasing part of the triangle
            };

            model
                .stream
                .send(move |audio| audio.envelope = envelope)
                .unwrap();
        }
    } else {
        model
            .stream
            .send(move |audio| {
                audio.envelope = 1.0;
            })
            .unwrap();
    }
}
