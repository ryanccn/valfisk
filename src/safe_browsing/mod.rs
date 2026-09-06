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
mod prefix_set;
mod rice;

use canonicalize::canonicalize;
use models::{
    ClientInfo, FindFullHashesRequest, FindFullHashesResponse, ListUpdateRequest, ThreatEntry,
    ThreatInfo, ThreatListConstraints, ThreatListUpdateRequest, ThreatListUpdateResponse,
    ThreatMatch, ThreatType,
};
use prefix_set::PrefixSet;

const MAX_UPDATE_ATTEMPTS: usize = 4;

/// There is only ever one set of removals, and its indices address the list as it
/// stood before the update, so all of them are decoded before any are applied.
fn decode_removals(removals: &[models::ListUpdateRemovals]) -> eyre::Result<Vec<usize>> {
    let mut indices = Vec::new();

    for removal in removals {
        if let Some(raw) = &removal.raw_indices {
            indices.extend(raw.indices.iter().copied());
        } else if let Some(rice) = &removal.rice_indices {
            let data = BASE64.decode(&rice.encoded_data)?;

            indices.extend(
                rice::decode(
                    rice.first_value,
                    rice.rice_parameter,
                    rice.num_entries,
                    &data,
                )?
                .into_iter()
                .map(|v| v as usize),
            );
        } else {
            return Err(eyre!("list update removal had no raw or rice indices"));
        }
    }

    indices.sort_unstable();
    indices.dedup();

    Ok(indices)
}

/// Added hash prefixes, decoded into one flat buffer instead of one allocation each.
struct Additions {
    data: Vec<u8>,
    offsets: Vec<usize>,
}

impl Additions {
    fn decode(additions: &[models::ListUpdateAdditions]) -> eyre::Result<Self> {
        let mut data = Vec::new();
        let mut offsets = vec![0];

        for addition in additions {
            if let Some(raw) = &addition.raw_hashes {
                if raw.prefix_size == 0 {
                    return Err(eyre!("list update addition had a zero prefix size"));
                }

                let hashes = BASE64.decode(&raw.raw_hashes)?;
                let mut chunks = hashes.chunks_exact(raw.prefix_size);

                for chunk in chunks.by_ref() {
                    data.extend_from_slice(chunk);
                    offsets.push(data.len());
                }

                if !chunks.remainder().is_empty() {
                    return Err(eyre!(
                        "list update addition was not a multiple of its prefix size"
                    ));
                }
            } else if let Some(rice) = &addition.rice_hashes {
                let encoded = BASE64.decode(&rice.encoded_data)?;

                for value in rice::decode(
                    rice.first_value,
                    rice.rice_parameter,
                    rice.num_entries,
                    &encoded,
                )? {
                    data.extend_from_slice(&value.to_le_bytes());
                    offsets.push(data.len());
                }
            } else {
                return Err(eyre!("list update addition had no raw or rice hashes"));
            }
        }

        Ok(Self { data, offsets })
    }

    fn len(&self) -> usize {
        self.offsets.len() - 1
    }

    fn iter(&self) -> impl Iterator<Item = &[u8]> {
        self.offsets.windows(2).map(|w| &self.data[w[0]..w[1]])
    }
}

