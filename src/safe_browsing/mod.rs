// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use eyre::eyre;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Instant,
};
use tokio::sync::RwLock;

use crate::{http::HTTP, utils::sha256};

mod canonicalize;
mod models;
mod rice;

use canonicalize::canonicalize;
use models::{
    ClientInfo, FindFullHashesRequest, FindFullHashesResponse, ListUpdateRequest, ThreatEntry,
    ThreatInfo, ThreatListConstraints, ThreatListUpdateRequest, ThreatListUpdateResponse,
    ThreatMatch, ThreatType,
};

#[derive(Debug, Clone)]
struct SafeBrowsingListState {
    state: String,
    prefixes: Vec<Vec<u8>>,
    lookup: HashSet<Vec<u8>>,
    lengths: Vec<usize>,
}

impl SafeBrowsingListState {
    fn new(state: String, prefixes: Vec<Vec<u8>>) -> Self {
        let lookup: HashSet<Vec<u8>> = prefixes.iter().cloned().collect();

        let mut lengths: Vec<usize> = lookup.iter().map(Vec::len).collect();
        lengths.sort_unstable();
        lengths.dedup();

        Self {
            state,
            prefixes,
            lookup,
            lengths,
        }
    }

    fn matching_prefix<'a>(&self, hash: &'a [u8]) -> Option<&'a [u8]> {
        self.lengths
            .iter()
            .filter_map(|&len| hash.get(..len))
            .find(|prefix| self.lookup.contains(*prefix))
    }
}

#[derive(Debug, Clone)]
pub struct SafeBrowsing {
    key: String,
    states: Arc<RwLock<HashMap<ThreatType, SafeBrowsingListState>>>,
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

            let current_states: HashMap<ThreatType, String> = {
                let states_lock = self.states.read().await;

                states_lock
                    .iter()
                    .map(|(k, v)| (*k, v.state.clone()))
                    .collect()
            };

            let request = ThreatListUpdateRequest {
                client: ClientInfo::default(),
                list_update_requests: ThreatType::VARIANTS
                    .map(|threat_type| ListUpdateRequest {
                        threat_type,
                        platform_type: "ANY_PLATFORM".to_owned(),
                        threat_entry_type: "URL".to_owned(),

                        state: current_states
                            .get(&threat_type)
                            .cloned()
                            .unwrap_or_default(),

                        constraints: ThreatListConstraints {
                            max_update_entries: 50000,
                            max_database_entries: 100000,
                            region: "US".to_owned(),
                            supported_compressions: vec!["RAW".to_owned(), "RICE".to_owned()],
                        },
                    })
                    .to_vec(),
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
                    let indices: HashSet<usize> = if let Some(raw) = &removal.raw_indices {
                        raw.indices.iter().copied().collect()
                    } else if let Some(rice) = &removal.rice_indices {
                        let data = BASE64.decode(&rice.encoded_data)?;
                        rice::decode(
                            rice.first_value,
                            rice.rice_parameter,
                            rice.num_entries,
                            &data,
                        )?
                        .into_iter()
                        .map(|v| v as usize)
                        .collect()
                    } else {
                        return Err(eyre!("list update removal had no raw or rice indices"));
                    };

                    let mut idx = 0;
                    current_prefixes.retain(|_| {
                        let keep = !indices.contains(&idx);
                        idx += 1;
                        keep
                    });
                }

                for addition in &list_update.additions {
                    if let Some(raw) = &addition.raw_hashes {
                        let hashes = BASE64.decode(&raw.raw_hashes)?;
                        current_prefixes.extend(hashes.chunks(raw.prefix_size).map(|c| c.to_vec()));
                    } else if let Some(rice) = &addition.rice_hashes {
                        let data = BASE64.decode(&rice.encoded_data)?;
                        current_prefixes.extend(
                            rice::decode(
                                rice.first_value,
                                rice.rice_parameter,
                                rice.num_entries,
                                &data,
                            )?
                            .into_iter()
                            .map(|v| v.to_le_bytes().to_vec()),
                        );
                    } else {
                        return Err(eyre!("list update addition had no raw or rice hashes"));
                    }
                }

