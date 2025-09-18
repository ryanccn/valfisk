// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;

use indexmap::IndexMap;
use poise::serenity_prelude::{
    CreateComponent, CreateContainer, CreateMessage, CreateTextDisplay, MessageFlags,
};
use serde::Deserialize;

// const fn default_to_false() -> bool {
//     false
// }

#[derive(Deserialize, Debug, Clone)]
pub struct Template {
    #[serde(default)]
    pub components: Vec<Component>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Component {
    Embed(EmbedComponent),
    Rules(RulesComponent),
    Links(LinksComponent),
    Text(TextComponent),
}

#[derive(Deserialize, Debug, Clone)]
pub struct EmbedComponent {
    #[serde(default)]
    embeds: Vec<EmbedComponentEmbed>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EmbedComponentEmbed {
    title: Option<String>,
    description: Option<String>,
    color: Option<u32>,
    fields: Option<Vec<EmbedComponentEmbedField>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EmbedComponentEmbedField {
    name: String,
    value: String,
    // #[serde(default = "default_to_false")]
    // inline: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RulesComponent {
    #[serde(default)]
    rules: IndexMap<String, String>,
    #[serde(default)]
    colors: Vec<u32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LinksComponent {
    title: String,
    color: Option<u32>,
    links: IndexMap<String, String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TextComponent {
    text: String,
}

impl Template {
    pub fn parse(source: &str) -> Result<Self> {
        Ok(toml::from_str(source)?)
    }
}

impl Template {
    pub fn to_messages(&self) -> Vec<CreateMessage<'_>> {
        self.components
            .iter()
            .enumerate()
            .map(|(idx, component)| match component {
                Component::Embed(data) => CreateMessage::default()
                    .flags(MessageFlags::IS_COMPONENTS_V2)
                    .components(
                        data.embeds
                            .iter()
                            .map(|data| {
                                let mut container = CreateContainer::new(&[]);

                                if let Some(title) = &data.title {
                                    container =
                                        container.add_component(CreateComponent::TextDisplay(
                                            CreateTextDisplay::new(format!(
                                                "{} {title}",
                                                if idx == 0 { "##" } else { "###" }
                                            )),
                                        ));
                                }

                                if let Some(description) = &data.description {
                                    container =
                                        container.add_component(CreateComponent::TextDisplay(
                                            CreateTextDisplay::new(description),
                                        ));
                                }

                                if let Some(fields) = &data.fields {
                                    for field in fields {
                                        container = container.add_component(
                                            CreateComponent::TextDisplay(CreateTextDisplay::new(
                                                format!("**{}**\n{}", field.name, field.value),
                                            )),
                                        );
                                    }
                                }

                                if let Some(color) = &data.color {
                                    container = container.accent_color(*color);
                                }

                                CreateComponent::Container(container)
                            })
                            .collect::<Vec<_>>(),
                    ),

                Component::Links(data) => CreateMessage::default()
                    .flags(MessageFlags::IS_COMPONENTS_V2)
                    .components({
                        let mut container =
                            CreateContainer::new(vec![CreateComponent::TextDisplay(
                                CreateTextDisplay::new(format!(
                                    "### {}\n{}",
                                    data.title,
                                    data.links
                                        .iter()
                                        .map(|(title, href)| format!("» [{title}]({href})"))
                                        .collect::<Vec<_>>()
                                        .join("\n")
                                )),
                            )]);

                        if let Some(color) = data.color {
                            container = container.accent_color(color);
                        }

                        vec![CreateComponent::Container(container)]
                    }),

                Component::Rules(data) => CreateMessage::default()
                    .flags(MessageFlags::IS_COMPONENTS_V2)
                    .components(
                        data.rules
                            .iter()
                            .enumerate()
                            .map(|(idx, (title, desc))| {
                                let mut container =
                                    CreateContainer::new(vec![CreateComponent::TextDisplay(
                                        CreateTextDisplay::new(format!(
                                            "### {}\u{200B}. {title}\n{desc}",
                                            idx + 1,
                                        )),
                                    )]);

                                if let Some(color) = data.colors.get(idx % data.colors.len()) {
                                    container = container.accent_color(*color);
                                }

                                CreateComponent::Container(container)
                            })
                            .collect::<Vec<_>>(),
                    ),

                Component::Text(data) => CreateMessage::default().content(&data.text),
            })
            .collect::<Vec<_>>()
    }
}
