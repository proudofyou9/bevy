use bevy::prelude::*;

fn main() {
    App::build()
        .add_default_plugins()
        .add_startup_system(setup.system())
        .add_system(placement_system.system())
        .add_plugin(DiagnosticsPlugin::default())
        .run();
}

fn placement_system(
    time: Res<Time>,
    materials: Res<Assets<ColorMaterial>>,
    mut node: ComMut<Node>,
    material_handle: Com<Handle<ColorMaterial>>,
) {
    let material = materials.get(&material_handle).unwrap();
    if material.color.r > 0.2 {
        node.position += Vec2::new(0.1 * time.delta_seconds, 0.0);
    }
}

fn setup(mut materials: ResMut<Assets<ColorMaterial>>, command_buffer: &mut CommandBuffer) {
    let mut builder = command_buffer.build();
    builder.entity_with(OrthographicCameraComponents::default());

    let mut prev = Vec2::default();
    let count = 1000;
    for i in 0..count {
        // 2d camera
        let cur = Vec2::new(1.0, 1.0) + prev;
        builder.entity_with(UiComponents {
            node: Node {
                position: Vec2::new(75.0, 75.0) + cur,
                anchors: Anchors::new(0.5, 0.5, 0.5, 0.5),
                margins: Margins::new(0.0, 100.0, 0.0, 100.0),
                ..Default::default()
            },
            material: materials.add(Color::rgb(0.0 + i as f32 / count as f32, 0.1, 0.1).into()),
            ..Default::default()
        });

        prev = cur;
    }
}
