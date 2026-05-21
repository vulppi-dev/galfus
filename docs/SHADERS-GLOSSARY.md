# Shader Helpers and Variables Glossary

This glossary lists helpers and variables currently available in Galfus shader composition paths.

## Material Shader Helpers

From material composer prelude:

- `sample_shadow_primary_light(world_position, world_normal)`
- `resolve_point_light_face(dir)`
- `sample_scene_color(uv)`
- `sample_history0(uv)`
- `sample_history1(uv)`
- `load_scene_depth(pixel)`
- `scene_resolution()`
- `scene_inv_resolution()`
- `current_frame_index()`
- `has_material_input(index)`
- `input_at(index)`
- `sample_material(tex_slot, sampler_index, uv)`
- `get_primary_light_direction(world_position)`
- `get_primary_light_color()`

## Material Shader Key Structs

- `VertexInput`
- `VertexOutput`
- `FragmentInput`
- `FragmentOutput`
- `Frame`
- `Camera`
- `MaterialParams`

## Material Globals (common)

- `frame`
- `camera`
- `lights`
- `material_params`

## Pass Shader Helpers (Shader DSL)

- `sample_scene_color(uv)`
- `sample_history0(uv)`
- `sample_history1(uv)`
- `load_scene_depth(pixel)`
- `scene_resolution()`
- `scene_inv_resolution()`
- `current_frame_index()`

For each declared logical pass input `foo`:

- `sample_foo(uv)`
- `load_foo(pixel)`

## Recommended Usage Patterns

- Use `camera.view_projection` to derive screen-space UV from world position when sampling history in model materials.
- For temporal effects, clamp/saturate blend weights.
- Prefer helper functions over duplicating shadow/light logic across shaders.
- Keep history usage capability-driven (`capabilities.semantics`) to avoid always-on cost.
