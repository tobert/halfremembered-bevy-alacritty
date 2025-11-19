use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_graph::{self, Node, NodeRunError, RenderGraph, RenderGraphContext, RenderLabel},
        render_resource::*,
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::GpuImage,
        Render, RenderApp, RenderSet,
    },
};
use std::borrow::Cow;
use crate::gpu_types::{GpuTerminalCell, TerminalUniforms};
use crate::gpu_prep::TerminalCpuBuffer;
use crate::renderer::TerminalTexture;
use crate::atlas::GlyphAtlas;
use log::{info, warn};

#[derive(Resource, ExtractResource, Clone)]
pub struct ExtractedTerminalData {
    pub cells: Vec<GpuTerminalCell>,
    pub texture_handle: Handle<Image>,
    pub atlas_texture_handle: Handle<Image>,
    pub term_cols: u32,
    pub term_rows: u32,
    pub cell_width: u32,
    pub cell_height: u32,
    pub atlas_cols: u32,
    pub atlas_rows: u32,
}

#[derive(Resource)]
pub struct TerminalGpuResources {
    pub cell_buffer: Buffer,
    pub uniform_buffer: Buffer,
    pub bind_group: BindGroup,
    pub pipeline_id: CachedComputePipelineId,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct TerminalComputeLabel;

pub struct TerminalComputePlugin;

impl Plugin for TerminalComputePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<ExtractedTerminalData>::default());
        app.add_systems(PostUpdate, update_extraction_resource);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<TerminalComputePipeline>()
            .add_systems(
                Render,
                (
                    prepare_gpu_resources.in_set(RenderSet::Prepare),
                ),
            );
            
        // Add node to graph
        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(TerminalComputeLabel, TerminalNode);
        render_graph.add_node_edge(TerminalComputeLabel, bevy::render::graph::CameraDriverLabel);
    }
}

fn update_extraction_resource(
    mut commands: Commands,
    cpu_buffer: Res<TerminalCpuBuffer>,
    term_texture: Option<Res<TerminalTexture>>,
    atlas: Option<Res<GlyphAtlas>>,
    term_state: Option<Res<crate::terminal::TerminalState>>,
) {
    if let (Some(texture), Some(atlas), Some(state)) = (term_texture, atlas, term_state) {
        let atlas_cols = atlas.atlas_width / atlas.cell_width;
        let atlas_rows = atlas.atlas_height / atlas.cell_height;
        
        if let Some(atlas_handle) = &atlas.texture_handle {
            commands.insert_resource(ExtractedTerminalData {
                cells: cpu_buffer.cells.clone(),
                texture_handle: texture.handle.clone(),
                atlas_texture_handle: atlas_handle.clone(),
                term_cols: state.cols as u32,
                term_rows: state.rows as u32,
                cell_width: atlas.cell_width,
                cell_height: atlas.cell_height,
                atlas_cols,
                atlas_rows,
            });
        }
    }
}

#[derive(Resource)]
pub struct TerminalComputePipeline {
    pub layout: BindGroupLayout,
    pub shader: Handle<Shader>,
}

impl FromWorld for TerminalComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(
            Some("terminal_compute_layout"),
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None, // Dynamic size array
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        );
        
        let shader = world.resource::<AssetServer>().load("shaders/terminal.wgsl");

        Self { layout, shader }
    }
}

fn prepare_gpu_resources(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
    compute_pipeline: Res<TerminalComputePipeline>,
    extracted: Option<Res<ExtractedTerminalData>>,
    gpu_images: Res<RenderAssets<GpuImage>>,
) {
    let Some(data) = extracted else { return };
    
    // 1. Uniforms
    let uniforms = TerminalUniforms {
        term_cols: data.term_cols,
        term_rows: data.term_rows,
        cell_width: data.cell_width,
        cell_height: data.cell_height,
        atlas_cols: data.atlas_cols,
        atlas_rows: data.atlas_rows,
        _padding: [0, 0],
    };
    let uniform_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("terminal_uniforms"),
        contents: bytemuck::bytes_of(&uniforms),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });

    // 2. Cell Grid
    let cell_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("terminal_grid"),
        contents: bytemuck::cast_slice(&data.cells),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    });

    // 3. Textures (Target)
    let Some(output_gpu_image) = gpu_images.get(&data.texture_handle) else { return };
    let Some(atlas_gpu_image) = gpu_images.get(&data.atlas_texture_handle) else { return };

    // 4. Pipeline
    let pipeline_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some(Cow::Borrowed("terminal_compute")),
        layout: vec![compute_pipeline.layout.clone()],
        push_constant_ranges: vec![],
        shader: compute_pipeline.shader.clone(),
        shader_defs: vec![],
        entry_point: Some(Cow::Borrowed("main")),
        zero_initialize_workgroup_memory: false,
    });

    // 5. Bind Group
    let bind_group = render_device.create_bind_group(
        Some("terminal_bind_group"),
        &compute_pipeline.layout,
        &[
            BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: cell_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(&atlas_gpu_image.texture_view),
            },
            BindGroupEntry {
                binding: 3,
                resource: BindingResource::TextureView(&output_gpu_image.texture_view),
            },
        ],
    );

    commands.insert_resource(TerminalGpuResources {
        cell_buffer,
        uniform_buffer,
        bind_group,
        pipeline_id,
    });
}

struct TerminalNode;
impl Node for TerminalNode {
     fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let Some(gpu_resources) = world.get_resource::<TerminalGpuResources>() else {
            // warn!("TerminalNode: Missing GpuResources");
            return Ok(());
        };
        let pipeline_cache = world.resource::<PipelineCache>();
        let Some(pipeline) = pipeline_cache.get_compute_pipeline(gpu_resources.pipeline_id) else {
            use std::sync::atomic::{AtomicU32, Ordering};
            static WAITING_COUNT: AtomicU32 = AtomicU32::new(0);
            let c = WAITING_COUNT.fetch_add(1, Ordering::Relaxed);
            if c % 60 == 0 {
                 info!("⏳ TerminalNode: Waiting for pipeline compilation... ({})", c);
            }
            return Ok(());
        };
        let extracted = world.resource::<ExtractedTerminalData>();

        // One-time log
        use std::sync::atomic::{AtomicBool, Ordering};
        static LOGGED: AtomicBool = AtomicBool::new(false);
        if !LOGGED.swap(true, Ordering::Relaxed) {
            info!("🚀 TerminalNode: FIRST DISPATCH! Pipeline ready.");
        }

        // Calculate dispatch size
        // One thread per pixel
        let width = extracted.term_cols * extracted.cell_width;
        let height = extracted.term_rows * extracted.cell_height;
        let workgroup_size = 8;
        let x_groups = (width + workgroup_size - 1) / workgroup_size;
        let y_groups = (height + workgroup_size - 1) / workgroup_size;

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor {
                label: Some("terminal_compute_pass"),
                ..default()
            });

        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &gpu_resources.bind_group, &[]);
        pass.dispatch_workgroups(x_groups, y_groups, 1);

        Ok(())
    }
}