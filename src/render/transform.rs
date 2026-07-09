use cgmath::{Matrix3, Matrix4, Quaternion, Rotation3, SquareMatrix, Vector3};

use super::model::Vertex;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformRaw {
    pub model: [[f32; 4]; 4],
    pub normal: [[f32; 3]; 3],
}

impl Transform {
    pub fn new(position: Vector3<f32>, rotation: Quaternion<f32>, scale: Vector3<f32>) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn identity() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::from_angle_z(cgmath::Deg(0.0)),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn to_raw(&self) -> TransformRaw {
        let model = Matrix4::from_translation(self.position)
            * Matrix4::from(self.rotation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);

        let inverse_scale = Matrix3::from_diagonal(Vector3::new(
            1.0 / self.scale.x,
            1.0 / self.scale.y,
            1.0 / self.scale.z,
        ));
        let normal_matrix = Matrix3::from(self.rotation) * inverse_scale;

        TransformRaw {
            model: model.into(),
            normal: normal_matrix.into(),
        }
    }
}

impl Vertex for TransformRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<TransformRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
