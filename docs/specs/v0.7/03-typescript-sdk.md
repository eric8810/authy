# 03 — TypeScript SDK

## Summary

Publish an `authy-secrets` npm package that wraps the `authy` CLI, providing a native TypeScript/Node.js API for vault operations. Thin subprocess wrapper with full type definitions.

## Motivation

TypeScript developers build MCP servers, n8n nodes, and agent tools. The MCP ecosystem (Claude Desktop, Cursor, Continue) is heavily TypeScript. A native package lowers friction:

```typescript
// Today — awkward
import { execSync } from "child_process";
const dbUrl = execSync("authy get db-url").toString().trim();
```

```typescript
// With SDK
import { Authy } from "authy-secrets";
const client = new Authy();
const dbUrl = await client.get("db-url");
```

## Design: CLI Wrapper

Same rationale as the Python SDK — subprocess over `authy --json`. The CLI is the stable interface. No WASM, no native addons, no N-API.

## API Surface

```typescript
import { Authy, SecretNotFound, AuthyError } from "authy-secrets";

// Construction
const client = new Authy();
const client = new Authy({ binary: "/usr/local/bin/authy" });
const client = new Authy({ passphrase: "..." });
const client = new Authy({ keyfile: "/path/to/key" });

// Core operations (all async)
const value: string = await client.get("db-url");           // throws SecretNotFound
const value: string | null = await client.getOrNull("db-url");
await client.store("db-url", "postgres://...");             // throws SecretAlreadyExists
await client.store("db-url", "postgres://...", { force: true });
const removed: boolean = await client.remove("db-url");
const version: number = await client.rotate("db-url", "new-value");

// List
const names: string[] = await client.list();
const names: string[] = await client.list({ scope: "deploy" });

// Run (subprocess injection)
const result = await client.run(["curl", "https://api.example.com"], { scope: "deploy" });

// Import
await client.importDotenv(".env");
await client.importFrom("1password", { vault: "Engineering" });

// Vault management
await client.init();
const ready: boolean = Authy.isInitialized();
```

### Type Definitions

```typescript
interface AuthyOptions {
  binary?: string;
  passphrase?: string;
  keyfile?: string;
}

interface StoreOptions {
  force?: boolean;
}

interface ListOptions {
  scope?: string;
}

interface RunOptions {
  scope?: string;
}

interface RunResult {
  exitCode: number;
  stdout: string;
  stderr: string;
}
```

### Error Types

```typescript
class AuthyError extends Error {
  exitCode: number;
  errorCode: string;
}

class SecretNotFound extends AuthyError {}
class SecretAlreadyExists extends AuthyError {}
class AuthFailed extends AuthyError {}
class PolicyDenied extends AuthyError {}
class VaultNotFound extends AuthyError {}
```

### Sync Variants

For scripts and CLIs that don't want async:

```typescript
import { AuthySync } from "authy-secrets";

const client = new AuthySync();
const value: string = client.get("db-url");  // synchronous
```

`AuthySync` uses `execSync` instead of `execFile`. Same API surface, no `await`.

## Implementation

### Internal: CLI Execution

```typescript
class Authy {
  private async runCmd(args: string[], stdin?: string): Promise<Record<string, unknown>> {
    const { stdout, stderr, exitCode } = await execFile(
      this.binary, ["--json", ...args],
      { env: { ...process.env, ...this.extraEnv }, input: stdin }
    );

    if (exitCode !== 0) {
      const error = JSON.parse(stderr).error;
      throw mapError(error, exitCode);
    }

    return stdout.trim() ? JSON.parse(stdout) : {};
  }
}
```

### Error Mapping

```typescript
function mapError(error: { code: string; message: string }, exitCode: number): AuthyError {
  switch (exitCode) {
    case 2: return new AuthFailed(error.message, exitCode, error.code);
    case 3: return new SecretNotFound(error.message, exitCode, error.code);
    case 4: return new PolicyDenied(error.message, exitCode, error.code);
    // ...
    default: return new AuthyError(error.message, exitCode, error.code);
  }
}
```

## Package Structure

```
packages/typescript/
  package.json
  tsconfig.json
  src/
    index.ts          # re-exports
    client.ts         # Authy async class
    sync.ts           # AuthySync class
    errors.ts         # error hierarchy
    types.ts          # interfaces
  tests/
    client.test.ts    # unit tests (mocked subprocess)
    integration.test.ts
  README.md
```

### `package.json`

```json
{
  "name": "authy-secrets",
  "version": "0.7.0",
  "description": "TypeScript SDK for the authy secrets manager",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "exports": {
    ".": {
      "import": "./dist/index.mjs",
      "require": "./dist/index.js",
      "types": "./dist/index.d.ts"
    }
  },
  "engines": { "node": ">=18" },
  "license": "MIT",
  "keywords": ["secrets", "vault", "agents", "ai", "mcp"]
}
```

No runtime dependencies. Uses `node:child_process` only.

Build with `tsup` or `tsc` — dual CJS/ESM output.

## Tests

### Unit Tests (mocked child_process)

- `get returns value` — mock authy output, assert value
- `get throws SecretNotFound` — mock exit 3, assert exception type
- `store passes value via stdin` — verify stdin written to subprocess
- `list returns names` — mock JSON list output
- `credentials passed via env` — verify AUTHY_KEYFILE set in spawn env
- `custom binary path` — verify path used

### Integration Tests (require `authy` binary)

- `init → store → get roundtrip`
- `list with scope filter`
- `store duplicate throws`
- `rotate increments version`

Integration tests check for `authy` on PATH and skip if missing.

## Acceptance Criteria

- [ ] `npm install authy-secrets` installs with zero native deps
- [ ] `new Authy().get("name")` retrieves a secret (async)
- [ ] `new AuthySync().get("name")` retrieves a secret (sync)
- [ ] `store()` passes values via stdin
- [ ] Errors map to typed TypeScript exceptions
- [ ] Full `.d.ts` type definitions published
- [ ] Dual CJS/ESM output
- [ ] Unit tests pass with mocked subprocess
- [ ] Integration tests pass against real authy binary
- [ ] Package published to npm as `authy-secrets`
