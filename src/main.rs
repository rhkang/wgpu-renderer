use wgpu_renderer::engine;

fn main() {
    pollster::block_on(engine::run(None));
}
