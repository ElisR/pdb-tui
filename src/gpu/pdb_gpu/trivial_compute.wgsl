//! Post-processing compute shader

@group(0) @binding(0) var input_texture: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, write>;

// Stupid compute shader that should act as the identity

@compute @workgroup_size(1, 1)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    // let size = vec2<u32>(textureDimensions(r_output_texture));
    let coord = global_invocation_id.xy;

    // Can access x, y, z & w
    let in_texel = textureLoad(input_texture, vec2<i32>(coord));

    textureStore(
        output_texture,
        vec2<i32>(coord),
        in_texel,
    );
}
