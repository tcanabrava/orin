// SPDX-License-Identifier: MIT

// Song-progress bar's waveform: a smoothed, mirrored envelope — unlike the
// oscilloscope's signed line trace (see oscilloscope.wgsl), this fills the
// area within the (interpolated) amplitude of the vertical center, so it
// reads as a classic audio waveform silhouette rather than a bar chart or a
// row of isolated spikes.

#import bevy_ui::ui_vertex_output::UiVertexOutput

@group(1) @binding(0) var<uniform> color: vec4<f32>;
@group(1) @binding(1) var<uniform> amplitudes: array<vec4<f32>, 75>; // 300 buckets, packed

const N: f32 = 300.0;
const EDGE_PX: f32 = 1.25; // soft-edge half-width, pixels

fn sample(i: u32) -> f32 {
    return amplitudes[i >> 2u][i & 3u];
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let size = in.size; // node size in pixels

    // The two waveform buckets bracketing this pixel's x, linearly
    // interpolated — this is what removes the hard bucket-to-bucket steps
    // a plain per-bucket bar chart has.
    let fx = clamp(in.uv.x * (N - 1.0), 0.0, N - 1.0);
    let i0 = u32(floor(fx));
    let i1 = min(i0 + 1u, u32(N) - 1u);
    let amp = mix(sample(i0), sample(i1), fract(fx));

    // Mirrored envelope: filled wherever this pixel's distance from the
    // node's vertical center is within half the interpolated amplitude,
    // with a soft (not hard-edged) transition.
    let half_amp = amp * 0.5;
    let dist_from_center = abs(in.uv.y - 0.5);
    let edge = EDGE_PX / size.y;
    let alpha = 1.0 - smoothstep(half_amp - edge, half_amp + edge, dist_from_center);

    return vec4<f32>(color.rgb, color.a * alpha);
}
