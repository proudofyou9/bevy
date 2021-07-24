use bevy_asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy_reflect::{TypeUuid, Uuid};
use bevy_utils::{tracing::error, BoxedFuture};
use std::marker::Copy;
use thiserror::Error;
use wgpu::ShaderStage;

/// An error that occurs during shader handling.
#[derive(Error, Debug)]
pub enum ShaderError {
    /// Shader compilation error.
    #[error("Shader compilation error:\n{0}")]
    Compilation(String),

    #[cfg(not(any(
        target_arch = "wasm32",
        all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"),
        all(target_arch = "x86_64", target_os = "macos"),
        all(target_arch = "aarch64", target_os = "android"),
        all(target_arch = "armv7", target_os = "androidabi"),
        all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    )))]
    /// shaderc error.
    #[error("shaderc error: {0}")]
    ShaderC(#[from] shaderc::Error),

    #[cfg(not(any(
        target_arch = "wasm32",
        all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"),
        all(target_arch = "x86_64", target_os = "macos"),
        all(target_arch = "aarch64", target_os = "android"),
        all(target_arch = "armv7", target_os = "androidabi"),
        all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    )))]
    #[error("Error initializing shaderc Compiler")]
    ErrorInitializingShadercCompiler,

    #[cfg(not(any(
        target_arch = "wasm32",
        all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"),
        all(target_arch = "x86_64", target_os = "macos"),
        all(target_arch = "aarch64", target_os = "android"),
        all(target_arch = "armv7", target_os = "androidabi"),
        all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    )))]
    #[error("Error initializing shaderc CompileOptions")]
    ErrorInitializingShadercCompileOptions,
}

#[cfg(any(
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"),
    all(target_arch = "x86_64", target_os = "macos"),
    all(target_arch = "aarch64", target_os = "android"),
    all(target_arch = "armv7", target_os = "androidabi"),
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
))]
fn convert_stage(s: ShaderStage) -> bevy_glsl_to_spirv::ShaderType {
    match s {
        ShaderStage::VERTEX => bevy_glsl_to_spirv::ShaderType::Vertex,
        ShaderStage::FRAGMENT => bevy_glsl_to_spirv::ShaderType::Fragment,
        ShaderStage::COMPUTE => bevy_glsl_to_spirv::ShaderType::Compute,
        _ => panic!("unsupported stage type"),
    }
}

#[cfg(any(
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"),
    all(target_arch = "x86_64", target_os = "macos"),
    all(target_arch = "aarch64", target_os = "android"),
    all(target_arch = "armv7", target_os = "androidabi"),
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
))]
pub fn glsl_to_spirv(
    glsl_source: &str,
    stage: ShaderStage,
    shader_defs: Option<&[String]>,
) -> Result<Vec<u32>, ShaderError> {
    bevy_glsl_to_spirv::compile(glsl_source, convert_stage(stage), shader_defs)
        .map_err(ShaderError::Compilation)
}

#[cfg(not(any(
    target_arch = "wasm32",
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"),
    all(target_arch = "x86_64", target_os = "macos"),
    all(target_arch = "aarch64", target_os = "android"),
    all(target_arch = "armv7", target_os = "androidabi"),
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
)))]
impl Into<shaderc::ShaderKind> for ShaderStage {
    fn into(self) -> shaderc::ShaderKind {
        match self {
            ShaderStage::Vertex => shaderc::ShaderKind::Vertex,
            ShaderStage::Fragment => shaderc::ShaderKind::Fragment,
            ShaderStage::Compute => shaderc::ShaderKind::Compute,
        }
    }
}

#[cfg(not(any(
    target_arch = "wasm32",
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"),
    all(target_arch = "x86_64", target_os = "macos"),
    all(target_arch = "aarch64", target_os = "android"),
    all(target_arch = "armv7", target_os = "androidabi"),
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
)))]
pub fn glsl_to_spirv(
    glsl_source: &str,
    stage: ShaderStage,
    shader_defs: Option<&[String]>,
) -> Result<Vec<u32>, ShaderError> {
    let mut compiler =
        shaderc::Compiler::new().ok_or(ShaderError::ErrorInitializingShadercCompiler)?;
    let mut options = shaderc::CompileOptions::new()
        .ok_or(ShaderError::ErrorInitializingShadercCompileOptions)?;
    if let Some(shader_defs) = shader_defs {
        for def in shader_defs.iter() {
            options.add_macro_definition(def, None);
        }
    }

    let binary_result = compiler.compile_into_spirv(
        glsl_source,
        stage.into(),
        "shader.glsl",
        "main",
        Some(&options),
    )?;

    Ok(binary_result.as_binary().to_vec())
}

