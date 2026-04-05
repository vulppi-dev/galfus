import type { GltfLoaderErrorCode } from './types';

/** Loader error with stable error code for callers. */
export class GltfLoaderError extends Error {
  constructor(public readonly code: GltfLoaderErrorCode, message: string) {
    super(message);
    this.name = 'GltfLoaderError';
  }
}
