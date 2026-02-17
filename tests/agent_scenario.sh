#!/usr/bin/env bash
# ───────────────────────────────────────────────────────────────────────
# Agent Usage Scenario — end-to-end test
#
# Demonstrates a full operator → agent lifecycle:
#   Operator sets up vault, secrets, and policies
#   Creates a scoped session token for an AI agent
#   Agent reads secrets, runs subprocesses
#   Operator reviews audit log and revokes the session
# ───────────────────────────────────────────────────────────────────────
set -uo pipefail

GREEN='\033[0;32m'
RED='\033[0;31m'
BOLD='\033[1m'
RESET='\033[0m'

PASS_COUNT=0
FAIL_COUNT=0

step() { printf "\n${BOLD}── Step %s: %s${RESET}\n" "$1" "$2"; }
pass() { printf "  ${GREEN}PASS${RESET}: %s\n" "$1"; PASS_COUNT=$((PASS_COUNT + 1)); }
fail() { printf "  ${RED}FAIL${RESET}: %s\n" "$1"; FAIL_COUNT=$((FAIL_COUNT + 1)); exit 1; }

# ── Build ────────────────────────────────────────────────────────────
step 0 "Build authy"
SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cargo build --quiet --manifest-path "$SCRIPT_DIR/Cargo.toml" 2>&1
AUTHY="$SCRIPT_DIR/target/debug/authy"
[ -x "$AUTHY" ] && pass "Binary built at $AUTHY" || fail "cargo build failed"

# ── Isolated temp environment ────────────────────────────────────────
TMPHOME=$(mktemp -d)
export HOME="$TMPHOME"
KEYFILE="$TMPHOME/master.key"

cleanup() { rm -rf "$TMPHOME"; }
trap cleanup EXIT

# Helper: run authy as the operator (master keyfile auth)
master() { AUTHY_KEYFILE="$KEYFILE" "$AUTHY" "$@"; }

# Helper: run authy as the agent (token + keyfile for vault decryption)
agent() { AUTHY_KEYFILE="$KEYFILE" AUTHY_TOKEN="$TOKEN" "$AUTHY" "$@"; }

# ═══════════════════════════════════════════════════════════════════════
# OPERATOR SETUP
# ═══════════════════════════════════════════════════════════════════════

# ── 1. Init vault ────────────────────────────────────────────────────
step 1 "Init vault with generated keyfile"
"$AUTHY" init --generate-keyfile "$KEYFILE" 2>/dev/null
[ -f "$TMPHOME/.authy/vault.age" ] && pass "Vault created" || fail "vault.age missing"
[ -f "$KEYFILE" ]                   && pass "Keyfile generated" || fail "Keyfile missing"

# ── 2. Store secrets ─────────────────────────────────────────────────
step 2 "Store secrets (db-host, db-port, db-password, api-key, ssh-key)"
echo -n "localhost"    | master store db-host     2>/dev/null
echo -n "5432"         | master store db-port     2>/dev/null
echo -n "s3cret!"      | master store db-password 2>/dev/null
echo -n "sk_live_xyz"  | master store api-key     2>/dev/null
echo -n "ssh-rsa AAAA" | master store ssh-key     2>/dev/null
COUNT=$(master list 2>/dev/null | wc -l)
[ "$COUNT" -eq 5 ] && pass "5 secrets stored" || fail "Expected 5, got $COUNT"

# ── 3. Create policy ─────────────────────────────────────────────────
step 3 "Create policy: allow db-* api-*, deny db-password"
master policy create deploy \
    --allow "db-*" "api-*" \
    --deny "db-password" \
    --description "Deploy scope — no raw DB password" 2>/dev/null
pass "Policy 'deploy' created"

# ── 4. Test policy rules ─────────────────────────────────────────────
step 4 "Verify policy rules with 'policy test'"
OUT=$(master policy test --scope deploy db-host 2>&1)
echo "$OUT" | grep -q "ALLOWED" && pass "db-host → ALLOWED"    || fail "db-host should be ALLOWED"
OUT=$(master policy test --scope deploy api-key 2>&1)
echo "$OUT" | grep -q "ALLOWED" && pass "api-key → ALLOWED"    || fail "api-key should be ALLOWED"
OUT=$(master policy test --scope deploy db-password 2>&1)
echo "$OUT" | grep -q "DENIED"  && pass "db-password → DENIED" || fail "db-password should be DENIED"
OUT=$(master policy test --scope deploy ssh-key 2>&1)
echo "$OUT" | grep -q "DENIED"  && pass "ssh-key → DENIED"     || fail "ssh-key should be DENIED"

