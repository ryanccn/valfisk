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

use crate::{Context, ucd};

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
                    ucd::unicode_notation(ch),
                    ucd::name_of(ch).map_or_else(String::new, |name| format!(" \u{2013} {name}")),
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        ctx.send(
            CreateReply::default()
                .flags(MessageFlags::IS_COMPONENTS_V2)
                .allowed_mentions(CreateAllowedMentions::new())
                .components(&[CreateComponent::Container(CreateContainer::new(&[
                    CreateContainerComponent::TextDisplay(CreateTextDisplay::new(compact_info)),
                    CreateContainerComponent::TextDisplay(CreateTextDisplay::new(format!(
                        "-# Unicode {}",
                        ucd::UNICODE_VERSION
                    ))),
                ]))]),
        )
        .await?;
    } else {
        let mut components = vec![];

        for character in input.chars() {
            let (cat_abbr, cat_name) = ucd::general_category_of(character);

            let char_components = vec![
                CreateContainerComponent::TextDisplay(CreateTextDisplay::new(format!(
                    r"### {}{}
## {character}",
                    ucd::unicode_notation(character),
                    ucd::name_of(character)
                        .map_or_else(String::new, |name| format!(" \u{2013} {name}")),
                ))),
                CreateContainerComponent::Separator(CreateSeparator::new().divider(true)),
                CreateContainerComponent::TextDisplay(CreateTextDisplay::new({
                    let mut s = format!(
                        "**Block**: {}\n**Script**: {}\n**General Category**: {} ({cat_abbr})\n**Bidi Class**: {}\n**Age**: {}",
                        ucd::block_of(character).unwrap_or("None"),
                        ucd::script_of(character).unwrap_or("Unknown"),
                        cat_name,
                        ucd::bidi_class_of(character).map_or_else(
                            || "Unknown".to_owned(),
                            |(abbr, name)| format!("{name} ({abbr})")
                        ),
                        ucd::age_of(character)
                            .map_or_else(|| "Unknown".to_owned(), |v| format!("Unicode {v}")),
                    );
                    if let Some(num_val) = ucd::numeric_value_of(character) {
                        s.push_str(&format!("\n**Numeric Value**: {num_val}"));
                    }
                    s
                })),
                CreateContainerComponent::Separator(CreateSeparator::new().divider(true)),
                CreateContainerComponent::TextDisplay(CreateTextDisplay::new(format!(
                    r"**Alphabetic**: {}
**Bidirectionally mirrored**: {}
**Case ignorable**: {}
**Cased**: {}
**Lowercase**: {}
**Uppercase**: {}
**Whitespace**: {}
**ID Start**: {}
**ID Continue**: {}
**Private use**: {}
**Noncharacter**: {}",
                    yesno(ucd::is_alphabetic(character)),
                    yesno(ucd::is_bidi_mirrored(character)),
                    yesno(ucd::is_case_ignorable(character)),
                    yesno(ucd::is_cased(character)),
                    yesno(ucd::is_lowercase(character)),
                    yesno(ucd::is_uppercase(character)),
                    yesno(ucd::is_white_space(character)),
                    yesno(ucd::is_id_start(character)),
                    yesno(ucd::is_id_continue(character)),
                    yesno(ucd::is_private_use(character)),
                    yesno(ucd::is_noncharacter(character)),
                ))),
                CreateContainerComponent::Separator(CreateSeparator::new().divider(true)),
                CreateContainerComponent::TextDisplay(CreateTextDisplay::new(format!(
                    r"**Emoji**: {}
**Emoji component**: {}
**Emoji modifier**: {}
**Emoji modifier base**: {}
**Emoji presentation**: {}",
                    yesno(ucd::is_emoji(character)),
                    yesno(ucd::is_emoji_component(character)),
                    yesno(ucd::is_emoji_modifier(character)),
                    yesno(ucd::is_emoji_modifier_base(character)),
                    yesno(ucd::is_emoji_presentation(character)),
                ))),
                CreateContainerComponent::TextDisplay(CreateTextDisplay::new(format!(
                    "-# Unicode {}",
                    ucd::UNICODE_VERSION
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
