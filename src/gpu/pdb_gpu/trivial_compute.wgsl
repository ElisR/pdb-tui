// Following constants should be prepended during `wgpu::ShaderSource::Wgsl`
// const grid_width: u32 = 1u;
// const grid_height: u32 = 2u;

const grid_size: u32 = grid_width * grid_height;

//! Post-processing compute shader

@group(0) @binding(0) var input_texture: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, write>;

// Stupid compute shader that should act as the identity

@compute @workgroup_size(1, 1)
fn main_1x1(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let coord = global_invocation_id.xy;
    let in_texel = textureLoad(input_texture, vec2<i32>(coord));
    textureStore(output_texture, vec2<i32>(coord), in_texel);
}

var<workgroup> workgroup_data: array<vec4<f32>, grid_size>;

@compute @workgroup_size(grid_width, grid_height)
fn main(
    @builtin(global_invocation_id) id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>
) {
    let coord = id.xy;

    let index = local_id.x + grid_width * local_id.y;
    let in_texel = textureLoad(input_texture, vec2<i32>(coord));
    workgroup_data[index] = in_texel;
    workgroupBarrier();

    var sum: f32 = 0.0;
    for (var i: u32 = 0u; i < grid_size; i++) {
        sum += workgroup_data[i].w;
    }
    let val = sum / f32(grid_size);
    let out_texel = vec4<f32>(in_texel.x, in_texel.y, in_texel.z, val);
    textureStore(output_texture, vec2<i32>(workgroup_id.xy), out_texel);
}
