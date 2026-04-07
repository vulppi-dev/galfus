import { WebIO, type Document, type GLTF, type JSONDocument } from '@gltf-transform/core';
import { decodeDataUri, detectFormat, normalizeResourceMap, toUint8Array } from './binary';
import { GltfLoaderError } from './errors';
import type { GltfLoadInput } from './types';

function parseGltfJsonDocument(
  bytes: Uint8Array,
  resources?: Record<string, import('./types').BinaryLike>
): JSONDocument {
  const decoder = new TextDecoder('utf-8');
  let json: GLTF.IGLTF;
  try {
    json = JSON.parse(decoder.decode(bytes)) as GLTF.IGLTF;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new GltfLoaderError('PARSE_ERROR', `Failed parsing glTF JSON: ${message}`);
  }

  const resourceMap = normalizeResourceMap(resources);

  const registerUri = (uri: string) => {
    if (resourceMap[uri]) return;
    if (uri.startsWith('data:')) {
      resourceMap[uri] = decodeDataUri(uri);
      return;
    }
    throw new GltfLoaderError(
      'MISSING_RESOURCE',
      `External resource not provided for URI "${uri}". Provide it in input.resources.`
    );
  };

  for (const buffer of json.buffers ?? []) {
    if (buffer.uri) registerUri(buffer.uri);
  }
  for (const image of json.images ?? []) {
    if (image.uri) registerUri(image.uri);
  }

  return {
    json,
    resources: resourceMap as unknown as { [s: string]: Uint8Array<ArrayBuffer> }
  };
}

/** Reads input payload into a glTF Transform document. */
export async function readDocument(input: GltfLoadInput): Promise<Document> {
  const io = new WebIO();
  const bytes = toUint8Array(input.data);
  const format = input.format ?? detectFormat(bytes);

  try {
    if (format === 'glb') {
      return await io.readBinary(bytes);
    }
    const jsonDoc = parseGltfJsonDocument(bytes, input.resources);
    return await io.readJSON(jsonDoc);
  } catch (error) {
    if (error instanceof GltfLoaderError) throw error;
    const message = error instanceof Error ? error.message : String(error);
    throw new GltfLoaderError('PARSE_ERROR', `Failed to read glTF document: ${message}`);
  }
}
