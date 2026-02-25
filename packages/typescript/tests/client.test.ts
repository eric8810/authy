import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

// We need to mock child_process before importing the modules under test.
// The async client uses execFile (callback) and the sync client uses execFileSync.
vi.mock("node:child_process", () => {
  const mockExecFile = vi.fn();
  const mockExecFileSync = vi.fn();
  return {
    execFile: mockExecFile,
    execFileSync: mockExecFileSync,
  };
});

import { execFile, execFileSync } from "node:child_process";
import { Authy } from "../src/client.js";
import { AuthySync } from "../src/sync.js";
import {
  AuthyError,
  SecretNotFound,
  SecretAlreadyExists,
  AuthFailed,
  PolicyDenied,
  VaultNotFound,
} from "../src/errors.js";

const mockedExecFile = execFile as unknown as Mock;
const mockedExecFileSync = execFileSync as unknown as Mock;

/** Helper: make the mocked execFile succeed with given stdout. */
function mockSuccess(stdout: string) {
  mockedExecFile.mockImplementation(
    (_cmd: string, _args: string[], _opts: unknown, cb: Function) => {
      cb(null, stdout, "");
      // Return a mock child process with a writable stdin
      return { stdin: { write: vi.fn(), end: vi.fn() } };
    },
  );
}

/** Helper: make the mocked execFile fail with given exit code and stderr JSON. */
function mockFailure(exitCode: number, stderrJson: string) {
  mockedExecFile.mockImplementation(
    (_cmd: string, _args: string[], _opts: unknown, cb: Function) => {
      const err = Object.assign(new Error(`exit ${exitCode}`), {
        status: exitCode,
        stderr: stderrJson,
        stdout: "",
      });
      cb(err, "", stderrJson);
      return { stdin: { write: vi.fn(), end: vi.fn() } };
    },
  );
}

beforeEach(() => {
  vi.clearAllMocks();
});

// ── Async Client Tests ──────────────────────────────────────────────