#[derive(Debug, Clone)]
struct SafeBrowsingListState {
    state: String,
    prefixes: PrefixSet,
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
        for attempt in 1..=MAX_UPDATE_ATTEMPTS {
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
                        platform_type: "ANY_PLATFORM",
                        threat_entry_type: "URL",

                        state: current_states
                            .get(&threat_type)
                            .cloned()
                            .unwrap_or_default(),

                        constraints: ThreatListConstraints {
                            max_update_entries: 50000,
                            max_database_entries: 100000,
                            region: "US",
                            supported_compressions: vec!["RAW", "RICE"],
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
                let removals = decode_removals(&list_update.removals)?;
                let additions = Additions::decode(&list_update.additions)?;

                let prefixes = {
                    let states = self.states.read().await;
                    let existing = states.get(&list_update.threat_type).map(|s| &s.prefixes);

                    let mut surviving =
                        Vec::with_capacity(existing.map_or(0, PrefixSet::len) + additions.len());

                    if let Some(existing) = existing {
                        let mut next_removal = removals.iter().peekable();

                        for index in 0..existing.len() {
                            if next_removal.peek() == Some(&&index) {
                                next_removal.next();
                            } else {
                                surviving.push(existing.get(index));
                            }
                        }
                    }

                    surviving.extend(additions.iter());

                    PrefixSet::build(surviving)
                };

                let checksum = BASE64.encode(sha256(prefixes.as_bytes()));

                if checksum == list_update.checksum.sha256 {
                    self.states.write().await.insert(
                        list_update.threat_type,
                        SafeBrowsingListState {
                            state: list_update.new_client_state,
                            prefixes,
                        },
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
                return Ok(());
            }

            if attempt == MAX_UPDATE_ATTEMPTS {
                return Err(eyre!(
                    "Safe Browsing list checksums kept drifting after {MAX_UPDATE_ATTEMPTS} attempts"
                ));
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

        let mut candidates = Vec::with_capacity(urls.len() * 2);

        for url in urls {
            candidates.push(*url);

            if let Some(trimmed) = url.strip_suffix([')', ']']) {
                candidates.push(trimmed);
            }
        }

        candidates.sort_unstable();
        candidates.dedup();

        let mut url_hashes: Vec<(&str, HashSet<[u8; 32]>)> = Vec::with_capacity(candidates.len());

        for candidate in candidates {
            match Self::url_expressions(candidate) {
                Ok(expressions) => url_hashes.push((
                    candidate,
                    expressions
                        .iter()
                        .map(|expression| sha256(expression.as_bytes()))
                        .collect(),
                )),

                // A malformed link must not stop the rest of the message being scanned.
                Err(err) => tracing::debug!(url = candidate, "could not expand URL: {err}"),
            }
        }

        let (matched_hash_prefixes, client_states) = {
            let states = self.states.read().await;

            let mut matched = HashSet::new();

            for (_, hashes) in &url_hashes {
                for hash in hashes {
                    for list_state in states.values() {
                        if let Some(prefix) = list_state.prefixes.matching_prefix(hash) {
                            matched.insert(prefix.to_vec());
                        }
                    }
                }
            }

            let client_states = states.values().map(|s| s.state.clone()).collect::<Vec<_>>();

            (matched, client_states)
        };

        if !matched_hash_prefixes.is_empty() {
            let request = FindFullHashesRequest {
                client: ClientInfo::default(),

                client_states,

                threat_info: ThreatInfo {
                    threat_types: ThreatType::VARIANTS.map(ThreatType::as_str).to_vec(),
                    platform_types: vec!["ANY_PLATFORM"],
                    threat_entry_types: vec!["URL"],
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
                    // A returned hash is 4-32 bytes, so it may be a prefix.
                    let threat_hash = BASE64.decode(&m.threat.hash).ok()?;

                    let (url, _) = url_hashes
                        .iter()
                        .find(|(_, hashes)| hashes.iter().any(|h| h.starts_with(&threat_hash)))?;

                    Some(((*url).to_owned(), m))
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
    fn host_suffixes(host: &str) -> Vec<String> {
        let components = host.split('.').collect::<Vec<_>>();
        let n = components.len();

        let mut suffixes = Vec::with_capacity(5);
        suffixes.push(host.to_owned());

        for len in 2..=n.min(5) {
            let suffix = components[n - len..].join(".");

            if suffix != host {
                suffixes.push(suffix);
            }
        }

        suffixes
    }

    /// Per <https://developers.google.com/safe-browsing/v4/urls-hashing#suffixprefix-expressions>,
    /// each host is combined with at most 6 paths: the exact path with and without the query,
    /// and up to 4 paths formed by starting at the root and successively appending a path
    /// component, each keeping its trailing slash. The scheme, credentials and port are
    /// discarded.
    fn url_expressions(url: &str) -> eyre::Result<Vec<String>> {
        let canonical_url = canonicalize(url)?;

        let hosts = match canonical_url
            .host()
            .ok_or_else(|| eyre!("URL has no host"))?
        {
            url::Host::Domain(host) => Self::host_suffixes(host),
            ip_host => vec![ip_host.to_string()],
        };

        let path = canonical_url.path();
        let query = canonical_url.query();

        let mut paths = Vec::with_capacity(5);
        paths.push(path);
        paths.extend(path.match_indices('/').take(4).map(|(i, _)| &path[..=i]));

        paths.sort_unstable();
        paths.dedup();

        let mut expressions = Vec::with_capacity(hosts.len() * (paths.len() + 1));

        for host in &hosts {
            if let Some(query) = query {
                expressions.push(format!("{host}{path}?{query}"));
            }

            for path in &paths {
                expressions.push(format!("{host}{path}"));
            }
        }

        Ok(expressions)
    }
}

#[cfg(test)]
mod tests {
    use super::SafeBrowsing;

    fn expressions(url: &str) -> Vec<String> {
        let mut expressions = SafeBrowsing::url_expressions(url).unwrap();
        expressions.sort();
        expressions
    }

    fn sorted(values: &[&str]) -> Vec<String> {
        let mut values = values.iter().map(|v| (*v).to_owned()).collect::<Vec<_>>();
        values.sort();
        values
    }

    #[test]
    fn host_suffixes_follow_the_spec() {
        assert_eq!(SafeBrowsing::host_suffixes("b.c"), ["b.c"]);
        assert_eq!(SafeBrowsing::host_suffixes("a.b.c"), ["a.b.c", "b.c"]);
        assert_eq!(
            SafeBrowsing::host_suffixes("a.b.c.d.e.f.g"),
            ["a.b.c.d.e.f.g", "f.g", "e.f.g", "d.e.f.g", "c.d.e.f.g"]
        );
    }

    /// The worked examples from
    /// <https://developers.google.com/safe-browsing/v4/urls-hashing#suffixprefix-expressions>.
    #[test]
    fn url_expressions_match_the_spec_examples() {
        assert_eq!(
            expressions("http://a.b.c/1/2.html?param=1"),
            sorted(&[
                "a.b.c/1/2.html?param=1",
                "a.b.c/1/2.html",
                "a.b.c/",
                "a.b.c/1/",
                "b.c/1/2.html?param=1",
                "b.c/1/2.html",
                "b.c/",
                "b.c/1/",
            ])
        );

        assert_eq!(
            expressions("http://a.b.c.d.e.f.g/1.html"),
            sorted(&[
                "a.b.c.d.e.f.g/1.html",
                "a.b.c.d.e.f.g/",
                "c.d.e.f.g/1.html",
                "c.d.e.f.g/",
                "d.e.f.g/1.html",
                "d.e.f.g/",
                "e.f.g/1.html",
                "e.f.g/",
                "f.g/1.html",
                "f.g/",
            ])
        );

        assert_eq!(
            expressions("http://1.2.3.4/1/"),
            sorted(&["1.2.3.4/1/", "1.2.3.4/"])
        );
    }

    #[test]
    fn url_expressions_cap_root_anchored_paths() {
        assert_eq!(
            expressions("http://a.b.c/1/2/3/4/5.html"),
            sorted(&[
                "a.b.c/1/2/3/4/5.html",
                "a.b.c/",
                "a.b.c/1/",
                "a.b.c/1/2/",
                "a.b.c/1/2/3/",
                "b.c/1/2/3/4/5.html",
                "b.c/",
                "b.c/1/",
                "b.c/1/2/",
                "b.c/1/2/3/",
            ])
        );
    }

    #[test]
    fn url_expressions_drop_the_scheme() {
        assert_eq!(expressions("http://example.com/"), ["example.com/"]);
        assert_eq!(expressions("https://example.com/"), ["example.com/"]);
    }

    #[test]
    fn url_expressions_drop_credentials_and_port() {
        assert_eq!(
            expressions("http://user:pass@example.com:8080/"),
            ["example.com/"]
        );
    }

    #[test]
    fn url_expressions_keep_empty_queries() {
        assert_eq!(
            expressions("http://www.google.com/q?"),
            sorted(&[
                "www.google.com/q?",
                "www.google.com/q",
                "www.google.com/",
                "google.com/q?",
                "google.com/q",
                "google.com/",
            ])
        );
    }
}
