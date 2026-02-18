# Authy Command Reference

## Agent Commands

| Command | Purpose |
|---------|---------|
| `authy run --scope <s> -- <cmd>` | Inject secrets into a subprocess |
| `authy list --json` | List all secret names (no values) |
| `authy list --scope <s> --json` | List secrets filtered by policy |
| `authy policy test --scope <s> <name> --json` | Check if a secret is accessible |

## Naming Transforms

Given a secret named `db-host`:

| Flags | Env Var |
|-------|---------|
| `--uppercase --replace-dash '_'` | `DB_HOST` |
| `--prefix 'APP_' --uppercase --replace-dash '_'` | `APP_DB_HOST` |

## Common Patterns

```bash
# Launch a service
authy run --scope backend --uppercase --replace-dash '_' -- node server.js

# Run tests with credentials
authy run --scope testing --uppercase --replace-dash '_' -- pytest tests/

# Check what secrets exist
authy list --scope deploy --json | jq '.secrets[].name'

# Write a script, then run it with secrets
cat > task.sh << 'SCRIPT'
#!/bin/bash
psql "$DATABASE_URL" -c "SELECT 1"
SCRIPT
chmod +x task.sh
authy run --scope db --uppercase --replace-dash '_' -- ./task.sh
rm task.sh
```

## Notes

- Your token is run-only — only the commands listed above are available
- Secrets are injected into subprocesses as env vars and never written to disk
- Do not echo, print, or redirect environment variables containing secrets
