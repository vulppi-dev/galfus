import { GLB_MAGIC, GLB_VERSION_2 } from './constants';
import { GltfLoaderError } from './errors';
import type { BinaryLike, GltfSourceFormat } from './types';

/** Converts supported binary-like inputs to Uint8Array with zero-copy when possible. */
export function toUint8Array(data: BinaryLike): Uint8Array {
  if (data instanceof Uint8Array) return data;
  if (data instanceof ArrayBuffer) return new Uint8Array(data);
  if (ArrayBuffer.isView(data)) {
    return new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
  }
  throw new GltfLoaderError('INVALID_INPUT', 'Unsupported binary input type.');
}

/** Detects whether payload appears to be .glb or .gltf JSON. */
export function detectFormat(bytes: Uint8Array): GltfSourceFormat {
  if (bytes.byteLength >= 12) {
    const dv = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    const magic = dv.getUint32(0, true);
    const version = dv.getUint32(4, true);
    if (magic === GLB_MAGIC && version === GLB_VERSION_2) {
      return 'glb';
    }
  }

  for (let i = 0; i < bytes.length; i++) {
    const c = bytes[i];
    if (c === undefined) continue;
    if (c <= 0x20) continue;
    if (c === 0x7b) return 'gltf'; // '{'
    break;
  }

  throw new GltfLoaderError(
    'UNSUPPORTED_FORMAT',
    'Unable to detect glTF format from payload bytes.',
  );
}

/** Decodes data URI payload into raw bytes. */
export function decodeDataUri(uri: string): Uint8Array {
  if (!uri.startsWith('data:')) {
    throw new GltfLoaderError('UNSUPPORTED_FORMAT', `Invalid data URI: ${uri}`);
  }
  const commaIdx = uri.indexOf(',');
  if (commaIdx < 0) {
    throw new GltfLoaderError('UNSUPPORTED_FORMAT', `Malformed data URI: ${uri}`);
  }

  const meta = uri.slice('data:'.length, commaIdx);
  const payload = uri.slice(commaIdx + 1);
  const isBase64 = meta.includes(';base64');

  if (isBase64) {
    return new Uint8Array(Buffer.from(payload, 'base64'));
  }
  return new Uint8Array(Buffer.from(decodeURIComponent(payload), 'utf8'));
}

/** Normalizes external resource map to Uint8Array values. */
export function normalizeResourceMap(
  resources?: Record<string, BinaryLike>,
): Record<string, Uint8Array> {
  if (!resources) return {};
  const out: Record<string, Uint8Array> = {};
  for (const [key, value] of Object.entries(resources)) {
    out[key] = toUint8Array(value);
  }
  return out;
}
