// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, utf8_percent_encode};
use url::Url;

static ENCODE_SET: AsciiSet = CONTROLS.add(b'#').add(b'%');

fn decode_encode(input: &str) -> String {
    let mut output = input.to_owned();

    loop {
        let decoded = percent_decode_str(&output).decode_utf8_lossy();
        if output == decoded {
            break;
        }
        output = decoded.into_owned();
    }

    utf8_percent_encode(&output, &ENCODE_SET).to_string()
}

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

    let segments = url
        .path_segments()
        .map(|path| path.collect::<Vec<_>>())
        .unwrap_or_default();

    let mut final_segments = Vec::new();

    for segment in segments
        .iter()
        .enumerate()
        .filter(|&(i, x)| x != &"." && (x != &"" || i == segments.len() - 1))
        .map(|(_, x)| (*x).to_string())
    {
        if segment == ".." {
            final_segments.pop();
        } else {
            final_segments.push(decode_encode(&segment));
        }
    }

    let path = if final_segments.is_empty() {
        "/"
    } else {
        &format!("/{}", final_segments.join("/"))
    };
    url.set_path(path);

    Ok(url)
}

#[cfg(test)]
mod tests {
    fn canonicalize(input: &str) -> String {
        super::canonicalize(input).unwrap().to_string()
    }

    /// <https://developers.google.com/safe-browsing/v4/urls-hashing#canonicalization>
    ///
    /// Tests with missing protocols were removed, since they are handled elsewhere;
    /// tests with invalid domains were also removed since `url` refuses to parse them.
    #[allow(clippy::too_many_lines)]
    #[test]
    fn canonicalize_works() {
        assert_eq!(canonicalize("http://host/%25%32%35"), "http://host/%25");
        assert_eq!(
            canonicalize("http://host/%25%32%35%25%32%35"),
            "http://host/%25%25"
        );
        assert_eq!(
            canonicalize("http://host/%2525252525252525"),
            "http://host/%25"
        );
        assert_eq!(
            canonicalize("http://host/asdf%25%32%35asd"),
            "http://host/asdf%25asd"
        );
        assert_eq!(
            canonicalize("http://host/%%%25%32%35asd%%"),
            "http://host/%25%25%25asd%25%25"
        );
        assert_eq!(
            canonicalize("http://www.google.com/"),
            "http://www.google.com/"
        );
        assert_eq!(
            canonicalize(
                "http://%31%36%38%2e%31%38%38%2e%39%39%2e%32%36/%2E%73%65%63%75%72%65/%77%77%77%2E%65%62%61%79%2E%63%6F%6D/"
            ),
            "http://168.188.99.26/.secure/www.ebay.com/"
        );
        assert_eq!(
            canonicalize(
                "http://195.127.0.11/uploads/%20%20%20%20/.verify/.eBaysecure=updateuserdataxplimnbqmn-xplmvalidateinfoswqpcmlx=hgplmcx/"
            ),
            "http://195.127.0.11/uploads/%20%20%20%20/.verify/.eBaysecure=updateuserdataxplimnbqmn-xplmvalidateinfoswqpcmlx=hgplmcx/"
        );
        assert_eq!(
            canonicalize("http://3279880203/blah"),
            "http://195.127.0.11/blah"
        );
        assert_eq!(
            canonicalize("http://www.google.com/blah/.."),
            "http://www.google.com/"
        );
        assert_eq!(
            canonicalize("http://www.evil.com/blah#frag"),
            "http://www.evil.com/blah"
        );
        assert_eq!(
            canonicalize("http://www.GOOgle.com/"),
            "http://www.google.com/"
        );
        assert_eq!(
            canonicalize("http://www.google.com.../"),
            "http://www.google.com/"
        );
        assert_eq!(
            canonicalize("http://www.google.com/foo\tbar\rbaz\n2"),
            "http://www.google.com/foobarbaz2"
        );
        assert_eq!(
            canonicalize("http://www.google.com/q?"),
            "http://www.google.com/q?"
        );
        assert_eq!(
            canonicalize("http://www.google.com/q?r?"),
            "http://www.google.com/q?r?"
        );
        assert_eq!(
            canonicalize("http://www.google.com/q?r?s"),
            "http://www.google.com/q?r?s"
        );
        assert_eq!(
            canonicalize("http://evil.com/foo#bar#baz"),
            "http://evil.com/foo"
        );
        assert_eq!(canonicalize("http://evil.com/foo;"), "http://evil.com/foo;");
        assert_eq!(
            canonicalize("http://evil.com/foo?bar;"),
            "http://evil.com/foo?bar;"
        );
        assert_eq!(
            canonicalize("http://notrailingslash.com"),
            "http://notrailingslash.com/"
        );
        assert_eq!(
            canonicalize("http://www.gotaport.com:1234/"),
            "http://www.gotaport.com/"
        );
        assert_eq!(
            canonicalize("  http://www.google.com/  "),
            "http://www.google.com/"
        );
        assert_eq!(
            canonicalize("https://www.securesite.com/"),
            "https://www.securesite.com/"
        );
        assert_eq!(
            canonicalize("http://host.com/ab%23cd"),
            "http://host.com/ab%23cd"
        );
        assert_eq!(
            canonicalize("http://host.com//twoslashes?more//slashes"),
            "http://host.com/twoslashes?more//slashes"
        );
    }
}
