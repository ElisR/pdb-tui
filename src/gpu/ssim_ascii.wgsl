//! Compute shader for picking the best SSIM values
// TODO Consider merging this into the other `wgsl` file for simplicity

const ASCII_START: u32 = 32u;
const ASCII_STOP: u32 = 127u;
const NUM_ASCII: u32 = ASCII_STOP - ASCII_START;

const grid_size: u32 = grid_width * grid_height;

struct AsciiStats {
    mu: f32,
    @size(12) sigma: f32,
}

struct AsciiPixel {
    @size(16) value: f32,
}

@group(0) @binding(0) var input_texture: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8uint, write>;

// TODO Need to find out if this can be written and stored to
/// Holding the scores for all of the ASCII glyphs
@group(1) @binding(0) var ssim_texture: texture_storage_3d<rgba8unorm, read_write>;

@group(2) @binding(0)
var<uniform> ascii_matrices: array<array<AsciiPixel, grid_size>, NUM_ASCII>;

// Storing the mean and standard deviation of each ASCII character since these will never change
@group(2) @binding(1)
var<uniform> ascii_stats: array<AsciiStats, NUM_ASCII>;

// NOTE Number of workgroups for z axis should still be 1 when dispatching
@compute @workgroup_size(1, 1, 1)
fn ascii_from_ssim(
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
) {
    var best_ssim: f32 = 0.0;
    // var best_ascii: u32 = 0u; // Will correspond to ' ' later
    var best_ascii: u32 = 1u; // Will correspond to ' ' later
    for (var i: u32 = 0u; i < NUM_ASCII; i++) {
        let ssim = textureLoad(ssim_texture, vec3<u32>(workgroup_id.x, workgroup_id.y, i)).w;
        if ssim > best_ssim {
            best_ssim = ssim;
            best_ascii = i;
        }
    }
    let ascii = best_ascii + ASCII_START;

    let ssim_texel = textureLoad(ssim_texture, vec3<u32>(workgroup_id.x, workgroup_id.y, best_ascii));
    let out_texel = vec4<u32>(u32(255.0 * ssim_texel.x), u32(255.0 * ssim_texel.y), u32(255.0 * ssim_texel.z), ascii);
    textureStore(output_texture, vec2<u32>(workgroup_id.xy), out_texel);
}
