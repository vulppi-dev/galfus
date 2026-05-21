import { decode, encode } from '@msgpack/msgpack';
import { Command } from 'commander';
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

type CoreVersionOptions = {
  attempts: number;
  transport: 'auto' | 'bun' | 'napi';
};

async function parseOptions(): Promise<CoreVersionOptions> {
  const program = new Command();
  program
    .name('core-version')
    .description('Query the runtime build version through a transport package.')
    .option('--transport <mode>', 'Transport selection: auto, bun or napi.', 'auto')
    .option(
      '--attempts <count>',
      'Number of tick attempts before failing.',
      (value) => {
        const parsed = Number.parseInt(value, 10);
        if (!Number.isFinite(parsed) || parsed <= 0) {
          throw new Error(`Invalid attempts value "${value}".`);
        }
        return parsed;
      },
      5
    );

  await program.parseAsync(process.argv);
  const options = program.opts<CoreVersionOptions>();
  return {
    attempts: options.attempts,
    transport: options.transport
  };
}

async function resolveTransportFactory(
  transport: CoreVersionOptions['transport']
): Promise<EngineTransportFactory> {
  if (transport === 'bun') {
    const bunTransport = await import('../packages/transport-bun/src/index');
    return bunTransport.transportBunFfi;
  }
  if (transport === 'napi') {
    const napiTransport = await import('../packages/transport-napi/src/index');
    return napiTransport.transportNapi;
  }
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
    throw new Error(`Invalid response payload type: expected array, got ${typeof decoded}`);
  }
  return decoded as ResponseEnvelope[];
}

async function main(): Promise<void> {
  const options = await parseOptions();
  const transportFactory = await resolveTransportFactory(options.transport);
  const core = transportFactory();
  const commandId = 1;

  try {
    const initResult = core.galfusInit();
    if (initResult !== RESULT_SUCCESS && initResult !== RESULT_ALREADY_INITIALIZED) {
      throw new Error(`galfusInit failed with result=${initResult}`);
    }

    // Clear stale responses before issuing the version command.
    core.galfusReceiveQueue();

    const payload: CmdEnvelope[] = [
      {
        id: commandId,
        type: 'cmd-system-build-version-get',
        content: {}
      }
    ];

    const sendResult = core.galfusSendQueue(encode(payload));
    if (sendResult !== RESULT_SUCCESS) {
      throw new Error(`galfusSendQueue failed with result=${sendResult}`);
    }

    for (let attempt = 0; attempt < options.attempts; attempt += 1) {
      const tickResult = core.galfusTick(Date.now(), 16);
      if (tickResult !== RESULT_SUCCESS) {
        throw new Error(`galfusTick failed with result=${tickResult}`);
      }

      const received = core.galfusReceiveQueue();
      if (received.result !== RESULT_SUCCESS) {
        throw new Error(`galfusReceiveQueue failed with result=${received.result}`);
      }
      if (received.buffer.byteLength === 0) {
        continue;
      }

      const responses = decodeResponses(received.buffer);
      const response = responses.find(
        (entry) => entry.id === commandId && entry.type === 'system-build-version-get'
      );

      if (!response) {
        continue;
      }

      if (!response.content?.success) {
        throw new Error(
          `Core rejected build-version request: ${response.content?.message ?? 'unknown error'}`
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
      `No response for system-build-version-get after ${options.attempts} tick attempts.`
    );
  } finally {
    core.galfusDispose();
  }
}

main().catch((error) => {
  console.error('[core-version] Failed:', error);
  process.exitCode = 1;
});
