import { execFile, spawn } from "node:child_process";
import { createInterface } from "node:readline";
import { homedir } from "node:os";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);

const VW = process.env.VARA_WALLET ?? `${homedir()}/.npm-global/bin/vara-wallet`;
const MAX_BUFFER = 4 * 1024 * 1024;

export interface CallOptions {
  idl: string;
  mnemonic: string;
  voucher?: string;
  value?: string;
}

export interface CallResult {
  txHash?: string;
  blockNumber?: number;
  result?: unknown;
  reason?: string;
  error?: string;
}

function lastJsonLine(stdout: string): string {
  const lines = stdout.trim().split("\n").filter((l) => l.trim().startsWith("{"));
  return lines[lines.length - 1] ?? "{}";
}

export async function call(
  programId: string,
  method: string,
  args: unknown[],
  opts: CallOptions,
): Promise<CallResult> {
  const cliArgs = [
    "--network", "mainnet", "--json", "--mnemonic", opts.mnemonic,
    "call", programId, method,
    "--args", JSON.stringify(args),
    "--idl", opts.idl,
  ];
  if (opts.voucher) cliArgs.push("--voucher", opts.voucher);
  if (opts.value) cliArgs.push("--value", opts.value);
  const { stdout } = await execFileAsync(VW, cliArgs, { maxBuffer: MAX_BUFFER });
  return JSON.parse(lastJsonLine(stdout)) as CallResult;
}

export function watch(
  programId: string,
  idl: string,
  onEvent: (raw: unknown) => void,
): () => void {
  const child = spawn(VW, ["--network", "mainnet", "watch", programId, "--idl", idl]);
  const reader = createInterface({ input: child.stdout });
  reader.on("line", (line) => {
    const trimmed = line.trim();
    if (!trimmed.startsWith("{")) return;
    try {
      onEvent(JSON.parse(trimmed));
    } catch {
      reader.resume();
    }
  });
  return () => child.kill();
}