describe("Authy (async)", () => {
  describe("get", () => {
    it("returns the secret value", async () => {
      const response = JSON.stringify({
        name: "db-url",
        value: "postgres://localhost/mydb",
        version: 1,
        created: "2025-01-01T00:00:00Z",
        modified: "2025-01-01T00:00:00Z",
      });

      mockSuccess(response);

      const client = new Authy();
      const value = await client.get("db-url");
      expect(value).toBe("postgres://localhost/mydb");

      // Verify --json flag is passed
      expect(mockedExecFile).toHaveBeenCalledWith(
        "authy",
        ["--json", "get", "db-url"],
        expect.any(Object),
        expect.any(Function),
      );
    });

    it("throws SecretNotFound on exit code 3", async () => {
      const errJson = JSON.stringify({
        error: {
          code: "not_found",
          message: "Secret not found: db-url",
          exit_code: 3,
        },
      });

      mockFailure(3, errJson);

      const client = new Authy();
      await expect(client.get("db-url")).rejects.toThrow(SecretNotFound);
      await expect(client.get("db-url")).rejects.toMatchObject({
        exitCode: 3,
        errorCode: "not_found",
      });
    });
  });

  describe("getOrNull", () => {
    it("returns null when secret not found", async () => {
      const errJson = JSON.stringify({
        error: {
          code: "not_found",
          message: "Secret not found: missing",
          exit_code: 3,
        },
      });

      mockFailure(3, errJson);

      const client = new Authy();
      const value = await client.getOrNull("missing");
      expect(value).toBeNull();
    });
  });

  describe("store", () => {
    it("passes value via stdin, not as argument", async () => {
      const mockWrite = vi.fn();
      const mockEnd = vi.fn();
      mockedExecFile.mockImplementation(
        (_cmd: string, _args: string[], _opts: unknown, cb: Function) => {
          cb(null, "", "");
          return { stdin: { write: mockWrite, end: mockEnd } };
        },
      );

      const client = new Authy();
      await client.store("api-key", "my-secret-value");

      // Verify stdin was used to pass the value
      expect(mockWrite).toHaveBeenCalledWith("my-secret-value");
      expect(mockEnd).toHaveBeenCalled();

      // Verify value is NOT in argv
      const callArgs = mockedExecFile.mock.calls[0];
      const cliArgs = callArgs[1] as string[];
      expect(cliArgs).not.toContain("my-secret-value");
      expect(cliArgs).toEqual(["--json", "store", "api-key"]);
    });

    it("passes --force when option is set", async () => {
      mockSuccess("");

      const client = new Authy();
      await client.store("api-key", "value", { force: true });

      const callArgs = mockedExecFile.mock.calls[0];
      const cliArgs = callArgs[1] as string[];
      expect(cliArgs).toEqual(["--json", "store", "api-key", "--force"]);
    });

    it("throws SecretAlreadyExists on exit code 5", async () => {
      const errJson = JSON.stringify({
        error: {
          code: "already_exists",
          message: "Secret already exists: api-key",
          exit_code: 5,
        },
      });

      mockFailure(5, errJson);

      const client = new Authy();
      await expect(client.store("api-key", "val")).rejects.toThrow(
        SecretAlreadyExists,
      );
    });
  });

  describe("list", () => {
    it("returns secret names", async () => {
      const response = JSON.stringify({
        secrets: [
          {
            name: "db-url",
            version: 1,
            created: "2025-01-01T00:00:00Z",
            modified: "2025-01-01T00:00:00Z",
          },
          {
            name: "api-key",
            version: 2,
            created: "2025-01-01T00:00:00Z",
            modified: "2025-01-02T00:00:00Z",
          },
        ],
      });

      mockSuccess(response);

      const client = new Authy();
      const names = await client.list();
      expect(names).toEqual(["db-url", "api-key"]);
    });

    it("passes scope option", async () => {
      mockSuccess(JSON.stringify({ secrets: [] }));

      const client = new Authy();
      await client.list({ scope: "deploy" });

      const callArgs = mockedExecFile.mock.calls[0];
      const cliArgs = callArgs[1] as string[];
      expect(cliArgs).toEqual(["--json", "list", "--scope", "deploy"]);
    });
  });

  describe("credentials via env", () => {
    it("passes passphrase via AUTHY_PASSPHRASE env var", async () => {
      mockedExecFile.mockImplementation(
        (_cmd: string, _args: string[], opts: any, cb: Function) => {
          expect(opts.env.AUTHY_PASSPHRASE).toBe("test-pass");
          cb(null, JSON.stringify({ secrets: [] }), "");
          return { stdin: { write: vi.fn(), end: vi.fn() } };
        },
      );

      const client = new Authy({ passphrase: "test-pass" });
      await client.list();
    });

    it("passes keyfile via AUTHY_KEYFILE env var", async () => {
      mockedExecFile.mockImplementation(
        (_cmd: string, _args: string[], opts: any, cb: Function) => {
          expect(opts.env.AUTHY_KEYFILE).toBe("/path/to/key");
          cb(null, JSON.stringify({ secrets: [] }), "");
          return { stdin: { write: vi.fn(), end: vi.fn() } };
        },
      );

      const client = new Authy({ keyfile: "/path/to/key" });
      await client.list();
    });
  });

  describe("custom binary path", () => {
    it("uses the provided binary path", async () => {
      mockSuccess(JSON.stringify({ secrets: [] }));

      const client = new Authy({ binary: "/usr/local/bin/authy" });
      await client.list();

      const callArgs = mockedExecFile.mock.calls[0];
      expect(callArgs[0]).toBe("/usr/local/bin/authy");
    });
  });

  describe("error mapping", () => {
    it("throws AuthFailed on exit code 2", async () => {
      const errJson = JSON.stringify({
        error: {
          code: "auth_failed",
          message: "Authentication failed",
          exit_code: 2,
        },
      });

      mockFailure(2, errJson);

      const client = new Authy();
      await expect(client.get("x")).rejects.toThrow(AuthFailed);
    });

    it("throws PolicyDenied on exit code 4", async () => {
      const errJson = JSON.stringify({
        error: {
          code: "access_denied",
          message: "Access denied",
          exit_code: 4,
        },
      });

      mockFailure(4, errJson);

      const client = new Authy();
      await expect(client.get("x")).rejects.toThrow(PolicyDenied);
    });

    it("throws VaultNotFound on exit code 7", async () => {
      const errJson = JSON.stringify({
        error: {
          code: "vault_not_initialized",
          message: "Vault not initialized",
          exit_code: 7,
        },
      });

      mockFailure(7, errJson);

      const client = new Authy();
      await expect(client.get("x")).rejects.toThrow(VaultNotFound);
    });

    it("throws generic AuthyError for unknown exit codes", async () => {
      const errJson = JSON.stringify({
        error: {
          code: "io_error",
          message: "IO error",
          exit_code: 1,
        },
      });

      mockFailure(1, errJson);

      const client = new Authy();
      await expect(client.get("x")).rejects.toThrow(AuthyError);
      await expect(client.get("x")).rejects.toMatchObject({
        exitCode: 1,
        errorCode: "io_error",
      });
    });
  });

  describe("remove", () => {
    it("returns true when secret is removed", async () => {
      mockSuccess("");

      const client = new Authy();
      const removed = await client.remove("api-key");
      expect(removed).toBe(true);
    });

    it("returns false when secret does not exist", async () => {
      const errJson = JSON.stringify({
        error: {
          code: "not_found",
          message: "Secret not found: api-key",
          exit_code: 3,
        },
      });

      mockFailure(3, errJson);

      const client = new Authy();
      const removed = await client.remove("api-key");
      expect(removed).toBe(false);
    });
  });

  describe("run", () => {
    it("passes scope and command correctly", async () => {
      // run uses runRaw (no --json), success path
      mockedExecFile.mockImplementation(
        (_cmd: string, _args: string[], _opts: unknown, cb: Function) => {
          cb(null, "output", "");
          return { stdin: { write: vi.fn(), end: vi.fn() } };
        },
      );

      const client = new Authy();
      const result = await client.run(["curl", "https://api.example.com"], {
        scope: "deploy",
      });
      expect(result.exitCode).toBe(0);
      expect(result.stdout).toBe("output");

      const callArgs = mockedExecFile.mock.calls[0];
      const cliArgs = callArgs[1] as string[];
      // run is not called with --json
      expect(cliArgs).toEqual([
        "run",
        "--scope",
        "deploy",
        "--",
        "curl",
        "https://api.example.com",
      ]);
    });
  });
});

