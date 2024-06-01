use bevy::audio::{AddAudioSource, AudioPlugin, Source};
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::utils::Duration;
use bevy::window::PrimaryWindow;
use std::time::Instant;

// structure:
// leftmost card is the oscillator, all other cards are modifiers
// different oscillators, different fms, different envelopes
// different sequences, different sequence modifiers
// global modifier cards for things like bpm

#[derive(Component, Debug)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component, Debug)]
struct NodeResources {
    target: Vec2,
    current: Vec2,
    interpolation_angle: f32,
    wobble_angle: f32,
    scale: f32,
    wobble_time: f32,
    is_wobbling: bool,
}

#[derive(Resource, Debug)]
struct MousePosition {
    x: f32,
    y: f32,
}

#[derive(Resource, Debug)]
struct GameState {
    pub is_dragging: bool,
    pub selected_node: Option<Entity>,
    pub updating_target: bool,
    pub updated_beat: bool,
    pub updated_audio: bool,
    pub updated_chain: bool,
}

#[derive(Bundle)]
struct CustomNodeBundle {
    position: Position,
    // class: Oscillator
    #[bundle()]
    sprite_bundle: MaterialMesh2dBundle<ColorMaterial>,
    node_resources: NodeResources,
    node_type: NodeType,
}

#[derive(Default, Resource, Debug)]
struct Nodes {
    nodes: Vec<Entity>,
}

#[derive(Resource, Debug)]
struct GridPositions {
    hand_positions: Vec<Vec2>,
    chain_positions: Vec<Vec2>,
}

#[derive(Default, Resource, Debug)]
struct NodeChain {
    nodes: Vec<Entity>,
}

#[derive(Asset, TypePath)]
struct SineAudio {
    frequency: f32,
}

struct SineDecoder {
    current_progress: f32,
    progress_per_frame: f32,
    period: f32,
    sample_rate: u32,
}

impl SineDecoder {
    fn new(frequency: f32) -> Self {
        let sample_rate = 44_100;
        SineDecoder {
            current_progress: 0.,
            progress_per_frame: frequency / sample_rate as f32,
            period: std::f32::consts::PI * 2.,
            sample_rate,
        }
    }
}

impl Iterator for SineDecoder {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        self.current_progress += self.progress_per_frame;
        // we loop back round to 0 to avoid floating point inaccuracies
        self.current_progress %= 1.;
        Some(f32::sin(self.period * self.current_progress))
    }
}

impl Source for SineDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Decodable for SineAudio {
    type Decoder = SineDecoder;

    type DecoderItem = <SineDecoder as Iterator>::Item;

    fn decoder(&self) -> Self::Decoder {
        SineDecoder::new(self.frequency)
    }
}

#[derive(Component, Debug)]
enum NodeType {
    Oscillator {
        frequency: f32,
    },
    Sequencer {
        sequence: Vec<f32>,
        current_index: usize,
        bpm: f32,
        last_played: Instant,
    },
}

