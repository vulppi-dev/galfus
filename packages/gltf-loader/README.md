# @galfus/gltf-loader

Loads `.gltf` and `.glb` content into `@galfus/engine` 3D worlds.

The loader follows the engine host-side math convention and exposes transforms as
`@galfus/engine/math` `vec3` / `quat`.

## Example

```ts
import { loadGltfAsset } from '@galfus/gltf-loader';

const asset = await loadGltfAsset({
  worldId,
  data: glbBytes
});

const instance = asset.instantiate({
  rootTransform: {
    position: [0, 0, 0]
  }
});

// Later:
instance.disposeEntities();
asset.disposeAll();
```
