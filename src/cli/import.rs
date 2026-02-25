use std::io::{self, BufRead};

use authy::audit;
use authy::auth;
use authy::error::{AuthyError, Result};
use authy::vault;
use authy::vault::secret::SecretEntry;

use super::import_sources::hcvault::HcVaultAdapter;
use super::import_sources::onepassword::OnePasswordAdapter;
use super::import_sources::pass::PassAdapter;
use super::import_sources::sops::SopsAdapter;
use super::import_sources::ImportAdapter;
use super::ImportSource;

#[allow(clippy::too_many_arguments)]
pub fn run(
    file: Option<&str>,
    from: Option<&ImportSource>,
    op_vault: Option<&str>,
    tag: Option<&str>,
    path: Option<&str>,
    mount: &str,
    keep_names: bool,
    prefix: Option<&str>,
    force: bool,
    dry_run: bool,
) -> Result<()> {
    let parsed = fetch_secrets(file, from, op_vault, tag, path, mount)?;

    if parsed.is_empty() {
        eprintln!("No secrets found in input.");
        return Ok(());
    }

    let (key, auth_ctx) = auth::resolve_auth(!dry_run)?;
    let mut vault_data = vault::load_vault(&key)?;

    let mut imported = 0usize;
    let mut skipped = 0usize;

    let material = audit::key_material(&key);
    let audit_key = audit::derive_audit_key(&material);

    for (raw_name, value) in &parsed {
        let name = transform_name(raw_name, keep_names, prefix);

        let exists = vault_data.secrets.contains_key(&name);

        if exists && !force {
            eprintln!("Skipping '{}' (already exists, use --force to overwrite)", name);
            skipped += 1;
            continue;
        }

        if dry_run {
            let action = if exists { "overwrite" } else { "create" };
            println!("[dry-run] {} {} = {}",
                action,
                name,
                if value.len() > 20 { format!("{}...", &value[..20]) } else { value.clone() }
            );
            imported += 1;
            continue;
        }

        if exists {
            // Force overwrite: bump version
            if let Some(entry) = vault_data.secrets.get_mut(&name) {
                entry.value = value.clone();
                entry.metadata.bump_version();
            }
        } else {
            vault_data
                .secrets
                .insert(name.clone(), SecretEntry::new(value.clone()));
        }

        // Audit each imported secret
        audit::log_event(
            &vault::audit_path(),
            "import",
            Some(&name),
            &auth_ctx.actor_name(),
            "success",
            Some(if exists { "overwrite" } else { "created" }),
            &audit_key,
        )?;

        imported += 1;
    }

    if !dry_run && imported > 0 {
        vault_data.touch();
        vault::save_vault(&vault_data, &key)?;
    }

    eprintln!(
        "{} secret(s) imported, {} skipped.{}",
        imported,
        skipped,
        if dry_run { " (dry run)" } else { "" }
    );

    Ok(())
}

/// Fetch secrets from the appropriate source.
fn fetch_secrets(
    file: Option<&str>,
    from: Option<&ImportSource>,
    op_vault: Option<&str>,
    tag: Option<&str>,
    path: Option<&str>,
    mount: &str,
) -> Result<Vec<(String, String)>> {
    match from {
        Some(ImportSource::OnePassword) => {
            let adapter = OnePasswordAdapter {
                vault: op_vault.map(String::from),
                tag: tag.map(String::from),
            };
            adapter.fetch()
        }
        Some(ImportSource::Pass) => {
            let adapter = PassAdapter {
                store_path: path.map(String::from),
            };
            adapter.fetch()
        }
        Some(ImportSource::Sops) => {
            let f = file.ok_or_else(|| {
                AuthyError::Other(
                    "SOPS import requires a file argument (e.g., authy import --from sops secrets.enc.yaml)"
                        .into(),
                )
            })?;
            let adapter = SopsAdapter {
                file: f.to_string(),
            };
            adapter.fetch()
        }
        Some(ImportSource::Vault) => {
            let p = path.ok_or_else(|| {
                AuthyError::Other(
                    "HashiCorp Vault import requires --path (e.g., authy import --from vault --path secret/myapp)"
                        .into(),
                )
            })?;
            let adapter = HcVaultAdapter {
                path: p.to_string(),
                mount: mount.to_string(),
            };
            adapter.fetch()
        }
        Some(ImportSource::Dotenv) | None => {
            // Existing .env import behavior
            let f = file.ok_or_else(|| {
                AuthyError::Other(
                    "Import requires a file argument (e.g., authy import .env)".into(),
                )
            })?;
            read_dotenv(f)
        }
    }
}

