use color_eyre::eyre::Result;

use indexmap::IndexMap;
use poise::serenity_prelude::{CreateEmbed, CreateMessage};
use serde::{Deserialize, Serialize};

fn default_to_false() -> bool {
    false
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Config {
    #[serde(default)]
    pub components: Vec<Component>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Component {
    Embed(EmbedComponent),
    Rules(RulesComponent),
    Links(LinksComponent),
    Text(TextComponent),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EmbedComponent {
    #[serde(default)]
    embeds: Vec<EmbedComponentEmbed>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EmbedComponentEmbed {
    title: Option<String>,
    description: Option<String>,
    color: Option<u64>,
    fields: Option<Vec<EmbedComponentEmbedField>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EmbedComponentEmbedField {
    name: String,
    value: String,
    #[serde(default = "default_to_false")]
    inline: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct RulesComponent {
    #[serde(default)]
    rules: IndexMap<String, String>,
    #[serde(default)]
    colors: Vec<u64>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct LinksComponent {
    title: String,
    color: Option<u64>,
    links: IndexMap<String, String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TextComponent {
    text: String,
}

impl Config {
    pub fn parse(source: &str) -> Result<Self> {
        Ok(toml::from_str(source)?)
    }
}

impl Config {
    pub fn to_messages(&self) -> Vec<CreateMessage> {
        self.components
            .iter()
            .map(|component| match component {
                Component::Embed(data) => {
                    let mut message = CreateMessage::default();

                    for embed_data in &data.embeds {
                        let mut embed = CreateEmbed::default();

                        if let Some(title) = &embed_data.title {
                            embed = embed.title(title);
                        }
                        if let Some(description) = &embed_data.description {
                            embed = embed.description(description);
                        }
                        if let Some(color) = &embed_data.color {
                            embed = embed.color(*color);
                        }
                        if let Some(fields) = &embed_data.fields {
                            embed =
                                embed.fields(fields.iter().map(|f| (&f.name, &f.value, f.inline)));
                        }

                        message = message.embed(embed);
                    }

                    message
                }

                Component::Links(data) => {
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

                    CreateMessage::default().embed(embed)
                }

                Component::Rules(data) => {
                    let mut message = CreateMessage::default();

                    for (idx, (title, desc)) in data.rules.iter().enumerate() {
                        let mut embed = CreateEmbed::default()
                            .title(format!("{}. {}", idx + 1, title))
                            .description(desc);

                        if let Some(color) = data.colors.get(idx % data.colors.len()) {
                            embed = embed.color(*color);
                        }

                        message = message.add_embed(embed);
                    }

                    message
                }

                Component::Text(data) => CreateMessage::default().content(&data.text),
            })
            .collect::<Vec<_>>()
    }
}
