//go:build integration

package authy

import (
	"context"
	"errors"
	"os"
	"os/exec"
	"testing"
)

func skipIfNoAuthy(t *testing.T) {
	t.Helper()
	if _, err := exec.LookPath("authy"); err != nil {
		t.Skip("authy binary not found on PATH, skipping integration test")
	}
}

func setupIntegrationClient(t *testing.T) *Client {
	t.Helper()
	skipIfNoAuthy(t)

	// Use a temp HOME to isolate the vault
	tmpHome := t.TempDir()
	t.Setenv("HOME", tmpHome)
	t.Setenv("AUTHY_PASSPHRASE", "test-passphrase-123")

	// Clear any existing keyfile/token env
	os.Unsetenv("AUTHY_KEYFILE")
	os.Unsetenv("AUTHY_TOKEN")

	client, err := New(WithPassphrase("test-passphrase-123"))
	if err != nil {
		t.Fatalf("failed to create client: %v", err)
	}
	return client
}

func TestInitStoreGetRoundtrip(t *testing.T) {
	client := setupIntegrationClient(t)
	ctx := context.Background()

	// Init vault
	err := client.Init(ctx)
	if err != nil {
		t.Fatalf("Init failed: %v", err)
	}

	// Store a secret
	err = client.Store(ctx, "db-url", "postgres://localhost/testdb")
	if err != nil {
		t.Fatalf("Store failed: %v", err)
	}

	// Get it back
	value, err := client.Get(ctx, "db-url")
	if err != nil {
		t.Fatalf("Get failed: %v", err)
	}
	if value != "postgres://localhost/testdb" {
		t.Errorf("expected 'postgres://localhost/testdb', got %q", value)
	}
}

func TestListWithScope(t *testing.T) {
	client := setupIntegrationClient(t)
	ctx := context.Background()

	err := client.Init(ctx)
	if err != nil {
		t.Fatalf("Init failed: %v", err)
	}

	// Store a few secrets
	for _, name := range []string{"api-key", "db-url", "redis-url"} {
		err = client.Store(ctx, name, "value-for-"+name)
		if err != nil {
			t.Fatalf("Store(%s) failed: %v", name, err)
		}
	}

	// List all
	names, err := client.List(ctx)
	if err != nil {
		t.Fatalf("List failed: %v", err)
	}
	if len(names) != 3 {
		t.Errorf("expected 3 secrets, got %d", len(names))
	}
}

func TestStoreDuplicateReturnsError(t *testing.T) {
	client := setupIntegrationClient(t)
	ctx := context.Background()

	err := client.Init(ctx)
	if err != nil {
		t.Fatalf("Init failed: %v", err)
	}

	err = client.Store(ctx, "dup-secret", "first-value")
	if err != nil {
		t.Fatalf("first Store failed: %v", err)
	}

	// Second store without Force should fail
	err = client.Store(ctx, "dup-secret", "second-value")
	if err == nil {
		t.Fatal("expected error for duplicate store, got nil")
	}
	if !errors.Is(err, ErrSecretAlreadyExists) {
		t.Errorf("expected ErrSecretAlreadyExists, got %v", err)
	}

	// With Force should succeed
	err = client.Store(ctx, "dup-secret", "second-value", Force())
	if err != nil {
		t.Fatalf("Store with Force failed: %v", err)
	}
}

func TestRotateIncrementsVersion(t *testing.T) {
	client := setupIntegrationClient(t)
	ctx := context.Background()

	err := client.Init(ctx)
	if err != nil {
		t.Fatalf("Init failed: %v", err)
	}

	err = client.Store(ctx, "rotate-me", "initial-value")
	if err != nil {
		t.Fatalf("Store failed: %v", err)
	}

	version, err := client.Rotate(ctx, "rotate-me", "rotated-value")
	if err != nil {
		t.Fatalf("Rotate failed: %v", err)
	}
	if version < 2 {
		t.Errorf("expected version >= 2 after rotation, got %d", version)
	}

	// Verify the new value
	value, err := client.Get(ctx, "rotate-me")
	if err != nil {
		t.Fatalf("Get after Rotate failed: %v", err)
	}
	if value != "rotated-value" {
		t.Errorf("expected 'rotated-value', got %q", value)
	}
}
