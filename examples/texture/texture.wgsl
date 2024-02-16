@group(0) @binding(0)
var v_texture: texture_2d<f32>;

@group(0) @binding(1)
var s_texture: sampler;

struct Uniform {
    width: f32,
    height: f32,
}

@group(1) @binding(0)
var<uniform> windowSize: Uniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;

    var pos = model.position;
    var ratio = windowSize.width / windowSize.height;
    pos.y *= ratio;

    out.clip_position = vec4<f32>(pos, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(v_texture, s_texture, in.tex_coords);
}