#[derive(Component)]
struct AudioPlayed;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AudioPlugin {
            global_volume: GlobalVolume::new(0.2),
            ..default()
        }))
        .insert_resource(GameState {
            is_dragging: false,
            selected_node: None,
            updating_target: false,
            updated_beat: false,
            updated_audio: false,
            updated_chain: false,
        })
        .insert_resource(MousePosition { x: 0.0, y: 0.0 })
        .insert_resource(Nodes::default())
        .insert_resource(NodeChain::default())
        .insert_resource(GridPositions {
            hand_positions: vec![
                // Bottom row
                Vec2::new(-260.0, -200.0),
                Vec2::new(-130.0, -200.0),
                Vec2::new(0.0, -200.0),
                Vec2::new(130.0, -200.0),
                Vec2::new(260.0, -200.0),
            ],
            chain_positions: vec![
                // Top row
                Vec2::new(-260.0, 100.0),
                Vec2::new(-130.0, 100.0),
                Vec2::new(0.0, 100.0),
                Vec2::new(130.0, 100.0),
                Vec2::new(260.0, 100.0),
            ],
        })
        .add_audio_source::<SineAudio>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                mouse_motion,
                mouse_button,
                update_target_position,
                interpolate_position,
                wobble,
                scale,
                update_transforms,
                snap_to_grid,
                update_check,
                update_node_chain.run_if(updating_chain),
                update_audio.run_if(updating_chain),
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut nodes: ResMut<Nodes>,
    grid_positions: Res<GridPositions>,
    audio_assets: Res<Assets<SineAudio>>,
) {
    commands.spawn(Camera2dBundle::default());

    let square_size = Vec2::new(120.0, 160.0);
    let square_mesh = meshes.add(Mesh::from(Rectangle {
        half_size: square_size / 2.0,
    }));

    for (i, &pos) in grid_positions.hand_positions.iter().enumerate() {
        // Create a unique color for each node
        let color = Color::hsl((i as f32 * 45.0) % 360.0, 0.7, 0.5);
        let square_material = materials.add(ColorMaterial::from(color));

        let node_type = if i == 0 {
            NodeType::Oscillator { frequency: 440.0 }
        } else if i == 1 {
            NodeType::Oscillator { frequency: 660.0 }
        } else if i == 2 {
            NodeType::Oscillator { frequency: 880.0 }
        } else {
            NodeType::Sequencer {
                sequence: vec![440.0, 880.0, 660.0],
                current_index: 0,
                bpm: 120.0,
                last_played: Instant::now(),
            }
        };

        let entity = commands
            .spawn(CustomNodeBundle {
                position: Position { x: pos.x, y: pos.y },
                sprite_bundle: MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(square_mesh.clone()),
                    material: square_material,
                    transform: Transform::from_xyz(pos.x, pos.y, 0.0),
                    ..Default::default()
                },
                node_resources: NodeResources {
                    target: pos,
                    current: pos,
                    interpolation_angle: 0.0,
                    wobble_angle: 0.0,
                    scale: 1.0,
                    wobble_time: 0.0,
                    is_wobbling: false,
                },
                node_type,
            })
            .id();
        nodes.nodes.push(entity);
    }
}

fn mouse_motion(
    mut mouse_position: ResMut<MousePosition>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = windows.get_single() {
        if let Some(position) = window.cursor_position() {
            mouse_position.x = position.x - window.width() / 2.0;
            mouse_position.y = -(position.y - window.height() / 2.0);
        }
    }
}

fn mouse_button(
    mut game_state: ResMut<GameState>,
    mouse_position: Res<MousePosition>,
    mut mousebtn_evr: EventReader<MouseButtonInput>,
    mut query: Query<(Entity, &Transform, &mut NodeResources)>,
) {
    use bevy::input::ButtonState;

    for ev in mousebtn_evr.read() {
        match ev.state {
            ButtonState::Pressed => {
                game_state.is_dragging = true;
                for (entity, transform, mut node_resources) in query.iter_mut() {
                    let distance = Vec2::new(mouse_position.x, mouse_position.y)
                        .distance(Vec2::new(transform.translation.x, transform.translation.y));
                    if distance < 60.0 {
                        game_state.selected_node = Some(entity);
                        node_resources.is_wobbling = true;
                        println!("Selected node: {:?}", entity);
                    }
                }
            }
            ButtonState::Released => {
                game_state.is_dragging = false;
                for (_entity, _transform, mut node_resources) in query.iter_mut() {
                    node_resources.is_wobbling = false;
                }
                println!("Released");
                game_state.selected_node = None;
                game_state.updating_target = true;
            }
        }
    }
}

fn update_target_position(
    mut query: Query<(&mut NodeResources, &Transform)>,
    game_state: Res<GameState>,
    mouse_position: Res<MousePosition>,
) {
    if let Some(selected_node) = game_state.selected_node {
        if let Ok((mut node_resources, _)) = query.get_mut(selected_node) {
            if game_state.is_dragging {
                node_resources.target = Vec2::new(mouse_position.x, mouse_position.y);
            }
        }
    }
}

