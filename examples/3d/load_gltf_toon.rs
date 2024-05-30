//! Loads and renders a glTF file as a scene with a custom standard material.

use bevy::{
    gltf::{FromStandardMaterial, GltfLoaderSettings, GltfPlugin},
    pbr::{
        CascadeShadowConfigBuilder, DirectionalLightShadowMap, ExtendedMaterial, MaterialExtension,
    },
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};
use serde::Deserialize;
use std::f32::consts::*;

fn main() {
    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins(DefaultPlugins.set(GltfPlugin::default().add_material::<ToonMaterial>("toon")))
        .add_plugins(MaterialPlugin::<ToonMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, animate_light_direction)
        .run();
}

#[derive(Debug, Clone, TypePath, AsBindGroup, Asset, Deserialize)]
struct ToonShader {
    #[uniform(100)]
    cutoff: f32,
    #[uniform(101)]
    dark: LinearRgba,
    #[uniform(102)]
    light: LinearRgba,
}

impl FromStandardMaterial for ToonShader {
    fn from_standard_material(_: StandardMaterial, gltf_extras: Option<&str>) -> Self {
        gltf_extras
            .and_then(|s| serde_json::from_str::<ToonShader>(s).ok())
            .unwrap_or(ToonShader {
                cutoff: 0.5,
                dark: LinearRgba::rgb(0.4, 0.4, 0.4),
                light: LinearRgba::rgb(0.8, 0.8, 0.8),
            })
    }
}

impl MaterialExtension for ToonShader {
    fn fragment_shader() -> ShaderRef {
        "shaders/toon_shader.wgsl".into()
    }
}

type ToonMaterial = ExtendedMaterial<StandardMaterial, ToonShader>;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(3.0, 2.0, 0.0)
                .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
            ..default()
        },
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 250.0,
        },
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        // This is a relatively small scene, so use tighter shadow
        // cascade bounds than the default for better quality.
        // We also adjusted the shadow map to be larger since we're
        // only using a single cascade.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .into(),
        ..default()
    });
    // Note: All assets are cached by path so you cannot load the same file with different settings
    // while this handle is alive, if multiple versions are needed, load with default settings and
    // generate each material from the standard materials.
    commands.spawn(SceneBundle {
        scene: asset_server.load_with_settings(
            "models/FlightHelmet/FlightHelmet.gltf#Scene0",
            |s: &mut GltfLoaderSettings| {
                s.with_default_material::<ToonMaterial>();
            },
        ),
        transform: Transform::from_translation(Vec3::new(-1., 0., 0.))
            .with_rotation(Quat::from_rotation_y(f32::to_radians(45.0)))
            .with_scale(Vec3::splat(1.4)),
        ..default()
    });

    // The balls are created in blender using custom attributes.
    commands.spawn(SceneBundle {
        scene: asset_server.load("models/ToonBalls/toon_balls.gltf#Scene0"),
        ..default()
    });
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_seconds() * PI / 5.0,
            -FRAC_PI_4,
        );
    }
}
