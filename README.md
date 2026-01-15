<!--
SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>

SPDX-License-Identifier: AGPL-3.0-only
-->

# Valfisk

[![Built with Nix](https://builtwithnix.org/badge.svg)](https://builtwithnix.org)

Valfisk is a next-generation general purpose Discord app, built with [Poise](https://github.com/serenity-rs/poise) and [Serenity](https://github.com/serenity-rs/serenity). It can be installed by both servers and users, with differing sets of features for each.

[**Try it out!**](https://discord.com/oauth2/authorize?client_id=1164562106713128990)

## Features

- Expand source code links from GitHub, Tangled, Tangled strings, Codeberg, GitLab, and the Rust and Go playgrounds
- Fetch [Lighthouse](https://developer.chrome.com/docs/lighthouse) metrics for websites
- Make DNS queries to a variety of DNS-over-HTTPS resolvers
- Reminders (public by default when installed in servers, private when not)
- Translate messages using a context menu command
- Show front page posts from [Hacker News](https://news.ycombinator.com/) and [Lobsters](https://lobste.rs/)
- Retrieve public information about a Discord user
- Render [Typst](https://typst.app/) documents into raster images
- Show comprehensive data on Unicode character(s) from the [Unicode Character Database](https://www.unicode.org/ucd/)

### Server-only

- [Google Safe Browsing](https://safebrowsing.google.com/) protection (privacy-friendly)
- Moderation commands (e.g. ban, kick, timeout)
- Auditing for message edits and deletions, and member joins and leaves
- Rotate logs channels by recreating them
- Configurable starboard
- Automatically reply to keyword triggers (supports regular expressions)
- Roles that rotate to random colors daily
- Apply TOML templates to channels
- Self-timeout command for members

## Self hosting

Valfisk's [Docker image](https://github.com/ryanccn/valfisk/pkgs/container/valfisk) is available at `ghcr.io/ryanccn/valfisk:latest`; it supports both `linux/amd64` and `linux/arm64`. A Nix package, from which the Docker image is built from, is also available in the repository's Nix flake.

Valfisk reads its configuration from environment variables:

- `DISCORD_TOKEN` is the only required environment variable. The registered Discord application should have the "Guild Members" and "Message Content" privileged intents enabled.
- `REDIS_URL` is a URL to a Redis or Redis-compatible server; it is optional but highly recommended, since some features will not work well or at all without it.
- `ADMIN_GUILD_ID` is a guild in which commands to manage Valfisk itself will be registered. `OWNERS` is a comma-separated list of user IDs that are allowed to run these commands; by default it is inferred from the Discord application's metadata.
- `ERROR_LOGS_CHANNEL` is a channel where internal errors from Valfisk will be logged. `DM_LOGS_CHANNEL` is one where direct messages sent to Valfisk will be logged.
- `PAGESPEED_API_KEY` and `SAFE_BROWSING_API_KEY` are [Google Cloud API keys](https://cloud.google.com/api-keys/docs/overview) for accessing the APIs required for certain features. (They can be set to the same key.)
- `HOST` and `PORT` form the address that the API server listens to. It defaults to `0.0.0.0:8080`.

## Privacy

See `PRIVACY.md`.
