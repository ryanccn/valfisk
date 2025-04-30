// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use eyre::eyre;
use sha2::{Digest as _, Sha256};

use rayon::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Instant,
};
use tokio::sync::RwLock;

use crate::http::HTTP;

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
    state: String,
    prefixes: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct SafeBrowsing {
    key: String,
    states: Arc<RwLock<HashMap<String, SafeBrowsingListState>>>,
}

impl SafeBrowsing {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_owned(),
            states: Arc::default(),
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn update(&self) -> eyre::Result<()> {
        loop {
            let mut failed = false;

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
                .error_for_status()?
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

                for removal in &list_update.removals {
                    let mut indices = removal.raw_indices.indices.clone();
                    indices.sort_unstable();

                    for idx in indices.into_iter().rev() {
                        if idx < current_prefixes.len() {
                            current_prefixes.remove(idx);
                        }
                    }
                }

                for addition in &list_update.additions {
                    let hashes = BASE64.decode(&addition.raw_hashes.raw_hashes)?;

                    current_prefixes.extend(
                        hashes
                            .chunks(addition.raw_hashes.prefix_size)
                            .map(|c| c.to_vec()),
                    );
                }

                current_prefixes.sort_unstable();

                let checksum = BASE64.encode(Sha256::digest(
                    current_prefixes
                        .clone()
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>(),
                ));

                if checksum == list_update.checksum.sha256 {
                    self.states.write().await.insert(
                        list_update.threat_type,
                        SafeBrowsingListState {
                            state: list_update.new_client_state,
                            prefixes: current_prefixes,
                        },
                    );
                } else {
                    tracing::error!(
                        r#type = list_update.threat_type,
                        actual = checksum,
                        expected = list_update.checksum.sha256,
                        "list checksum has drifted, resetting",
                    );

                    self.states.write().await.remove(&list_update.threat_type);
                    failed = true;
                }
            }

            let prefixes = self
                .states
                .read()
                .await
                .values()
                .map(|v| v.prefixes.len())
                .sum::<usize>();

            tracing::info!(prefixes, "updated Safe Browsing database");

            if !failed {
                break;
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn check_urls(&self, urls: &[&str]) -> eyre::Result<Vec<(String, ThreatMatch)>> {
        if urls.is_empty() {
            return Ok(Vec::new());
        }

        let bench_start = Instant::now();

        let mut url_hashes: HashMap<String, HashSet<Vec<u8>>> = HashMap::new();

        for url in urls {
            for url_prefix in Self::generate_url_prefixes(url)? {
                let url_hash = Sha256::digest(&url_prefix).to_vec();

                url_hashes
                    .entry((*url).to_string())
                    .or_default()
                    .insert(url_hash);
            }
        }

        let matched_hash_prefixes = self
            .states
            .read()
            .await
            .values()
            .par_bridge()
            .map(|list_state| {
                url_hashes
                    .values()
                    .flatten()
                    .par_bridge()
                    .map(|hash| {
                        list_state
                            .prefixes
                            .par_iter()
                            .filter(|prefix| hash.starts_with(prefix))
                            .map(|p| p.to_owned())
                    })
                    .flatten()
            })
            .flatten()
            .collect::<HashSet<_>>();

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
                        .par_iter()
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
                .into_par_iter()
                .filter_map(|m| {
                    if let Ok(raw_threat_hash) = BASE64.decode(&m.threat.hash) {
                        if let Some((url, _)) = url_hashes
                            .par_iter()
                            .find_any(|(_, h)| h.contains(&raw_threat_hash))
                        {
                            return Some((url.to_owned(), m));
                        }
                    }

                    None
                })
                .collect::<Vec<_>>();

            tracing::trace!(
                urls = urls.len(),
                matches = matches.len(),
                elapsed = ?bench_start.elapsed(),
                "scanned with Safe Browsing (prefixes matched)",
            );

            return Ok(matches);
        }

        tracing::trace!(
            urls = urls.len(),
            elapsed = ?bench_start.elapsed(),
            "scanned with Safe Browsing (no prefixes matched)",
        );

        Ok(Vec::new())
    }

    fn generate_url_prefixes(url: &str) -> eyre::Result<impl Iterator<Item = String>> {
        let mut url = canonicalize(url)?;

        let mut prefixes = HashSet::new();
        prefixes.insert(url.to_string());

        if url.query().is_some() {
            url.set_query(None);
            prefixes.insert(url.to_string());
        }

        while url.path() != "/" {
            url.path_segments_mut()
                .map_err(|()| eyre!("could not obtain path segments"))?
                .pop();

            prefixes.insert(url.to_string());
        }

        Ok(prefixes.into_iter().map(|v| {
            v.strip_prefix("http://")
                .unwrap_or(&v)
                .strip_prefix("https://")
                .unwrap_or(&v)
                .to_owned()
        }))
    }
}
