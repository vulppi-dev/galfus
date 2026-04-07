import { DEMO_IDS, type DemoId } from './index';

function normalizeDemoId(raw: string | undefined): DemoId | null {
  if (!raw) return null;
  const trimmed = raw.trim();
  const padded = /^\d+$/.test(trimmed) ? trimmed.padStart(3, '0') : trimmed;
  if ((DEMO_IDS as readonly string[]).includes(padded)) {
    return padded as DemoId;
  }
  return null;
}

function printUsage(): void {
  console.error(`Usage: bun run demo <id>\\nAvailable demos: ${DEMO_IDS.join(', ')}`);
}

async function main(): Promise<void> {
  const demoId = normalizeDemoId(process.argv[2]);
  if (!demoId) {
    printUsage();
    process.exitCode = 1;
    return;
  }

  await import(`./${demoId}.demo.ts`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
