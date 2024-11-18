// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreatListUpdateRequest {
    pub client: ClientInfo,
    pub list_update_requests: Vec<ListUpdateRequest>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    pub client_id: String,
    pub client_version: String,
}

impl Default for ClientInfo {
    fn default() -> Self {
        Self {
            client_id: env!("CARGO_PKG_NAME").to_string(),
            client_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUpdateRequest {
    pub threat_type: String,
    pub platform_type: String,
    pub threat_entry_type: String,
    pub state: String,
    pub constraints: ThreatListConstraints,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreatListConstraints {
    pub max_update_entries: u32,
    pub max_database_entries: u32,
    pub region: String,
    pub supported_compressions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreatListUpdateResponse {
    pub list_update_responses: Vec<ListUpdateResponse>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUpdateResponse {
    pub threat_type: String,
    pub new_client_state: String,
    pub checksum: ListUpdateChecksum,

    #[serde(default)]
    pub additions: Vec<ListUpdateAdditions>,
    #[serde(default)]
    pub removals: Vec<ListUpdateRemovals>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUpdateChecksum {
    pub sha256: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUpdateAdditions {
    pub raw_hashes: RawHashes,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUpdateRemovals {
    pub raw_indices: RawIndices,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawHashes {
    pub prefix_size: usize,
    pub raw_hashes: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawIndices {
    pub indices: Vec<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FindFullHashesRequest {
    pub client: ClientInfo,
    pub client_states: Vec<String>,
    pub threat_info: ThreatInfo,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreatInfo {
    pub threat_types: Vec<String>,
    pub platform_types: Vec<String>,
    pub threat_entry_types: Vec<String>,
    pub threat_entries: Vec<ThreatEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreatEntry {
    pub hash: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindFullHashesResponse {
    #[serde(default)]
    pub matches: Vec<ThreatMatch>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreatMatch {
    pub threat_type: String,
    pub threat: ThreatEntry,
}
