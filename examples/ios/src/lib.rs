use bevy::{prelude::*, window::WindowMode};

// the `bevy_main` proc_macro generates the required ios boilerplate
#[bevy_main]
fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            vsync: true,
            resizable: false,
            mode: WindowMode::BorderlessFullscreen,
            ..Default::default()
        })
        .add_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .run();
}
/// set up a simple 3D scene
fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // add entities to the world
    commands
        // plane
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
            material: materials.add(Color::rgb(0.1, 0.2, 0.1).into()),
            ..Default::default()
        })
        // cube
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.5, 0.4, 0.3).into()),
            transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            ..Default::default()
        })
        // sphere
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                subdivisions: 4,
                radius: 0.5,
            })),
            material: materials.add(Color::rgb(0.1, 0.4, 0.8).into()),
            transform: Transform::from_translation(Vec3::new(1.5, 1.5, 1.5)),
            ..Default::default()
        })
        // light
        .spawn(LightComponents {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..Default::default()
        })
        // camera
        .spawn(Camera3dComponents {
            transform: Transform::from_translation(Vec3::new(-3.0, 5.0, 8.0))
                .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        });
}
