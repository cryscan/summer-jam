use crate::config::{ARENA_HEIGHT, ARENA_WIDTH};
use bevy::{
    ecs::system::lifetimeless::SRes,
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::{PrepareAssetError, RenderAsset},
        render_resource::{
            std140::{AsStd140, Std140},
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer,
            BufferBindingType, BufferInitDescriptor, BufferSize, BufferUsages, ShaderStages,
        },
        renderer::RenderDevice,
    },
    sprite::{Material2d, Material2dPipeline, Material2dPlugin, MaterialMesh2dBundle},
};

const BACKGROUND_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1038182793939033549);

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        shaders.set_untracked(
            BACKGROUND_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("shaders/background.wgsl").replace("\r\n", "\n")),
        );

        app.add_plugin(Material2dPlugin::<BackgroundMaterial>::default())
            .add_startup_system(setup)
            .add_system(update);
    }
}

#[derive(Component, Debug, Clone, TypeUuid, AsStd140)]
#[uuid = "b4f62ce0-3227-4d22-a027-50eed7dbc5f5"]
struct BackgroundMaterial {
    time: f32,
    velocity: Vec3,
}

impl RenderAsset for BackgroundMaterial {
    type ExtractedAsset = BackgroundMaterial;
    type PreparedAsset = GpuBackgroundMaterial;
    type Param = (SRes<RenderDevice>, SRes<Material2dPipeline<Self>>);

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, pipeline): &mut bevy::ecs::system::SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: None,
            contents: extracted_asset.as_std140().as_bytes(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            layout: &pipeline.material2d_layout,
        });

        Ok(GpuBackgroundMaterial {
            _buffer: buffer,
            bind_group,
        })
    }
}

impl Material2d for BackgroundMaterial {
    fn vertex_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.get_handle(BACKGROUND_SHADER_HANDLE))
    }

    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.get_handle(BACKGROUND_SHADER_HANDLE))
    }

    fn bind_group(material: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &material.bind_group
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(Self::std140_size_static() as u64),
                },
                count: None,
            }],
        })
    }
}

#[derive(Clone)]
struct GpuBackgroundMaterial {
    _buffer: Buffer,
    bind_group: BindGroup,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
) {
    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, -0.09),
            scale: Vec3::new(ARENA_WIDTH, ARENA_HEIGHT, 1.0),
            ..Default::default()
        },
        material: materials
            .add(BackgroundMaterial {
                time: 0.0,
                velocity: Vec3::new(2.0, 1.0, 0.0),
            })
            .into(),
        ..Default::default()
    });
}

fn update(time: Res<Time>, mut materials: ResMut<Assets<BackgroundMaterial>>) {
    for (_, mut material) in materials.iter_mut() {
        material.time += time.delta_seconds();
    }
}
