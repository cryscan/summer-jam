use crate::{config::HIT_EFFECT_TIME_STEP, TimeScale};
use bevy::{
    prelude::*,
    sprite::{
        Material2d,
        Material2dPlugin, Material2dKey, ColorMaterialUniform, ColorMaterialFlags,
    }, render::{render_resource::{AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError, BlendState, BlendComponent, BlendFactor, BlendOperation, ShaderRef, AsBindGroupShaderType}, mesh::InnerMeshVertexBufferLayout, render_asset::RenderAssets}, reflect::TypeUuid, utils::{Hashed, FixedState},
};
use std::time::Duration;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraShakeTimer(Timer::from_seconds(0.02, false)))
            .add_event::<CameraShakeEvent>()
            .add_plugin(Material2dPlugin::<DeathEffectMaterial>::default())
            .add_system(death_effect_system)
            .add_system(hit_effect_system)
            .add_system(camera_shake_system);
    }
}

// pub type DeathEffectMaterial = ColorMaterial;
#[derive(Debug, Clone, TypeUuid, AsBindGroup)]
#[uuid = "8afb68fd-de70-4be5-be04-72f5dd29d1e2"]
#[uniform(0, ColorMaterialUniform)]
pub struct DeathEffectMaterial{
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
        _key: Material2dKey<Self>

    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment {
            if let Some(t) = &mut fragment.targets[0] {
                t.blend = Some(BlendState {
                    color: BlendComponent {
                        src_factor: BlendFactor::OneMinusDst,
                        dst_factor: BlendFactor::OneMinusSrcAlpha,
                        operation: BlendOperation::Add,
                    },
                    alpha: BlendComponent::OVER,
                });
            }
        }

        Ok(())
    }
}


#[derive(Clone, Component)]
pub struct DeathEffect {
    pub timer: Timer,
    pub speed: f32,
    pub acceleration: f32,
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
            timer: Timer::from_seconds(HIT_EFFECT_TIME_STEP, true),
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

#[derive(Deref, DerefMut)]
pub struct CameraShakeTimer(Timer);

pub struct CameraShakeEvent {
    pub amplitude: Vec2,
}

fn camera_shake_system(
    mut events: EventReader<CameraShakeEvent>,
    time: Res<Time>,
    mut timer: ResMut<CameraShakeTimer>,
    mut cameras: Query<(&mut Transform, &Camera)>,
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
