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

// TODO Need to find out if this can be written and stored to
/// Holding the scores for all of the ASCII glyphs
@group(1) @binding(0) var ssim_texture: texture_storage_3d<rgba8unorm, read_write>;

@group(2) @binding(0)
var<uniform> ascii_matrices: array<array<AsciiPixel, grid_size>, NUM_ASCII>;

// Storing the mean and standard deviation of each ASCII character since these will never change
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
    let this_ssim = ssim(grid_mean, ascii_stat.mu, sigma_grid, ascii_stat.sigma, sigma_grid_ascii);

    // TODO Need to pick the central colour
    let ssim_texel = vec4<f32>(0.5, 0.5, 0.5, this_ssim);
    textureStore(ssim_texture, vec3<u32>(workgroup_id.x, workgroup_id.y, workgroup_id.z), ssim_texel);
}
