import { Packr, Unpackr } from 'msgpackr';
import type { TransportBuffer } from '@vulfram/transport-types';
import type {
  CommandResponseEnvelope,
  EngineCmdEnvelope,
} from '../../types/cmds';
import type { EngineEvent } from '../../types/events';

/**
 * Vulfram Core/ABI Invariants:
 *
 * 1. Threading:
 *    - The Core expects to be ticked and communicated with from a single host thread (Main Thread).
 *    - FFI/N-API calls are synchronous but the logic they trigger in the core (GPU work) is asynchronous.
 *
 * 2. Temporal Stability (Tick):
 *    - `vulframTick` must be called once per frame.
 *    - Incorrect or skipped ticks can lead to uneven animations, input lag, or resource starvation.
 *
 * 3. Command Batching:
 *    - All High-Level Logic commands are batched into a single MessagePack-encoded buffer.
 *    - The batch is sent once per frame via `vulframSendQueue`.
 *    - Maximum batch size is defined by the Core (currently 64KB for safety, but adjustable).
 *
 * 4. Events & Responses:
 *    - Responses: Every command sent in a batch will eventually produce a response in `vulframReceiveQueue`.
 *      Responses are matched back to commands using the `id` field in the envelope.
 *    - Events: System-level events (Keyboard, Mouse, Window resize) are polled via `vulframReceiveEvents`.
 *
 * 5. Resource Uploads:
 *    - Larger data (Textures, Geometry data) are sent via `vulframUploadBuffer` to avoid bloating the command batch.
 *    - These are out-of-band and acknowledged via specific events or response IDs.
 *
 * 6. ID Management:
 *    - The Host (this ECS) is responsible for generating and managing Logical IDs for all entities, components, and resources.
 *    - The Core uses these IDs to track objects. Reusing an ID without disposing of the previous object will cause errors or undefined behavior.
 */

/**
 * Batch Scheme:
 * A batch is simply an array of EngineCmdEnvelope objects.
 */
export type CoreCommandBatch = EngineCmdEnvelope[];

const packr = new Packr({ useRecords: false });
const unpackr = new Unpackr({ useRecords: false });

function normalizeForMsgpack(value: unknown): unknown {
  if (value === undefined || value === null) return value;
  if (Array.isArray(value)) {
    let out: unknown[] | undefined;
    for (let i = 0; i < value.length; i++) {
      const item = value[i];
      const normalizedItem = normalizeForMsgpack(item);
      const normalized = normalizedItem === undefined ? null : normalizedItem;
      if (!out) {
        if (normalized !== item) {
          out = value.slice(0, i);
          out.push(normalized);
        }
      } else {
        out.push(normalized);
      }
    }
    return out ?? value;
  }
  if (ArrayBuffer.isView(value) || value instanceof ArrayBuffer) {
    return value;
  }
  if (typeof value === 'object') {
    const input = value as Record<string, unknown>;
    const entries = Object.entries(input);
    let out: Record<string, unknown> | undefined;

    for (let i = 0; i < entries.length; i++) {
      const [key, item] = entries[i]!;
      if (item === undefined) continue;
      const normalized = normalizeForMsgpack(item);
      if (normalized === undefined) {
        continue;
      }
      if (!out) {
        if (normalized !== item) {
          out = {};
          for (let j = 0; j < i; j++) {
            const [prevKey, prevItem] = entries[j]!;
            if (prevItem !== undefined) {
              out[prevKey] = prevItem;
            }
          }
          out[key] = normalized;
        }
      } else {
        out[key] = normalized;
      }
    }
    if (!out) {
      for (const [, item] of entries) {
        if (item === undefined) {
          out = {};
          for (const [prevKey, prevItem] of entries) {
            if (prevItem !== undefined) {
              out[prevKey] = prevItem;
            }
          }
          break;
        }
      }
    }
    return out ?? value;
  }
  return value;
}

/**
 * Minimal Command Payload Requirement:
 * Most scene/window commands include a `windowId` so the Core can route them
 * to the correct world/surface. Some commands (e.g. camera update or global
 * maintenance commands) are window-agnostic.
 */
export interface BaseCommandArgs {
  windowId: number;
}

/**
 * Serializes a batch of commands into a byte buffer ready for vulframSendQueue.
 */
export function serializeBatch(batch: CoreCommandBatch): TransportBuffer {
  const normalizedBatch = normalizeForMsgpack(batch) as CoreCommandBatch;
  return packr.pack(normalizedBatch);
}

/**
 * Deserializes responses from vulframReceiveQueue.
 */
export function deserializeResponses(
  buffer: TransportBuffer,
): CommandResponseEnvelope[] {
  if (buffer.length === 0) return [];
  return unpackr.unpack(buffer);
}

/**
 * Deserializes events from vulframReceiveEvents.
 */
export function deserializeEvents(buffer: TransportBuffer): EngineEvent[] {
  if (buffer.length === 0) return [];
  return unpackr.unpack(buffer);
}
