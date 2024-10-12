// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::net::SocketAddr;

use hickory_resolver::{
    config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts},
    error::ResolveErrorKind,
    proto::rr::RecordType,
    TokioAsyncResolver,
};

use crate::Context;
use poise::{
    serenity_prelude::{CreateEmbed, CreateEmbedFooter, Timestamp},
    ChoiceParameter, CreateReply,
};

use eyre::Result;
use once_cell::sync::Lazy;

pub static BOOTSTRAP_RESOLVER: Lazy<TokioAsyncResolver> = Lazy::new(|| {
    TokioAsyncResolver::tokio(ResolverConfig::cloudflare_https(), make_resolver_opts(true))
});

#[allow(clippy::upper_case_acronyms)]
#[derive(ChoiceParameter, Debug, Copy, Clone)]
enum RecordTypeChoice {
    A,
    AAAA,
    ANAME,
    ANY,
    CAA,
    CNAME,
    DNSKEY,
    DS,
    HTTPS,
    KEY,
    MX,
    NS,
    PTR,
    RRSIG,
    SIG,
    SOA,
    SRV,
    SVCB,
    TLSA,
    TXT,
}

impl RecordTypeChoice {
    fn as_record_type(self) -> RecordType {
        match self {
            Self::A => RecordType::A,
            Self::AAAA => RecordType::AAAA,
            Self::ANAME => RecordType::ANAME,
            Self::ANY => RecordType::ANY,
            Self::CAA => RecordType::CAA,
            Self::CNAME => RecordType::CNAME,
            Self::DNSKEY => RecordType::DNSKEY,
            Self::DS => RecordType::DS,
            Self::HTTPS => RecordType::HTTPS,
            Self::KEY => RecordType::KEY,
            Self::MX => RecordType::MX,
            Self::NS => RecordType::NS,
            Self::PTR => RecordType::PTR,
            Self::RRSIG => RecordType::RRSIG,
            Self::SIG => RecordType::SIG,
            Self::SOA => RecordType::SOA,
            Self::SRV => RecordType::SRV,
            Self::SVCB => RecordType::SVCB,
            Self::TLSA => RecordType::TLSA,
            Self::TXT => RecordType::TXT,
        }
    }
}

impl std::fmt::Display for RecordTypeChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_record_type().fmt(f)
    }
}

#[derive(ChoiceParameter, Debug, Copy, Clone)]
enum ResolverChoice {
    Cloudflare,
    #[name = "Cloudflare (security)"]
    CloudflareSecurity,
    #[name = "Cloudflare (family)"]
    CloudflareFamily,
    Google,
    Quad9,
    #[name = "dns0.eu"]
    DNS0EU,
    Mullvad,
    #[name = "Mullvad (adblock)"]
    MullvadAdblock,
    #[name = "Mullvad (base)"]
    MullvadBase,
    #[name = "Mullvad (extended)"]
    MullvadExtended,
    #[name = "Mullvad (family)"]
    MullvadFamily,
    #[name = "Mullvad (all)"]
    MullvadAll,
    AdGuard,
    #[name = "AdGuard (non-filtering)"]
    AdGuardNonfiltering,
    #[name = "AdGuard (family)"]
    AdGuardFamily,
    OpenDNS,
    #[name = "OpenDNS (FamilyShield)"]
    OpenDNSFamilyShield,
    #[name = "Wikimedia DNS"]
    Wikimedia,
}

impl ResolverChoice {
    fn domain(self) -> String {
        match self {
            Self::Cloudflare => "cloudflare-dns.com",
            Self::CloudflareSecurity => "security.cloudflare-dns.com",
            Self::CloudflareFamily => "family.cloudflare-dns.com",
            Self::Google => "dns.google",
            Self::Quad9 => "dns.quad9.net",
            Self::DNS0EU => "dns0.eu",
            Self::Mullvad => "dns.mullvad.net",
            Self::MullvadAdblock => "adblock.dns.mullvad.net",
            Self::MullvadBase => "base.dns.mullvad.net",
            Self::MullvadExtended => "extended.dns.mullvad.net",
            Self::MullvadFamily => "family.dns.mullvad.net",
            Self::MullvadAll => "all.dns.mullvad.net",
            Self::AdGuard => "dns.adguard-dns.com",
            Self::AdGuardNonfiltering => "unfiltered.adguard-dns.com",
            Self::AdGuardFamily => "family.adguard-dns.com",
            Self::OpenDNS => "doh.opendns.com",
            Self::OpenDNSFamilyShield => "doh.familyshield.opendns.com",
            Self::Wikimedia => "wikimedia-dns.org",
        }
        .into()
    }

    async fn doh_config(&self) -> Result<ResolverConfig> {
        let name_servers = BOOTSTRAP_RESOLVER
            .lookup_ip(self.domain() + ".")
            .await?
            .into_iter()
            .map(|ip| NameServerConfig {
                socket_addr: SocketAddr::new(ip, 443),
                tls_dns_name: Some(self.domain()),
                protocol: Protocol::Https,
                bind_addr: None,
                tls_config: None,
                trust_negative_responses: true,
            })
            .collect::<Vec<_>>();

        let mut config = ResolverConfig::new();
        for name_server in name_servers {
            config.add_name_server(name_server);
        }

        Ok(config)
    }
}

fn make_resolver_opts(bootstrap: bool) -> ResolverOpts {
    let mut opts = ResolverOpts::default();

    opts.attempts = 5;
    opts.use_hosts_file = false;
    opts.validate = true;

    if !bootstrap {
        opts.cache_size = 0;
    }

    opts
}

/// Make a DNS lookup
#[poise::command(slash_command, guild_only)]
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
pub async fn dig(
    ctx: Context<'_>,
    #[description = "The domain name to query for"] name: String,
    #[description = "The record type to query for"] r#type: RecordTypeChoice,
    #[description = "The resolver to use for queries"] resolver: Option<ResolverChoice>,
) -> Result<()> {
    let resolver = resolver.unwrap_or(ResolverChoice::Cloudflare);

    ctx.defer().await?;

    let hickory =
        TokioAsyncResolver::tokio(resolver.doh_config().await?, make_resolver_opts(false));

    match hickory.lookup(&name, r#type.as_record_type()).await {
        Ok(response) => {
            ctx.send(
                CreateReply::default().embed(
                    CreateEmbed::default()
                        .title(format!("{type} records on {name}"))
                        .description(
                            response
                                .record_iter()
                                .map(|r| {
                                    format!(
                                        "`{} {} {}`",
                                        r.name(),
                                        r.record_type(),
                                        r.data()
                                            .map_or_else(|| "<none>".to_owned(), |d| d.to_string())
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join("\n"),
                        )
                        .color(0xffa94d)
                        .footer(CreateEmbedFooter::new(resolver.domain()))
                        .timestamp(Timestamp::now()),
                ),
            )
            .await?;
        }

        Err(err) => {
            if matches!(err.kind(), ResolveErrorKind::NoRecordsFound { .. }) {
                ctx.say("No records found!").await?;
            } else {
                return Err(err.into());
            }
        }
    }

    Ok(())
}
