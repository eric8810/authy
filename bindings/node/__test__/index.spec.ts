import { describe, it, expect, beforeEach } from "vitest";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

// Note: in real usage this would be:
// import { Authy } from "authy-cli";
// For testing the built native module:
const { Authy } = require("../index.js");

function isolatedHome() {
  const tmp = mkdtempSync(join(tmpdir(), "authy-test-"));
  process.env.HOME = tmp;
  return tmp;
}

describe("Authy native binding", () => {
  beforeEach(() => {
    isolatedHome();
  });

  it("should init vault and store/get a secret", () => {
    const client = new Authy({ passphrase: "test-pass" });
    client.initVault();
    client.store("api-key", "sk-secret-123");
    expect(client.get("api-key")).toBe("sk-secret-123");
  });

  it("should throw on get not found", () => {
    const client = new Authy({ passphrase: "test-pass" });
    client.initVault();
    expect(() => client.get("nonexistent")).toThrow(/not found/i);
  });

  it("should return null for getOrNull on missing secret", () => {
    const client = new Authy({ passphrase: "test-pass" });
    client.initVault();
    expect(client.getOrNull("nonexistent")).toBeNull();
  });

  it("should throw on store duplicate", () => {
    const client = new Authy({ passphrase: "test-pass" });
    client.initVault();
    client.store("dup", "v1");
    expect(() => client.store("dup", "v2")).toThrow(/already exists/i);
  });

  it("should store with force overwrite", () => {
    const client = new Authy({ passphrase: "test-pass" });
    client.initVault();
    client.store("key", "v1");
    client.store("key", "v2", { force: true });
    expect(client.get("key")).toBe("v2");
  });

  it("should remove a secret", () => {
    const client = new Authy({ passphrase: "test-pass" });
    client.initVault();
    client.store("to-remove", "val");
    expect(client.remove("to-remove")).toBe(true);
    expect(client.getOrNull("to-remove")).toBeNull();
  });

  it("should rotate a secret", () => {
    const client = new Authy({ passphrase: "test-pass" });
    client.initVault();
    client.store("rotating", "v1");
    const version = client.rotate("rotating", "v2");
    expect(version).toBe(2);
    expect(client.get("rotating")).toBe("v2");
  });

  it("should list secrets", () => {
    const client = new Authy({ passphrase: "test-pass" });
    client.initVault();
    client.store("alpha", "a");
    client.store("beta", "b");
    const names = client.list();
    expect(names).toContain("alpha");
    expect(names).toContain("beta");
  });

  it("should check isInitialized", () => {
    expect(Authy.isInitialized()).toBe(false);
    const client = new Authy({ passphrase: "test-pass" });
    client.initVault();
    expect(Authy.isInitialized()).toBe(true);
  });

  it("should require passphrase or keyfile", () => {
    expect(() => new Authy({})).toThrow();
  });
});
