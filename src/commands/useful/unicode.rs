// SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::{
    CreateReply,
    serenity_prelude::{
        CreateComponent, CreateContainer, CreateSeparator, CreateTextDisplay, MessageFlags,
    },
};

use crate::Context;

fn yesno(value: bool) -> &'static str {
    if value { "Yes" } else { "No" }
}

/// Show data on Unicode character(s) from the Unicode Character Database
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    install_context = "Guild | User",
    interaction_context = "Guild | BotDm | PrivateChannel"
)]
pub async fn unicode(
    ctx: Context<'_>,
    #[description = "Unicode character(s)"] input: String,
    #[description = "Only show codepoints and names"] compact: Option<bool>,
) -> Result<()> {
    if compact.unwrap_or_else(|| input.chars().count() > 1) {
        let compact_info = input
            .chars()
            .map(|ch| {
                format!(
                    "`{ch}` {}{}",
                    unic::char::basics::unicode_notation(ch),
                    if let Some(name) = unic::ucd::Name::of(ch) {
                        format!(" \u{2013} {name}")
                    } else {
                        String::new()
                    },
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        ctx.send(
            CreateReply::new()
                .components(&[CreateComponent::Container(CreateContainer::new(&[
                    CreateComponent::TextDisplay(CreateTextDisplay::new(compact_info)),
                    CreateComponent::TextDisplay(CreateTextDisplay::new(format!(
                        "-# Unicode {} \u{00B7} [Unicode and Internationalization Crates for Rust](https://docs.rs/unic/)",
                        unic::UNICODE_VERSION
                    )))
                ]))])
                .flags(MessageFlags::IS_COMPONENTS_V2),
        )
        .await?;
    } else {
        let mut components = vec![];

        for character in input.chars() {
            let char_components = vec![
                CreateComponent::TextDisplay(CreateTextDisplay::new(format!(
                    r"### {}{}
## {character}",
                    unic::char::basics::unicode_notation(character),
                    if let Some(name) = unic::ucd::Name::of(character) {
                        format!(" \u{2013} {name}")
                    } else {
                        String::new()
                    },
                ))),
                CreateComponent::Separator(CreateSeparator::new(true)),
                CreateComponent::TextDisplay(CreateTextDisplay::new(format!(
                    r"**Block**: {}
**General Category**: {}
**Age**: {}",
                    match unic::ucd::Block::of(character) {
                        Some(block) => block.name,
                        None => "None",
                    },
                    unic::ucd::GeneralCategory::of(character),
                    match unic::ucd::Age::of(character) {
                        Some(age) => format!("Unicode {}", age.actual()),
                        None => "Unknown".to_owned(),
                    },
                ))),
                CreateComponent::Separator(CreateSeparator::new(true)),
                CreateComponent::TextDisplay(CreateTextDisplay::new(format!(
                    r"**Alphabetic**: {}
**Bidirectionally mirrored**: {}
**Case ignorable**: {}
**Cased**: {}
**Lowercase**: {}
**Uppercase**: {}
**Whitespace**: {}
**Private use**: {}
**Noncharacter**: {}",
                    yesno(unic::ucd::is_alphabetic(character)),
                    yesno(unic::ucd::is_bidi_mirrored(character)),
                    yesno(unic::ucd::is_case_ignorable(character)),
                    yesno(unic::ucd::is_cased(character)),
                    yesno(unic::ucd::is_lowercase(character)),
                    yesno(unic::ucd::is_uppercase(character)),
                    yesno(unic::ucd::is_white_space(character)),
                    yesno(unic::char::basics::is_private_use(character)),
                    yesno(unic::char::basics::is_noncharacter(character)),
                ))),
                CreateComponent::Separator(CreateSeparator::new(true)),
                CreateComponent::TextDisplay(CreateTextDisplay::new(format!(
                    r"**Emoji**: {}
**Emoji component**: {}
**Emoji modifier**: {}
**Emoji modifier base**: {}
**Emoji presentation**: {}",
                    yesno(unic::emoji::char::is_emoji(character)),
                    yesno(unic::emoji::char::is_emoji_component(character)),
                    yesno(unic::emoji::char::is_emoji_modifier(character)),
                    yesno(unic::emoji::char::is_emoji_modifier_base(character)),
                    yesno(unic::emoji::char::is_emoji_presentation(character)),
                ))),
                CreateComponent::TextDisplay(CreateTextDisplay::new(format!(
                    "-# Unicode {} \u{00B7} [Unicode and Internationalization Crates for Rust](https://docs.rs/unic/)",
                    unic::UNICODE_VERSION
                ))),
            ];

            components.push(CreateComponent::Container(CreateContainer::new(
                char_components,
            )));
        }

        ctx.send(
            CreateReply::new()
                .components(components)
                .flags(MessageFlags::IS_COMPONENTS_V2),
        )
        .await?;
    }

    Ok(())
}
