import { chmod, mkdir, mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { log } from "./lib/log.ts";

const CURRENT_VERSION = "2.1.280";
const LATEST_VERSION = "2.1.285";
const FIXTURES_DIR = "tests/fixtures";
const BINARY = "target/debug/claude-statusline";
const OUTPUT = "screenshot.png";

const HOUR_MS = 60 * 60 * 1000;
const MINUTE_MS = 60 * 1000;
const DAY_MS = 24 * HOUR_MS;

const ANSI_DEFAULT_FG = "\x1b[39m";
const ANSI_RESET = "\x1b[0m";
const ZERO_WIDTH_SPACE = "​";

const versionCache = (now: Date) => ({
  fetched_at: now.toISOString(),
  latest_version: LATEST_VERSION,
});

const usageCache = (now: Date) => ({
  fetched_at: now.toISOString(),
  response: {
    five_hour: {
      utilization: 35.0,
      resets_at: new Date(
        now.getTime() + 2 * HOUR_MS + 30 * MINUTE_MS,
      ).toISOString(),
    },
    seven_day: {
      utilization: 18.0,
      resets_at: new Date(now.getTime() + 4 * DAY_MS + 6 * HOUR_MS).toISOString(),
    },
  },
});

const fakeClaudeScript = () =>
  `#!/bin/sh\necho '${CURRENT_VERSION} (Claude Code)'\n`;

const writeFakeHome = async (home: string): Promise<void> => {
  const now = new Date();
  const cacheDir = join(home, ".cache", "claude-statusline");
  const binDir = join(home, "bin");
  const claude = join(binDir, "claude");
  const versionPath = join(cacheDir, "version.json");
  const usagePath = join(cacheDir, "usage.json");
  const version = versionCache(now);
  const usage = usageCache(now);
  const versionJson = JSON.stringify(version);
  const usageJson = JSON.stringify(usage);
  const claudeScript = fakeClaudeScript();

  await mkdir(cacheDir, { recursive: true });
  await mkdir(binDir, { recursive: true });
  await Bun.write(versionPath, versionJson);
  await Bun.write(usagePath, usageJson);
  await Bun.write(claude, claudeScript);
  await chmod(claude, 0o755);
};

const runStatusline = async (home: string, root: string): Promise<string> => {
  const fixtures = join(root, FIXTURES_DIR);
  const binary = join(root, BINARY);
  const binDir = join(home, "bin");
  const stdinPath = join(fixtures, "demo_stdin.json");
  const stdin = Bun.file(stdinPath);
  const proc = Bun.spawn([binary], {
    cwd: fixtures,
    env: { ...process.env, HOME: home, PATH: `${binDir}:${process.env["PATH"]}` },
    stdin,
    stdout: "pipe",
    stderr: "pipe",
  });
  const stdout = await new Response(proc.stdout).text();
  const stderr = await new Response(proc.stderr).text();
  const code = await proc.exited;
  if (code !== 0) throw new Error(`claude-statusline failed: ${stderr.trim()}`);
  return stdout;
};

const cleanOutput = (output: string): string =>
  output.replaceAll(ANSI_DEFAULT_FG, ANSI_RESET).replaceAll(ZERO_WIDTH_SPACE, "").trim();

const freezePng = async (input: string, output: string): Promise<void> => {
  const proc = Bun.spawn(
    ["freeze", "--language", "ansi", "--padding", "20", "--output", output],
    {
      stdin: new TextEncoder().encode(input),
      stdout: "pipe",
      stderr: "pipe",
    },
  );
  const stderr = await new Response(proc.stderr).text();
  const code = await proc.exited;
  if (code !== 0) throw new Error(`freeze failed: ${stderr.trim()}`);
};

const main = async () => {
  const root = process.cwd();
  const tmp = tmpdir();
  const prefix = join(tmp, "readme-screenshot-");
  const home = await mkdtemp(prefix);
  const outputPath = join(root, OUTPUT);

  try {
    await writeFakeHome(home);
    const output = await runStatusline(home, root);
    const cleaned = cleanOutput(output);
    await freezePng(cleaned, outputPath);
    log.ok(`wrote ${OUTPUT}`);
  } catch (err) {
    log.error(err instanceof Error ? err.message : String(err));
    process.exitCode = 1;
  } finally {
    await rm(home, { recursive: true });
  }
};

await main();
