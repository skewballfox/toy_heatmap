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

@group(0) @binding(2) var<storage> kernel: array<f32>;
@group(0) @binding(3) var<uniform> data_size: vec2<f32>;
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    const kernel_size = 8.0;
    var color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var intensity: f32 = 0.0;

    for (var y: f32 = -kernel_size; y <= kernel_size; y += 1.0) {
        let offsettedY = y + in.color_index.y;
        if (offsettedY >= 0.0 && offsettedY <= data_size.y) {
            for (var x: f32 = -kernel_size; x <= kernel_size; x += 1.0) {
                let offsettedX = x + in.color_index.x;
                if (offsettedX >= 0.0 && offsettedX <= data_size.x) {
                    let indexY = u32(y + kernel_size);
                    let indexX = u32(x + kernel_size);
                    let index = indexY * (u32(kernel_size) * 2 + 1) + indexX;

                    let tex_coord = vec2(offsettedX / data_size.x, offsettedY / data_size.y);
                    let gaussian_v = kernel[index];
                    let c: vec4<f32> = textureSampleLevel(colormap_texture, colormap_sampler, tex_coord, 0.0);
                    color += c * gaussian_v;
                    intensity += gaussian_v;
                }
            }
        }
    }

    color /= intensity;
    color.w = 1.0;
    return color;

}
