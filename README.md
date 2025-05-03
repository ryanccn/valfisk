<!--
SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>

SPDX-License-Identifier: AGPL-3.0-only
-->

# Valfisk

[![Built with Nix](https://builtwithnix.org/badge.svg)](https://builtwithnix.org)

Valfisk is a next-generation general purpose Discord app, built with [Poise](https://github.com/serenity-rs/poise) and [Serenity](https://github.com/serenity-rs/serenity). It supports being installed to both servers and users, with differing feature sets.

[**Try it out!**](https://discord.com/oauth2/authorize?client_id=1164562106713128990)

## Features

- Expand source code links (supports GitHub, Codeberg, GitLab, the Rust playground, and the Go playground)
- Fetch Lighthouse metrics using Google's [PageSpeed Insights API](https://developers.google.com/speed/docs/insights/v5/about)
- Make DNS queries to a variety of DNS-over-HTTPS resolvers
- Reminders (public by default when installed by servers and private when not)
- Translate messages using Google's [Cloud Translation API](https://cloud.google.com/translate/docs/)
- Show front page posts from [Hacker News](https://news.ycombinator.com/) and [Lobsters](https://lobste.rs/)

### Servers

- [Google Safe Browsing](https://safebrowsing.google.com/) protection (privacy-friendly)
- Moderation commands (e.g. ban, kick, timeout)
- Auditing for message edits and deletions, and member joins and leaves
- Configurable starboard
- Autoreply to keyword triggers (supports regular expressions)
- Roles that rotate to random colors daily
- Apply TOML templates to channels
- Self-timeout command for members
