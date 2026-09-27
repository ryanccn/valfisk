// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude as serenity;

use super::{Expansion, plural};
use crate::utils::truncate;

const MAX_COMPONENTS: usize = 40;
const MAX_CHARACTERS: usize = 4000;
const MIN_BODY: usize = 150;
const NOTE_RESERVE: usize = 48;

fn shorten(text: &str, length: usize, code: bool) -> String {
    if text.chars().count() <= length {
        return text.to_owned();
    }

    let cut = truncate(text, length.saturating_sub(2));

    if code {
        return cut
            .rfind('\n')
            .map_or(cut, |index| &cut[..index])
            .to_owned()
            + "\n…";
    }

    let floor = cut.len() * 4 / 5;
    let end = ["\n\n", ". ", "\n"]
        .into_iter()
        .find_map(|boundary| {
            cut.rfind(boundary)
                .filter(|&index| index >= floor)
                .map(|index| index + usize::from(boundary == ". "))
        })
        .or_else(|| cut.rfind(' '))
        .unwrap_or(cut.len());

    cut[..end].trim_end().to_owned() + "…"
}

pub fn render(
    expansions: Vec<Expansion>,
    mut omitted: usize,
) -> Vec<serenity::CreateComponent<'static>> {
    let total = expansions.len();
    let mut kept = Vec::new();
    let mut needs = Vec::new();
    let mut components = 0;
    let mut characters = NOTE_RESERVE;
    let mut minimum = 0;

    for mut expansion in expansions {
        expansion.body = expansion.body.filter(|body| !body.text.is_empty());

        let heading = expansion
            .heading
            .as_ref()
            .map(|heading| format!("### {heading}"));
        let footer = format!(
            "-# [{}]({}) · {}",
            expansion.source,
            expansion.link,
            serenity::FormattedTimestamp::now()
        );

        let count = 3
            + usize::from(expansion.body.is_some())
            + usize::from(expansion.meta.is_some())
            + 2 * usize::from(expansion.parent.is_some());

        let fixed = heading
            .iter()
            .chain([&footer])
            .chain(&expansion.parent)
            .chain(&expansion.meta)
            .map(|text| text.chars().count())
            .sum::<usize>()
            + expansion
                .body
                .as_ref()
                .and_then(|body| body.language.as_ref())
                .map_or(0, |language| language.len() + 8);

        let need = expansion.body.as_ref().map_or(0, |body| {
            let max = if body.language.is_some() { 2048 } else { 1500 };
            body.text.chars().count().min(max)
        });

        if components + count > MAX_COMPONENTS
            || characters + minimum + fixed + need.min(MIN_BODY) > MAX_CHARACTERS
        {
            break;
        }

        components += count;
        characters += fixed;
        minimum += need.min(MIN_BODY);
        kept.push((expansion, heading, footer));
        needs.push(need);
    }

    omitted += total - kept.len();

    // Bodies share the remaining characters evenly, with shares unused by shorter bodies going to longer ones.
    let mut available = MAX_CHARACTERS - characters;
    let mut order = (0..needs.len()).collect::<Vec<_>>();
    order.sort_unstable_by_key(|&index| needs[index]);

    let mut allotments = vec![0; needs.len()];
    for (n, &index) in order.iter().enumerate() {
        allotments[index] = needs[index].min(available / (order.len() - n));
        available -= allotments[index];
    }

    let mut rendered = Vec::new();

    for ((expansion, heading, footer), allotment) in kept.into_iter().zip(allotments) {
        let body = expansion.body.map(|body| match body.language {
            Some(language) => format!(
                "```{language}\n{}\n```",
                shorten(&body.text, allotment, true)
            ),
            None => shorten(&body.text, allotment, false),
        });

        let inner = expansion
            .meta
            .into_iter()
            .chain(body)
            .map(serenity::CreateTextDisplay::new);

        // Expansions with headings are rendered as is, while those without are placed in a container.
        if let Some(heading) = heading {
            rendered.push(serenity::CreateComponent::TextDisplay(
                serenity::CreateTextDisplay::new(heading),
            ));
            rendered.extend(inner.map(serenity::CreateComponent::TextDisplay));
        } else {
            let mut components = Vec::new();

            if let Some(parent) = expansion.parent {
                components.push(serenity::CreateContainerComponent::TextDisplay(
                    serenity::CreateTextDisplay::new(parent),
                ));
                components.push(serenity::CreateContainerComponent::Separator(
                    serenity::CreateSeparator::new()
                        .divider(false)
                        .spacing(serenity::SeparatorSpacingSize::Large),
                ));
            }

            components.extend(inner.map(serenity::CreateContainerComponent::TextDisplay));

            let mut container = serenity::CreateContainer::new(components);

            if let Some(accent) = expansion.accent {
                container = container.accent_color(accent);
            }

            rendered.push(serenity::CreateComponent::Container(container));
        }

        rendered.push(serenity::CreateComponent::TextDisplay(
            serenity::CreateTextDisplay::new(footer),
        ));
        rendered.push(serenity::CreateComponent::Separator(
            serenity::CreateSeparator::new()
                .divider(true)
                .spacing(serenity::SeparatorSpacingSize::Large),
        ));
    }

    rendered.pop();

    if omitted > 0 && !rendered.is_empty() {
        rendered.push(serenity::CreateComponent::TextDisplay(
            serenity::CreateTextDisplay::new(format!(
                "-# {} not expanded",
                plural(omitted as u64, "more link")
            )),
        ));
    }

    rendered
}

#[cfg(test)]
mod tests {
    use super::*;

    fn measure(components: &[serenity::CreateComponent<'static>]) -> (usize, usize, Vec<String>) {
        fn walk(value: &serde_json::Value, out: &mut (usize, usize, Vec<String>)) {
            match value {
                serde_json::Value::Object(object) => {
                    if object.contains_key("type") {
                        out.0 += 1;
                    }
                    if let Some(serde_json::Value::String(content)) = object.get("content") {
                        out.1 += content.chars().count();
                        out.2.push(content.clone());
                    }
                    object.values().for_each(|value| walk(value, out));
                }
                serde_json::Value::Array(array) => array.iter().for_each(|value| walk(value, out)),
                _ => {}
            }
        }

        let mut out = (0, 0, Vec::new());
        walk(&serde_json::to_value(components).unwrap(), &mut out);
        out
    }

    fn snippet(text: &str) -> Expansion {
        Expansion::snippet(
            "a/b file.rs L1".to_owned(),
            "rs",
            text,
            "Test",
            "https://example.com",
        )
    }

    #[test]
    fn render_fits_large_bodies() {
        let code = "let x = 1;\n".repeat(1000);
        let (components, characters, contents) =
            measure(&render((0..8).map(|_| snippet(&code)).collect(), 0));

        assert!(components <= MAX_COMPONENTS);
        assert!(characters <= MAX_CHARACTERS);
        assert_eq!(contents.iter().filter(|c| c.starts_with("### ")).count(), 8);
        assert!(contents.iter().all(|c| !c.contains("not expanded")));
    }
}
