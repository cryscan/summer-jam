use crate::{
    constants::{ARENA_HEIGHT, ARENA_WIDTH, BACKGROUND_SHADER},
    TimeScale,
};
use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<BackgroundMaterial>::default())
            .add_startup_system(setup)
            .add_system(update);
    }
}

#[derive(Debug, Clone, TypeUuid, AsBindGroup)]
#[uuid = "b4f62ce0-3227-4d22-a027-50eed7dbc5f5"]
struct BackgroundMaterial {
    #[uniform(0)]
    time: f32,
    #[uniform(0)]
    velocity: Vec3,
}

impl Material2d for BackgroundMaterial {
    fn vertex_shader() -> ShaderRef {
        BACKGROUND_SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        BACKGROUND_SHADER.into()
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
) {
    let size = Vec2::new(ARENA_WIDTH + 16.0, ARENA_HEIGHT + 16.0);
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::new(size))).into(),
        transform: Transform::from_xyz(0.0, 0.0, -0.09),
        material: materials.add(BackgroundMaterial {
            time: 0.0,
            velocity: Vec3::new(2.0, 1.0, 0.0),
        }),
        ..Default::default()
    });
}

fn update(
    time: Res<Time>,
    time_scale: Res<TimeScale>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
) {
    for (_, mut material) in materials.iter_mut() {
        material.time += time.delta_seconds() * time_scale.0;
    }
}