# ── 5. Create session token ──────────────────────────────────────────
step 5 "Create scoped session token (scope=deploy, ttl=1h)"
TOKEN=$(master session create --scope deploy --ttl 1h --label agent-test 2>/dev/null)
echo "$TOKEN" | grep -q "^authy_v1\." \
    && pass "Token issued: ${TOKEN:0:24}…" \
    || fail "Invalid token format"

# ═══════════════════════════════════════════════════════════════════════
# AGENT OPERATIONS (using scoped token)
# ═══════════════════════════════════════════════════════════════════════

# ── 6. Read allowed secret ────────────────────────────────────────────
step 6 "Agent reads allowed secret (db-host)"
VALUE=$(agent get db-host 2>/dev/null)
[ "$VALUE" = "localhost" ] && pass "db-host = localhost" || fail "Expected 'localhost', got '$VALUE'"

# ── 7. Read denied secret ────────────────────────────────────────────
step 7 "Agent reads denied secret (db-password)"
if agent get db-password 2>/dev/null; then
    fail "db-password should be denied by policy"
else
    pass "db-password correctly denied"
fi

# ── 8. Read out-of-scope secret ──────────────────────────────────────
step 8 "Agent reads out-of-scope secret (ssh-key)"
if agent get ssh-key 2>/dev/null; then
    fail "ssh-key should be denied (not in policy)"
else
    pass "ssh-key correctly denied"
fi

# ── 9. Agent tries to write (read-only enforcement) ──────────────────
step 9 "Agent attempts write operation"
WRITE_ERR=$(echo -n "hacked" | agent store evil-secret 2>&1) || true
echo "$WRITE_ERR" | grep -q "read-only" \
    && pass "Write rejected: tokens are read-only" \
    || fail "Expected read-only error, got: $WRITE_ERR"

# ── 10. Run subprocess with env injection ─────────────────────────────
step 10 "Run subprocess with scoped env vars (uppercase, replace-dash)"
ENV_OUT=$(master run --scope deploy --uppercase --replace-dash _ -- env 2>/dev/null)
echo "$ENV_OUT" | grep -q "DB_HOST=localhost"    && pass "DB_HOST injected"      || fail "DB_HOST missing"
echo "$ENV_OUT" | grep -q "DB_PORT=5432"         && pass "DB_PORT injected"      || fail "DB_PORT missing"
echo "$ENV_OUT" | grep -q "API_KEY=sk_live_xyz"  && pass "API_KEY injected"      || fail "API_KEY missing"
echo "$ENV_OUT" | grep -q "DB_PASSWORD"          && fail "DB_PASSWORD leaked"    || pass "DB_PASSWORD excluded"
echo "$ENV_OUT" | grep -q "SSH_KEY"              && fail "SSH_KEY leaked"        || pass "SSH_KEY excluded"

# ═══════════════════════════════════════════════════════════════════════
# OPERATOR AUDIT & REVOCATION
# ═══════════════════════════════════════════════════════════════════════

# ── 11. Audit show ────────────────────────────────────────────────────
step 11 "Review audit log"
AUDIT=$(master audit show 2>/dev/null)
echo "$AUDIT" | grep -q "store"          && pass "Audit records store events"   || fail "Missing store events"
echo "$AUDIT" | grep -q "get"            && pass "Audit records get events"     || fail "Missing get events"
echo "$AUDIT" | grep -q "session.create" && pass "Audit records session create" || fail "Missing session.create"

# ── 12. Audit verify ─────────────────────────────────────────────────
step 12 "Verify audit log integrity"
VERIFY_OUT=$(master audit verify 2>&1)
echo "$VERIFY_OUT" | grep -q "verified" \
    && pass "Audit chain intact" \
    || fail "Chain verification failed: $VERIFY_OUT"

# ── 13. Revoke session ───────────────────────────────────────────────
step 13 "Revoke agent session"
SESSION_ID=$(master session list 2>/dev/null | head -1 | awk '{print $1}')
[ -n "$SESSION_ID" ] || fail "Could not parse session ID"
master session revoke "$SESSION_ID" 2>/dev/null
pass "Session $SESSION_ID revoked"

# ── 14. Verify revoked token is rejected ──────────────────────────────
step 14 "Agent uses revoked token"
if agent get db-host 2>/dev/null; then
    fail "Revoked token should be rejected"
else
    pass "Revoked token correctly rejected"
fi

# ═══════════════════════════════════════════════════════════════════════
# SUMMARY
# ═══════════════════════════════════════════════════════════════════════
printf "\n${BOLD}════════════════════════════════════════════${RESET}\n"
printf "  ${GREEN}%d checks passed${RESET}" "$PASS_COUNT"
if [ "$FAIL_COUNT" -gt 0 ]; then
    printf ", ${RED}%d failed${RESET}" "$FAIL_COUNT"
fi
printf "\n${BOLD}════════════════════════════════════════════${RESET}\n"

exit "$FAIL_COUNT"
