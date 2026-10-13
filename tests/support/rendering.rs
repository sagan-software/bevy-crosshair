//! Shared rendering conformance through the public API.

use super::*;

/// Exercises the optional backend through a real offscreen Bevy camera.
fn application() -> App {
    use bevy::app::PluginsState;
    use bevy::render::RenderPlugin;
    use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
    use bevy::window::ExitCondition;
    use bevy::winit::WinitPlugin;

    // Disable the OS event loop and render thread so the test can inspect pipeline state.
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(RenderPlugin {
                synchronous_pipeline_compilation: true,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            })
            .disable::<WinitPlugin>()
            .disable::<PipelinedRenderingPlugin>(),
    )
    .add_plugins(CrosshairPlugin::<HitMarker>::default());
    // Async plugin initialization must complete before calling finish and cleanup.
    while app.plugins_state() == PluginsState::Adding {
        bevy::tasks::tick_global_task_pools_on_main_thread();
    }
    app.finish();
    app.cleanup();
    app
}

/// Creates an image target and explicitly selects its camera for UI rendering.
fn target(app: &mut App) -> Entity {
    use bevy::camera::RenderTarget;
    use bevy::render::render_resource::TextureFormat;
    // An image camera needs the default-UI marker because there is no primary window.
    let image = app
        .world_mut()
        .get_resource_mut::<Assets<Image>>()
        .expect("image assets are installed")
        .add(Image::new_target_texture(
            256,
            256,
            TextureFormat::Rgba8UnormSrgb,
            None,
        ));
    app.world_mut().spawn((
        Camera2d,
        IsDefaultUiCamera,
        RenderTarget::Image(image.into()),
    ));
    app.world_mut()
        .spawn((
            Crosshair::default(),
            CrosshairInput {
                viewport: Vec2::splat(256.0),
                ..default()
            },
        ))
        .id()
}

/// Runs every style and marker upload through an actual GPU-backed Bevy app.
pub(crate) fn run() -> App {
    // Finish plugin initialization before creating image assets and target entities.
    let mut app = application();
    let target = target(&mut app);
    render_modes(&mut app, target);
    render_extensions(&mut app, target);
    // Confirm material compilation before despawning, so an empty scene cannot pass.
    for _ in 0..20 {
        app.update();
    }
    await_material(&mut app);
    // A host may remove layout components before despawning its crosshair.
    app.world_mut()
        .get_entity_mut(target)
        .expect("target is alive")
        .remove::<Node>();
    app.update();
    app.world_mut().despawn(target);
    app.update();
    app
}

/// Fails if the material shader cannot create an actual GPU pipeline.
fn await_material(app: &mut App) {
    use bevy::render::RenderApp;
    use bevy::render::render_resource::CachedPipelineState;
    use bevy::render::render_resource::PipelineCache;
    use bevy::render::render_resource::PipelineDescriptor;
    // Bound shader initialization so a missing asset fails instead of hanging the suite.
    let started = std::time::Instant::now();
    loop {
        app.update();
        let cache = app
            .sub_app(RenderApp)
            .world()
            .get_resource::<PipelineCache>()
            .expect("render pipeline cache exists");
        // Inspect the UI material specifically; an unrelated ready camera pipeline is insufficient.
        if let Some(pipeline) = cache.pipelines().find(|pipeline| {
            matches!(&pipeline.descriptor, PipelineDescriptor::RenderPipelineDescriptor(descriptor)
                if descriptor.label.as_deref() == Some("ui_material_pipeline"))
        }) {
            let state = &pipeline.state;
            assert!(
                !matches!(state, CachedPipelineState::Err(_)),
                "material pipeline failed: {state:?}"
            );
            if matches!(state, CachedPipelineState::Ok(_)) {
                return;
            }
        }
        // Yield between attempts so asynchronous asset tasks can publish their results.
        assert!(
            started.elapsed() < Duration::from_secs(30),
            "material pipeline did not become ready"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Exercises changed material values and stable updates for every rendering mode.
fn render_modes(app: &mut App, target: Entity) {
    let styles = 0..10;
    // All ten identifiers must create a material; each hit adds rotated rectangles.
    for code in styles {
        app.world_mut()
            .get_entity_mut(target)
            .expect("the target is alive")
            .insert(
                Crosshair::default()
                    .with_style(Style::try_from(code).expect("the fixture supplies a valid value")),
            );
        app.world_mut().trigger(Hit {
            entity: target,
            metadata: HitMarker::default(),
        });
        // The second update covers stable material synchronization.
        app.update();
        app.update();
    }
}

/// Submits the new visual extensions through the same material pipeline.
fn render_extensions(app: &mut App, target: Entity) {
    render_layers(app, target);
    // Filled interiors, circular dots, and each animated marker use the same backend.
    app.world_mut()
        .get_entity_mut(target)
        .expect("the target is alive")
        .insert(
            "cl_crosshairdot true; cl_crosshair_gap 24"
                .parse::<Crosshair>()
                .expect("the filled fixture is valid")
                .with_fill(LinearRgba::BLUE.with_alpha(0.25))
                .with_dot_shape(bevy_crosshair::DotShape::Circle),
        );
    for shape in [
        bevy_crosshair::MarkerShape::Cross,
        bevy_crosshair::MarkerShape::Plus,
        bevy_crosshair::MarkerShape::Chevron,
        bevy_crosshair::MarkerShape::Diamond,
        bevy_crosshair::MarkerShape::Ring,
    ] {
        for animation in [
            bevy_crosshair::MarkerAnimation::Fade,
            bevy_crosshair::MarkerAnimation::Expand,
            bevy_crosshair::MarkerAnimation::Contract,
            bevy_crosshair::MarkerAnimation::Pulse,
        ] {
            let mut marker = HitMarker::default()
                .with_shape(shape)
                .with_animation(animation);
            // Submit a visible frame directly; a real hit would restart the fade-in clock.
            marker.advance(Duration::from_millis(90));
            app.world_mut()
                .get_entity_mut(target)
                .expect("the target is alive")
                .insert(marker);
            app.update();
        }
    }
}

/// Submits arcs and fading bars, which use their own shader tags.
fn render_layers(app: &mut App, target: Entity) {
    // One frame with every custom mark reaches the encoder and the shader.
    app.world_mut()
        .get_entity_mut(target)
        .expect("the target is alive")
        .insert(super::layered());
    app.update();
}
