// Debug-only validation helpers for the renderer.
// These help catch mismatches between Rust uniform structs and WGSL shader
// expectations early during development. They compile to no-ops in release.

#[cfg(debug_assertions)]
pub fn validate_uniform_struct_sizes() {
    use std::mem::size_of;

    let camera_size = size_of::<crate::render::camera::CameraUniform>();
    let light_size = size_of::<crate::render::state::LightUniform>();

    debug_assert_eq!(
        camera_size, 80,
        "CameraUniform size ({}) != 80 bytes (shader expectation)",
        camera_size
    );
    debug_assert_eq!(
        light_size, 32,
        "LightUniform size ({}) != 32 bytes (shader expectation)",
        light_size
    );
}

#[cfg(not(debug_assertions))]
pub fn validate_uniform_struct_sizes() {}
