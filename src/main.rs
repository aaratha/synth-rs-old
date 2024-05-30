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
struct Square {
    id: usize,
}

#[derive(Resource, Debug)]
struct MousePosition {
    x: f32,
    y: f32,
}
#[derive(Resource, Debug)]
struct SquarePositions {
    first: Vec2,
    second: Vec2,
}

#[derive(Resource, Debug)]
pub struct GameState {
    pub is_playing: bool,
    pub is_dragging: bool,
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
        })
        .insert_resource(MousePosition { x: 0.0, y: 0.0 })
        .insert_resource(SquarePositions {
            first: Vec2::ZERO,
            second: Vec2::ZERO,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                mouse_motion,
                mouse_button,
                update_position,
                update_square_positions,
                interpolate_position,
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

    let square_size = Vec2::new(100.0, 100.0);
    let square_mesh = meshes.add(Mesh::from(bevy::math::primitives::Rectangle {
        half_size: square_size / 2.0,
    }));
    let square_material_purple =
        materials.add(ColorMaterial::from(Color::rgba(0.5, 0.0, 0.5, 0.0)));
    let square_material_blue = materials.add(ColorMaterial::from(Color::BLUE));

    // Spawn the first square
    commands.spawn((
        CustomNodeBundle {
            position: Position { x: 0.0, y: 0.0 },
            sprite_bundle: MaterialMesh2dBundle {
                mesh: Mesh2dHandle(square_mesh.clone()),
                material: square_material_purple.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..Default::default()
            },
        },
        Square { id: 1 },
    ));

    // Spawn the second square, positioned above the first one
    commands.spawn((
        CustomNodeBundle {
            position: Position { x: 0.0, y: 100.0 }, // Positioned 100 units above
            sprite_bundle: MaterialMesh2dBundle {
                mesh: Mesh2dHandle(square_mesh.clone()), // Clone the mesh handle
                material: square_material_blue.clone(),  // Clone the material handle
                transform: Transform::from_xyz(0.0, 100.0, 0.0), // 100 units above in the y-axis
                ..Default::default()
            },
        },
        Square { id: 2 },
    ));
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
            }
            ButtonState::Released => {
                game_state.is_dragging = false;
            }
        }
    }
}

fn update_position(
    mut query: Query<(&Square, &mut Position, &mut Transform)>,
    game_state: Res<GameState>,
    mouse_position: Res<MousePosition>,
) {
    for (square, mut position, mut transform) in query.iter_mut() {
        if game_state.is_dragging && square.id == 1 {
            position.x = mouse_position.x;
            position.y = mouse_position.y;
        }
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}

fn update_square_positions(
    query: Query<(&Square, &Position)>,
    mut square_positions: ResMut<SquarePositions>,
) {
    for (square, position) in query.iter() {
        if square.id == 1 {
            square_positions.first = Vec2::new(position.x, position.y);
        } else if square.id == 2 {
            square_positions.second = Vec2::new(position.x, position.y);
        }
    }
}

fn interpolate_position(
    mut query: Query<(&Square, &mut Position, &mut Transform)>,
    square_positions: Res<SquarePositions>,
) {
    let t = 0.3; // interpolation factor

    for (square, mut position, mut transform) in query.iter_mut() {
        if square.id == 2 {
            // Interpolate the position of the second square towards the first square
            let new_position = position.as_vec2().lerp(square_positions.first, t);
            position.x = new_position.x;
            position.y = new_position.y;
            transform.translation.x = position.x;
            transform.translation.y = position.y;
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

// Helper function to convert Position to Vec2
impl Position {
    fn as_vec2(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}
