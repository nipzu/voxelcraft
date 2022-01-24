struct VertexOutput {
    [[builtin(position)]] clip_pos: vec4<f32>;
    [[location(0)]] screen_pos: vec2<f32>;
};

[[stage(vertex)]]
fn vert_main(
    [[builtin(vertex_index)]] index: u32
) -> VertexOutput {
    let xored_0_2 = index ^ (index >> 2u);
    let x = -1.0 + 2.0 * f32(xored_0_2 & 1u);
    let y = -1.0 + 2.0 * f32((xored_0_2 ^ (index >> 1u)) & 1u);
    return VertexOutput(
        vec4<f32>(x, y, 0.0, 1.0),
        vec2<f32>(x, y),
    );
}

struct CameraCanvas {
    origin: vec3<f32>;
    canvas_mid_delta: vec3<f32>;
    canvas_top_delta: vec3<f32>;
    canvas_right_delta: vec3<f32>;
};

[[group(0), binding(0)]]
var<uniform> camera: CameraCanvas;

[[stage(fragment)]]
fn frag_main(
    [[location(0)]] pos: vec2<f32>
) -> [[location(0)]] vec4<f32> {
    let dst = camera.canvas_mid_delta
        + pos.x * camera.canvas_right_delta
        + pos.y * camera.canvas_top_delta;
    let dir = normalize(dst);

    var c = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    if (dir.y > 0.0) {
        c = c + vec4<f32>(1.0, 0.0, 0.0, 0.0);
    }
    if (dir.x > 0.0) {
        c = c + vec4<f32>(0.0, 1.0, 0.0, 0.0);
    }
    if (dir.z > 0.0) {
        c = c + vec4<f32>(0.0, 0.0, 1.0, 0.0);
    }
    if (abs(dir.y) < 0.01) {
        c = vec4<f32>(0.3,0.3,0.3,1.0);
    }

    return c;
}

fn intersect_cube(
    target: vec3<f32>,
    ray_origin: vec3<f32>,
    dir_inv: vec3<f32>,
) -> f32 {
    let d = (target - ray_origin) * dir_inv;

    return min(min(d.x, d.y), d.z);
}
