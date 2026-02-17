use serde::Serialize;

/// JSON response for `authy get --json`.
#[derive(Serialize)]
pub struct GetResponse {
    pub name: String,
    pub value: String,
    pub version: u32,
    pub created: String,
    pub modified: String,
}

/// JSON response for `authy list --json`.
#[derive(Serialize)]
pub struct ListResponse {
    pub secrets: Vec<SecretListItem>,
}

#[derive(Serialize)]
pub struct SecretListItem {
    pub name: String,
    pub version: u32,
    pub created: String,
    pub modified: String,
}

/// JSON response for `authy policy show --json`.
#[derive(Serialize)]
pub struct PolicyShowResponse {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub allow: Vec<String>,
    pub deny: Vec<String>,
    pub run_only: bool,
    pub created: String,
    pub modified: String,
}

/// JSON response for `authy policy list --json`.
#[derive(Serialize)]
pub struct PolicyListResponse {
    pub policies: Vec<PolicyListItem>,
}

#[derive(Serialize)]
pub struct PolicyListItem {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub allow_count: usize,
    pub deny_count: usize,
}

/// JSON response for `authy policy test --json`.
#[derive(Serialize)]
pub struct PolicyTestResponse {
    pub scope: String,
    pub secret: String,
    pub allowed: bool,
}

/// JSON response for `authy session create --json`.
#[derive(Serialize)]
pub struct SessionCreateResponse {
    pub token: String,
    pub session_id: String,
    pub scope: String,
    pub run_only: bool,
    pub expires: String,
}

/// JSON response for `authy session list --json`.
#[derive(Serialize)]
pub struct SessionListResponse {
    pub sessions: Vec<SessionListItem>,
}

#[derive(Serialize)]
pub struct SessionListItem {
    pub id: String,
    pub scope: String,
    pub status: String,
    pub run_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub created: String,
    pub expires: String,
}

/// JSON response for `authy audit show --json`.
#[derive(Serialize)]
pub struct AuditShowResponse {
    pub entries: Vec<AuditEntryItem>,
    pub shown: usize,
    pub total: usize,
}

#[derive(Serialize)]
pub struct AuditEntryItem {
    pub timestamp: String,
    pub operation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    pub actor: String,
    pub outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}
