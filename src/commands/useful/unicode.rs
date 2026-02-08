// SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::{
    CreateReply,
    serenity_prelude::{
        CreateAllowedMentions, CreateComponent, CreateContainer, CreateContainerComponent,
        CreateSeparator, CreateTextDisplay, MessageFlags,
    },
};

use crate::Context;

const fn yesno(value: bool) -> &'static str {
    if value { "Yes" } else { "No" }
}

/// Show data on Unicode character(s) from the Unicode Character Database
#[tracing::instrument(skip(ctx), fields(ctx.channel = ctx.channel_id().get(), ctx.author = ctx.author().id.get()))]
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
                    unic::ucd::Name::of(ch)
                        .map_or_else(String::new, |name| format!(" \u{2013} {name}")),
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        ctx.send(
            CreateReply::default()
                .flags(MessageFlags::IS_COMPONENTS_V2)
                .allowed_mentions(CreateAllowedMentions::new())
                .components(&[
                    CreateComponent::Container(CreateContainer::new(&[
                        CreateContainerComponent::TextDisplay(CreateTextDisplay::new(compact_info)),
                        CreateContainerComponent::TextDisplay(CreateTextDisplay::new(format!(
                            "-# Unicode {} \u{00B7} [Unicode and Internationalization Crates for Rust](https://docs.rs/unic/)",
                            unic::UNICODE_VERSION
                        ))),
                    ])),
                ]),
        )
        .await?;
    } else {
        let mut components = vec![];

        for character in input.chars() {
            let char_components = vec![
                CreateContainerComponent::TextDisplay(CreateTextDisplay::new(format!(
                    r"### {}{}
## {character}",
                    unic::char::basics::unicode_notation(character),
                    unic::ucd::Name::of(character)
                        .map_or_else(String::new, |name| format!(" \u{2013} {name}")),
                ))),
                CreateContainerComponent::Separator(CreateSeparator::new(true)),
                CreateContainerComponent::TextDisplay(CreateTextDisplay::new(format!(
                    r"**Block**: {}
**General Category**: {}
**Age**: {}",
                    unic::ucd::Block::of(character).map_or("None", |block| block.name),
                    unic::ucd::GeneralCategory::of(character),
                    unic::ucd::Age::of(character).map_or_else(
                        || "Unknown".to_owned(),
                        |age| format!("Unicode {}", age.actual())
                    ),
                ))),
                CreateContainerComponent::Separator(CreateSeparator::new(true)),
                CreateContainerComponent::TextDisplay(CreateTextDisplay::new(format!(
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
                CreateContainerComponent::Separator(CreateSeparator::new(true)),
                CreateContainerComponent::TextDisplay(CreateTextDisplay::new(format!(
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
                CreateContainerComponent::TextDisplay(CreateTextDisplay::new(format!(
                    "-# Unicode {} \u{00B7} [Unicode and Internationalization Crates for Rust](https://docs.rs/unic/)",
                    unic::UNICODE_VERSION
                ))),
            ];

            components.push(CreateComponent::Container(CreateContainer::new(
                char_components,
            )));
        }

        ctx.send(
            CreateReply::default()
                .flags(MessageFlags::IS_COMPONENTS_V2)
                .allowed_mentions(CreateAllowedMentions::new())
                .components(components),
        )
        .await?;
    }

    Ok(())
}
