pub mod app;
pub mod assets;
pub mod render;
pub mod sim;

// pub fn run() -> anyhow::Result<()> {
//     app::run()
// }

// use std::{iter, sync::Arc};

// use model::{Model, Vertex};
// use wgpu::util::DeviceExt;
// use winit::{
//     application::ApplicationHandler,
//     event::*,
//     event_loop::{ActiveEventLoop, EventLoop},
//     keyboard::{KeyCode, PhysicalKey},
//     window::Window,
// };

// // u32s for 4-byte alignment
// #[repr(C)]
// #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// struct LightUniform {
//     position: [f32; 3],
//     _padding: u32,
//     colour: [f32; 3],
//     _padding2: u32,
// }

// struct Instance {
//     position: cgmath::Vector3<f32>,
//     rotation: cgmath::Quaternion<f32>,
// }

// #[repr(C)]
// #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// struct InstanceRaw {
//     model: [[f32; 4]; 4],
//     normal: [[f32; 3]; 3],
// }

// #[repr(C)]
// #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// pub enum CompareFunction {
//     Undefined = 0,
//     Never = 1,
//     Less = 2,
//     Equal = 3,
//     LessEqual = 4,
//     Greater = 5,
//     NotEqual = 6,
//     GreaterEqual = 7,
//     Always = 8,
// }

// impl Instance {
//     fn to_raw(&self) -> InstanceRaw {
//         InstanceRaw {
//             model: (cgmath::Matrix4::from_translation(self.position)
//                 * cgmath::Matrix4::from(self.rotation))
//             .into(),

//             normal: cgmath::Matrix3::from(self.rotation).into(),
//         }
//     }
// }

// impl model::Vertex for InstanceRaw {
//     fn desc() -> wgpu::VertexBufferLayout<'static> {
//         use std::mem;
//         wgpu::VertexBufferLayout {
//             array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
//             step_mode: wgpu::VertexStepMode::Instance,
//             attributes: &[
//                 wgpu::VertexAttribute {
//                     offset: 0,
//                     shader_location: 5,
//                     format: wgpu::VertexFormat::Float32x4,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
//                     shader_location: 6,
//                     format: wgpu::VertexFormat::Float32x4,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
//                     shader_location: 7,
//                     format: wgpu::VertexFormat::Float32x4,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
//                     shader_location: 8,
//                     format: wgpu::VertexFormat::Float32x4,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
//                     shader_location: 9,
//                     format: wgpu::VertexFormat::Float32x3,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
//                     shader_location: 10,
//                     format: wgpu::VertexFormat::Float32x3,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
//                     shader_location: 11,
//                     format: wgpu::VertexFormat::Float32x3,
//                 },
//             ],
//         }
//     }
// }

// pub fn run() -> anyhow::Result<()> {
//     #[cfg(not(target_arch = "wasm32"))]
//     {
//         env_logger::init();
//     }
//     #[cfg(target_arch = "wasm32")]
//     {
//         console_log::init_with_level(log::Level::Info).unwrap_throw();
//     }

//     let event_loop = EventLoop::with_user_event().build()?;
//     let mut app = App::new(
//         #[cfg(target_arch = "wasm32")]
//         &event_loop,
//     );
//     event_loop.run_app(&mut app)?;

//     Ok(())
// }
