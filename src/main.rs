use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::window::{WindowDescriptor, WindowPlugin};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rand::Rng;
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
const RECT_WIDTH: f32 = 100.0;
const RECT_HEIGHT: f32 = 100.0;
const SAMPLE_RATE: u32 = 44100;
const AMPLITUDE: f32 = 28000.0;
const INTERPOLATION_SPEED: f32 = 0.4;
const SCALE_SPEED: f32 = 4.0;
const EPSILON: f32 = 20.0;
const DT: f32 = 0.016;

#[derive(Component)]
struct Node;

#[derive(Component)]
struct OscillatorNode {
    frequency: f32,
    phase: f32,
}

#[derive(Component)]
struct LfoNode {
    frequency: f32,
    phase: f32,
}

#[derive(Component)]
struct OutputNode;

#[derive(Component)]
struct ConnectedTo(Entity);

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Scale {
    scale: f32,
}

#[derive(Component)]
struct PickedUp;

#[derive(Component)]
struct Resetting;

#[derive(Default)]
struct AudioBuffer(Vec<f32>);

impl Resource for AudioBuffer {}

struct AudioStream {
    stream: cpal::Stream,
}

// Remove `Resource` implementation for `AudioStream` as it can't be shared between threads

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Bevy App".to_string(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(InputPlugin)
        .add_plugin(WindowPlugin)
        .add_startup_systems((setup,))
        .add_system(handle_input)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("output.png"), // Replace with actual path
            transform: Transform::from_xyz(200.0, 200.0, 0.0),
            sprite: Sprite {
                custom_size: Some(Vec2::new(RECT_WIDTH, RECT_HEIGHT)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Node)
        .insert(OutputNode)
        .insert(Position { x: 200.0, y: 200.0 })
        .insert(Velocity { x: 0.0, y: 0.0 })
        .insert(Scale { scale: 1.0 });
}

fn node_system(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Res<Windows>,
    mut query: Query<(
        Entity,
        &mut Transform,
        &mut Position,
        &mut Velocity,
        &mut Scale,
        &Node,
    )>,
) {
    let window = windows.primary();
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = window.cursor_position() {
            for (entity, mut transform, mut position, _velocity, _scale, _node) in query.iter_mut()
            {
                if cursor_pos.x >= position.x
                    && cursor_pos.x <= position.x + RECT_WIDTH
                    && cursor_pos.y >= position.y
                    && cursor_pos.y <= position.y + RECT_HEIGHT
                {
                    commands.entity(entity).insert(PickedUp);
                    break;
                }
            }
        }
    }

    if mouse_button_input.just_released(MouseButton::Left) {
        for (entity, _transform, _position, _velocity, _scale, _node) in query.iter_mut() {
            commands
                .entity(entity)
                .remove::<PickedUp>()
                .insert(Resetting);
        }
    }

    if keyboard_input.just_pressed(KeyCode::O) {
        let x = rand::thread_rng().gen_range(0.0..(WINDOW_WIDTH - RECT_WIDTH));
        let y = rand::thread_rng().gen_range(0.0..(WINDOW_HEIGHT - RECT_HEIGHT));
        let frequency = 220.0 + (y / RECT_HEIGHT) * 20.0;
        commands
            .spawn(SpriteBundle {
                texture: asset_server.load("oscillator.png"), // Replace with actual path
                transform: Transform::from_xyz(x, y, 0.0),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(RECT_WIDTH, RECT_HEIGHT)),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Node)
            .insert(OscillatorNode {
                frequency,
                phase: 0.0,
            })
            .insert(Position { x, y })
            .insert(Velocity { x: 0.0, y: 0.0 })
            .insert(Scale { scale: 1.0 });
    }

    if keyboard_input.just_pressed(KeyCode::L) {
        let x = rand::thread_rng().gen_range(0.0..(WINDOW_WIDTH - RECT_WIDTH));
        let y = rand::thread_rng().gen_range(0.0..(WINDOW_HEIGHT - RECT_HEIGHT));
        let frequency = 1.0 + (y / RECT_HEIGHT) * 0.1;
        commands
            .spawn(SpriteBundle {
                texture: asset_server.load("lfo.png"), // Replace with actual path
                transform: Transform::from_xyz(x, y, 0.0),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(RECT_WIDTH, RECT_HEIGHT)),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Node)
            .insert(LfoNode {
                frequency,
                phase: 0.0,
            })
            .insert(Position { x, y })
            .insert(Velocity { x: 0.0, y: 0.0 })
            .insert(Scale { scale: 1.0 });
    }

    for (entity, mut transform, mut position, mut velocity, mut scale, _node) in query.iter_mut() {
        if query.get_component::<PickedUp>(entity).is_ok() {
            if let Some(cursor_pos) = window.cursor_position() {
                position.x = cursor_pos.x - RECT_WIDTH / 2.0;
                position.y = cursor_pos.y - RECT_HEIGHT / 2.0;
                scale.scale = (scale.scale + SCALE_SPEED * DT).min(1.3);
            }
        } else if query.get_component::<Resetting>(entity).is_ok() {
            let target_x = (position.x / RECT_WIDTH).round() * RECT_WIDTH;
            let target_y = (position.y / RECT_HEIGHT).round() * RECT_HEIGHT;
            velocity.x = INTERPOLATION_SPEED * (target_x - position.x);
            velocity.y = INTERPOLATION_SPEED * (target_y - position.y);
            position.x += velocity.x;
            position.y += velocity.y;
            if (velocity.x.abs() < EPSILON && velocity.y.abs() < EPSILON)
                || (position.x - target_x).abs() < EPSILON
                || (position.y - target_y).abs() < EPSILON
            {
                position.x = target_x;
                position.y = target_y;
                velocity.x = 0.0;
                velocity.y = 0.0;
                scale.scale = (scale.scale - SCALE_SPEED * DT).max(1.0);
                if scale.scale <= 1.0 {
                    commands.entity(entity).remove::<Resetting>();
                }
            }
        } else {
            velocity.x *= 0.9;
            velocity.y *= 0.9;
        }
        transform.translation.x = position.x;
        transform.translation.y = position.y;
        transform.scale = Vec3::splat(scale.scale);
    }
}

