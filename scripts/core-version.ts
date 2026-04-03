import { decode, encode } from '@msgpack/msgpack';
import type { EngineTransportFactory } from '../packages/transport-types/src/index';

type CmdEnvelope = {
  id: number;
  type: string;
  content: Record<string, unknown>;
};

type ResponseEnvelope = {
  id?: number;
  type?: string;
  content?: {
    success?: boolean;
    message?: string;
    buildVersion?: string;
  };
};

const RESULT_SUCCESS = 0;
const RESULT_ALREADY_INITIALIZED = 3;

async function resolveTransportFactory(): Promise<EngineTransportFactory> {
  if (typeof Bun !== 'undefined') {
    const bunTransport = await import('../packages/transport-bun/src/index');
    return bunTransport.transportBunFfi;
  }
  const napiTransport = await import('../packages/transport-napi/src/index');
  return napiTransport.transportNapi;
}

function decodeResponses(bytes: Uint8Array): ResponseEnvelope[] {
  const decoded = decode(bytes);
  if (!Array.isArray(decoded)) {
    throw new Error(
      `Invalid response payload type: expected array, got ${typeof decoded}`,
    );
  }
  return decoded as ResponseEnvelope[];
}

async function main(): Promise<void> {
  const transportFactory = await resolveTransportFactory();
  const core = transportFactory();
  const commandId = 1;

  try {
    const initResult = core.vulframInit();
    if (
      initResult !== RESULT_SUCCESS &&
      initResult !== RESULT_ALREADY_INITIALIZED
    ) {
      throw new Error(`vulframInit failed with result=${initResult}`);
    }

    // Clear stale responses before issuing the version command.
    core.vulframReceiveQueue();

    const payload: CmdEnvelope[] = [
      {
        id: commandId,
        type: 'cmd-system-build-version-get',
        content: {},
      },
    ];

    const sendResult = core.vulframSendQueue(encode(payload));
    if (sendResult !== RESULT_SUCCESS) {
      throw new Error(`vulframSendQueue failed with result=${sendResult}`);
    }

    for (let attempt = 0; attempt < 5; attempt += 1) {
      const tickResult = core.vulframTick(Date.now(), 16);
      if (tickResult !== RESULT_SUCCESS) {
        throw new Error(`vulframTick failed with result=${tickResult}`);
      }

      const received = core.vulframReceiveQueue();
      if (received.result !== RESULT_SUCCESS) {
        throw new Error(
          `vulframReceiveQueue failed with result=${received.result}`,
        );
      }
      if (received.buffer.byteLength === 0) {
        continue;
      }

      const responses = decodeResponses(received.buffer);
      const response = responses.find(
        (entry) => entry.id === commandId && entry.type === 'system-build-version-get',
      );

      if (!response) {
        continue;
      }

      if (!response.content?.success) {
        throw new Error(
          `Core rejected build-version request: ${response.content?.message ?? 'unknown error'}`,
        );
      }

      const version = response.content.buildVersion;
      if (!version) {
        throw new Error('Core response missing buildVersion');
      }

      console.log(version);
      return;
    }

    throw new Error(
      'No response for system-build-version-get after 5 tick attempts.',
    );
  } finally {
    core.vulframDispose();
  }
}

main().catch((error) => {
  console.error('[core-version] Failed:', error);
  process.exitCode = 1;
});
