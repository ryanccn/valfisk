// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;

use indexmap::IndexMap;
use poise::serenity_prelude::{CreateEmbed, CreateMessage};
use serde::{Deserialize, Serialize};

const fn default_to_false() -> bool {
    false
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Template {
    #[serde(default)]
    pub components: Vec<Component>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Component {
    Embed(EmbedComponent),
    Rules(RulesComponent),
    Links(LinksComponent),
    Text(TextComponent),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct EmbedComponent {
    #[serde(default)]
    embeds: Vec<EmbedComponentEmbed>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct EmbedComponentEmbed {
    title: Option<String>,
    description: Option<String>,
    color: Option<u32>,
    fields: Option<Vec<EmbedComponentEmbedField>>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct EmbedComponentEmbedField {
    name: String,
    value: String,
    #[serde(default = "default_to_false")]
    inline: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct RulesComponent {
    #[serde(default)]
    rules: IndexMap<String, String>,
    #[serde(default)]
    colors: Vec<u32>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct LinksComponent {
    title: String,
    color: Option<u32>,
    links: IndexMap<String, String>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct TextComponent {
    text: String,
}

impl Template {
    pub fn parse(source: &str) -> Result<Self> {
        Ok(toml::from_str(source)?)
    }
}

impl Template {
    pub fn to_messages(&self) -> Vec<CreateMessage> {
        self.components
            .iter()
            .map(|component| match component {
                Component::Embed(data) => CreateMessage::default().embeds(
                    data.embeds
                        .iter()
                        .map(|data| {
                            let mut embed = CreateEmbed::default();

                            if let Some(title) = &data.title {
                                embed = embed.title(title);
                            }
                            if let Some(description) = &data.description {
                                embed = embed.description(description);
                            }
                            if let Some(color) = &data.color {
                                embed = embed.color(*color);
                            }
                            if let Some(fields) = &data.fields {
                                embed = embed
                                    .fields(fields.iter().map(|f| (&f.name, &f.value, f.inline)));
                            }

                            embed
                        })
                        .collect::<Vec<_>>(),
                ),

                Component::Links(data) => CreateMessage::default().embed({
                    let mut embed = CreateEmbed::default().title(&data.title).description(
                        data.links
                            .iter()
                            .map(|(title, href)| format!("» [{title}]({href})"))
                            .collect::<Vec<_>>()
                            .join("\n"),
                    );

                    if let Some(color) = data.color {
                        embed = embed.color(color);
                    }

                    embed
                }),

                Component::Rules(data) => CreateMessage::default().embeds(
                    data.rules
                        .iter()
                        .enumerate()
                        .map(|(idx, (title, desc))| {
                            let mut embed = CreateEmbed::default()
                                .title(format!("{}. {}", idx + 1, title))
                                .description(desc);

                            if let Some(color) = data.colors.get(idx % data.colors.len()) {
                                embed = embed.color(*color);
                            }

                            embed
                        })
                        .collect::<Vec<_>>(),
                ),

                Component::Text(data) => CreateMessage::default().content(&data.text),
            })
            .collect::<Vec<_>>()
    }
}
