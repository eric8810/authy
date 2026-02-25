import { execFile, execFileSync, type ExecFileOptions } from "node:child_process";
import { mapError, AuthyError } from "./errors.js";
import type {
  AuthyOptions,
  StoreOptions,
  ListOptions,
  RunOptions,
  RunResult,
  GetResponse,
  ListResponse,
  JsonErrorResponse,
} from "./types.js";

/**
 * Build the environment object for spawning the authy CLI.
 * Credentials are passed via env vars, never as argv.
 */
function buildEnv(opts: AuthyOptions): NodeJS.ProcessEnv {
  const env: NodeJS.ProcessEnv = { ...process.env };
  if (opts.passphrase) {
    env.AUTHY_PASSPHRASE = opts.passphrase;
  }
  if (opts.keyfile) {
    env.AUTHY_KEYFILE = opts.keyfile;
  }
  return env;
}

/**
 * Parse a JSON error from stderr and throw a typed exception.
 * Falls back to a generic AuthyError if stderr is not valid JSON.
 */
function throwFromStderr(stderr: string, exitCode: number): never {
  try {
    const parsed: JsonErrorResponse = JSON.parse(stderr);
    throw mapError(parsed.error, exitCode);
  } catch (e) {
    if (e instanceof AuthyError) throw e;
    throw new AuthyError(
      stderr.trim() || `authy exited with code ${exitCode}`,
      exitCode,
      "unknown",
    );
  }
}

/**
 * Promise wrapper around execFile that supports stdin via input option.
 * We avoid promisify to make mocking simpler in tests.
 */
function execFilePromise(
  binary: string,
  args: string[],
  options: ExecFileOptions & { input?: string },
): Promise<{ stdout: string; stderr: string }> {
  return new Promise((resolve, reject) => {
    const child = execFile(
      binary,
      args,
      { ...options, encoding: "utf-8" } as ExecFileOptions,
      (error, stdout, stderr) => {
        if (error) {
          const e = error as NodeJS.ErrnoException & {
            status?: number;
            code?: string | number;
          };
          // Attach stderr/stdout to error for callers
          reject(
            Object.assign(e, {
              stdout: (stdout ?? "").toString(),
              stderr: (stderr ?? "").toString(),
              status: e.status ?? (typeof e.code === "number" ? e.code : 1),
            }),
          );
        } else {
          resolve({
            stdout: (stdout ?? "").toString(),
            stderr: (stderr ?? "").toString(),
          });
        }
      },
    );
    if (options.input !== undefined && child.stdin) {
      child.stdin.write(options.input);
      child.stdin.end();
    }
  });
}

/** Async authy client. Wraps the authy CLI via child_process.execFile. */
export class Authy {
  private readonly binary: string;
  private readonly env: NodeJS.ProcessEnv;

  constructor(opts: AuthyOptions = {}) {
    this.binary = opts.binary ?? "authy";
    this.env = buildEnv(opts);
  }

  /**
   * Execute an authy CLI command with --json and return parsed stdout.
   * Secret values are passed via stdin, never as arguments.
   */
  private async runCmd(
    args: string[],
    stdin?: string,
  ): Promise<Record<string, unknown>> {
    try {
      const { stdout } = await execFilePromise(
        this.binary,
        ["--json", ...args],
        {
          env: this.env,
          maxBuffer: 10 * 1024 * 1024,
          ...(stdin !== undefined ? { input: stdin } : {}),
        },
      );
      const out = stdout.trim();
      return out ? (JSON.parse(out) as Record<string, unknown>) : {};
    } catch (err: unknown) {
      const e = err as {
        code?: string;
        status?: number;
        stderr?: string;
        stdout?: string;
      };
      const exitCode = e.status ?? 1;
      const stderr = (e.stderr ?? "").toString();
      throwFromStderr(stderr, exitCode);
    }
  }

  /**
   * Execute an authy CLI command without --json.
   * Used for commands like `run` that should not parse JSON output.
   */
  private async runRaw(
    args: string[],
    stdin?: string,
  ): Promise<{ exitCode: number; stdout: string; stderr: string }> {
    try {
      const { stdout, stderr } = await execFilePromise(this.binary, args, {
        env: this.env,
        maxBuffer: 10 * 1024 * 1024,
        ...(stdin !== undefined ? { input: stdin } : {}),
      });
      return { exitCode: 0, stdout, stderr };
    } catch (err: unknown) {
      const e = err as {
        code?: string;
        status?: number;
        stderr?: string;
        stdout?: string;
      };
      return {
        exitCode: e.status ?? 1,
        stdout: (e.stdout ?? "").toString(),
        stderr: (e.stderr ?? "").toString(),
      };
    }
  }

  /** Retrieve a secret value. Throws SecretNotFound if it does not exist. */
  async get(name: string): Promise<string> {
    const result = (await this.runCmd(["get", name])) as unknown as GetResponse;
    return result.value;
  }

  /** Retrieve a secret value, returning null if it does not exist. */
  async getOrNull(name: string): Promise<string | null> {
    try {
      return await this.get(name);
    } catch (err) {
      if (err instanceof AuthyError && err.exitCode === 3) {
        return null;
      }
      throw err;
    }
  }

  /**
   * Store a secret. Value is passed via stdin.
   * Throws SecretAlreadyExists unless force is true.
   */
  async store(
    name: string,
    value: string,
    opts: StoreOptions = {},
  ): Promise<void> {
    const args = ["store", name];
    if (opts.force) args.push("--force");
    await this.runCmd(args, value);
  }

  /** Remove a secret. Returns true if it existed, false otherwise. */
  async remove(name: string): Promise<boolean> {
    try {
      await this.runCmd(["remove", name]);
      return true;
    } catch (err) {
      if (err instanceof AuthyError && err.exitCode === 3) {
        return false;
      }
      throw err;
    }
  }

  /** Rotate a secret to a new value. Returns the new version number. */
  async rotate(name: string, value: string): Promise<number> {
    await this.runCmd(["rotate", name], value);
    const result = (await this.runCmd(["get", name])) as unknown as GetResponse;
    return result.version;
  }

  /** List secret names, optionally filtered by scope. */
  async list(opts: ListOptions = {}): Promise<string[]> {
    const args = ["list"];
    if (opts.scope) {
      args.push("--scope", opts.scope);
    }
    const result = (await this.runCmd(args)) as unknown as ListResponse;
    return result.secrets.map((s) => s.name);
  }

  /** Run a subprocess with secrets injected as environment variables. */
  async run(command: string[], opts: RunOptions = {}): Promise<RunResult> {
    const args = ["run"];
    if (opts.scope) {
      args.push("--scope", opts.scope);
    }
    args.push("--", ...command);
    return this.runRaw(args);
  }

  /** Import secrets from a .env file. */
  async importDotenv(
    path: string,
    opts: { force?: boolean } = {},
  ): Promise<void> {
    const args = ["import", path];
    if (opts.force) args.push("--force");
    await this.runCmd(args);
  }

  /** Initialize a new vault. */
  async init(opts: { keyfile?: string } = {}): Promise<void> {
    const args = ["init"];
    if (opts.keyfile) {
      args.push("--generate-keyfile", opts.keyfile);
    }
    await this.runCmd(args);
  }

  /** Check whether a vault is initialized. This is a static check. */
  static isInitialized(binary = "authy"): boolean {
    try {
      execFileSync(binary, ["--json", "list"], {
        encoding: "utf-8",
        stdio: ["pipe", "pipe", "pipe"],
      });
      return true;
    } catch {
      return false;
    }
  }
}
