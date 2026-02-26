/** Options for creating an Authy client. */
export interface AuthyOptions {
  /** Vault passphrase for authentication. */
  passphrase?: string;
  /** Path to an age keyfile for authentication. */
  keyfile?: string;
}

/** Options for storing a secret. */
export interface StoreOptions {
  /** Overwrite existing secret if true. */
  force?: boolean;
}

/** Options for listing secrets. */
export interface ListOptions {
  /** Policy scope to filter secrets by. */
  scope?: string;
}

/**
 * Native Authy client backed by the Rust vault engine.
 *
 * No authy binary on PATH needed — the vault engine is compiled
 * directly into this Node.js module.
 */
export class Authy {
  constructor(opts: AuthyOptions);

  /** Retrieve a secret value. Throws if not found. */
  get(name: string): string;

  /** Retrieve a secret value, returning null if not found. */
  getOrNull(name: string): string | null;

  /** Store a secret. Throws if it already exists unless force is set. */
  store(name: string, value: string, opts?: StoreOptions): void;

  /** Remove a secret. Returns true if it existed. */
  remove(name: string): boolean;

  /** Rotate a secret to a new value. Returns the new version number. */
  rotate(name: string, newValue: string): number;

  /** List secret names, optionally filtered by a policy scope. */
  list(opts?: ListOptions): string[];

  /** Build an environment variable map from secrets matching a policy scope. */
  buildEnvMap(scope: string, uppercase?: boolean, replaceDash?: string): Record<string, string>;

  /** Test whether a policy allows access to a secret. */
  testPolicy(scope: string, secretName: string): boolean;

  /** Initialize a new vault. */
  initVault(): void;

  /** Check whether a vault is initialized (static, no auth needed). */
  static isInitialized(): boolean;
}
