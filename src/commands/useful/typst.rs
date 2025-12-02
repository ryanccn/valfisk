// SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

// Implementation partially inspired by https://github.com/mattfbacon/typst-bot

use eyre::{Result, eyre};
use std::time::Duration;
use tokio::{task, time::timeout};

use poise::{CreateReply, serenity_prelude as serenity};

use typst::{
    Library, LibraryExt,
    diag::{FileError, FileResult},
    foundations::{Bytes, Datetime},
    layout::{Abs, PagedDocument},
    syntax::{FileId, Source, VirtualPath},
    text::{Font, FontBook},
    utils::LazyHash,
};

use crate::Context;

#[derive(Debug, Clone)]
struct ValfiskTypstWorld {
    source: Source,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    library: LazyHash<Library>,
}

impl ValfiskTypstWorld {
    fn new(source: &str) -> Self {
        let source = Source::new(
            FileId::new_fake(VirtualPath::new("source")),
            source.to_owned(),
        );

        let fonts = typst_assets::fonts()
            .flat_map(|f| Font::iter(Bytes::new(f)))
            .collect::<Vec<_>>();

        Self {
            source,
            book: LazyHash::new(FontBook::from_fonts(&fonts)),
            fonts,
            library: LazyHash::new(Library::default()),
        }
    }
}

impl typst::World for ValfiskTypstWorld {
    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn file(&self, _id: FileId) -> FileResult<Bytes> {
        Err(FileError::AccessDenied)
    }

    fn font(&self, index: usize) -> Option<typst::text::Font> {
        self.fonts.get(index).cloned()
    }

    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn main(&self) -> FileId {
        self.source.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            Err(FileError::AccessDenied)
        }
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        Datetime::from_ymd_hms(1970, 1, 1, 0, 0, 0)
    }
}

/// Render a Typst document
#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    install_context = "Guild | User",
    interaction_context = "Guild | BotDm | PrivateChannel"
)]
pub async fn typst(
    ctx: Context<'_>,
    #[description = "The source of the Typst document"] source: String,
) -> Result<()> {
    ctx.defer().await?;

    let source = "#set page(width: auto, height: auto, margin: 10pt)\n".to_owned() + &source;

    match timeout(
        Duration::from_secs(5),
        task::spawn_blocking(move || {
            let world = ValfiskTypstWorld::new(&source);

            let compiled = typst::compile::<PagedDocument>(&world)
                .output
                .map_err(|err| eyre!("internal Typst error: {err:?}"))?;

            let png = typst_render::render_merged(&compiled, 4., Abs::zero(), None).encode_png()?;
            let optimized =
                oxipng::optimize_from_memory(&png, &oxipng::Options::max_compression())?;

            eyre::Ok(optimized)
        }),
    )
    .await
    {
        Ok(result) => {
            ctx.send(
                CreateReply::default()
                    .attachment(serenity::CreateAttachment::bytes(result??, "typst.png")),
            )
            .await?;
        }
        Err(_) => {
            ctx.say("render task timed out").await?;
        }
    }

    Ok(())
}