fn snap_to_grid(
    mut query: Query<&mut NodeResources>,
    mouse_position: Res<MousePosition>,
    nodes: Res<Nodes>,
    grid_positions: Res<GridPositions>,
    game_state: Res<GameState>,
) {
    if !nodes.nodes.is_empty() {
        let mut occupied_positions = vec![];

        for entity in nodes.nodes.iter() {
            if let Ok(mut node_resources) = query.get_mut(*entity) {
                if game_state.is_dragging {
                    if let Some(selected_node) = game_state.selected_node {
                        if let Ok(mut node_resources) = query.get_mut(selected_node) {
                            node_resources.target = Vec2::new(mouse_position.x, mouse_position.y);
                        }
                    }
                } else {
                    let mut closest_position = Vec2::ZERO;
                    let mut closest_distance = f32::MAX;
                    let mut positions = grid_positions.hand_positions.clone();
                    positions.extend(grid_positions.chain_positions.clone());
                    for &grid_pos in positions.iter() {
                        if occupied_positions.contains(&grid_pos) {
                            continue;
                        }
                        let distance = node_resources.target.distance(grid_pos);
                        if distance < closest_distance {
                            closest_distance = distance;
                            closest_position = grid_pos;
                        }
                    }

                    node_resources.target = closest_position;
                    occupied_positions.push(closest_position);
                }
            }
        }
    }
}

fn interpolate_position(mut query: Query<&mut NodeResources>) {
    let t = 0.4; // interpolation factor

    for mut node_resources in query.iter_mut() {
        let new_position = node_resources.current.lerp(node_resources.target, t);
        node_resources.current = new_position;

        let delta = node_resources.target.x - node_resources.current.x;
        node_resources.interpolation_angle = delta / 200.0;
    }
}

fn wobble(time: Res<Time>, mut query: Query<&mut NodeResources>, game_state: Res<GameState>) {
    let decay_rate = 5.0;
    let wobble_amplitude = 0.8;
    let wobble_speed = 1.3;
    let frequency = 20.0;

    if let Some(selected_node) = game_state.selected_node {
        if let Ok(mut node_resources) = query.get_mut(selected_node) {
            if node_resources.is_wobbling {
                node_resources.wobble_time += wobble_speed * time.delta_seconds();
                let wobble_factor = (node_resources.wobble_time * frequency).sin()
                    * wobble_amplitude
                    * (-decay_rate * node_resources.wobble_time).exp();

                node_resources.wobble_angle = wobble_factor;
            } else {
                node_resources.wobble_angle = node_resources
                    .wobble_angle
                    .lerp(0.0, time.delta_seconds() * 6.0);
                node_resources.wobble_time = 0.0;
            }
        }
    } else {
        for mut node_resources in query.iter_mut() {
            node_resources.wobble_angle = node_resources
                .wobble_angle
                .lerp(0.0, time.delta_seconds() * 6.0);
            node_resources.wobble_time = 0.0;
        }
    }
}

fn scale(time: Res<Time>, mut query: Query<&mut NodeResources>, game_state: Res<GameState>) {
    let scale_rate = 4.0;

    if let Some(selected_node) = game_state.selected_node {
        if let Ok(mut node_resources) = query.get_mut(selected_node) {
            if node_resources.is_wobbling {
                let scale_increase = scale_rate * time.delta_seconds();
                node_resources.scale = (node_resources.scale + scale_increase).min(1.3);
            } else {
                let scale_decrease = scale_rate * time.delta_seconds();
                node_resources.scale = (node_resources.scale - scale_decrease).max(1.0);
            }
        }
    } else {
        // When no node is selected, gradually decrease the scale of all nodes back to 1.0
        for mut node_resources in query.iter_mut() {
            let scale_decrease = scale_rate * time.delta_seconds();
            node_resources.scale = (node_resources.scale - scale_decrease).max(1.0);
        }
    }
}

