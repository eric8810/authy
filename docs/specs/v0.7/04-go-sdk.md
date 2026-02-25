# 04 — Go SDK

## Summary

Publish a `go.authy.dev/secrets` Go module that wraps the `authy` CLI, providing a native Go API for vault operations. Thin subprocess wrapper with idiomatic Go error handling.

## Motivation

Go is the dominant language for infrastructure and CLI tooling: Kubernetes operators, Terraform providers, Docker tools, and increasingly agent frameworks (e.g., langchaingo). A Go SDK lets these tools integrate authy natively:

```go
// Today — awkward
out, _ := exec.Command("authy", "get", "db-url").Output()
dbURL := strings.TrimSpace(string(out))
```

```go
// With SDK
client, _ := authy.New()
dbURL, _ := client.Get(ctx, "db-url")
```

## Design: CLI Wrapper

Same approach as Python and TypeScript — subprocess over `authy --json`. No CGO, no Rust FFI. Pure Go, zero dependencies beyond stdlib.

## API Surface

```go
package authy

import "context"

// Construction
client, err := authy.New()                                    // finds authy on PATH
client, err := authy.New(authy.WithBinary("/usr/local/bin/authy"))
client, err := authy.New(authy.WithPassphrase("..."))
client, err := authy.New(authy.WithKeyfile("/path/to/key"))

// Core operations
value, err := client.Get(ctx, "db-url")                       // returns ErrSecretNotFound
value, ok, err := client.GetOpt(ctx, "db-url")                // ok=false if missing, no error
err = client.Store(ctx, "db-url", "postgres://...")            // returns ErrSecretAlreadyExists
err = client.Store(ctx, "db-url", "postgres://...", authy.Force())
removed, err := client.Remove(ctx, "db-url")
version, err := client.Rotate(ctx, "db-url", "new-value")

// List
names, err := client.List(ctx)
names, err := client.List(ctx, authy.WithScope("deploy"))

// Run (subprocess injection)
result, err := client.Run(ctx, []string{"curl", "https://api.example.com"}, authy.WithScope("deploy"))

// Import
err = client.ImportDotenv(ctx, ".env")
err = client.ImportFrom(ctx, "1password", authy.ImportVault("Engineering"))

// Vault management
err = client.Init(ctx)
ok := authy.IsInitialized()  // package-level, no auth
```

### Option Pattern

```go
type Option func(*config)

func WithBinary(path string) Option { ... }
func WithPassphrase(pass string) Option { ... }
func WithKeyfile(path string) Option { ... }

type CallOption func(*callConfig)

func Force() CallOption { ... }
func WithScope(scope string) CallOption { ... }
```

### Error Types

```go
var (
    ErrSecretNotFound      = &AuthyError{ExitCode: 3, Code: "secret_not_found"}
    ErrSecretAlreadyExists = &AuthyError{ExitCode: 5, Code: "secret_exists"}
    ErrAuthFailed          = &AuthyError{ExitCode: 2, Code: "auth_failed"}
    ErrPolicyDenied        = &AuthyError{ExitCode: 4, Code: "policy_denied"}
    ErrVaultNotFound       = &AuthyError{ExitCode: 1, Code: "vault_not_found"}
)

type AuthyError struct {
    ExitCode int
    Code     string
    Message  string
}

func (e *AuthyError) Error() string { return e.Message }

// errors.Is support
func IsSecretNotFound(err error) bool { ... }
func IsAuthFailed(err error) bool { ... }
```

Errors are parsed from `--json` stderr and matched by exit code. Sentinel errors support `errors.Is`.

### Context Support

All methods take `context.Context` for cancellation and timeouts:

```go
ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
defer cancel()
value, err := client.Get(ctx, "db-url")
```

The context is passed to `exec.CommandContext`, so long-running `authy` calls can be cancelled.

## Implementation

### Internal: CLI Execution

```go
func (c *Client) runCmd(ctx context.Context, args []string, stdin string) (map[string]any, error) {
    cmd := exec.CommandContext(ctx, c.binary, append([]string{"--json"}, args...)...)
    cmd.Env = append(os.Environ(), c.extraEnv...)
    if stdin != "" {
        cmd.Stdin = strings.NewReader(stdin)
    }

    var stdout, stderr bytes.Buffer
    cmd.Stdout = &stdout
    cmd.Stderr = &stderr

    if err := cmd.Run(); err != nil {
        return nil, parseError(stderr.Bytes(), cmd.ProcessState.ExitCode())
    }

    if stdout.Len() == 0 {
        return nil, nil
    }

    var result map[string]any
    if err := json.Unmarshal(stdout.Bytes(), &result); err != nil {
        return nil, fmt.Errorf("authy: invalid JSON output: %w", err)
    }
    return result, nil
}
```

### `Get` Example

```go
func (c *Client) Get(ctx context.Context, name string) (string, error) {
    result, err := c.runCmd(ctx, []string{"get", name}, "")
    if err != nil {
        return "", err
    }
    value, ok := result["value"].(string)
    if !ok {
        return "", fmt.Errorf("authy: unexpected response format")
    }
    return value, nil
}
```

## Module Structure

```
packages/go/
  go.mod
  authy.go            # Client struct, New(), options
  operations.go       # Get, Store, Remove, Rotate, List, Run
  errors.go           # AuthyError, sentinel errors, parseError
  authy_test.go       # unit tests (mock exec)
  integration_test.go # integration tests (real binary)
  README.md
```

### `go.mod`

```
module github.com/eric8810/authy-go

go 1.21

// No external dependencies — stdlib only
```

Zero dependencies. Uses `os/exec`, `encoding/json`, `bytes`, `context`, `strings` only.

## Tests

### Unit Tests (mock exec)

Use `exec.Command` override pattern (inject a test binary via `WithBinary`):

- `TestGet_ReturnsValue`
- `TestGet_SecretNotFound`
- `TestStore_PassesStdin`
- `TestList_ReturnsNames`
- `TestCredentialsInEnv`
- `TestContextCancellation`

### Integration Tests (require `authy` binary)

Build-tagged with `//go:build integration`:

- `TestInitStoreGetRoundtrip`
- `TestListWithScope`
- `TestStoreDuplicateReturnsError`
- `TestRotateIncrementsVersion`

Skipped if `authy` is not on PATH.

## Acceptance Criteria

- [ ] `go get github.com/eric8810/authy-go` installs with zero deps
- [ ] `client.Get(ctx, "name")` retrieves a secret
- [ ] `client.Store(ctx, "name", "value")` passes value via stdin
- [ ] Errors support `errors.Is` with sentinel values
- [ ] All methods accept `context.Context` for cancellation
- [ ] Idiomatic Go option pattern for configuration
- [ ] Unit tests pass with mocked binary
- [ ] Integration tests pass against real authy binary
- [ ] Module published and importable via `go get`
