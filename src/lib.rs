pub mod app;
pub mod assets;
pub mod render;
pub mod sim;

pub use render::model;
pub use render::texture;

pub fn run() -> anyhow::Result<()> {
    app::run()
}
