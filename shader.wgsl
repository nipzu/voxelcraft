struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) screen_pos: vec2<f32>,
};

@vertex
fn vert_main(
    @builtin(vertex_index) index: u32
) -> VertexOutput {
    let xored_0_2 = index ^ (index >> 2u);
    let x = -1.0 + 2.0 * f32(xored_0_2 & 1u);
    let y = -1.0 + 2.0 * f32((xored_0_2 ^ (index >> 1u)) & 1u);
    return VertexOutput(
        vec4<f32>(x, y, 0.0, 1.0),
        vec2<f32>(x, y),
    );
}

struct Camera {
    origin: vec3<f32>,
    canvas_mid_delta: vec3<f32>,
    canvas_top_delta: vec3<f32>,
    canvas_right_delta: vec3<f32>,
};

@group(0)
@binding(0)
var<uniform> camera: Camera;

// two vectors to please alignment requirements
struct ocnode {
    x0: vec4<u32>,
    x1: vec4<u32>,
}

@group(0)
@binding(1)
var<storage, read> voxels: array<ocnode>;

// TODO: advancing two axes at once

fn ray_cast(origin: vec3<f32>, dir_in: vec3<f32>) -> vec4<f32> {
    // constants

    // smallest positive normal number
    // note that 1/m is a normal number
    let m = 1.1754944e-38;
    let m_vec = vec3<f32>(m);
    // let max_dist = 100.0;

    let is_zero = -m_vec < dir_in & dir_in < m_vec;
    let dir = select(dir_in, m_vec, is_zero);
    let dir_inv = 1.0 / dir;
    // let sky = sky_color(dir);

    var cur: vec3<f32> = origin;

    var stack: array<u32, 32>;
    var scale: u32 = 0u;

    // TODO: input octree root

    var cur_ocnode: ocnode = voxels[0u];

    let tmp_idx = select(vec3<u32>(0u), vec3<u32>(1u, 2u, 4u), cur > vec3<f32>(1.0));
    var cur_subnode_idx = tmp_idx.x + tmp_idx.y + tmp_idx.z;

    var ascended: bool = false;
    // var trgt: vec3<f32> = vec3<f32>(dir > vec3<f32>(0.0)) + vec3<f32>(cur > vec3<f32>(1.0));

    for(var i = 0; i < 60; i += 1) {
        // idx & b1 == b0 => void
        // idx & b1 == b1 => not void
        // idx & b11 == b01 => non-leaf
        // idx & b11 == b11 => leaf

        // if (i == 5) {
        //     return vec4<f32>(
        //         // f32(scale) / 3.0,
        //         // f32(scale) / 3.0,
        //         // f32(scale) / 3.0,
        //         f32((cur_subnode_idx & 1u) != 0u),
        //         f32((cur_subnode_idx & 2u) != 0u),
        //         f32((cur_subnode_idx & 4u) != 0u),
        //         1.0
        //     );
        // }

        // TODO: remove
        // if (any((cur < vec3<f32>(0.0)) || (cur > vec3<f32>(2.0)))) {
        //     discard;
        // }

        let subnode_mask = (vec3<u32>(cur_subnode_idx) & vec3<u32>(1u, 2u, 4u)) != vec3<u32>(0u);

        let oc_half = select( cur_ocnode.x0, cur_ocnode.x1, subnode_mask.x );
        let oc_quarter = select( oc_half.xy, oc_half.zw, subnode_mask.y );
        let cur_subnode_val = select( oc_quarter.x, oc_quarter.y, subnode_mask.z );

        if ((cur_subnode_val & 1u) == 1u && !ascended) {
            // subnode not void

            if ((cur_subnode_val & 2u) == 0u) {
                // did not hit a leaf, descend
                stack[scale] += cur_subnode_idx;

                cur_ocnode = voxels[cur_subnode_val >> 2u];

                let sub_offset = vec3<f32>(subnode_mask);

                cur -= sub_offset;
                cur *= 2.0;
                // cur = fma(vec3<f32>(2.0), cur, -2.0 * sub_offset);
                scale += 1u;

                // trgt = vec3<f32>(dir > vec3<f32>(0.0)) + vec3<f32>(cur > vec3<f32>(1.0));

                let tmp_idx = select(vec3<u32>(0u), vec3<u32>(1u, 2u, 4u), cur > vec3<f32>(1.0));
                cur_subnode_idx = tmp_idx.x + tmp_idx.y + tmp_idx.z;
                stack[scale] = ((cur_subnode_val >> 2u) << 3u);
            } else {
                // hit leaf
                return vec4<f32>(0.0, 0.7, 0.0, 1.0);
            }
        } else {
            // subnode void

            ascended = false;
            // advance

            let subnode_offset = vec3<f32>(subnode_mask);
            let trgt = vec3<f32>(dir > vec3<f32>(0.0)) + subnode_offset;
            let d = (trgt - cur) * dir_inv;

            // pick minimum distance
            let t = min(min(d.x, d.y), d.z);
            let adv_axis = (d == vec3<f32>(t));

            let snormal = select(vec3<f32>(0.0), sign(dir), adv_axis);

            let neg = (snormal.x + snormal.y + snormal.z) < 0.0;

            let tmp_idx_offset = select(vec3<u32>(0u), vec3<u32>(1u, 2u, 4u), adv_axis);
            let idx_offset = tmp_idx_offset.x + tmp_idx_offset.y + tmp_idx_offset.z;

            cur = fma(vec3<f32>(t), dir, cur);
            // TODO: remove sometime
            // cur = select(cur, round(cur), d == vec3<f32>(t));

            if ((cur_subnode_idx & idx_offset) != u32(neg) * idx_offset) {
                // ascend, change coordinates
                if (scale == 0u) {
                    // return vec4<f32>(0.0, 0.0, 0.6, 1.0);
                    return vec4<f32>(vec3<f32>(adv_axis), 1.0);
                }

                scale -= 1u;
                // cur *= 0.5;
                let stack_top = stack[scale];
                cur_ocnode = voxels[stack_top >> 3u];
                cur_subnode_idx = stack_top & 7u;

                let subnode_offset = vec3<f32>((vec3<u32>(cur_subnode_idx) & vec3<u32>(1u, 2u, 4u)) != vec3<u32>(0u));
                cur = fma(vec3<f32>(0.5), cur, subnode_offset);
                // cur += subnode_offset;

                ascended = true;
                // trgt = vec3<f32>(dir > vec3<f32>(0.0)) + subnode_offset;
            } else {
                // TODO: idk man used signed or smth
                cur_subnode_idx = select(cur_subnode_idx + idx_offset, cur_subnode_idx - idx_offset, neg);
                // trgt += select(vec3<f32>(0.0), sign(dir), adv_axis);
            }
        }
    }

    return vec4<f32>(0.4, 0.0, 0.7, 1.0);
}

@fragment
fn frag_main(
    @location(0) pos: vec2<f32>
) -> @location(0) vec4<f32> {
    let dst = camera.canvas_mid_delta
        + pos.x * camera.canvas_right_delta
        + pos.y * camera.canvas_top_delta;
    let dir = normalize(dst);

    //return vec4(dir, 1.0);

    return ray_cast(camera.origin, dir);
}
