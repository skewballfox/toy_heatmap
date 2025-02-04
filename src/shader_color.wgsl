// Vertex shader
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color_index: vec2<f32>,
}

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.color_index = in.tex_coords;
    return out;
}


@group(0) @binding(0) var colormap_texture: texture_2d<f32>;
@group(0) @binding(1) var colormap_sampler: sampler;
//@group(0) @binding(2) var data_texture: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(colormap_texture, colormap_sampler, in.color_index);

}
