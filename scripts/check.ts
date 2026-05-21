type Command = {
  args: string[];
  label: string;
};

const COMMANDS: Command[] = [
  {
    label: 'cargo check -p galfus-bindings-ffi --lib',
    args: ['cargo', 'check', '-p', 'galfus-bindings-ffi', '--lib']
  },
  {
    label: 'cargo check -p galfus-runtime --lib',
    args: ['cargo', 'check', '-p', 'galfus-runtime', '--lib']
  },
  {
    label: 'cargo run --bin wgsl_check',
    args: ['cargo', 'run', '-p', 'galfus-devtools', '--bin', 'wgsl_check']
  },
  {
    label: 'cargo test -p galfus-runtime --lib',
    args: ['cargo', 'test', '-p', 'galfus-runtime', '--lib']
  },
  {
    label: 'cargo fmt --all',
    args: ['cargo', 'fmt', '--all']
  }
];

async function runCommand(command: Command): Promise<void> {
  console.log(`[check] ${command.label}`);
  const proc = Bun.spawn({
    cmd: command.args,
    cwd: process.cwd(),
    stdout: 'inherit',
    stderr: 'inherit',
    stdin: 'inherit'
  });
  const exitCode = await proc.exited;
  if (exitCode !== 0) {
    throw new Error(`Command failed (${exitCode}): ${command.label}`);
  }
}

async function main(): Promise<void> {
  for (const command of COMMANDS) {
    await runCommand(command);
  }
}

main().catch((error) => {
  console.error('[check] Failed:', error);
  process.exitCode = 1;
});