fn update_transforms(
    mut query: Query<(&Position, &mut Transform, &NodeResources)>,
    game_state: Res<GameState>,
) {
    for (_position, mut transform, node_resources) in query.iter_mut() {
        transform.translation.x = node_resources.current.x;
        transform.translation.y = node_resources.current.y;

        transform.scale = Vec3::splat(node_resources.scale);

        let combined_angle = node_resources.interpolation_angle + node_resources.wobble_angle;
        transform.rotation = Quat::from_rotation_z(combined_angle);

        // Reset z-level for all nodes
        transform.translation.z = 0.0;
    }

    if let Some(selected_node) = game_state.selected_node {
        if let Ok((_position, mut transform, _node_resources)) = query.get_mut(selected_node) {
            // Set z-level for the selected node
            transform.translation.z = 1.0;
        }
    }
}

fn update_node_chain(
    mut node_chain: ResMut<NodeChain>,
    grid_positions: Res<GridPositions>,
    mut game_state: ResMut<GameState>,
    query: Query<(Entity, &NodeResources)>,
) {
    for (entity, node_resources) in query.iter() {
        if grid_positions
            .chain_positions
            .contains(&node_resources.target)
        {
            node_chain.nodes.push(entity);
            // println!("Node chain: {:?}", node_chain.nodes);
        }
    }
    println!("Node chain: {:?}", node_chain.nodes);
    game_state.updated_chain = true;
}

fn updating_chain(game_state: Res<GameState>) -> bool {
    game_state.updating_target
}

fn update_check(mut game_state: ResMut<GameState>) {
    if game_state.updating_target && game_state.updated_chain && game_state.updated_audio {
        game_state.updating_target = false;
        game_state.updated_chain = false;
        game_state.updated_audio = false;
    }
}

fn updating_beat(game_state: Res<GameState>) -> bool {
    game_state.updated_beat
}

fn update_audio(
    node_chain: Res<NodeChain>,
    mut query: Query<(Entity, &NodeResources, &mut NodeType), Without<AudioPlayed>>,
    mut commands: Commands,
    mut audio_assets: ResMut<Assets<SineAudio>>,
    mut game_state: ResMut<GameState>,
) {
    let current_time = Instant::now();

    for &entity in node_chain.nodes.iter() {
        if let Ok((_entity, _node_resources, mut node_type)) = query.get_mut(entity) {
            match &mut *node_type {
                NodeType::Oscillator { frequency } => {
                    // Mark this entity as having its audio played
                    commands.entity(entity).insert(AudioPlayed);
                    let audio_handle = audio_assets.add(SineAudio {
                        frequency: *frequency,
                    });
                    commands.spawn(AudioSourceBundle {
                        source: audio_handle,
                        ..default()
                    });
                }
                NodeType::Sequencer {
                    sequence,
                    current_index,
                    bpm,
                    last_played,
                } => {}
            }
        }
    }
    game_state.updated_audio = true
}

fn update_sequence(
    node_chain: Res<NodeChain>,
    mut query: Query<(Entity, &NodeResources, &mut NodeType), Without<AudioPlayed>>,
    mut commands: Commands,
    mut audio_assets: ResMut<Assets<SineAudio>>,
    mut game_state: ResMut<GameState>,
) {
    let current_time = Instant::now();

    for &entity in node_chain.nodes.iter() {
        if let Ok((_entity, _node_resources, mut node_type)) = query.get_mut(entity) {
            match &mut *node_type {
                NodeType::Oscillator { frequency } => {}
                NodeType::Sequencer {
                    sequence,
                    current_index,
                    bpm,
                    last_played,
                } => {
                    let interval = 60.0 / *bpm as f32;

                    if current_time.duration_since(*last_played).as_secs_f32() >= interval {
                        let frequency = sequence[*current_index];
                        let audio_handle = audio_assets.add(SineAudio { frequency });
                        commands.spawn(AudioSourceBundle {
                            source: audio_handle,
                            ..default()
                        });

                        // Update the sequencer's state
                        *current_index = (*current_index + 1) % sequence.len();
                        *last_played = current_time;
                        commands.entity(entity).insert(AudioPlayed);
                    }
                }
            }
        }
    }
}
