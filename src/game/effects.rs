use crate::{
    constants::{ARENA_HEIGHT, ARENA_WIDTH, DEATH_EFFECT_LAYER, HIT_EFFECT_TIME_STEP},
    MainCamera, TimeScale,
};
use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    reflect::TypeUuid,
    render::{
        camera::RenderTarget, mesh::InnerMeshVertexBufferLayout, render_asset::RenderAssets,
        render_resource::*, texture::BevyDefault,
    },
    sprite::{
        ColorMaterialFlags, ColorMaterialUniform, Material2d, Material2dKey, Material2dPlugin,
        MaterialMesh2dBundle,
    },
    utils::{FixedState, Hashed},
};
use std::time::Duration;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<DeathEffectMaterial>::default())
            .init_resource::<DeathEffectTexture>()
            .insert_resource(CameraShakeTimer(Timer::from_seconds(0.02, TimerMode::Once)))
            .add_event::<CameraShakeEvent>()
            .add_startup_system(setup)
            .add_system(death_effect_system)
            .add_system(hit_effect_system)
            .add_system(camera_shake_system);
    }
}

#[derive(Resource)]
pub struct DeathEffectTexture(Handle<Image>);

impl FromWorld for DeathEffectTexture {
    fn from_world(world: &mut World) -> Self {
        let size = Extent3d {
            width: ARENA_WIDTH as u32,
            height: ARENA_HEIGHT as u32,
            depth_or_array_layers: 1,
        };
        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                usage: TextureUsages::RENDER_ATTACHMENT
                    | TextureUsages::COPY_DST
                    | TextureUsages::TEXTURE_BINDING,
            },
            ..Default::default()
        };
        image.resize(size);

        let mut images = world.resource_mut::<Assets<Image>>();
        Self(images.add(image))
    }
}

// pub type DeathEffectMaterial = ColorMaterial;
#[derive(Debug, Clone, TypeUuid, AsBindGroup)]
#[uuid = "8afb68fd-de70-4be5-be04-72f5dd29d1e2"]
#[uniform(0, ColorMaterialUniform)]
pub struct DeathEffectMaterial {
    pub color: Color,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
}

impl Default for DeathEffectMaterial {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            texture: None,
        }
    }
}

impl From<Handle<Image>> for DeathEffectMaterial {
    fn from(image: Handle<Image>) -> Self {
        Self {
            texture: Some(image),
            ..Default::default()
        }
    }
}

impl AsBindGroupShaderType<ColorMaterialUniform> for DeathEffectMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<Image>) -> ColorMaterialUniform {
        let mut flags = ColorMaterialFlags::NONE;
        if self.texture.is_some() {
            flags |= ColorMaterialFlags::TEXTURE;
        }

        ColorMaterialUniform {
            color: self.color.as_linear_rgba_f32().into(),
            flags: flags.bits(),
        }
    }
}

impl Material2d for DeathEffectMaterial {
    fn fragment_shader() -> ShaderRef {
        ColorMaterial::fragment_shader()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &Hashed<InnerMeshVertexBufferLayout, FixedState>,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(target) = descriptor
            .fragment
            .as_mut()
            .and_then(|fragment| fragment.targets[0].as_mut())
        {
            let blend = BlendComponent {
                src_factor: BlendFactor::OneMinusDst,
                dst_factor: BlendFactor::OneMinusSrc,
                operation: BlendOperation::Add,
            };
            target.blend = Some(BlendState {
                color: blend,
                alpha: blend,
            });
        }

        Ok(())
    }
}

#[derive(Component)]
pub struct DeathEffectCamera;

#[derive(Component)]
pub struct DeathEffect {
    pub timer: Timer,
    pub speed: f32,
    pub acceleration: f32,
}

fn setup(
    mut commands: Commands,
    texture: Res<DeathEffectTexture>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DeathEffectMaterial>>,
) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                priority: -1,
                target: RenderTarget::Image(texture.0.clone()),
                ..Default::default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::NONE),
            },
            ..Default::default()
        },
        UiCameraConfig { show_ui: false },
        DeathEffectCamera,
        DEATH_EFFECT_LAYER,
    ));

    let size = Vec2::new(ARENA_WIDTH, ARENA_HEIGHT);
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(shape::Quad::new(size).into()).into(),
        material: materials.add(texture.0.clone().into()),
        transform: Transform::from_xyz(0.0, 0.0, 0.1),
        ..Default::default()
    });
}

fn death_effect_system(
    mut commands: Commands,
    time: Res<Time>,
    time_scale: Res<TimeScale>,
    mut query: Query<(Entity, &mut Transform, &mut DeathEffect)>,
) {
    for (entity, mut transform, mut effect) in query.iter_mut() {
        if effect
            .timer
            .tick(Duration::from_secs_f32(time.delta_seconds() * time_scale.0))
            .just_finished()
        {
            commands.entity(entity).despawn();
            continue;
        }

        effect.speed += effect.acceleration * time.delta_seconds() * time_scale.0;
        transform.scale += effect.speed * time.delta_seconds() * time_scale.0;
    }
}

#[derive(Clone, Component)]
pub struct HitEffect {
    timer: Timer,
}

impl Default for HitEffect {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(HIT_EFFECT_TIME_STEP, TimerMode::Repeating),
        }
    }
}

fn hit_effect_system(
    mut commands: Commands,
    time: Res<Time>,
    time_scale: Res<TimeScale>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        Entity,
        &mut HitEffect,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (entity, mut effect, mut sprite, texture_atlas_handle) in query.iter_mut() {
        if effect
            .timer
            .tick(Duration::from_secs_f32(time.delta_seconds() * time_scale.0))
            .just_finished()
        {
            if let Some(texture_atlas) = texture_atlases.get(texture_atlas_handle) {
                if sprite.index + 1 < texture_atlas.len() {
                    sprite.index += 1;
                } else {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct CameraShakeTimer(Timer);

pub struct CameraShakeEvent {
    pub amplitude: Vec2,
}

fn camera_shake_system(
    mut events: EventReader<CameraShakeEvent>,
    time: Res<Time>,
    mut timer: ResMut<CameraShakeTimer>,
    mut cameras: Query<(&mut Transform, &Camera), With<MainCamera>>,
    mut camera_position: Local<Option<Vec3>>,
) {
    for (mut transform, camera) in cameras.iter_mut() {
        if camera.is_active {
            if camera_position.is_none() {
                *camera_position = Some(transform.translation);
            }

            if timer.tick(time.delta()).just_finished() {
                transform.translation = camera_position.unwrap_or_default();
            }

            for event in events.iter() {
                if timer.finished() {
                    *camera_position = Some(transform.translation);
                    transform.translation += event.amplitude.extend(0.0);
                    timer.reset();
                }
            }
        }
    }
}
