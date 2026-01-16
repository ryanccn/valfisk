// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use reqwest::Client;
use std::{sync::LazyLock, time::Duration};

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub static HTTP: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(30))
        .https_only(true)
        .tls_backend_preconfigured(
            rustls::ClientConfig::builder_with_protocol_versions(rustls::DEFAULT_VERSIONS)
                .with_root_certificates(rustls::RootCertStore {
                    roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
                })
                .with_no_client_auth(),
        )
        .build()
        .unwrap()
});
