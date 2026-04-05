/**
 * JSON-like values that are safe to serialize across the host/core boundary.
 */
export type JsonValue =
  | null
  | boolean
  | number
  | string
  | JsonValue[]
  | { [key: string]: JsonValue };

/**
 * Convenience alias for object-shaped JSON payloads.
 */
export type JsonObject = { [key: string]: JsonValue };
