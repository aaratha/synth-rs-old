use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::window::PrimaryWindow;

#[derive(Component, Debug)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Resource, Debug)]
struct MousePosition {
    x: f32,
    y: f32,
}

#[derive(Resource, Debug)]
struct SquareResources {
    target: Vec2,
    current: Vec2,
    interpolation_angle: f32,
    wobble_angle: f32,
    scale: f32,
}

#[derive(Resource, Debug)]
pub struct GameState {
    pub is_playing: bool,
    pub is_dragging: bool,
    pub is_wobbling: bool,
    pub wobble_time: f32,
}

#[derive(Bundle)]
struct CustomNodeBundle {
    position: Position,
    #[bundle()]
    sprite_bundle: MaterialMesh2dBundle<ColorMaterial>,
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
            is_wobbling: false,
            wobble_time: 0.0,
        })
        .insert_resource(MousePosition { x: 0.0, y: 0.0 })
        .insert_resource(SquareResources {
            target: Vec2::ZERO,
            current: Vec2::ZERO,
            interpolation_angle: 0.0,
            wobble_angle: 0.0,
            scale: 1.0,
        })
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
                update_transforms,
                scale,
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
    mut mousebtn_evr: EventReader<MouseButtonInput>,
) {
    use bevy::input::ButtonState;

    for ev in mousebtn_evr.read() {
        match ev.state {
            ButtonState::Pressed => {
                game_state.is_dragging = true;
                game_state.is_wobbling = true;
            }
            ButtonState::Released => {
                game_state.is_dragging = false;
                game_state.is_wobbling = false;
            }
        }
    }
}

fn update_target_position(
    mut square_resources: ResMut<SquareResources>,
    game_state: Res<GameState>,
    mouse_position: Res<MousePosition>,
) {
    if game_state.is_dragging {
        square_resources.target = Vec2::new(mouse_position.x, mouse_position.y);
    }
}

fn interpolate_position(mut square_resources: ResMut<SquareResources>, game_state: Res<GameState>) {
    let t = 0.3; // interpolation factor

    if game_state.is_dragging {
        let new_position = square_resources.current.lerp(square_resources.target, t);
        square_resources.current = new_position;

        let delta = square_resources.target.x - square_resources.current.x;
        square_resources.interpolation_angle = delta / 200.0;
    }
}

fn wobble(
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    mut square_resources: ResMut<SquareResources>,
) {
    let decay_rate = 3.0;
    let wobble_amplitude = 0.8;
    let wobble_speed = 1.3;
    let frequency = 20.0;

    if game_state.is_wobbling {
        game_state.wobble_time += wobble_speed * time.delta_seconds();
        let wobble_factor = (game_state.wobble_time * frequency).sin()
            * wobble_amplitude
            * (-decay_rate * game_state.wobble_time).exp();

        square_resources.wobble_angle = wobble_factor;
    } else {
        // Smoothly interpolate wobble_angle to 0 instead of setting it abruptly
        square_resources.wobble_angle = square_resources
            .wobble_angle
            .lerp(0.0, time.delta_seconds() * 6.0);
        game_state.wobble_time = 0.0;
    }
}

fn scale(
    time: Res<Time>,
    game_state: Res<GameState>,
    mut square_resources: ResMut<SquareResources>,
) {
    let decay_rate = 3.0;
    if game_state.is_wobbling {
        // Increase the scale up to 1.5
        let scale_increase = 4.0 * time.delta_seconds();
        square_resources.scale = (square_resources.scale + scale_increase).min(1.5);
    } else {
        // Gradually decrease the scale back to 1.0
        let scale_decrease = 4.0 * time.delta_seconds();
        square_resources.scale = (square_resources.scale - scale_decrease).max(1.0);
    }
}

fn update_transforms(
    mut query: Query<(&Position, &mut Transform)>,
    square_resources: Res<SquareResources>,
) {
    for (position, mut transform) in query.iter_mut() {
        transform.translation.x = square_resources.current.x;
        transform.translation.y = square_resources.current.y;

        transform.scale = Vec3::splat(square_resources.scale);

        // Combine the angles from interpolation and wobble
        let combined_angle = square_resources.interpolation_angle + square_resources.wobble_angle;
        transform.rotation = Quat::from_rotation_z(combined_angle);
    }
}

fn print_fps(diagnostics: Res<DiagnosticsStore>) {
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            println!("FPS: {:.2}", average);
        }
    }
}