fn bytes_to_words(bytes: &[u8]) -> Vec<u32> {
    let mut words = Vec::new();
    for bytes4 in bytes.chunks(4) {
        words.push(u32::from_le_bytes([
            bytes4[0], bytes4[1], bytes4[2], bytes4[3],
        ]));
    }

    words
}

/// The full "source" of a shader
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum ShaderSource {
    Spirv(Vec<u32>),
    Glsl(String),
}

impl ShaderSource {
    pub fn spirv_from_bytes(bytes: &[u8]) -> ShaderSource {
        ShaderSource::Spirv(bytes_to_words(bytes))
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct ShaderId(Uuid);

impl ShaderId {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        ShaderId(Uuid::new_v4())
    }
}

/// A shader, as defined by its [ShaderSource] and [ShaderStage]
#[derive(Clone, Debug, TypeUuid)]
#[uuid = "d95bc916-6c55-4de3-9622-37e7b6969fda"]
pub struct Shader {
    pub source: ShaderSource,
    pub stage: ShaderStage,
}

impl Shader {
    pub fn new(stage: ShaderStage, source: ShaderSource) -> Shader {
        Shader { source, stage }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_spirv(spirv: &[u8]) -> Result<Shader, ShaderError> {
        use spirv_reflect::{types::ReflectShaderStageFlags, ShaderModule};

        let module = ShaderModule::load_u8_data(spirv)
            .map_err(|msg| ShaderError::Compilation(msg.to_string()))?;
        let stage = match module.get_shader_stage() {
            ReflectShaderStageFlags::VERTEX => ShaderStage::VERTEX,
            ReflectShaderStageFlags::FRAGMENT => ShaderStage::FRAGMENT,
            other => panic!("cannot load {:?} shader", other),
        };

        Ok(Shader {
            source: ShaderSource::spirv_from_bytes(spirv),
            stage,
        })
    }

    pub fn from_glsl(stage: ShaderStage, glsl: &str) -> Shader {
        Shader {
            source: ShaderSource::Glsl(glsl.to_string()),
            stage,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_spirv(&self, macros: Option<&[String]>) -> Result<Vec<u32>, ShaderError> {
        match self.source {
            ShaderSource::Spirv(ref bytes) => Ok(bytes.clone()),
            ShaderSource::Glsl(ref source) => glsl_to_spirv(&source, self.stage, macros),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_spirv_shader(&self, macros: Option<&[String]>) -> Result<Shader, ShaderError> {
        Ok(Shader {
            source: ShaderSource::Spirv(self.get_spirv(macros)?),
            stage: self.stage,
        })
    }
}

/// All stages in a shader program
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ShaderStages {
    pub vertex: ShaderId,
    pub fragment: Option<ShaderId>,
}

/// All stages in a compute shader program
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ComputeShaderStages {
    pub compute: ShaderId,
}

#[derive(Default)]
pub struct ShaderLoader;

impl AssetLoader for ShaderLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let ext = load_context.path().extension().unwrap().to_str().unwrap();

            let shader = match ext {
                "vert" => Shader::from_glsl(ShaderStage::VERTEX, std::str::from_utf8(bytes)?),
                "frag" => Shader::from_glsl(ShaderStage::FRAGMENT, std::str::from_utf8(bytes)?),
                #[cfg(not(target_arch = "wasm32"))]
                "spv" => Shader::from_spirv(bytes)?,
                #[cfg(target_arch = "wasm32")]
                "spv" => panic!("cannot load .spv file on wasm"),
                _ => panic!("unhandled extension: {}", ext),
            };

            load_context.set_default_asset(LoadedAsset::new(shader));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["vert", "frag", "spv"]
    }
}
