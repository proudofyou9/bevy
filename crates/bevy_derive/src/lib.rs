extern crate proc_macro;

mod app_plugin;
mod bytes;
mod entity_archetype;
mod modules;
mod resource;
mod uniforms;

use proc_macro::TokenStream;

#[proc_macro_derive(FromResources, attributes(module))]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    resource::derive_resource(input)
}

#[proc_macro_derive(Bytes, attributes(module))]
pub fn derive_bytes(input: TokenStream) -> TokenStream {
    bytes::derive_bytes(input)
}

#[proc_macro_derive(Uniform, attributes(uniform, module))]
pub fn derive_uniform(input: TokenStream) -> TokenStream {
    uniforms::derive_uniform(input)
}

#[proc_macro_derive(EntityArchetype, attributes(tag, module))]
pub fn derive_entity_archetype(input: TokenStream) -> TokenStream {
    entity_archetype::derive_entity_archetype(input)
}

#[proc_macro_derive(Uniforms, attributes(uniform, module))]
pub fn derive_uniforms(input: TokenStream) -> TokenStream {
    uniforms::derive_uniforms(input)
}

#[proc_macro_derive(DynamicAppPlugin)]
pub fn derive_app_plugin(input: TokenStream) -> TokenStream {
    app_plugin::derive_app_plugin(input)
}
