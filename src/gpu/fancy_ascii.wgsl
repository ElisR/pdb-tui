//! Compute shader for averaging across non-trivial grid sizes

// Following constants should be prepended during `wgpu::ShaderSource::Wgsl`
// const grid_width: u32 = 1u;
// const grid_height: u32 = 2u;

const ASCII_START: u32 = 32u;
const ASCII_STOP: u32 = 127u;
const NUM_ASCII: u32 = ASCII_STOP - ASCII_START;
const grid_size: u32 = grid_width * grid_height;

@group(0) @binding(0) var input_texture: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8uint, write>;

@group(1) @binding(0)
var<uniform> ascii_matrices: array<array<f32, grid_size>, NUM_ASCII>;

var<workgroup> workgroup_data: array<f32, grid_size>;


/// Calculate mean of an array 
fn mean(x: array<f32, grid_size>) -> f32 {
    var total: f32 = 0.0;
    for (var i: u32 = 0; i < grid_size; i++) {
        total += x[i];
    }
    return total / float(grid_size);
}

/// Calculate the standard deviation of an array
fn std(x: array<f32, grid_size>) -> f32 {
    var sum_squares: f32 = 0.0;
    for (var i: u32 = 0; i < grid_size; i++) {
        sum_squares += pow(x[i], 2);
    }
    let mean_squares = sum_squares / float(grid_size);
    return sqrt(mean_squares);
}

/// Calculate the covariance of two arrays (not accounting for means)
fn cov(x: array<f32, grid_size>, y: array<f32, grid_size>) {
    var sum_xy: f32 = 0.0;
    for (var i: u32 = 0; i < grid_size; i++) {
        sum_xy += x[i] * y[i];
    }
    return sum_squares / float(grid_size);
}

/// Calculate structural similarity between two character grids
fn ssim(x: array<f32, grid_size>, y: array<f32, grid_size>) -> f32 {
    let mu_x = mean(x);
    let mu_y = mean(y);

    let sigma_x = std(x);
    let sigma_y = std(y);
    let sigma_xy = cov(x, y) - mu_x * mu_y;

    let l = (2.0 * mu_x * mu_y) / (pow(mu_x, 2) * pow(mu_y, 2));
    let c = (2.0 * sigma_x * sigma_y) / (pow(sigma_x, 2) * pow(sigma_y, 2));
    let s = sigma_xy / (sigma_x * sigma_y);

    return l * c * s;
}

@compute @workgroup_size(grid_width, grid_height)
fn rasterize(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>
) {
    let coord = global_id.xy;

    let index = local_id.x + grid_width * local_id.y;
    let in_texel = textureLoad(input_texture, vec2<i32>(coord));
    workgroup_data[index] = in_texel.w;
    workgroupBarrier();

    var best_i: u32;
    var best_ssim: f32 = 0.0;
    for (var i: u32 = ASCII_START; i < ASCII_STOP; i++) {
        let ascii_matrix = ascii_matrices[i - ASCII_START];
        let current_ssim = ssim(workgroup_data, ascii_matrix);
        if current_ssim > best_ssim {
            best_i = i;
        }
    }

    let out_texel = vec4<u32>(u32(255.0 * in_texel.x), u32(255.0 * in_texel.y), u32(255.0 * in_texel.z), best_i);
    textureStore(output_texture, vec2<i32>(workgroup_id.xy), out_texel);
}
