use bevy::{prelude::*, render::mesh::Mesh, sprite::MaterialMesh2dBundle};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(MaterialMesh2dBundle {
        mesh: bevy::sprite::Mesh2dHandle(meshes.add(bevy::render::mesh::Mesh::from(
            bevy_math::prelude::Circle::new(50.),
        ))),
        material: materials.add(ColorMaterial::from(Color::PURPLE)),
        transform: Transform::from_translation(Vec3::new(-150., 0., 0.)),
        ..default()
    });

    // Rectangle
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.25, 0.25, 0.75),
            custom_size: Some(Vec2::new(50.0, 100.0)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(-50., 0., 0.)),
        ..default()
    });

    // Quad
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes
            .add(Into::<Mesh>::into(bevy_math::prelude::Rectangle::new(
                50., 100.,
            )))
            .into(),
        material: materials.add(ColorMaterial::from(Color::LIME_GREEN)),
        transform: Transform::from_translation(Vec3::new(50., 0., 0.)),
        ..default()
    });

    // Hexagon
    let regular_polygon = bevy_math::prelude::RegularPolygon {
        sides: 6,
        circumcircle: bevy_math::prelude::Circle::new(50.),
    };

    commands.spawn(MaterialMesh2dBundle {
        mesh: Into::<bevy::sprite::Mesh2dHandle>::into(meshes.add(regular_polygon)),
        material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
        transform: Transform::from_translation(Vec3::new(150., 0., 0.)),
        ..default()
    });
}
