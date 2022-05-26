use crate::utils::*;
use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::{PrepareAssetError, RenderAsset, RenderAssets},
        render_resource::{
            std140::{AsStd140, Std140},
            *,
        },
        renderer::RenderDevice,
    },
    sprite::{
        ColorMaterialFlags, ColorMaterialUniformData, GpuColorMaterial, Material2d,
        Material2dPipeline,
    },
};

#[derive(Component, Debug, Clone, TypeUuid, Deref, DerefMut)]
#[uuid = "8afb68fd-de70-4be5-be04-72f5dd29d1e2"]
pub struct ColorReversionMaterial(ColorMaterial);

impl From<Handle<Image>> for ColorReversionMaterial {
    fn from(image: Handle<Image>) -> Self {
        Self(image.into())
    }
}

impl RenderAsset for ColorReversionMaterial {
    type ExtractedAsset = ColorReversionMaterial;
    type PreparedAsset = GpuColorMaterial;
    type Param = (
        SRes<RenderDevice>,
        SRes<Material2dPipeline<Self>>,
        SRes<RenderAssets<Image>>,
    );

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        material: Self::ExtractedAsset,
        (render_device, pipeline, gpu_images): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let (texture_view, sampler) = if let Some(result) = pipeline
            .mesh2d_pipeline
            .get_image_texture(gpu_images, &material.texture)
        {
            result
        } else {
            return Err(PrepareAssetError::RetryNextUpdate(material));
        };

        let mut flags = ColorMaterialFlags::NONE;
        if material.texture.is_some() {
            flags |= ColorMaterialFlags::TEXTURE;
        }

        let value = ColorMaterialUniformData {
            color: material.color.as_linear_rgba_f32().into(),
            flags: flags.bits(),
        };
        let value_std140 = value.as_std140();

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("color_material_uniform_buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: value_std140.as_bytes(),
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(texture_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(sampler),
                },
            ],
            label: Some("color_material_bind_group"),
            layout: &pipeline.material2d_layout,
        });

        let texture = material.texture.clone();
        Ok(GpuColorMaterial {
            buffer,
            bind_group,
            flags,
            texture,
        })
    }
}

impl Material2d for ColorReversionMaterial {
    fn bind_group(material: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &material.bind_group
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        ColorMaterial::bind_group_layout(render_device)
    }

    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        ColorMaterial::fragment_shader(asset_server)
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment {
            fragment.targets[0].blend = Some(BlendState {
                color: BlendComponent {
                    src_factor: BlendFactor::OneMinusDst,
                    dst_factor: BlendFactor::OneMinusSrcAlpha,
                    operation: BlendOperation::Add,
                },
                alpha: BlendComponent::OVER,
            });
        }

        Ok(())
    }
}

#[derive(Component, Clone)]
pub struct DeathEffect {
    pub timer: Timer,
    pub speed: f32,
}

pub fn death_ring_system(
    mut commands: Commands,
    time: Res<Time>,
    time_scale: Res<TimeScale>,
    mut query: Query<(Entity, &mut Transform, &mut DeathEffect)>,
) {
    for (entity, mut transform, mut effect) in query.iter_mut() {
        if effect.timer.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        transform.scale += effect.speed * (1.0 + time.delta_seconds()).powf(2.0) * time_scale.0;
    }
}
