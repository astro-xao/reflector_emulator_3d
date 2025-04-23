#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

#import bevy_pbr::{
    mesh_view_bindings::globals,
    // forward_io::VertexOutput,
}

@group(2) @binding(0) var<storage, read> buffer: array<f32, 3880>;
@group(2) @binding(1) var<uniform> enable_boundary: u32;
@group(2) @binding(2) var<uniform> interpolate_algo: u32;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,

    // The world-space position of the vertex.
    @location(0) position: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) i_height: f32,
};

//包含 10 种 rgb color 的 scale 
// #661325 rgb: 102, 19, 37   divide 255: 0.4, 0.07, 0.15
// #7E1944 rgb: 126, 25, 68   divide 255: 0.5, 0.1, 0.27
// #95206B rgb: 149, 32, 107  divide 255: 0.58, 0.12, 0.42
// #AC2898 rgb: 172, 40, 152  divide 255: 0.67, 0.16, 0.6
// #B930C2 rgb: 185, 48, 194  divide 255: 0.73, 0.19, 0.76
// #AC4ACC rgb: 172, 74, 172  divide 255: 0.67, 0.29, 0.67
// #A464D6 rgb: 164, 100, 214 divide 255: 0.64, 0.39, 0.84
// #A37EDF rgb: 163, 126, 223 divide 255: 0.64, 0.49, 0.87
// #A99AE7 rgb: 169, 154, 231 divide 255: 0.66, 0.6, 0.91
// #B6B6EF rgb: 182, 182, 239 divide 255: 0.71, 0.71, 0.94
// #D3D9F6 rgb: 211, 217, 246 divide 255: 0.83, 0.85, 0.96
var<private> scale_0 = array<vec4<f32>, 11>(
        vec4<f32>(0.4, 0.07, 0.15, 1.0),
        vec4<f32>(0.5, 0.1, 0.27, 1.0),
        vec4<f32>(0.58, 0.12, 0.42, 1.0),
        vec4<f32>(0.67, 0.16, 0.6, 1.0),
        vec4<f32>(0.73, 0.19, 0.76, 1.0),
        vec4<f32>(0.67, 0.29, 0.67, 1.0),
        vec4<f32>(0.64, 0.39, 0.84, 1.0),
        vec4<f32>(0.64, 0.49, 0.87, 1.0),
        vec4<f32>(0.66, 0.6, 0.91, 1.0),
        vec4<f32>(0.71, 0.71, 0.94, 1.0),
        vec4<f32>(0.83, 0.85, 0.96, 1.0)
    );

// #7B802F rgb: 123, 128, 47   divide 255: 0.48, 0.5, 0.18
// #9F9A3D rgb: 159, 154, 61   divide 255: 0.62, 0.6, 0.24
// #BF9E4B rgb: 191, 158, 75   divide 255: 0.75, 0.62, 0.29
// #DF9B5A rgb: 223, 155, 90   divide 255: 0.87, 0.61, 0.35
// #FF9169 rgb: 255, 145, 105  divide 255: 1.0, 0.57, 0.41
// #FF867B rgb: 255, 134, 123  divide 255: 1.0, 0.53, 0.48
// #FF8D9A rgb: 255, 141, 154  divide 255: 1.0, 0.55, 0.6
// #FFA1BD rgb: 255, 161, 189  divide 255: 1.0, 0.63, 0.74
// #FFB4DA rgb: 255, 180, 218  divide 255: 1.0, 0.71, 0.85
// #FFC9F0 rgb: 255, 201, 240  divide 255: 1.0, 0.79, 0.94
// #FFDEFD rgb: 255, 222, 253  divide 255: 1.0, 0.87, 0.99
var<private> scale_1 = array<vec4<f32>, 11>(
        vec4<f32>(0.48, 0.5, 0.18, 1.0),
        vec4<f32>(0.62, 0.6, 0.24, 1.0),
        vec4<f32>(0.75, 0.62, 0.29, 1.0),
        vec4<f32>(0.87, 0.61, 0.35, 1.0),
        vec4<f32>(1.0, 0.57, 0.41, 1.0),
        vec4<f32>(1.0, 0.53, 0.48, 1.0),
        vec4<f32>(1.0, 0.55, 0.6, 1.0),
        vec4<f32>(1.0, 0.63, 0.74, 1.0),
        vec4<f32>(1.0, 0.71, 0.85, 1.0),
        vec4<f32>(1.0, 0.79, 0.94, 1.0),
        vec4<f32>(1.0, 0.87, 0.99, 1.0),
    );

