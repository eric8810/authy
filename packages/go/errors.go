package authy

import (
	"encoding/json"
	"errors"
	"fmt"
)

// AuthyError represents an error returned by the authy CLI.
type AuthyError struct {
	ExitCode int
	Code     string
	Message  string
}

func (e *AuthyError) Error() string {
	if e.Message != "" {
		return e.Message
	}
	return fmt.Sprintf("authy: %s (exit code %d)", e.Code, e.ExitCode)
}

// Is supports errors.Is matching by comparing the Code field.
func (e *AuthyError) Is(target error) bool {
	var t *AuthyError
	if errors.As(target, &t) {
		return e.Code == t.Code
	}
	return false
}

// Sentinel errors matching authy CLI exit codes and error codes.
var (
	ErrSecretNotFound      = &AuthyError{ExitCode: 3, Code: "not_found"}
	ErrSecretAlreadyExists = &AuthyError{ExitCode: 5, Code: "already_exists"}
	ErrAuthFailed          = &AuthyError{ExitCode: 2, Code: "auth_failed"}
	ErrPolicyDenied        = &AuthyError{ExitCode: 4, Code: "access_denied"}
	ErrVaultNotFound       = &AuthyError{ExitCode: 7, Code: "vault_not_initialized"}
)

// jsonErrorResponse represents the JSON error format from authy --json stderr.
type jsonErrorResponse struct {
	Error jsonErrorDetail `json:"error"`
}

type jsonErrorDetail struct {
	Code     string `json:"code"`
	Message  string `json:"message"`
	ExitCode int    `json:"exit_code"`
}

// parseError parses a JSON error from stderr, falling back to a generic error.
func parseError(stderr []byte, exitCode int) error {
	var resp jsonErrorResponse
	if err := json.Unmarshal(stderr, &resp); err == nil && resp.Error.Code != "" {
		return &AuthyError{
			ExitCode: resp.Error.ExitCode,
			Code:     resp.Error.Code,
			Message:  resp.Error.Message,
		}
	}

	// Fallback: could not parse JSON, create generic error from exit code
	msg := string(stderr)
	if msg == "" {
		msg = fmt.Sprintf("authy exited with code %d", exitCode)
	}
	return &AuthyError{
		ExitCode: exitCode,
		Code:     exitCodeToCode(exitCode),
		Message:  msg,
	}
}

func exitCodeToCode(exitCode int) string {
	switch exitCode {
	case 1:
		return "internal_error"
	case 2:
		return "auth_failed"
	case 3:
		return "not_found"
	case 4:
		return "access_denied"
	case 5:
		return "already_exists"
	case 6:
		return "invalid_token"
	case 7:
		return "vault_not_initialized"
	default:
		return "unknown_error"
	}
}
