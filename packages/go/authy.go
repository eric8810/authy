// Package authy provides a Go client for the authy CLI secrets manager.
//
// It wraps the authy binary as a subprocess and communicates via --json mode,
// passing secret values through stdin (never as command-line arguments).
package authy

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"strings"
)

// Client is the main interface to the authy CLI.
type Client struct {
	binary   string
	extraEnv []string
}

type config struct {
	binary     string
	passphrase string
	keyfile    string
}

// Option configures a Client.
type Option func(*config)

// WithBinary sets the path to the authy binary.
func WithBinary(path string) Option {
	return func(c *config) {
		c.binary = path
	}
}

// WithPassphrase sets the vault passphrase via the AUTHY_PASSPHRASE env var.
func WithPassphrase(pass string) Option {
	return func(c *config) {
		c.passphrase = pass
	}
}

// WithKeyfile sets the path to the keyfile via the AUTHY_KEYFILE env var.
func WithKeyfile(path string) Option {
	return func(c *config) {
		c.keyfile = path
	}
}

// New creates a new authy Client. It verifies the binary exists on PATH
// (or at the specified path) and returns an error if not found.
func New(opts ...Option) (*Client, error) {
	cfg := &config{}
	for _, opt := range opts {
		opt(cfg)
	}

	binary := cfg.binary
	if binary == "" {
		found, err := exec.LookPath("authy")
		if err != nil {
			return nil, fmt.Errorf("authy: binary not found on PATH: %w", err)
		}
		binary = found
	}

	var extraEnv []string
	if cfg.passphrase != "" {
		extraEnv = append(extraEnv, "AUTHY_PASSPHRASE="+cfg.passphrase)
	}
	if cfg.keyfile != "" {
		extraEnv = append(extraEnv, "AUTHY_KEYFILE="+cfg.keyfile)
	}

	return &Client{
		binary:   binary,
		extraEnv: extraEnv,
	}, nil
}

// CallOption configures individual method calls.
type CallOption func(*callConfig)

type callConfig struct {
	force bool
	scope string
}

// Force enables the --force flag for operations like Store.
func Force() CallOption {
	return func(c *callConfig) {
		c.force = true
	}
}

// WithScope sets the --scope flag for operations like List and Run.
func WithScope(scope string) CallOption {
	return func(c *callConfig) {
		c.scope = scope
	}
}

// runCmd executes the authy CLI with the given arguments and optional stdin.
// It returns the parsed JSON output from stdout, or an error parsed from stderr.
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
		exitCode := -1
		if cmd.ProcessState != nil {
			exitCode = cmd.ProcessState.ExitCode()
		}
		return nil, parseError(stderr.Bytes(), exitCode)
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

// IsInitialized checks whether an authy vault exists at the default location.
// This is a package-level check that does not require authentication.
func IsInitialized() bool {
	home, err := os.UserHomeDir()
	if err != nil {
		return false
	}
	vaultPath := home + "/.authy/vault.age"
	_, err = os.Stat(vaultPath)
	return err == nil
}
