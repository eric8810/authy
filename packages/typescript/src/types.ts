/** Options for constructing an Authy client. */
export interface AuthyOptions {
  /** Path to the authy binary. Defaults to "authy". */
  binary?: string;
  /** Vault passphrase for authentication. */
  passphrase?: string;
  /** Path to a keyfile for authentication. */
  keyfile?: string;
}

/** Options for the store operation. */
export interface StoreOptions {
  /** Overwrite an existing secret. */
  force?: boolean;
}

/** Options for the list operation. */
export interface ListOptions {
  /** Filter secrets by policy scope. */
  scope?: string;
}

/** Options for the run operation. */
export interface RunOptions {
  /** Policy scope for secret injection. */
  scope?: string;
}

/** Result from running a subprocess via authy run. */
export interface RunResult {
  exitCode: number;
  stdout: string;
  stderr: string;
}

/** JSON response from `authy get --json`. */
export interface GetResponse {
  name: string;
  value: string;
  version: number;
  created: string;
  modified: string;
}

/** JSON response from `authy list --json`. */
export interface ListResponse {
  secrets: SecretListItem[];
}

/** A single item in the list response. */
export interface SecretListItem {
  name: string;
  version: number;
  created: string;
  modified: string;
}

/** JSON error response from authy --json on stderr. */
export interface JsonErrorResponse {
  error: {
    code: string;
    message: string;
    exit_code: number;
  };
}
