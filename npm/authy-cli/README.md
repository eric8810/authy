# authy-cli

Encrypted secrets for AI agents. Single binary, no server, no accounts.

## Install

```bash
npm install -g authy-cli
authy --help
```

## 30-Second Start

```bash
authy init --generate-keyfile ~/.authy/keys/master.key
authy store api-key                          # type value, Ctrl+D
authy run --scope "*" -- ./my-script.sh      # script sees $API_KEY in its env
```

Secret is encrypted in the vault, injected into the subprocess, never in shell history or `.env` files.

## Config File Placeholders

```bash
authy resolve config.yaml.tpl --scope deploy --output config.yaml
```

Replace `<authy:key-name>` placeholders in config files with real values. Safe for run-only mode.

## Give Agents Scoped Access

```bash
# Create a policy + token — agent can inject secrets but never read values
authy policy create backend --allow "db-*" --run-only
authy session create --scope backend --ttl 1h --run-only

# Agent uses the token
export AUTHY_TOKEN="authy_v1...."
export AUTHY_KEYFILE=~/.authy/keys/master.key
authy run --scope backend --uppercase --replace-dash '_' -- node server.js
```

## MCP Server

Run as an MCP server for AI agent platforms (Claude Desktop, Cursor, Windsurf):

```bash
authy serve --mcp
```

Exposes 5 tools over stdio JSON-RPC 2.0: `get_secret`, `list_secrets`, `store_secret`, `remove_secret`, `test_policy`.

## Language SDKs

Thin CLI wrappers with zero native deps:

```bash
pip install authy-secrets          # Python
npm install authy-secrets          # TypeScript
go get github.com/eric8810/authy-go  # Go
```

## Migrate Your Secrets

```bash
authy import .env                                  # .env files
authy import --from 1password --vault Engineering  # 1Password
authy import --from pass                           # pass (password-store)
authy import --from sops secrets.enc.yaml          # Mozilla SOPS
authy import --from vault --path secret/myapp      # HashiCorp Vault
```

## Supported Platforms

| Platform | Architecture |
|----------|-------------|
| Linux | x64, arm64 |
| macOS | x64, arm64 |
| Windows | x64 |

## Documentation

Full docs at [github.com/eric8810/authy](https://github.com/eric8810/authy).

## License

MIT
