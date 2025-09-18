// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::{
    ChoiceParameter, CreateReply,
    serenity_prelude::{
        CreateComponent, CreateContainer, CreateTextDisplay, FormattedTimestamp, MessageFlags,
    },
};

use hickory_resolver::{
    Name, TokioResolver,
    config::{NameServerConfigGroup, ResolveHosts, ResolverConfig, ResolverOpts},
    name_server::TokioConnectionProvider,
    proto::rr::RecordType,
};

use eyre::Result;
use std::{fmt, sync::LazyLock};

use crate::Context;

fn fqdn(name: &str) -> Result<Name> {
    let mut name = Name::from_utf8(name)?;
    name.set_fqdn(true);
    Ok(name)
}

fn set_resolver_opts(options: &mut ResolverOpts, bootstrap: bool) {
    options.attempts = 5;
    options.use_hosts_file = ResolveHosts::Never;
    options.validate = true;

    if !bootstrap {
        options.cache_size = 0;
    }
}

pub static BOOTSTRAP_RESOLVER: LazyLock<TokioResolver> = LazyLock::new(|| {
    let mut builder = TokioResolver::builder_with_config(
        ResolverConfig::cloudflare_https(),
        TokioConnectionProvider::default(),
    );

    set_resolver_opts(builder.options_mut(), true);

    builder.build()
});

#[expect(clippy::upper_case_acronyms)]
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
    const fn as_record_type(self) -> RecordType {
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

impl fmt::Display for RecordTypeChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    fn domain(self) -> &'static str {
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
    }

    async fn resolver_config(&self) -> Result<ResolverConfig> {
        let domain = self.domain();

        let ips = BOOTSTRAP_RESOLVER
            .lookup_ip(fqdn(domain)?)
            .await?
            .into_iter()
            .collect::<Vec<_>>();

        let name_servers =
            NameServerConfigGroup::from_ips_https(&ips, 443, domain.to_owned(), true);

        Ok(ResolverConfig::from_parts(None, Vec::new(), name_servers))
    }
}

/// Make a DNS lookup
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    install_context = "Guild | User",
    interaction_context = "Guild | BotDm | PrivateChannel"
)]
pub async fn dig(
    ctx: Context<'_>,
    #[description = "The domain name to query for"] name: String,
    #[description = "The record type to query for"] r#type: RecordTypeChoice,
    #[description = "The resolver to use for queries"] resolver: Option<ResolverChoice>,
) -> Result<()> {
    let resolver = resolver.unwrap_or(ResolverChoice::Cloudflare);

    ctx.defer().await?;

    let hickory = {
        let mut builder = TokioResolver::builder_with_config(
            resolver.resolver_config().await?,
            TokioConnectionProvider::default(),
        );

        set_resolver_opts(builder.options_mut(), false);

        builder.build()
    };

    let Ok(fqdn) = fqdn(&name) else {
        ctx.say("Invalid domain name provided!").await?;
        return Ok(());
    };

    match hickory.lookup(fqdn, r#type.as_record_type()).await {
        Ok(response) => {
            ctx.send(
                CreateReply::default()
                    .flags(MessageFlags::IS_COMPONENTS_V2)
                    .components(&[CreateComponent::Container(
                        CreateContainer::new(&[
                            CreateComponent::TextDisplay(CreateTextDisplay::new(format!(
                                "### {type} records on {name}"
                            ))),
                            CreateComponent::TextDisplay(CreateTextDisplay::new(format!(
                                "```\n{}\n```",
                                response
                                    .record_iter()
                                    .map(|r| format!(
                                        "{}  {}  {}",
                                        r.name(),
                                        r.record_type(),
                                        r.data()
                                    ))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            ))),
                            CreateComponent::TextDisplay(CreateTextDisplay::new(format!(
                                "-# {} \u{00B7} {}",
                                resolver.domain(),
                                FormattedTimestamp::now()
                            ))),
                        ])
                        .accent_color(0xffa94d),
                    )]),
            )
            .await?;
        }

        Err(err) => {
            if err.is_nx_domain() {
                ctx.say("The domain does not exist.").await?;
            } else if err.is_no_records_found() {
                ctx.say("No records were returned.").await?;
            } else {
                return Err(err.into());
            }
        }
    }

    Ok(())
}
