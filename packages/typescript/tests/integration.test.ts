import { describe, it, expect, beforeAll, beforeEach, afterEach } from "vitest";
import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Authy } from "../src/client.js";
import { SecretAlreadyExists, SecretNotFound } from "../src/errors.js";

/**
 * Integration tests that require the `authy` binary on PATH.
 * Skipped entirely if the binary is not available.
 */

let hasAuthy = false;

beforeAll(() => {
  try {
    execFileSync("authy", ["--help"], { stdio: "pipe" });
    hasAuthy = true;
  } catch {
    hasAuthy = false;
  }
});

// Helper to create an isolated vault in a temp directory
function createIsolatedClient(): {
  client: Authy;
  tmpDir: string;
  cleanup: () => void;
} {
  const tmpDir = mkdtempSync(join(tmpdir(), "authy-ts-test-"));
  const env = { ...process.env, HOME: tmpDir, AUTHY_PASSPHRASE: "test-pass" };
  // Override process.env temporarily for init
  const origEnv = { ...process.env };
  Object.assign(process.env, env);

  // Initialize vault
  execFileSync("authy", ["init"], {
    env,
    encoding: "utf-8",
    stdio: "pipe",
  });

  // Restore env
  Object.keys(env).forEach((k) => {
    if (origEnv[k] === undefined) delete process.env[k];
    else process.env[k] = origEnv[k];
  });

  const client = new Authy({ passphrase: "test-pass" });
  // We need to set HOME for the client's subprocess calls
  const clientEnv = { ...process.env, HOME: tmpDir };
  // Re-create client that will use the right HOME
  const isolatedClient = new (class extends Authy {
    constructor() {
      super({ passphrase: "test-pass" });
      // Override the env to point HOME at our temp dir
      (this as any).env = { ...clientEnv, AUTHY_PASSPHRASE: "test-pass" };
    }
  })();

  return {
    client: isolatedClient,
    tmpDir,
    cleanup: () => {
      try {
        rmSync(tmpDir, { recursive: true, force: true });
      } catch {
        // ignore cleanup errors
      }
    },
  };
}

describe.skipIf(!hasAuthy)("Integration tests", () => {
  let client: Authy;
  let tmpDir: string;
  let cleanup: () => void;

  beforeEach(() => {
    const ctx = createIsolatedClient();
    client = ctx.client;
    tmpDir = ctx.tmpDir;
    cleanup = ctx.cleanup;
  });

  afterEach(() => {
    cleanup();
  });

  it("init -> store -> get roundtrip", async () => {
    await client.store("db-url", "postgres://localhost/test");
    const value = await client.get("db-url");
    expect(value).toBe("postgres://localhost/test");
  });

  it("store duplicate throws SecretAlreadyExists", async () => {
    await client.store("api-key", "key-1");
    await expect(client.store("api-key", "key-2")).rejects.toThrow(
      SecretAlreadyExists,
    );
  });

  it("store with force overwrites", async () => {
    await client.store("api-key", "key-1");
    await client.store("api-key", "key-2", { force: true });
    const value = await client.get("api-key");
    expect(value).toBe("key-2");
  });

  it("getOrNull returns null for missing secret", async () => {
    const value = await client.getOrNull("nonexistent");
    expect(value).toBeNull();
  });

  it("list returns stored secret names", async () => {
    await client.store("alpha", "a");
    await client.store("beta", "b");
    const names = await client.list();
    expect(names).toContain("alpha");
    expect(names).toContain("beta");
  });

  it("remove deletes a secret", async () => {
    await client.store("to-remove", "value");
    const removed = await client.remove("to-remove");
    expect(removed).toBe(true);
    const value = await client.getOrNull("to-remove");
    expect(value).toBeNull();
  });

  it("remove returns false for nonexistent secret", async () => {
    const removed = await client.remove("nonexistent");
    expect(removed).toBe(false);
  });

  it("rotate increments version", async () => {
    await client.store("rotating", "v1");
    const version = await client.rotate("rotating", "v2");
    expect(version).toBe(2);
    const value = await client.get("rotating");
    expect(value).toBe("v2");
  });
});
