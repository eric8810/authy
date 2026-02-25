package authy

import (
	"context"
	"fmt"
)

// Get retrieves the value of a secret by name.
// Returns ErrSecretNotFound if the secret does not exist.
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

// GetOpt retrieves a secret, returning (value, true, nil) if found, or
// ("", false, nil) if the secret does not exist. Other errors are returned
// as the third value.
func (c *Client) GetOpt(ctx context.Context, name string) (string, bool, error) {
	result, err := c.runCmd(ctx, []string{"get", name}, "")
	if err != nil {
		if isNotFound(err) {
			return "", false, nil
		}
		return "", false, err
	}
	value, ok := result["value"].(string)
	if !ok {
		return "", false, fmt.Errorf("authy: unexpected response format")
	}
	return value, true, nil
}

// Store creates a new secret. Returns ErrSecretAlreadyExists if the secret
// already exists (unless Force() is passed).
// The secret value is passed via stdin, never as a command-line argument.
func (c *Client) Store(ctx context.Context, name, value string, opts ...CallOption) error {
	cfg := &callConfig{}
	for _, opt := range opts {
		opt(cfg)
	}
	args := []string{"store", name}
	if cfg.force {
		args = append(args, "--force")
	}
	_, err := c.runCmd(ctx, args, value)
	return err
}

// Remove deletes a secret by name. Returns true if the secret was removed,
// or an error (including ErrSecretNotFound) if it did not exist.
func (c *Client) Remove(ctx context.Context, name string) (bool, error) {
	_, err := c.runCmd(ctx, []string{"remove", name}, "")
	if err != nil {
		return false, err
	}
	return true, nil
}

// Rotate updates the value of an existing secret and increments its version.
// Returns the new version number. The new value is passed via stdin.
func (c *Client) Rotate(ctx context.Context, name, newValue string) (int, error) {
	_, err := c.runCmd(ctx, []string{"rotate", name}, newValue)
	if err != nil {
		return 0, err
	}
	// The rotate command does not return JSON output with the version,
	// so we fetch the secret to get the current version.
	result, err := c.runCmd(ctx, []string{"get", name}, "")
	if err != nil {
		return 0, err
	}
	version, ok := result["version"].(float64)
	if !ok {
		return 0, fmt.Errorf("authy: unexpected response format for version")
	}
	return int(version), nil
}

// ListResult holds a single entry from the list output.
type ListResult struct {
	Name     string
	Version  int
	Created  string
	Modified string
}

// List returns the names of all secrets, optionally filtered by scope.
func (c *Client) List(ctx context.Context, opts ...CallOption) ([]string, error) {
	cfg := &callConfig{}
	for _, opt := range opts {
		opt(cfg)
	}
	args := []string{"list"}
	if cfg.scope != "" {
		args = append(args, "--scope", cfg.scope)
	}
	result, err := c.runCmd(ctx, args, "")
	if err != nil {
		return nil, err
	}
	if result == nil {
		return []string{}, nil
	}

	secretsRaw, ok := result["secrets"].([]any)
	if !ok {
		return []string{}, nil
	}

	names := make([]string, 0, len(secretsRaw))
	for _, item := range secretsRaw {
		m, ok := item.(map[string]any)
		if !ok {
			continue
		}
		name, ok := m["name"].(string)
		if !ok {
			continue
		}
		names = append(names, name)
	}
	return names, nil
}

// RunResult holds the exit code from a subprocess run.
type RunResult struct {
	ExitCode int
}

// Run executes a command with secrets injected as environment variables.
func (c *Client) Run(ctx context.Context, command []string, opts ...CallOption) (*RunResult, error) {
	cfg := &callConfig{}
	for _, opt := range opts {
		opt(cfg)
	}
	args := []string{"run"}
	if cfg.scope != "" {
		args = append(args, "--scope", cfg.scope)
	}
	args = append(args, "--")
	args = append(args, command...)
	_, err := c.runCmd(ctx, args, "")
	if err != nil {
		// For run, a non-zero exit from the child process is also an error.
		if ae, ok := err.(*AuthyError); ok {
			return &RunResult{ExitCode: ae.ExitCode}, nil
		}
		return nil, err
	}
	return &RunResult{ExitCode: 0}, nil
}

// ImportDotenv imports secrets from a .env file.
func (c *Client) ImportDotenv(ctx context.Context, path string) error {
	_, err := c.runCmd(ctx, []string{"import", path}, "")
	return err
}

// ImportOption configures import operations.
type ImportOption func(*importConfig)

type importConfig struct {
	vault string
}

// ImportVault sets the vault name for external import sources.
func ImportVault(name string) ImportOption {
	return func(c *importConfig) {
		c.vault = name
	}
}

// ImportFrom imports secrets from an external source (e.g., "1password").
func (c *Client) ImportFrom(ctx context.Context, source string, opts ...ImportOption) error {
	cfg := &importConfig{}
	for _, opt := range opts {
		opt(cfg)
	}
	args := []string{"import", "--from", source}
	if cfg.vault != "" {
		args = append(args, "--vault", cfg.vault)
	}
	_, err := c.runCmd(ctx, args, "")
	return err
}

// Init initializes a new authy vault.
func (c *Client) Init(ctx context.Context) error {
	_, err := c.runCmd(ctx, []string{"init"}, "")
	return err
}

// isNotFound checks whether an error represents a secret-not-found condition.
func isNotFound(err error) bool {
	ae, ok := err.(*AuthyError)
	if !ok {
		return false
	}
	return ae.Code == "not_found"
}
