mod anchors;
pub mod entity;
mod margins;
mod node;
mod render;
mod ui_update_system;
pub mod widget;

pub use anchors::*;
pub use margins::*;
pub use node::*;
pub use render::*;
pub use ui_update_system::*;

use bevy_app::{stage, AppBuilder, AppPlugin};
use bevy_render::render_graph::RenderGraph;
use legion::prelude::IntoSystem;
use widget::Label;

#[derive(Default)]
pub struct UiPlugin;

impl AppPlugin for UiPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(stage::POST_UPDATE, ui_update_system())
            .add_system_to_stage(stage::POST_UPDATE, Label::label_system.system());

        let resources = app.resources();
        let mut render_graph = resources.get_mut::<RenderGraph>().unwrap();
        render_graph.add_ui_graph(resources);
    }
}
