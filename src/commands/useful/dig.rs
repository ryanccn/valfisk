use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    error::ResolveErrorKind,
    proto::rr::RecordType,
    TokioAsyncResolver,
};

use crate::Context;
use poise::{
    serenity_prelude::{CreateEmbed, CreateEmbedFooter, Timestamp},
    ChoiceParameter, CreateReply,
};

use color_eyre::eyre::Result;
use once_cell::sync::Lazy;

pub static RESOLVER: Lazy<TokioAsyncResolver> = Lazy::new(|| {
    TokioAsyncResolver::tokio(ResolverConfig::cloudflare_https(), ResolverOpts::default())
});

#[allow(clippy::upper_case_acronyms)]
#[derive(ChoiceParameter, Debug, Copy, Clone)]
enum DiscordRecordType {
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

impl DiscordRecordType {
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

impl std::fmt::Display for DiscordRecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_record_type().fmt(f)
    }
}

/// Make a DNS lookup
#[poise::command(slash_command, guild_only)]
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
pub async fn dig(
    ctx: Context<'_>,
    #[description = "The domain name to query for"] name: String,
    #[description = "The record type to query for"] r#type: DiscordRecordType,
) -> Result<()> {
    ctx.defer().await?;

    match RESOLVER.lookup(&name, r#type.as_record_type()).await {
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
                        .footer(CreateEmbedFooter::new(
                            "https://cloudflare-dns.com/dns-query",
                        ))
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
