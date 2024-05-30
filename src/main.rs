use bevy::core_pipeline::deferred::node;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::ecs::query::QueryData;
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::window::PrimaryWindow;

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
    pub is_playing: bool,
    pub is_dragging: bool,
    pub selected_node: Option<Entity>,
}

#[derive(Bundle)]
struct CustomNodeBundle {
    position: Position,
    #[bundle()]
    sprite_bundle: MaterialMesh2dBundle<ColorMaterial>,
    node_resources: NodeResources,
}

#[derive(Default, Resource, Debug)]
struct NodeChain {
    nodes: Vec<Entity>,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FrameTimeDiagnosticsPlugin))
        .insert_resource(GameState {
            is_playing: true,
            is_dragging: false,
            selected_node: None,
        })
        .insert_resource(MousePosition { x: 0.0, y: 0.0 })
        .insert_resource(NodeChain::default())
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
                // print_fps,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut node_chain: ResMut<NodeChain>,
) {
    commands.spawn(Camera2dBundle::default());

    let square_size = Vec2::new(120.0, 160.0);
    let square_mesh = meshes.add(Mesh::from(bevy::math::primitives::Rectangle {
        half_size: square_size / 2.0,
    }));
    let square_material = materials.add(ColorMaterial::from(Color::BLUE));

    for i in 0..5 {
        let entity = commands
            .spawn(CustomNodeBundle {
                position: Position {
                    x: i as f32 * 130.0,
                    y: 0.0,
                },
                sprite_bundle: MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(square_mesh.clone()),
                    material: square_material.clone(),
                    transform: Transform::from_xyz(i as f32 * 130.0, 0.0, 0.0),
                    ..Default::default()
                },
                node_resources: NodeResources {
                    target: Vec2::ZERO,
                    current: Vec2::new(i as f32 * 130.0, 0.0),
                    interpolation_angle: 0.0,
                    wobble_angle: 0.0,
                    scale: 1.0,
                    wobble_time: 0.0,
                    is_wobbling: false,
                },
            })
            .id();
        node_chain.nodes.push(entity);
    }

    println!("NodeChain: {:?}", node_chain.nodes);
}

fn mouse_motion(
    mut mouse_position: ResMut<MousePosition>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = windows.get_single() {
        if let Some(position) = window.cursor_position() {
            // Convert the mouse position from window space to world space
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
                        // Assuming half size of the squares is 60.0
                        game_state.selected_node = Some(entity);
                        node_resources.is_wobbling = true;
                        println!("Selected node: {:?}", entity);
                        break;
                    }
                }
            }
            ButtonState::Released => {
                game_state.is_dragging = false;
                for (entity, transform, mut node_resources) in query.iter_mut() {
                    // Assuming half size of the squares is 60.0
                    game_state.selected_node = Some(entity);
                    node_resources.is_wobbling = false;
                    println!("Selected node: {:?}", entity);
                    break;
                }
            }
        }
    }
}

fn update_target_position(
    mut query: Query<&mut NodeResources>,
    game_state: Res<GameState>,
    mouse_position: Res<MousePosition>,
) {
    if let Some(selected_node) = game_state.selected_node {
        if let Ok(mut node_resources) = query.get_mut(selected_node) {
            if game_state.is_dragging {
                node_resources.target = Vec2::new(mouse_position.x, mouse_position.y);
            }
        }
    }
}

fn interpolate_position(mut query: Query<&mut NodeResources>, game_state: Res<GameState>) {
    let t = 0.3; // interpolation factor

    if let Some(selected_node) = game_state.selected_node {
        if let Ok(mut node_resources) = query.get_mut(selected_node) {
            if game_state.is_dragging {
                let new_position = node_resources.current.lerp(node_resources.target, t);
                node_resources.current = new_position;

                let delta = node_resources.target.x - node_resources.current.x;
                node_resources.interpolation_angle = delta / 200.0;
            }
        }
    }
}

fn wobble(time: Res<Time>, game_state: Res<GameState>, mut query: Query<&mut NodeResources>) {
    let decay_rate = 3.0;
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
                // Smoothly interpolate wobble_angle to 0 instead of setting it abruptly
                node_resources.wobble_angle = node_resources
                    .wobble_angle
                    .lerp(0.0, time.delta_seconds() * 6.0);
                node_resources.wobble_time = 0.0;
            }
        }
    }
}

fn scale(time: Res<Time>, game_state: Res<GameState>, mut query: Query<&mut NodeResources>) {
    let scale_rate = 4.0;

    if let Some(selected_node) = game_state.selected_node {
        if let Ok(mut node_resources) = query.get_mut(selected_node) {
            if node_resources.is_wobbling {
                // Increase the scale up to 1.5
                let scale_increase = scale_rate * time.delta_seconds();
                node_resources.scale = (node_resources.scale + scale_increase).min(1.5);
            } else {
                // Gradually decrease the scale back to 1.0
                let scale_decrease = scale_rate * time.delta_seconds();
                node_resources.scale = (node_resources.scale - scale_decrease).max(1.0);
            }
        }
    }
}

fn update_transforms(
    mut query: Query<(Entity, &Position, &mut Transform, &NodeResources)>,
    game_state: Res<GameState>,
) {
    if let Some(selected_node) = game_state.selected_node {
        for (entity, position, mut transform, node_resources) in query.iter_mut() {
            if entity == selected_node {
                transform.translation.x = node_resources.current.x;
                transform.translation.y = node_resources.current.y;

                transform.scale = Vec3::splat(node_resources.scale);

                // Combine the angles from interpolation and wobble
                let combined_angle =
                    node_resources.interpolation_angle + node_resources.wobble_angle;
                transform.rotation = Quat::from_rotation_z(combined_angle);
            }
        }
    }
}

fn print_fps(diagnostics: Res<DiagnosticsStore>) {
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            println!("FPS: {:.2}", average);
        }
    }
}
