// Compute shader for turning rendered pixels into ASCII characters

// ASCII codes used for rasterizer 
const codes = array<u32, 10>(
64u, // '@'
37u, // '%'
35u, // '#'
42u, // '*'
43u, // '+'
61u, // '='
45u, // '-'
58u, // ':'
46u, // '.'
32u, // ' '
);

// Thresholds used to set boundaries between ASCII codes
const thresholds = array<f32, 10>(
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
fn find_best_code(intensity: f32) -> u32 {
    var code = codes[9];

    // NOTE Would like to do this with a for loop, but didn't let me index into constant above
    // TODO This could be made faster with binary search
    if (intensity > thresholds[0]) && (intensity <= thresholds[1]) {
        code = codes[0];
    } else if (intensity > thresholds[1]) && (intensity <= thresholds[2]) {
        code = codes[1];
    } else if (intensity > thresholds[2]) && (intensity <= thresholds[3]) {
        code = codes[2];
    } else if (intensity > thresholds[3]) && (intensity <= thresholds[4]) {
        code = codes[3];
    } else if (intensity > thresholds[4]) && (intensity <= thresholds[5]) {
        code = codes[4];
    } else if (intensity > thresholds[5]) && (intensity <= thresholds[6]) {
        code = codes[5];
    } else if (intensity > thresholds[6]) && (intensity <= thresholds[7]) {
        code = codes[6];
    } else if (intensity > thresholds[7]) && (intensity <= thresholds[8]) {
        code = codes[7];
    } else if (intensity > thresholds[8]) && (intensity <= thresholds[9]) {
        code = codes[8];
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
    let code = find_best_code(in_texel.w);

    let out_texel = vec4<u32>(u32(255.0 * in_texel.x), u32(255.0 * in_texel.y), u32(255.0 * in_texel.z), code);
    textureStore(output_texture, vec2<i32>(workgroup_id.xy), out_texel);
}
