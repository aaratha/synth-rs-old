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

#[derive(Component, Debug)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Resource, Debug)]
struct MousePosition {
    x: f32,
    y: f32,
}

#[derive(Resource, Debug)]
pub struct GameState {
    pub is_playing: bool,
    pub is_dragging: bool,
}

#[derive(Bundle)]
struct CustomNodeBundle {
    position: Position,
    velocity: Velocity,
    #[bundle()]
    sprite_bundle: MaterialMesh2dBundle<ColorMaterial>,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FrameTimeDiagnosticsPlugin))
        .insert_resource(GameState {
            is_playing: true,
            is_dragging: false,
        })
        .insert_resource(MousePosition { x: 0.0, y: 0.0 })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                mouse_motion,
                mouse_button,
                update_position,
                update_transform,
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
    const PURPLE: Color = Color::rgba(1.0, 0.0, 1.0, 1.0);
    let square_size = Vec2::new(100.0, 100.0);
    let square_mesh = meshes.add(Mesh::from(bevy::math::primitives::Rectangle {
        half_size: square_size / 2.0,
    }));
    let square_material = materials.add(ColorMaterial::from(PURPLE));
    commands.spawn(CustomNodeBundle {
        position: Position { x: 0.0, y: 0.0 },
        velocity: Velocity { x: 50.0, y: 50.0 },
        sprite_bundle: MaterialMesh2dBundle {
            mesh: Mesh2dHandle(square_mesh),
            material: square_material,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        },
    });
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
            // println!("mouse position: {:?}", mouse_position)
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
            }
            ButtonState::Released => {
                game_state.is_dragging = false;
            }
        }
        // println!("is dragging: {:?}", game_state.is_dragging)
    }
}

fn update_position(
    mut query: Query<(&Velocity, &mut Position)>,
    time: Res<Time>,
    game_state: Res<GameState>,
    mouse_position: Res<MousePosition>,
) {
    for (velocity, mut position) in query.iter_mut() {
        if game_state.is_dragging {
            position.x = mouse_position.x;
            position.y = mouse_position.y;
        } else {
            position.x += velocity.x * time.delta_seconds();
            position.y += velocity.y * time.delta_seconds();
        }
    }
}

fn update_transform(mut query: Query<(&Position, &mut Transform)>) {
    for (position, mut transform) in query.iter_mut() {
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}

fn print_fps(diagnostics: Res<DiagnosticsStore>) {
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            println!("FPS: {:.2}", average);
        }
    }
}