                current_prefixes.sort_unstable();

                let checksum = BASE64.encode(sha256(
                    &current_prefixes
                        .iter()
                        .flatten()
                        .copied()
                        .collect::<Vec<_>>(),
                ));

                if checksum == list_update.checksum.sha256 {
                    self.states.write().await.insert(
                        list_update.threat_type,
                        SafeBrowsingListState::new(list_update.new_client_state, current_prefixes),
                    );
                } else {
                    tracing::error!(
                        r#type = ?list_update.threat_type,
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
    pub async fn check(&self, urls: &[&str]) -> eyre::Result<Vec<(String, ThreatMatch)>> {
        if urls.is_empty() {
            return Ok(Vec::new());
        }

        let bench_start = Instant::now();

        let mut url_hashes: HashMap<String, HashSet<Vec<u8>>> = HashMap::new();

        for url in urls {
            for url_prefix in Self::generate_url_prefixes(url)? {
                url_hashes
                    .entry((*url).to_string())
                    .or_default()
                    .insert(sha256(url_prefix.as_bytes()));
            }

            if let Some(url_without_end_parens) = url.strip_suffix([')', ']']) {
                for url_prefix in Self::generate_url_prefixes(url_without_end_parens)? {
                    url_hashes
                        .entry(url_without_end_parens.to_string())
                        .or_default()
                        .insert(sha256(url_prefix.as_bytes()));
                }
            }
        }

        let (matched_hash_prefixes, client_states) = {
            let states = self.states.read().await;

            let matched = states
                .values()
                .flat_map(|list_state| {
                    url_hashes
                        .values()
                        .flatten()
                        .filter_map(|hash| list_state.matching_prefix(hash))
                        .map(<[u8]>::to_vec)
                })
                .collect::<HashSet<_>>();

            let client_states = states.values().map(|s| s.state.clone()).collect::<Vec<_>>();

            (matched, client_states)
        };

        if !matched_hash_prefixes.is_empty() {
            let request = FindFullHashesRequest {
                client: ClientInfo::default(),

                client_states,

                threat_info: ThreatInfo {
                    threat_types: ThreatType::VARIANTS.map(|v| v.to_string()).to_vec(),
                    platform_types: vec!["ANY_PLATFORM".to_owned()],
                    threat_entry_types: vec!["URL".to_owned()],
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
                .into_iter()
                .filter_map(|m| {
                    if let Ok(raw_threat_hash) = BASE64.decode(&m.threat.hash)
                        && let Some((url, _)) = url_hashes
                            .iter()
                            .find(|(_, h)| h.contains(&raw_threat_hash))
                    {
                        return Some((url.to_owned(), m));
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

    /// Per <https://developers.google.com/safe-browsing/v4/urls-hashing#suffixprefix-expressions>,
    /// at most 5 hostnames are checked: the exact host, and up to 4 more formed from its last 5
    /// components by successively removing the leading one (down to 2 components). IP-literal
    /// hosts are only ever checked as themselves.
    fn host_suffixes(host: &str) -> HashSet<String> {
        let components = host.split('.').collect::<Vec<_>>();
        let n = components.len();

        let mut suffixes = HashSet::new();
        suffixes.insert(host.to_owned());

        for len in 2..=n.min(5) {
            suffixes.insert(components[n - len..].join("."));
        }

        suffixes
    }

    fn generate_url_prefixes(url: &str) -> eyre::Result<impl Iterator<Item = String>> {
        let canonical_url = canonicalize(url)?;

        let hosts = match canonical_url
            .host()
            .ok_or_else(|| eyre!("URL has no host"))?
        {
            url::Host::Domain(host) => Self::host_suffixes(host),
            ip_host => HashSet::from([ip_host.to_string()]),
        };

        let mut prefixes = HashSet::new();

        for host in hosts {
            let mut url = canonical_url.clone();
            url.set_host(Some(&host))?;

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
