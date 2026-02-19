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

## Migrate from .env

```bash
authy import .env
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
