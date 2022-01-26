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

// fn sky_color(dir: vec3<f32>) -> vec4<f32> {
//     return vec4<f32>(0.2, 0.3, dir.y * 0.35 + 0.65, 1.0);
// }

fn ray_cast(origin: vec3<f32>, dir_in: vec3<f32>) -> vec4<f32> {
    // constants
    
    // smallest positive normal number
    // note that 1/m is a normal number
    let m = 1.1754944e-38; 
    let m_vec = vec3<f32>(m);
    let max_dist = 100.0;

    let is_zero = -m_vec < dir_in & dir_in < m_vec;
    let dir = select(dir_in, m_vec, is_zero);
    let dir_inv = 1.0 / dir;
    // let sky = sky_color(dir);

    var d: vec3<f32> = sign(dir) * fract(-sign(dir) * origin) * dir_inv;
    // var stack: array<i32, 32>;
    // var scale: u32 = 0u;

    // for (var dist: f32 = 0.0; dist < max_dist;) {
        let t = min(min(d.x, d.y), d.z);
        // let normal = step(vec3<f32>(-t), -d);
        let normal = select(vec3<f32>(0.0), vec3<f32>(1.0), d == vec3<f32>(t));

        d = fma(normal, dir, d);

        return vec4<f32>(normal.x * 0.5 + 0.5, normal.y * 0.5 + 0.5, normal.z * 0.5 + 0.5, 1.0);
    // }

    // return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

[[stage(fragment)]]
fn frag_main(
    [[location(0)]] pos: vec2<f32>
) -> [[location(0)]] vec4<f32> {
    let dst = camera.canvas_mid_delta
        + pos.x * camera.canvas_right_delta
        + pos.y * camera.canvas_top_delta;
    let dir = normalize(dst);

    return ray_cast(camera.origin, dir);
}
