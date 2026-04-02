struct SkyboxParams {
    inv_view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    intensity: vec4<f32>,
    ground_color: vec4<f32>,
    horizon_color: vec4<f32>,
    sky_color: vec4<f32>,
    params: vec4<f32>,
    sun_dirs: array<vec4<f32>, 4>,
    sun_colors: array<vec4<f32>, 4>,
    sun_sizes: array<vec4<f32>, 4>,
    sun_meta: vec4<f32>,
};

@group(0) @binding(0) var<uniform> u_sky: SkyboxParams;
@group(0) @binding(1) var t_sky: texture_2d<f32>;
@group(0) @binding(2) var s_sky: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VsOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );

    let pos = positions[idx];
    var out: VsOut;
    out.pos = vec4<f32>(pos, 0.0, 1.0);
    out.uv = pos * 0.5 + vec2<f32>(0.5, 0.5);
    return out;
}

fn rotate_y(dir: vec3<f32>, angle: f32) -> vec3<f32> {
    let s = sin(angle);
    let c = cos(angle);
    return vec3<f32>(c * dir.x + s * dir.z, dir.y, -s * dir.x + c * dir.z);
}

fn equirect_uv(dir: vec3<f32>) -> vec2<f32> {
    let n = normalize(dir);
    let u = atan2(n.z, n.x) * 0.15915494 + 0.5; // 1 / (2 * PI)
    let v = 1.0 - (asin(clamp(n.y, -1.0, 1.0)) * 0.318309886 + 0.5); // 1 / PI
    return vec2<f32>(fract(u), clamp(v, 0.0, 1.0));
}

fn procedural_sky(dir_in: vec3<f32>) -> vec3<f32> {
    let dir = normalize(dir_in);
    let y = clamp(dir.y * 0.5 + 0.5, 0.0, 1.0);
    let horizon_center = 0.5;
    let upper = clamp((y - horizon_center) * 2.0, 0.0, 1.0);
    let lower = clamp((horizon_center - y) * 2.0, 0.0, 1.0);
    let ground = u_sky.ground_color.xyz;
    let horizon = u_sky.horizon_color.xyz;
    let sky = u_sky.sky_color.xyz;

    // params.z / params.w are influence factors:
    // 0.0 -> horizon-only on that hemisphere, 1.0 -> dominant pole color with zero-width blend.
    let ground_influence = clamp(u_sky.params.z, 0.0, 1.0);
    let sky_influence = clamp(u_sky.params.w, 0.0, 1.0);
    let ground_blend_range = max(1e-4, 1.0 - ground_influence);
    let sky_blend_range = max(1e-4, 1.0 - sky_influence);

    let to_ground = select(
        0.0,
        smoothstep(0.0, ground_blend_range, lower),
        ground_influence > 0.0
    );
    let to_sky = select(
        0.0,
        smoothstep(0.0, sky_blend_range, upper),
        sky_influence > 0.0
    );

    var color = horizon;
    color = mix(color, ground, to_ground);
    color = mix(color, sky, to_sky);

    let sun_count = min(u32(round(max(u_sky.sun_meta.x, 0.0))), 4u);
    var sun_accum = vec3<f32>(0.0);
    for (var i = 0u; i < 4u; i++) {
        if (i >= sun_count) {
            break;
        }
        let sun_dir = normalize(u_sky.sun_dirs[i].xyz);
        let sun_rgb = u_sky.sun_colors[i].xyz;
        let sun_intensity = max(u_sky.sun_colors[i].w, 0.0);
        // Normalized size:
        // 0.0 -> no coverage, 1.0 -> hemisphere coverage, 2.0 -> full-sphere coverage.
        let solid_size = max(u_sky.sun_sizes[i].x, 0.0);
        let gradient_size = max(u_sky.sun_sizes[i].y, solid_size + 1e-4);
        let solid_ratio = min(solid_size, 2.0);
        let gradient_ratio = min(gradient_size, 2.0);
        let sun_disk_edge = 1.0 - solid_ratio;
        let sun_halo_edge = 1.0 - gradient_ratio;
        let alignment = clamp(dot(dir, sun_dir), 0.0, 1.0);
        let disk = select(
            smoothstep(sun_disk_edge, 1.0, alignment),
            1.0,
            solid_size >= 2.0
        );
        let halo_base = select(
            smoothstep(sun_halo_edge, 1.0, alignment),
            1.0,
            gradient_size >= 2.0
        );
        let halo = halo_base * (1.0 - disk);
        sun_accum += sun_rgb * sun_intensity * (disk * 4.0 + halo * 1.2);
    }

    return color + sun_accum;
}

fn is_finite3(v: vec3<f32>) -> bool {
    let mag = abs(v);
    return all(v == v) && all(mag < vec3<f32>(1e30, 1e30, 1e30));
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let ndc_xy = in.uv * 2.0 - 1.0;
    let near_clip = vec4<f32>(ndc_xy, 1.0, 1.0);
    let far_clip = vec4<f32>(ndc_xy, 0.0, 1.0);

    let near_world = u_sky.inv_view_proj * near_clip;
    let far_world = u_sky.inv_view_proj * far_clip;

    let near_w = select(near_world.w, 1.0, abs(near_world.w) < 1e-6);
    let far_w = select(far_world.w, 1.0, abs(far_world.w) < 1e-6);

    let near_pos = near_world.xyz / near_w;
    let far_pos = far_world.xyz / far_w;

    var dir = normalize(far_pos - near_pos);
    if (!is_finite3(dir)) {
        dir = normalize(near_pos - u_sky.camera_pos.xyz);
    }
    if (!is_finite3(dir)) {
        dir = vec3<f32>(0.0, 1.0, 0.0);
    }
    dir = rotate_y(dir, u_sky.params.x);

    let mode = u_sky.params.y;
    let intensity = u_sky.intensity.x;

    if (mode < 1.5) {
        let color = procedural_sky(dir);
        let final_color = color * intensity;
        return vec4<f32>(final_color, 1.0);
    }

    let uv = equirect_uv(dir);
    let texel = textureSample(t_sky, s_sky, uv).rgb;
    return vec4<f32>(texel * intensity, 1.0);
}
