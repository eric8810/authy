import { execFileSync } from "node:child_process";
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

/** Synchronous authy client. Wraps the authy CLI via child_process.execFileSync. */
export class AuthySync {
  private readonly binary: string;
  private readonly env: NodeJS.ProcessEnv;

  constructor(opts: AuthyOptions = {}) {
    this.binary = opts.binary ?? "authy";
    this.env = buildEnv(opts);
  }

  /**
   * Execute an authy CLI command with --json and return parsed stdout.
   */
  private runCmd(
    args: string[],
    stdin?: string,
  ): Record<string, unknown> {
    try {
      const stdout = execFileSync(this.binary, ["--json", ...args], {
        env: this.env,
        encoding: "utf-8",
        maxBuffer: 10 * 1024 * 1024,
        stdio: ["pipe", "pipe", "pipe"],
        ...(stdin !== undefined ? { input: stdin } : {}),
      });
      const out = stdout.trim();
      return out ? (JSON.parse(out) as Record<string, unknown>) : {};
    } catch (err: unknown) {
      const e = err as {
        status?: number;
        stderr?: Buffer | string;
        stdout?: Buffer | string;
      };
      const exitCode = e.status ?? 1;
      const stderr = (e.stderr ?? "").toString();
      throwFromStderr(stderr, exitCode);
    }
  }

  /**
   * Execute an authy CLI command without --json.
   */
  private runRaw(
    args: string[],
    stdin?: string,
  ): { exitCode: number; stdout: string; stderr: string } {
    try {
      const stdout = execFileSync(this.binary, args, {
        env: this.env,
        encoding: "utf-8",
        maxBuffer: 10 * 1024 * 1024,
        stdio: ["pipe", "pipe", "pipe"],
        ...(stdin !== undefined ? { input: stdin } : {}),
      });
      return { exitCode: 0, stdout: stdout ?? "", stderr: "" };
    } catch (err: unknown) {
      const e = err as {
        status?: number;
        stderr?: Buffer | string;
        stdout?: Buffer | string;
      };
      return {
        exitCode: e.status ?? 1,
        stdout: (e.stdout ?? "").toString(),
        stderr: (e.stderr ?? "").toString(),
      };
    }
  }

  /** Retrieve a secret value. Throws SecretNotFound if it does not exist. */
  get(name: string): string {
    const result = this.runCmd(["get", name]) as unknown as GetResponse;
    return result.value;
  }

  /** Retrieve a secret value, returning null if it does not exist. */
  getOrNull(name: string): string | null {
    try {
      return this.get(name);
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
  store(name: string, value: string, opts: StoreOptions = {}): void {
    const args = ["store", name];
    if (opts.force) args.push("--force");
    this.runCmd(args, value);
  }

  /** Remove a secret. Returns true if it existed, false otherwise. */
  remove(name: string): boolean {
    try {
      this.runCmd(["remove", name]);
      return true;
    } catch (err) {
      if (err instanceof AuthyError && err.exitCode === 3) {
        return false;
      }
      throw err;
    }
  }

  /** Rotate a secret to a new value. Returns the new version number. */
  rotate(name: string, value: string): number {
    this.runCmd(["rotate", name], value);
    const result = this.runCmd(["get", name]) as unknown as GetResponse;
    return result.version;
  }

  /** List secret names, optionally filtered by scope. */
  list(opts: ListOptions = {}): string[] {
    const args = ["list"];
    if (opts.scope) {
      args.push("--scope", opts.scope);
    }
    const result = this.runCmd(args) as unknown as ListResponse;
    return result.secrets.map((s) => s.name);
  }

  /** Run a subprocess with secrets injected as environment variables. */
  run(command: string[], opts: RunOptions = {}): RunResult {
    const args = ["run"];
    if (opts.scope) {
      args.push("--scope", opts.scope);
    }
    args.push("--", ...command);
    return this.runRaw(args);
  }

  /** Import secrets from a .env file. */
  importDotenv(path: string, opts: { force?: boolean } = {}): void {
    const args = ["import", path];
    if (opts.force) args.push("--force");
    this.runCmd(args);
  }

  /** Initialize a new vault. */
  init(opts: { keyfile?: string } = {}): void {
    const args = ["init"];
    if (opts.keyfile) {
      args.push("--generate-keyfile", opts.keyfile);
    }
    this.runCmd(args);
  }

  /** Check whether a vault is initialized. */
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
