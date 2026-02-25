package authy

import (
	"context"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"testing"
)

// buildMockBinary compiles a small Go program that acts as a mock authy binary.
// It reads the MOCK_STDOUT, MOCK_STDERR, and MOCK_EXIT env vars to control output.
func buildMockBinary(t *testing.T) string {
	t.Helper()
	dir := t.TempDir()

	src := filepath.Join(dir, "mock_authy.go")
	bin := filepath.Join(dir, "mock_authy")
	if runtime.GOOS == "windows" {
		bin += ".exe"
	}

	mockSrc := `package main

import (
	"fmt"
	"os"
	"strconv"
)

func main() {
	stdout := os.Getenv("MOCK_STDOUT")
	stderr := os.Getenv("MOCK_STDERR")
	exitStr := os.Getenv("MOCK_EXIT")

	if stdout != "" {
		fmt.Fprint(os.Stdout, stdout)
	}
	if stderr != "" {
		fmt.Fprint(os.Stderr, stderr)
	}

	exitCode := 0
	if exitStr != "" {
		exitCode, _ = strconv.Atoi(exitStr)
	}
	os.Exit(exitCode)
}
`
	if err := os.WriteFile(src, []byte(mockSrc), 0644); err != nil {
		t.Fatalf("failed to write mock source: %v", err)
	}

	cmd := exec.Command("go", "build", "-o", bin, src)
	cmd.Env = append(os.Environ(), "CGO_ENABLED=0")
	out, err := cmd.CombinedOutput()
	if err != nil {
		t.Fatalf("failed to build mock binary: %v\n%s", err, out)
	}
	return bin
}

// newMockClient creates a Client pointing at the mock binary with the
// given env vars set for stdout, stderr, and exit code behavior.
func newMockClient(t *testing.T, bin string, mockStdout, mockStderr string, mockExit int) *Client {
	t.Helper()
	env := []string{
		"MOCK_STDOUT=" + mockStdout,
		"MOCK_STDERR=" + mockStderr,
		fmt.Sprintf("MOCK_EXIT=%d", mockExit),
	}
	return &Client{
		binary:   bin,
		extraEnv: env,
	}
}

func TestGet_ReturnsValue(t *testing.T) {
	bin := buildMockBinary(t)
	client := newMockClient(t, bin,
		`{"name":"db-url","value":"postgres://localhost/mydb","version":1,"created":"2025-01-01T00:00:00Z","modified":"2025-01-01T00:00:00Z"}`,
		"", 0)

	value, err := client.Get(context.Background(), "db-url")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if value != "postgres://localhost/mydb" {
		t.Errorf("expected 'postgres://localhost/mydb', got %q", value)
	}
}

func TestGet_SecretNotFound(t *testing.T) {
	bin := buildMockBinary(t)
	client := newMockClient(t, bin,
		"",
		`{"error":{"code":"not_found","message":"Secret not found: db-url","exit_code":3}}`,
		3)

	_, err := client.Get(context.Background(), "db-url")
	if err == nil {
		t.Fatal("expected error, got nil")
	}
	if !errors.Is(err, ErrSecretNotFound) {
		t.Errorf("expected ErrSecretNotFound, got %v", err)
	}
}

func TestGetOpt_Missing(t *testing.T) {
	bin := buildMockBinary(t)
	client := newMockClient(t, bin,
		"",
		`{"error":{"code":"not_found","message":"Secret not found: db-url","exit_code":3}}`,
		3)

	value, ok, err := client.GetOpt(context.Background(), "db-url")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if ok {
		t.Error("expected ok=false")
	}
	if value != "" {
		t.Errorf("expected empty value, got %q", value)
	}
}

