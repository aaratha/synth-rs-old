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
    angle: f32,
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
            angle: 0.0,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                mouse_motion,
                mouse_button,
                update_target_position,
                interpolate_position,
                wobble,
                // print_fps,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let square_size = Vec2::new(120.0, 160.0);
    let square_mesh = meshes.add(Mesh::from(bevy::math::primitives::Rectangle {
        half_size: square_size / 2.0,
    }));
    let square_material_blue = materials.add(ColorMaterial::from(Color::BLUE));

    // Spawn the square
    commands.spawn((CustomNodeBundle {
        position: Position { x: 0.0, y: 100.0 }, // Initially positioned
        sprite_bundle: MaterialMesh2dBundle {
            mesh: Mesh2dHandle(square_mesh.clone()),
            material: square_material_blue.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        },
    },));
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
    mut square_positions: ResMut<SquareResources>,
    game_state: Res<GameState>,
    mouse_position: Res<MousePosition>,
) {
    if game_state.is_dragging {
        square_positions.target = Vec2::new(mouse_position.x, mouse_position.y);
    }
}

fn interpolate_position(
    mut query: Query<(&mut Position, &mut Transform)>,
    mut square_resources: ResMut<SquareResources>,
) {
    let t = 0.3; // interpolation factor

    for (mut position, mut transform) in query.iter_mut() {
        // Interpolate the position of the square towards the target position
        let delta = square_resources.target.x - square_resources.current.x;
        square_resources.angle = delta / 400.0;
        let new_position = square_resources.current.lerp(square_resources.target, t);
        position.x = new_position.x;
        position.y = new_position.y;
        transform.translation.x = position.x;
        transform.translation.y = position.y;
        transform.rotation = Quat::from_rotation_z(square_resources.angle);
        square_resources.current = new_position;
    }
}

fn wobble(
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    mut square_resources: ResMut<SquareResources>,
    mut query: Query<(&mut Transform, &Position)>,
) {
    let decay_rate = 3.0;
    let wobble_amplitude = 20.0;
    let wobble_speed = 1.3;

    if game_state.is_wobbling {
        game_state.wobble_time += wobble_speed * time.delta_seconds();
        let wobble_factor = (game_state.wobble_time * wobble_amplitude).sin()
            * (-decay_rate * game_state.wobble_time).exp();

        for (mut transform, position) in query.iter_mut() {
            // Update the rotation around the square's own center
            transform.rotate_around(
                Vec3::new(position.x, position.y, 0.0),
                Quat::from_rotation_z(wobble_factor),
            );
        }
    } else {
        game_state.wobble_time = 0.0;
    }
}

fn print_fps(diagnostics: Res<DiagnosticsStore>) {
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            println!("FPS: {:.2}", average);
        }
    }
}
