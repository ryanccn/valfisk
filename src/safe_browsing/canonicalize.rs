// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use url::Url;

pub fn canonicalize(input: &str) -> eyre::Result<Url> {
    let input = input.trim().replace(['\t', '\r', '\n'], "");
    let mut url = Url::parse(&input)?;

    let _ = url.set_port(None);
    url.set_fragment(None);

    if let Some(host) = url.host_str() {
        let host = host.trim_matches('.').to_lowercase();

        let canonical_host = host
            .split('.')
            .filter(|&x| !x.is_empty())
            .collect::<Vec<&str>>()
            .join(".");

        url.set_host(Some(&canonical_host))?;
    }

    let segments: Vec<String> = url
        .path_segments()
        .map(|path| path.collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .filter(|&x| !x.is_empty() && x != ".")
        .map(|s| s.to_string())
        .collect();

    let mut final_segments = Vec::new();
    for segment in segments {
        if segment == ".." {
            final_segments.pop();
        } else {
            final_segments.push(segment);
        }
    }

    let new_path = if final_segments.is_empty() {
        "/"
    } else {
        &format!("/{}", final_segments.join("/"))
    };

    url.set_path(new_path);

    Ok(url)
}
