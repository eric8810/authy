/** Base error for all authy SDK errors. */
export class AuthyError extends Error {
  /** CLI exit code that produced this error. */
  readonly exitCode: number;
  /** Machine-readable error code from the CLI JSON output. */
  readonly errorCode: string;

  constructor(message: string, exitCode: number, errorCode: string) {
    super(message);
    this.name = "AuthyError";
    this.exitCode = exitCode;
    this.errorCode = errorCode;
  }
}

/** The requested secret was not found in the vault. */
export class SecretNotFound extends AuthyError {
  constructor(message: string, exitCode: number, errorCode: string) {
    super(message, exitCode, errorCode);
    this.name = "SecretNotFound";
  }
}

/** A secret with that name already exists. */
export class SecretAlreadyExists extends AuthyError {
  constructor(message: string, exitCode: number, errorCode: string) {
    super(message, exitCode, errorCode);
    this.name = "SecretAlreadyExists";
  }
}

/** Authentication failed (wrong passphrase, bad keyfile, etc.). */
export class AuthFailed extends AuthyError {
  constructor(message: string, exitCode: number, errorCode: string) {
    super(message, exitCode, errorCode);
    this.name = "AuthFailed";
  }
}

/** Access denied by policy. */
export class PolicyDenied extends AuthyError {
  constructor(message: string, exitCode: number, errorCode: string) {
    super(message, exitCode, errorCode);
    this.name = "PolicyDenied";
  }
}

/** Vault not initialized. Run `authy init` first. */
export class VaultNotFound extends AuthyError {
  constructor(message: string, exitCode: number, errorCode: string) {
    super(message, exitCode, errorCode);
    this.name = "VaultNotFound";
  }
}

/**
 * Map a CLI exit code and JSON error payload to a typed exception.
 */
export function mapError(
  error: { code: string; message: string },
  exitCode: number,
): AuthyError {
  switch (exitCode) {
    case 2:
      return new AuthFailed(error.message, exitCode, error.code);
    case 3:
      return new SecretNotFound(error.message, exitCode, error.code);
    case 4:
      return new PolicyDenied(error.message, exitCode, error.code);
    case 5:
      return new SecretAlreadyExists(error.message, exitCode, error.code);
    case 7:
      return new VaultNotFound(error.message, exitCode, error.code);
    default:
      return new AuthyError(error.message, exitCode, error.code);
  }
}