fn interpolate_color_0(i_height: f32) -> vec4<f32> {
    var h = i_height * 10.0;
    var low_index: u32 = u32(floor(h));
    var high_index: u32 = u32(ceil(h));
    low_index = max(u32(0), low_index);
    high_index = min(u32(10), high_index);
    return mix(scale_0[low_index], scale_0[high_index], i_height);
}

fn interpolate_color_1(i_height: f32) -> vec4<f32> {
    var h = i_height * 10.0;
    var low_index: u32 = u32(floor(h));
    var high_index: u32 = u32(ceil(h));
    low_index = max(u32(0), low_index);
    high_index = min(u32(10), high_index);
    return mix(scale_1[low_index], scale_1[high_index], i_height);
}

// 红 到 紫色 七种颜色
var<private> colors = array<vec3<f32>, 3>(
    // 红
    vec3<f32>(1.0, 0.0, 0.0),
    // // 橙
    // vec3<f32>(1.0, 0.5, 0.0),
    // // 黄
    vec3<f32>(1.0, 1.0, 0.0),
    // 绿
    vec3<f32>(0.0, 1.0, 0.0),
    // 青
    // vec3<f32>(0.0, 1.0, 1.0),
    // 蓝
    // vec3<f32>(0.0, 0.0, 1.0),
    // 紫
    // vec3<f32>(1.0, 0.0, 1.0),
);

fn interpolate_color(factor: f32) -> vec4<f32> {
    let segment_count = f32(3 - 1);
    let segment = factor * segment_count;
    let low_index = u32(floor(segment));
    let high_index = u32(ceil(segment));
    let t = fract(segment);
    return vec4<f32>(mix(colors[low_index], colors[high_index], t), 1.0);
}

fn interpolate_color_oklab(factor: f32) -> vec4<f32> {

    // blending is done in a perceptual color space: https://bottosson.github.io/posts/oklab/
    let red = vec3<f32>(0.627955, 0.224863, 0.125846);
    let green = vec3<f32>(0.86644, -0.233887, 0.179498);
    let blue = vec3<f32>(0.701674, 0.274566, -0.169156);
    let white = vec3<f32>(1.0, 0.0, 0.0);
    var mixed = vec3<f32>(0.0, 0.0, 0.0);
    // mix the colors with factor
    if factor < 0.5 {
        mixed = mix(red, green, factor * 2.0); // 插值红色和黄色
    } else {
        mixed= mix(green, blue, (factor - 0.5) * 2.0); // 插值黄色和绿色
    }

    return vec4<f32>(oklab_to_linear_srgb(mixed), 1.0);
}

fn oklab_to_linear_srgb(c: vec3<f32>) -> vec3<f32> {
    let L = c.x;
    let a = c.y;
    let b = c.z;

    let l_ = L + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = L - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = L - 0.0894841775 * a - 1.2914855480 * b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    return vec3<f32>(
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    );
}

@vertex
fn vertex(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(vertex.instance_index), 
        vec4<f32>(vertex.position, 1.0)
    );

    // 使用 i_height 在 red green 中混合
    let red = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    let green = vec4<f32>(0.0, 1.0, 0.0, 1.0);
    let height = buffer[vertex.vertex_index % 3880];
    out.i_height = height;
    if (interpolate_algo == 0u) {
        out.color = interpolate_color_0(height);
    } else if (interpolate_algo == 1u) {
        out.color = interpolate_color_1(height);
    } else if (interpolate_algo == 2u) {
        out.color = interpolate_color(height);
    } else if (interpolate_algo == 3u) {
        out.color = interpolate_color_oklab(height);
    }
    
    out.uv = vertex.uv;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // 通过 uv 值判断当前片元是否属于边缘，如果是则渲染 border 为黑色
    if (enable_boundary == 1u && (in.uv.x <= 0.015 || in.uv.y <= 0.015 || in.uv.x >= 0.985 || in.uv.y >= 0.985)) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    let color = in.color;
    return vec4<f32>(color.xyz, 1.0);
}
