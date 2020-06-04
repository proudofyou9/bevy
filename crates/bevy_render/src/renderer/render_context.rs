use super::RenderResourceContext;
use crate::{
    pass::{PassDescriptor, RenderPass},
    render_resource::{RenderResource, RenderResourceAssignments},
    texture::{Extent3d},
};

pub trait RenderContext {
    fn resources(&self) -> &dyn RenderResourceContext;
    fn resources_mut(&mut self) -> &mut dyn RenderResourceContext;
    fn copy_buffer_to_buffer(
        &mut self,
        source_buffer: RenderResource,
        source_offset: u64,
        destination_buffer: RenderResource,
        destination_offset: u64,
        size: u64,
    );
    fn copy_buffer_to_texture(
        &mut self,
        source_buffer: RenderResource,
        source_offset: u64,
        source_bytes_per_row: u32,
        destination_texture: RenderResource,
        destination_origin: [u32; 3],
        destination_mip_level: u32,
        size: Extent3d,
    );
    fn begin_pass(
        &mut self,
        pass_descriptor: &PassDescriptor,
        render_resource_assignments: &RenderResourceAssignments,
        run_pass: &mut dyn Fn(&mut dyn RenderPass),
    );
}
