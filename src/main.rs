use bevy::math::primitives::Rectangle;
use bevy::render::color::Color::Rgba;
use bevy::ui::update;
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

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
pub struct GameState {
    pub is_playing: bool,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (update_position, print_position, update_transform))
        .insert_resource(GameState { is_playing: true })
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
    let square_mesh = meshes.add(Mesh::from(shape::Quad::new(square_size)));
    let square_material = materials.add(ColorMaterial::from(PURPLE));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(square_mesh),
            material: square_material,
            transform: Transform::default(),
            ..Default::default()
        },
        Position { x: 0.0, y: 0.0 },
        Velocity { x: 50.0, y: 50.0 },
    ));
}

fn update_position(mut query: Query<(&Velocity, &mut Position)>, time: Res<Time>) {
    for (velocity, mut position) in query.iter_mut() {
        position.x += velocity.x * time.delta_seconds();
        position.y += velocity.y * time.delta_seconds();
    }
}

fn update_transform(mut query: Query<(&Position, &mut Transform)>) {
    for (position, mut transform) in query.iter_mut() {
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}

fn print_position(query: Query<(Entity, &Position)>) {
    for (entity, position) in query.iter() {
        println!("Entity {:?} is at Position {:?}", entity, position);
    }
}