/// Read and parse a dotenv file (or stdin with "-").
fn read_dotenv(file: &str) -> Result<Vec<(String, String)>> {
    let content = if file == "-" {
        let mut buf = String::new();
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line?;
            buf.push_str(&line);
            buf.push('\n');
        }
        buf
    } else {
        std::fs::read_to_string(file)?
    };

    parse_dotenv(&content)
}

/// Transform a raw secret name using the shared pipeline.
fn transform_name(raw_name: &str, keep_names: bool, prefix: Option<&str>) -> String {
    if keep_names {
        let mut n = raw_name.to_string();
        if let Some(p) = prefix {
            n = format!("{}{}", p, n);
        }
        n
    } else {
        let transformed = to_lower_kebab(raw_name);
        if let Some(p) = prefix {
            format!("{}{}", p, transformed)
        } else {
            transformed
        }
    }
}

/// Transform names to lower-kebab-case.
/// Replaces `_`, `/`, spaces, and `.` with `-`, then lowercases.
fn to_lower_kebab(name: &str) -> String {
    name.to_lowercase()
        .replace(['_', '/', ' ', '.'], "-")
}

/// Parse a dotenv-format string into (key, value) pairs.
fn parse_dotenv(content: &str) -> Result<Vec<(String, String)>> {
    let mut result = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Strip optional `export ` prefix
        let line = trimmed
            .strip_prefix("export ")
            .or_else(|| trimmed.strip_prefix("export\t"))
            .unwrap_or(trimmed);

        // Split on first '='
        let Some(eq_pos) = line.find('=') else {
            continue;
        };

        let key = line[..eq_pos].trim().to_string();
        let raw_value = line[eq_pos + 1..].to_string();

        if key.is_empty() {
            continue;
        }

        let value = parse_dotenv_value(&raw_value);
        result.push((key, value));
    }

    Ok(result)
}

/// Parse a dotenv value, handling quoted and unquoted forms.
fn parse_dotenv_value(raw: &str) -> String {
    let trimmed = raw.trim();

    if trimmed.is_empty() {
        return String::new();
    }

    // Double-quoted value: handle escape sequences
    if trimmed.starts_with('"') {
        if let Some(end) = find_closing_quote(trimmed, '"') {
            let inner = &trimmed[1..end];
            return unescape_double_quoted(inner);
        }
    }

    // Single-quoted value: literal (no escaping)
    if trimmed.starts_with('\'') {
        if let Some(end) = find_closing_quote(trimmed, '\'') {
            return trimmed[1..end].to_string();
        }
    }

    // Unquoted value: strip inline comments
    if let Some(comment_pos) = trimmed.find(" #") {
        trimmed[..comment_pos].trim().to_string()
    } else {
        trimmed.to_string()
    }
}

/// Find the position of the closing quote character, respecting backslash escapes.
fn find_closing_quote(s: &str, quote: char) -> Option<usize> {
    let mut chars = s.char_indices().skip(1); // skip opening quote
    while let Some((i, c)) = chars.next() {
        if c == '\\' && quote == '"' {
            chars.next(); // skip escaped char
            continue;
        }
        if c == quote {
            return Some(i);
        }
    }
    None
}

/// Unescape double-quoted dotenv values.
fn unescape_double_quoted(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}