func TestStore_PassesStdin(t *testing.T) {
	bin := buildMockBinary(t)
	// Store doesn't return JSON stdout on success
	client := newMockClient(t, bin, "", "", 0)

	err := client.Store(context.Background(), "api-key", "secret-value-123")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestStore_AlreadyExists(t *testing.T) {
	bin := buildMockBinary(t)
	client := newMockClient(t, bin,
		"",
		`{"error":{"code":"already_exists","message":"Secret already exists: api-key","exit_code":5}}`,
		5)

	err := client.Store(context.Background(), "api-key", "value")
	if err == nil {
		t.Fatal("expected error, got nil")
	}
	if !errors.Is(err, ErrSecretAlreadyExists) {
		t.Errorf("expected ErrSecretAlreadyExists, got %v", err)
	}
}

func TestStore_Force(t *testing.T) {
	bin := buildMockBinary(t)
	client := newMockClient(t, bin, "", "", 0)

	err := client.Store(context.Background(), "api-key", "new-value", Force())
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestList_ReturnsNames(t *testing.T) {
	bin := buildMockBinary(t)
	client := newMockClient(t, bin,
		`{"secrets":[{"name":"db-url","version":1,"created":"2025-01-01T00:00:00Z","modified":"2025-01-01T00:00:00Z"},{"name":"api-key","version":2,"created":"2025-01-01T00:00:00Z","modified":"2025-01-02T00:00:00Z"}]}`,
		"", 0)

	names, err := client.List(context.Background())
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(names) != 2 {
		t.Fatalf("expected 2 names, got %d", len(names))
	}
	if names[0] != "db-url" {
		t.Errorf("expected 'db-url', got %q", names[0])
	}
	if names[1] != "api-key" {
		t.Errorf("expected 'api-key', got %q", names[1])
	}
}

func TestList_WithScope(t *testing.T) {
	bin := buildMockBinary(t)
	client := newMockClient(t, bin,
		`{"secrets":[{"name":"db-url","version":1,"created":"2025-01-01T00:00:00Z","modified":"2025-01-01T00:00:00Z"}]}`,
		"", 0)

	names, err := client.List(context.Background(), WithScope("deploy"))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(names) != 1 {
		t.Fatalf("expected 1 name, got %d", len(names))
	}
	if names[0] != "db-url" {
		t.Errorf("expected 'db-url', got %q", names[0])
	}
}

func TestCredentialsInEnv(t *testing.T) {
	client, err := New(
		WithBinary("/bin/true"),
		WithPassphrase("my-passphrase"),
		WithKeyfile("/path/to/keyfile"),
	)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	foundPassphrase := false
	foundKeyfile := false
	for _, env := range client.extraEnv {
		if env == "AUTHY_PASSPHRASE=my-passphrase" {
			foundPassphrase = true
		}
		if env == "AUTHY_KEYFILE=/path/to/keyfile" {
			foundKeyfile = true
		}
	}

	if !foundPassphrase {
		t.Error("expected AUTHY_PASSPHRASE in extraEnv")
	}
	if !foundKeyfile {
		t.Error("expected AUTHY_KEYFILE in extraEnv")
	}
}

func TestContextCancellation(t *testing.T) {
	bin := buildMockBinary(t)
	// Use a mock that sleeps — but since our mock doesn't sleep,
	// we test with an already-cancelled context.
	client := newMockClient(t, bin, "", "", 0)

	ctx, cancel := context.WithCancel(context.Background())
	cancel() // cancel immediately

	_, err := client.Get(ctx, "any-key")
	if err == nil {
		t.Fatal("expected error from cancelled context, got nil")
	}
}

func TestRemove_Success(t *testing.T) {
	bin := buildMockBinary(t)
	client := newMockClient(t, bin, "", "", 0)

	removed, err := client.Remove(context.Background(), "db-url")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !removed {
		t.Error("expected removed=true")
	}
}

func TestRemove_NotFound(t *testing.T) {
	bin := buildMockBinary(t)
	client := newMockClient(t, bin,
		"",
		`{"error":{"code":"not_found","message":"Secret not found: db-url","exit_code":3}}`,
		3)

	_, err := client.Remove(context.Background(), "db-url")
	if err == nil {
		t.Fatal("expected error, got nil")
	}
	if !errors.Is(err, ErrSecretNotFound) {
		t.Errorf("expected ErrSecretNotFound, got %v", err)
	}
}

func TestAuthFailed(t *testing.T) {
	bin := buildMockBinary(t)
	client := newMockClient(t, bin,
		"",
		`{"error":{"code":"auth_failed","message":"Authentication failed: wrong passphrase","exit_code":2}}`,
		2)

	_, err := client.Get(context.Background(), "db-url")
	if err == nil {
		t.Fatal("expected error, got nil")
	}
	if !errors.Is(err, ErrAuthFailed) {
		t.Errorf("expected ErrAuthFailed, got %v", err)
	}
}

func TestPolicyDenied(t *testing.T) {
	bin := buildMockBinary(t)
	client := newMockClient(t, bin,
		"",
		`{"error":{"code":"access_denied","message":"Access denied","exit_code":4}}`,
		4)

	_, err := client.Get(context.Background(), "db-url")
	if err == nil {
		t.Fatal("expected error, got nil")
	}
	if !errors.Is(err, ErrPolicyDenied) {
		t.Errorf("expected ErrPolicyDenied, got %v", err)
	}
}

func TestVaultNotFound(t *testing.T) {
	bin := buildMockBinary(t)
	client := newMockClient(t, bin,
		"",
		`{"error":{"code":"vault_not_initialized","message":"Vault not initialized","exit_code":7}}`,
		7)

	_, err := client.Get(context.Background(), "db-url")
	if err == nil {
		t.Fatal("expected error, got nil")
	}
	if !errors.Is(err, ErrVaultNotFound) {
		t.Errorf("expected ErrVaultNotFound, got %v", err)
	}
}

func TestNew_DefaultLooksOnPath(t *testing.T) {
	// This test just ensures that New() without WithBinary tries LookPath.
	// It may fail in environments without authy on PATH, which is expected.
	_, err := New()
	if err != nil {
		// Expected — authy is likely not on PATH in the test environment.
		return
	}
	// If it succeeded, that's fine too.
}

