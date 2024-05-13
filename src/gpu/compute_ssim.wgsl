//! Compute shader for averaging across non-trivial grid sizes

// Following constants should be prepended during `wgpu::ShaderSource::Wgsl`
// const grid_width: u32 = 1u;
// const grid_height: u32 = 2u;
// TODO Look into pipeline overridable constants like `@id override grid_width`, once this gets added to `wgpu`

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

/// Holding the scores for all of the ASCII glyphs
@group(1) @binding(0) var ssim_texture: texture_storage_3d<rgba8unorm, read_write>;

// FIXME Uniform is not allowed to be very big, so will have to change this to storage
@group(2) @binding(0)
var<storage, read> ascii_matrices: array<array<AsciiPixel, grid_size>, NUM_ASCII>;

/// Storing the mean and standard deviation of each ASCII character since these will never change
@group(2) @binding(1)
var<uniform> ascii_stats: array<AsciiStats, NUM_ASCII>;

/// Calculate structural similarity between two character grids, given their moments
fn ssim(mu_x: f32, mu_y: f32, sigma_x: f32, sigma_y: f32, sigma_xy: f32) -> f32 {
    let l = (2.0 * mu_x * mu_y) / (mu_x * mu_x * mu_y * mu_y);
    let c = (2.0 * sigma_x * sigma_y) / (sigma_x * sigma_x * sigma_y * sigma_y);
    let s = sigma_xy / (sigma_x * sigma_y);
    return l * c * s;
}

@compute @workgroup_size(1, 1, 1)
fn compute_ssim(
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
) {
    let ascii_index = workgroup_id.z;

    var grid_mean: f32 = 0.0;
    var grid_squared_mean: f32 = 0.0;
    var grid_ascii_mean: f32 = 0.0;
    for (var i: u32 = 0u; i < grid_width; i++) {
        for (var j: u32 = 0u; j < grid_height; j++) {
            let img_coord = vec2<u32>(workgroup_id.x * grid_width + i, workgroup_id.y * grid_height + j);
            let grid_texel: f32 = textureLoad(input_texture, img_coord).w;
            let ascii_texel = ascii_matrices[ascii_index][i + j * grid_width].value;

            grid_mean += grid_texel;
            grid_squared_mean += (grid_texel * grid_texel);
            grid_ascii_mean += (grid_texel * ascii_texel);

            // TODO Need to deal with colour
        }
    }
    grid_mean /= f32(grid_size);
    grid_squared_mean /= f32(grid_size);
    grid_ascii_mean /= f32(grid_size);

    // Use moments to calculate SSIM
    let ascii_stat = ascii_stats[ascii_index];
    let mu_grid = grid_mean;
    let sigma_grid = sqrt(grid_squared_mean);
    let sigma_grid_ascii = grid_ascii_mean - mu_grid * ascii_stat.mu;
    // FIXME Swap back to let after debugging 
    var this_ssim: f32 = ssim(grid_mean, ascii_stat.mu, sigma_grid, ascii_stat.sigma, sigma_grid_ascii);
    // NOTE This value is not being respected by the next compute shader
    // if ascii_index == 3u {
    //     this_ssim = 1.0;
    // } else {
    //     this_ssim = 0.0;
    // }

    // TODO Need to pick the central colour
    let ssim_texel = vec4<f32>(0.5, 0.5, 0.5, this_ssim);
    textureStore(ssim_texture, vec3<u32>(workgroup_id.x, workgroup_id.y, ascii_index), ssim_texel);

    // // FIXME Swap back after debugging
    // let out_texel = vec4<u32>(u32(255.0 * ssim_texel.x), u32(255.0 * ssim_texel.y), u32(255.0 * ssim_texel.z), 33u + u32(95.0 * this_ssim));
    // textureStore(output_texture, vec2<u32>(workgroup_id.xy), out_texel);
}


// NOTE Number of workgroups for z axis should still be 1 when dispatching
@compute @workgroup_size(1, 1, 1)
fn ascii_from_ssim(
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
) {
    var best_ssim: f32 = 0.0;
    var best_index: u32 = 0u; // Will correspond to ' ' later
    // var best_index: u32 = 1u; // Will correspond to '!' later
    for (var ascii_index: u32 = 0u; ascii_index < NUM_ASCII; ascii_index++) {
        let ssim = textureLoad(ssim_texture, vec3<u32>(workgroup_id.x, workgroup_id.y, ascii_index)).w;
        if ssim > best_ssim {
            best_ssim = ssim;
            best_index = ascii_index;
        }
    }
    let ascii_char = best_index + ASCII_START;

    let ssim_texel = textureLoad(ssim_texture, vec3<u32>(workgroup_id.x, workgroup_id.y, best_index));
    let out_texel = vec4<u32>(u32(255.0 * ssim_texel.x), u32(255.0 * ssim_texel.y), u32(255.0 * ssim_texel.z), ascii_char);
    textureStore(output_texture, vec2<u32>(workgroup_id.xy), out_texel);
}
