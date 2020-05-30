use crate::{ColorMaterial, Rect};
use bevy_asset::{Assets, Handle};
use bevy_render::{
    base_render_graph,
    draw_target::AssignedMeshesDrawTarget,
    pipeline::{state_descriptors::*, PipelineDescriptor},
    render_graph::{
        nodes::{AssetUniformNode, PassNode, UniformNode},
        RenderGraph,
    },
    shader::{Shader, ShaderStage, ShaderStages},
    texture::TextureFormat,
};
use legion::prelude::Resources;

pub const SPRITE_PIPELINE_HANDLE: Handle<PipelineDescriptor> =
    Handle::from_u128(278534784033876544639935131272264723170);

pub fn build_sprite_pipeline(shaders: &mut Assets<Shader>) -> PipelineDescriptor {
    PipelineDescriptor {
        rasterization_state: Some(RasterizationStateDescriptor {
            front_face: FrontFace::Ccw,
            cull_mode: CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        }),
        depth_stencil_state: Some(DepthStencilStateDescriptor {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            stencil_front: StencilStateFaceDescriptor::IGNORE,
            stencil_back: StencilStateFaceDescriptor::IGNORE,
            stencil_read_mask: 0,
            stencil_write_mask: 0,
        }),
        color_states: vec![ColorStateDescriptor {
            format: TextureFormat::Bgra8UnormSrgb,
            color_blend: BlendDescriptor {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
            alpha_blend: BlendDescriptor {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
            write_mask: ColorWrite::ALL,
        }],
        ..PipelineDescriptor::new(ShaderStages {
            vertex: shaders.add(Shader::from_glsl(
                ShaderStage::Vertex,
                include_str!("sprite.vert"),
            )),
            fragment: Some(shaders.add(Shader::from_glsl(
                ShaderStage::Fragment,
                include_str!("sprite.frag"),
            ))),
        })
    }
}

pub mod node {
    pub const COLOR_MATERIAL: &'static str = "color_material";
    pub const RECT: &'static str = "rect";
}

pub trait SpriteRenderGraphBuilder {
    fn add_sprite_graph(&mut self, resources: &Resources) -> &mut Self;
}

impl SpriteRenderGraphBuilder for RenderGraph {
    fn add_sprite_graph(&mut self, resources: &Resources) -> &mut Self {
        self.add_system_node(
            node::COLOR_MATERIAL,
            AssetUniformNode::<ColorMaterial>::new(false),
        );
        self.add_node_edge(node::COLOR_MATERIAL, base_render_graph::node::MAIN_PASS)
            .unwrap();

        self.add_system_node(node::RECT, UniformNode::<Rect>::new(false));
        self.add_node_edge(node::RECT, base_render_graph::node::MAIN_PASS)
            .unwrap();

        let mut pipelines = resources.get_mut::<Assets<PipelineDescriptor>>().unwrap();
        let mut shaders = resources.get_mut::<Assets<Shader>>().unwrap();
        pipelines.set(SPRITE_PIPELINE_HANDLE, build_sprite_pipeline(&mut shaders));
        let main_pass: &mut PassNode = self
            .get_node_mut(base_render_graph::node::MAIN_PASS)
            .unwrap();
        main_pass.add_pipeline(
            SPRITE_PIPELINE_HANDLE,
            vec![Box::new(AssignedMeshesDrawTarget)],
        );
        self
    }
}
