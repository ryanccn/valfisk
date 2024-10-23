// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use eyre::eyre;
use sha2::{Digest, Sha256};

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::RwLock;

use crate::reqwest_client::HTTP;

mod canonicalize;
mod models;

use canonicalize::canonicalize;
use models::{
    ClientInfo, FindFullHashesRequest, FindFullHashesResponse, ListUpdateRequest, ThreatEntry,
    ThreatInfo, ThreatListConstraints, ThreatListUpdateRequest, ThreatListUpdateResponse,
    ThreatMatch,
};

static THREAT_TYPES: [&str; 3] = ["MALWARE", "SOCIAL_ENGINEERING", "UNWANTED_SOFTWARE"];

#[derive(Debug, Clone)]
struct SafeBrowsingListState {
    pub state: String,
    pub prefixes: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct SafeBrowsing {
    key: String,
    states: Arc<RwLock<HashMap<String, SafeBrowsingListState>>>,
}

impl SafeBrowsing {
    pub fn new(key: &str) -> Self {
        SafeBrowsing {
            key: key.to_owned(),
            states: Arc::default(),
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn update(&self) -> eyre::Result<()> {
        let current_states: HashMap<String, String> = {
            let states_lock = self.states.read().await;

            states_lock
                .iter()
                .map(|(k, v)| (k.clone(), v.state.clone()))
                .collect()
        };

        let request = ThreatListUpdateRequest {
            client: ClientInfo::default(),
            list_update_requests: THREAT_TYPES
                .into_iter()
                .map(|threat_type| ListUpdateRequest {
                    threat_type: threat_type.to_string(),
                    platform_type: "ANY_PLATFORM".to_string(),
                    threat_entry_type: "URL".to_string(),
                    state: current_states.get(threat_type).cloned().unwrap_or_default(),

                    constraints: ThreatListConstraints {
                        max_update_entries: 50000,
                        max_database_entries: 100000,
                        region: "US".to_string(),
                        supported_compressions: vec!["RAW".to_string()],
                    },
                })
                .collect(),
        };

        let response: ThreatListUpdateResponse = HTTP
            .post("https://safebrowsing.googleapis.com/v4/threatListUpdates:fetch")
            .query(&[("key", &self.key)])
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        for list_update in response.list_update_responses {
            let mut current_prefixes = self
                .states
                .read()
                .await
                .get(&list_update.threat_type)
                .map(|s| s.prefixes.clone())
                .unwrap_or_default();

            if let Some(removals) = list_update.removals {
                for entry_set in removals {
                    if let Some(raw_indices) = entry_set.raw_indices {
                        for index in raw_indices.indices {
                            if (index as usize) < current_prefixes.len() {
                                current_prefixes.remove(index as usize);
                            }
                        }
                    }
                }
            }

            if let Some(additions) = list_update.additions {
                for entry_set in additions {
                    if let Some(raw_hashes) = entry_set.raw_hashes {
                        let hashes = BASE64.decode(raw_hashes.raw_hashes)?;

                        current_prefixes.extend(
                            hashes
                                .chunks(raw_hashes.prefix_size as usize)
                                .map(|c| c.to_vec()),
                        );
                    }
                }
            }

            current_prefixes.sort_unstable();

            self.states.write().await.insert(
                list_update.threat_type,
                SafeBrowsingListState {
                    state: list_update.new_client_state,
                    prefixes: current_prefixes,
                },
            );
        }

        tracing::info!(
            "Updated Safe Browsing database => {} hash prefixes",
            self.states
                .read()
                .await
                .values()
                .map(|v| v.prefixes.len())
                .sum::<usize>()
        );

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn check_urls(&self, urls: &[&str]) -> eyre::Result<Vec<(String, ThreatMatch)>> {
        let mut url_hashes: HashMap<String, HashSet<Vec<u8>>> = HashMap::new();

        for url in urls {
            let url_prefixes = Self::generate_url_prefixes(url)?;

            for url_prefix in url_prefixes {
                let url_hash = Self::sha256(&url_prefix);

                if let Some(v) = url_hashes.get_mut(*url) {
                    v.insert(url_hash);
                } else {
                    let mut hs = HashSet::new();
                    hs.insert(url_hash);
                    url_hashes.insert((*url).to_string(), hs);
                }
            }
        }

        let mut matched_hash_prefixes = HashSet::new();
        let states = self.states.read().await;

        for hash in url_hashes.values().flatten() {
            let hash_prefix = hash[0..4].to_vec();

            for list_state in states.values() {
                if list_state.prefixes.contains(&hash_prefix) {
                    matched_hash_prefixes.insert(hash_prefix.clone());
                }
            }
        }

        drop(states);

        if !matched_hash_prefixes.is_empty() {
            let request = FindFullHashesRequest {
                client: ClientInfo::default(),

                client_states: self
                    .states
                    .read()
                    .await
                    .values()
                    .map(|s| s.state.clone())
                    .collect(),

                threat_info: ThreatInfo {
                    threat_types: THREAT_TYPES.map(String::from).to_vec(),
                    platform_types: vec!["ANY_PLATFORM".to_string()],
                    threat_entry_types: vec!["URL".to_string()],
                    threat_entries: matched_hash_prefixes
                        .iter()
                        .map(|hash| ThreatEntry {
                            hash: BASE64.encode(hash),
                        })
                        .collect(),
                },
            };

            let response: FindFullHashesResponse = HTTP
                .post("https://safebrowsing.googleapis.com/v4/fullHashes:find")
                .query(&[("key", &self.key)])
                .json(&request)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;

            let matches = response
                .matches
                .unwrap_or_default()
                .into_iter()
                .filter_map(|m| {
                    for (url, hashes) in &url_hashes {
                        if let Ok(raw_threat_hash) = BASE64.decode(&m.threat.hash) {
                            if hashes.contains(&raw_threat_hash) {
                                return Some((url.to_owned(), m));
                            }
                        }
                    }

                    None
                })
                .collect::<Vec<_>>();

            return Ok(matches);
        }

        Ok(Vec::new())
    }

    fn generate_url_prefixes(url: &str) -> eyre::Result<Vec<String>> {
        let mut url = canonicalize(url)?;

        let mut prefixes = Vec::new();
        prefixes.push(url.to_string());

        if url.query().is_some() {
            url.set_query(None);
            prefixes.push(url.to_string());
        }

        while url.path() != "/" {
            url.path_segments_mut()
                .map_err(|()| eyre!("could not obtain path segments"))?
                .pop();

            prefixes.push(url.to_string());
        }

        let prefixes = prefixes
            .into_iter()
            .map(|v| {
                v.trim_start_matches("http://")
                    .trim_start_matches("https://")
                    .to_owned()
            })
            .collect();

        Ok(prefixes)
    }

    fn sha256(input: &str) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hasher.finalize().to_vec()
    }
}