fn connection_system(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(
        Entity,
        &mut Transform,
        &mut Position,
        &mut Velocity,
        &mut Scale,
        &Node,
    )>,
) {
    if mouse_button_input.just_pressed(MouseButton::Right) {
        let mut selected_entity: Option<Entity> = None;
        for (entity, _transform, _position, _velocity, _scale, _node) in query.iter_mut() {
            if query.get_component::<PickedUp>(entity).is_ok() {
                selected_entity = Some(entity);
                break;
            }
        }

        if let Some(selected_entity) = selected_entity {
            for (entity, _transform, position, _velocity, _scale, _node) in query.iter_mut() {
                if entity != selected_entity {
                    // Implement connection logic here
                    // Example: If entities are close enough, create a connection
                    if (position.x - RECT_WIDTH).abs() < 50.0
                        && (position.y - RECT_HEIGHT).abs() < 50.0
                    {
                        commands.entity(selected_entity).insert(ConnectedTo(entity));
                        break;
                    }
                }
            }
        }
    }
}

fn audio_setup(mut commands: Commands) {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("Failed to get default output device");
    let config = device
        .default_output_config()
        .expect("Failed to get default output config");

    let sample_rate = config.sample_rate().0 as f32;
    let channels = config.channels() as usize;

    let audio_data = Arc::new(Mutex::new(AudioBuffer(vec![0.0; SAMPLE_RATE as usize])));

    let audio_data_clone = audio_data.clone();
    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut audio_data = audio_data_clone.lock().unwrap();
                for frame in data.chunks_mut(channels) {
                    let sample = audio_data.0.remove(0);
                    for sample_out in frame.iter_mut() {
                        *sample_out = sample;
                    }
                }
            },
            |err| eprintln!("An error occurred on the audio stream: {}", err),
            None,
        )
        .expect("Failed to build output stream");

    stream.play().expect("Failed to play the stream");

    commands.insert_resource(AudioStream { stream });
    commands.insert_resource(AudioBuffer(vec![0.0; SAMPLE_RATE as usize]));
}

fn audio_system(
    mut audio_buffer: ResMut<AudioBuffer>,
    query: Query<(
        &Node,
        Option<&OscillatorNode>,
        Option<&LfoNode>,
        Option<&OutputNode>,
        Option<&ConnectedTo>,
    )>,
) {
    let buffer_len = audio_buffer.0.len();
    for i in 0..buffer_len {
        audio_buffer.0[i] = 0.0;
    }

    // Find the output node
    for (_node, oscillator, lfo, output, connection) in query.iter() {
        if let Some(_output) = output {
            // Process audio chain starting from OutputNode
            if let Some(connected_to) = connection {
                let mut sample = 0.0;
                let mut current_node = Some(connected_to.0);
                while let Some(entity) = current_node {
                    let node_data = query.get(entity).unwrap();
                    let oscillator = node_data.1;
                    let lfo = node_data.2;
                    let connection = node_data.4;

                    sample = if let Some(oscillator) = oscillator {
                        oscillator.process(sample)
                    } else if let Some(lfo) = lfo {
                        lfo.process(sample)
                    } else {
                        sample
                    };

                    current_node = connection.map(|c| c.0);
                }

                for i in 0..buffer_len {
                    audio_buffer.0[i] += sample;
                }
            }
        }
    }
}

impl OscillatorNode {
    fn process(&mut self, sample: f32) -> f32 {
        let sample =
            AMPLITUDE * (2.0 * PI * self.frequency * self.phase / SAMPLE_RATE as f32).sin();
        self.phase = (self.phase + self.frequency / SAMPLE_RATE as f32) % 1.0;
        sample
    }
}

impl LfoNode {
    fn process(&mut self, sample: f32) -> f32 {
        self.phase = (self.phase + self.frequency / SAMPLE_RATE as f32) % 1.0;
        let modulation = (2.0 * PI * self.phase).sin();
        sample * (1.0 + modulation * 0.1)
    }
}

fn close_on_esc(keyboard_input: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        let window = windows.get_primary_mut().unwrap();
        window.set_should_close(true);
    }
}
