use bevy::{
    anti_alias::smaa::smaa,
    core_pipeline::{Core2d, Core2dSystems, FullscreenShader, tonemapping::tonemapping},
    prelude::*,
    render::{
        RenderApp, RenderStartup,
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, ViewQuery},
        view::ViewTarget,
    },
};

const SHADER_ASSET_PATH: &str = "shaders/crt_shader.wesl";
pub struct CrtPlugin;

impl Plugin for CrtPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // The settings will be a component that lives in the main world but will
            // be extracted to the render world every frame.
            // This makes it possible to control the effect from the main world.
            // This plugin will take care of extracting it automatically.
            // It's important to derive [`ExtractComponent`] on [`PostProcessingSettings`]
            // for this plugin to work correctly.
            ExtractComponentPlugin::<PostProcessSettings>::default(),
            // The settings will also be the data used in the shader.
            // This plugin will prepare the component for the GPU by creating a uniform buffer
            // and writing the data to that buffer every frame.
            UniformComponentPlugin::<PostProcessSettings>::default(),
        ));

        // Add the update system to animate the shader effect
        app.add_systems(Update, update_settings);

        // We need to get the render app from the main app
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(RenderStartup, init_post_process_pipeline)
            // The pass runs once per 2D camera with `PostProcessSettings`, on the tonemapped
            // image.
            .add_systems(
                Core2d,
                post_process
                    .after(tonemapping)
                    .after(smaa)
                    .in_set(Core2dSystems::PostProcess),
            );
    }
}

fn post_process(
    // This makes sure the pass only runs on cameras with the PostProcessSettings component.
    // As there could be multiple post processing components sent to the GPU (one per camera),
    // we need to get the index of the one that is associated with the current view.
    view: ViewQuery<(
        &ViewTarget,
        &PostProcessSettings,
        &DynamicUniformIndex<PostProcessSettings>,
    )>,
    post_process_pipeline: Option<Res<PostProcessPipeline>>,
    // The pipeline cache is a cache of all previously created pipelines.
    // It is required to avoid creating a new pipeline each frame,
    // which is expensive due to shader compilation.
    pipeline_cache: Res<PipelineCache>,
    settings_uniforms: Res<ComponentUniforms<PostProcessSettings>>,
    mut render_context: RenderContext,
) {
    let Some(post_process_pipeline) = post_process_pipeline else {
        return;
    };
    let (view_target, _post_process_settings, settings_index) = view.into_inner();

    // Get the pipeline from the cache
    let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id)
    else {
        return;
    };

    // Get the settings uniform binding
    let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
        return;
    };

    // This will start a new "post process write", obtaining two texture
    // views from the view target - a `source` and a `destination`.
    // `source` is the "current" main texture and you _must_ write into
    // `destination` because calling `post_process_write()` on the
    // [`ViewTarget`] will internally flip the [`ViewTarget`]'s main
    // texture to the `destination` texture. Failing to do so will cause
    // the current main texture information to be lost.
    let post_process = view_target.post_process_write();

    // The bind_group gets created each frame, because each post_process_write
    // alternates the source/destination.
    let bind_group = render_context.render_device().create_bind_group(
        "post_process_bind_group",
        &pipeline_cache.get_bind_group_layout(&post_process_pipeline.layout),
        // It's important for this to match the BindGroupLayout defined in the PostProcessPipeline
        &BindGroupEntries::sequential((
            // Make sure to use the source view
            post_process.source,
            // Use the sampler created for the pipeline
            &post_process_pipeline.sampler,
            // Set the settings binding
            settings_binding.clone(),
        )),
    );

    // Begin the render pass
    let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("post_process_pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            // We need to specify the post process destination view here
            // to make sure we write to the appropriate texture.
            view: post_process.destination,
            depth_slice: None,
            resolve_target: None,
            ops: Operations::default(),
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });

    // This is mostly just wgpu boilerplate for drawing a fullscreen triangle,
    // using the pipeline/bind_group created above
    render_pass.set_render_pipeline(pipeline);
    // By passing in the index of the post process settings on this view, we ensure
    // that in the event that multiple settings were sent to the GPU (as would be the
    // case with multiple cameras), we use the correct one.
    render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
    render_pass.draw(0..3, 0..1);
}

// This contains global data used by the render pipeline. This will be created once on startup.
#[derive(Resource)]
struct PostProcessPipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

fn init_post_process_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    // We need to define the bind group layout used for our pipeline
    let layout = BindGroupLayoutDescriptor::new(
        "post_process_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            // The layout entries will only be visible in the fragment stage
            ShaderStages::FRAGMENT,
            (
                // The screen texture
                texture_2d(TextureSampleType::Float { filterable: true }),
                // The sampler that will be used to sample the screen texture
                sampler(SamplerBindingType::Filtering),
                // The settings uniform that will control the effect
                uniform_buffer::<PostProcessSettings>(true),
            ),
        ),
    );

    // We can create the sampler here since it won't change at runtime and doesn't depend on the view
    let sampler = render_device.create_sampler(&SamplerDescriptor::default());

    // This will add the pipeline to the cache and queue its creation
    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("post_process_pipeline".into()),
        layout: vec![layout.clone()],
        // This will setup a fullscreen triangle for the vertex state
        vertex: fullscreen_shader.to_vertex_state(),
        fragment: Some(FragmentState {
            shader: asset_server.load(SHADER_ASSET_PATH),
            targets: vec![Some(ColorTargetState {
                // Use Rgba16Float to match the HDR camera format
                format: TextureFormat::Rgba16Float,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    });

    commands.insert_resource(PostProcessPipeline {
        layout,
        sampler,
        pipeline_id,
    });
}

// This is the component that will get passed to the shader
#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
#[extract_app(RenderApp)]
pub struct PostProcessSettings {
    pub intensity: f32, // Controls the overall CRT effect strength
    pub time: f32,      // Time value for animations
    pub ambient_light: f32, // Controls background brightness/glow
                        // // WebGL2 structs must be 16 byte aligned.
                        // #[cfg(feature = "webgl2")]
                        // _webgl2_padding: Vec3,
}

// Update the shader parameters to control and animate the CRT effect
fn update_settings(mut settings: Query<&mut PostProcessSettings>, time: Res<Time>) {
    for mut setting in &mut settings {
        // Update the time value for shader animations
        setting.time = time.elapsed_secs();
    }
}