// ── Sync Client Tests ───────────────────────────────────────────────

describe("AuthySync", () => {
  describe("get", () => {
    it("returns the secret value", () => {
      const response = JSON.stringify({
        name: "db-url",
        value: "postgres://localhost/mydb",
        version: 1,
        created: "2025-01-01T00:00:00Z",
        modified: "2025-01-01T00:00:00Z",
      });

      mockedExecFileSync.mockReturnValue(response);

      const client = new AuthySync();
      const value = client.get("db-url");
      expect(value).toBe("postgres://localhost/mydb");
    });

    it("throws SecretNotFound on exit code 3", () => {
      const errJson = JSON.stringify({
        error: {
          code: "not_found",
          message: "Secret not found: db-url",
          exit_code: 3,
        },
      });

      mockedExecFileSync.mockImplementation(() => {
        throw Object.assign(new Error("exit 3"), {
          status: 3,
          stderr: errJson,
          stdout: "",
        });
      });

      const client = new AuthySync();
      expect(() => client.get("db-url")).toThrow(SecretNotFound);
    });
  });

  describe("store", () => {
    it("passes value via stdin option", () => {
      mockedExecFileSync.mockReturnValue("");

      const client = new AuthySync();
      client.store("api-key", "my-secret-value");

      const callArgs = mockedExecFileSync.mock.calls[0];
      const cliArgs = callArgs[1] as string[];
      const opts = callArgs[2] as Record<string, unknown>;
      expect(cliArgs).toEqual(["--json", "store", "api-key"]);
      expect(opts.input).toBe("my-secret-value");
    });
  });

  describe("list", () => {
    it("returns secret names", () => {
      const response = JSON.stringify({
        secrets: [
          {
            name: "a",
            version: 1,
            created: "2025-01-01T00:00:00Z",
            modified: "2025-01-01T00:00:00Z",
          },
          {
            name: "b",
            version: 1,
            created: "2025-01-01T00:00:00Z",
            modified: "2025-01-01T00:00:00Z",
          },
        ],
      });

      mockedExecFileSync.mockReturnValue(response);

      const client = new AuthySync();
      const names = client.list();
      expect(names).toEqual(["a", "b"]);
    });
  });

  describe("getOrNull", () => {
    it("returns null when secret not found", () => {
      const errJson = JSON.stringify({
        error: {
          code: "not_found",
          message: "Secret not found: missing",
          exit_code: 3,
        },
      });

      mockedExecFileSync.mockImplementation(() => {
        throw Object.assign(new Error("exit 3"), {
          status: 3,
          stderr: errJson,
          stdout: "",
        });
      });

      const client = new AuthySync();
      expect(client.getOrNull("missing")).toBeNull();
    });
  });

  describe("credentials via env", () => {
    it("passes keyfile via AUTHY_KEYFILE env var", () => {
      mockedExecFileSync.mockReturnValue(
        JSON.stringify({ secrets: [] }),
      );

      const client = new AuthySync({ keyfile: "/path/to/key" });
      client.list();

      const callArgs = mockedExecFileSync.mock.calls[0];
      const opts = callArgs[2] as Record<string, any>;
      expect(opts.env.AUTHY_KEYFILE).toBe("/path/to/key");
    });
  });
});
