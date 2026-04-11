import type { GltfLoaderErrorCode } from './types';

/**
 * Loader error with a stable error code for caller handling.
 *
 * @example
 * ```ts
 * import { GltfLoaderError, loadGltfAsset } from '@vulfram/gltf-loader';
 *
 * try {
 *   await loadGltfAsset({ worldId, data });
 * } catch (error) {
 *   if (error instanceof GltfLoaderError) {
 *     console.error(error.code, error.message);
 *   }
 * }
 * ```
 */
export class GltfLoaderError extends Error {
  constructor(
    public readonly code: GltfLoaderErrorCode,
    message: string
  ) {
    super(message);
    this.name = 'GltfLoaderError';
  }
}
