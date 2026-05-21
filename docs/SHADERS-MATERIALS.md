# Custom Material Shaders

This guide explains how to create custom material shaders in Galfus today.

## 1. Contract Model

Custom material shaders are defined by:

- `MaterialDefinition` (`cmd-material-definition-upsert`)
- `Material` instance (`cmd-material-upsert`) referencing definition `slug`

Minimal flow:

1. upsert a material definition with `shaderSource`
2. upsert a material instance with the same `slug`
3. assign material ID to one or more models

## 2. Required Logical Entry Points

Your logical WGSL snippet must provide:

- `fn vertex(input: VertexInput) -> VertexOutput`
- `fn fragment(input: FragmentInput) -> FragmentOutput`

The engine composes physical WGSL around this snippet.

## 3. Core-Provided Inputs

The composer prelude injects global uniforms and storage bindings, including:

- `frame` (time, delta, frame index)
- `camera` (position, view/projection)
- lights and visibility buffers
- material parameter/texture bindings
- frame semantics resources (scene color/depth, history)

Use only the logical structs/functions documented in this file and in `SHADERS-GLOSSARY.md`.

## 4. History/Semantics on Demand

To request semantic resources like history textures, set capabilities in definition:

- `capabilities.semantics = ["history0"]` or `history1`

If omitted/empty, semantics are considered not required by that definition.

## 5. Example: Fresnel + Ghost Trail

```wgsl
fn project_world_to_screen_uv(world_position: vec3<f32>) -> vec2<f32> {
  let clip = camera.view_projection * vec4<f32>(world_position, 1.0);
  let inv_w = select(0.0, 1.0 / clip.w, abs(clip.w) > 1e-6);
  let ndc = clip.xy * inv_w;
  return vec2<f32>(ndc.x * 0.5 + 0.5, -ndc.y * 0.5 + 0.5);
}

fn vertex(input: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.world_position = input.position;
  out.world_normal = input.normal;
  out.uv = input.uv;
  out.clip_position = vec4<f32>(0.0);
  return out;
}

fn fragment(input: FragmentInput) -> FragmentOutput {
  var out: FragmentOutput;
  let view_dir = normalize(camera.position.xyz - input.world_position);
  let fresnel = pow(1.0 - max(dot(normalize(input.world_normal), view_dir), 0.0), 2.2);

  let screen_uv = project_world_to_screen_uv(input.world_position);
  let history = sample_history0(screen_uv + vec2<f32>(0.01, 0.0)).rgb;
  let ghost = history * 0.96 * vec3<f32>(0.2, 0.9, 1.0) * 0.5;

  let base = mix(vec3<f32>(0.1, 0.6, 0.95), vec3<f32>(1.0), fresnel * 0.4);
  out.color = vec4<f32>(max(base, base + ghost), 1.0);
  out.emissive = vec4<f32>(vec3<f32>(0.0), 1.0);
  return out;
}
```

## 6. Current Limits

- Shader contract is logical WGSL, not raw pipeline WGSL.
- No public contract for custom bind-group layout overrides.
- Keep outputs in `FragmentOutput` contract shape.
- Large binary data is not returned through `get/list`; resources stay core-authoritative.
