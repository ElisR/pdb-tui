// Compute shader for turning rendered pixels into ASCII characters

// ASCII codes used for rasterizer 
const basic_rasterizer_codes = array<u8, 10>(
64, // '@'
37, // '%'
35, // '#'
42, // '*'
43, // '+'
61, // '='
45, // '-'
58, // ':'
46, // '.'
32, // ' '
);

// Thresholds used to set boundaries between ASCII codes
const basic_rasterizer_thresholds = array<f32, 9>(
 -0.1,
 0.2,
 0.3,
 0.4,
 0.5,
 0.6,
 0.7,
 0.8,
 0.9,
 0.999,
);

@group(0) @binding(0) var input_texture: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8uint, write>;


/// Find the code that lies in between the thresholds
fn find_best_code(intensity: f32) -> u8 {
    var code = basic_rasterizer_codes[9];

    for (var i: u32 = 0u; i < 9; i++) {
        let lower = basic_rasterizer_thresholds[i];
        let upper = basic_rasterizer_thresholds[i];

        if (intensity > lower) && (intensity <= upper) {
            code = basic_rasterizer_codes[i];
            break;
        }
    }

    return code;
}


@compute @workgroup_size(1, 1)
fn basic_ascii_rasterizer(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>
) {
    let coord = global_id.xy;
    let in_texel = textureLoad(input_texture, vec2<i32>(coord));
    let code = find_best_code(in_texel.w)
    
}
