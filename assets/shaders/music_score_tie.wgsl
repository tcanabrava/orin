// SPDX-License-Identifier: MIT

// A tied-note connector for the shared music-notation staff
// (`music_score`), drawn as a real curved arc rather than the flat
// rectangle a plain `bevy_ui` `Node`/`BackgroundColor` is limited to —
// see `music_score::tie_material`'s own module doc comment.
//
// uv.x spans the node horizontally (0 = left end, 1 = right end); uv.y = 0
// is the node's top edge (nearest the notehead the tie starts from), 1 is
// the bottom edge. The arc's two ends sit at uv.y = 0 and dip toward
// uv.y = 1 at the middle — the same "curves away from the notehead" shape
// a real tie drawn below the staff has.

#import bevy_ui::ui_vertex_output::UiVertexOutput

// x = arc depth (fraction of the node's own height the middle dips to),
// y = line thickness (fraction of the node's own height).
@group(1) @binding(0) var<uniform> color: vec4<f32>;
@group(1) @binding(1) var<uniform> params: vec4<f32>;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let depth = params.x;
    let thickness = params.y;

    // A parabola pinned to 0 at uv.x = 0/1, peaking at `depth` at the
    // midpoint — cheap, and plenty smooth-looking at this scale; no need
    // for a true cubic bezier for a mark this small.
    let curve_y = depth * 4.0 * uv.x * (1.0 - uv.x);
    let dist = abs(uv.y - curve_y);
    let aa = fwidth(uv.y) + 0.01;
    let alpha = 1.0 - smoothstep(thickness - aa, thickness + aa, dist);

    return vec4<f32>(color.rgb, alpha * color.a);
}
