<!--
SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>

SPDX-License-Identifier: AGPL-3.0-only
-->

# Privacy Policy

Last updated: September 27, 2026

This Privacy Policy describes how the instance of Valfisk operated by Ryan Cao ("we") handles your data. Instances operated by third parties are not covered by this policy.

## Data we collect

We automatically collect data for moderation purposes. Messages sent in servers where Valfisk is installed have their content, author ID, and attachments stored for a period of **1 day** before they are deleted; edits and deletes may be logged to a channel that is configured per guild, where they remain under the control of that server.

When moderators warn you with the `/warn` command, the number of warnings you have received in the guild is stored indefinitely until it is reset by a moderator.

Messages that receive enough reactions to be posted to starboards have their message IDs stored for **2 weeks** in order to relate the original message to the message on the starboard.

Messages with expanded links have their message IDs and a hash of their content stored for **5 minutes** in order to update or delete the expansion when the original message is edited or deleted.

Guild-level configurations, such as those set using the `/config` and `/autoreply` commands, are stored indefinitely and associated with the guild ID.

When you set reminders, the content of the reminder and your user ID are stored until the reminder is completed (i.e. sent to you after the specified duration has elapsed).

When you interact with Valfisk's intelligence features, your messages and generated responses to your messages are stored temporarily within a window of **5 minutes** in order to construct a continuous conversational context. Once you consent to the use of these features, your user ID is stored indefinitely in order to remember your consent.

Direct messages sent to Valfisk, as well as commands that fail (including their inputs), may be logged to a channel accessible to Valfisk's operators.

We may collect analytics events and store them indefinitely in order to track usage of and improve Valfisk's commands and features. Only command names and guild IDs are currently associated with analytics data.

## Data we share with third parties

Valfisk protects servers it is installed in with [Google Safe Browsing](https://safebrowsing.google.com/). If a sent link is considered suspicious (by matching with a list of hash prefixes), the hash prefix is sent to Google in order to obtain a full list of URL hashes. **The links that you send are never sent to Google.**

When you send links to source code, issues, pull requests, commits, or comments on [GitHub](https://github.com), [Tangled](https://tangled.org), [Codeberg](https://codeberg.org), [GitLab](https://gitlab.com), or the [Rust](https://play.rust-lang.org) and [Go](https://go.dev/play) playgrounds, the relevant parts of those links (such as repository names, file paths, and identifiers) are sent to their respective services in order to fetch the linked content. Rust playground links are fetched from [GitHub Gist](https://gist.github.com).

When you use the `/lighthouse` command, the URL that is being tested is sent to Google. The usage of APIs provided by Google Cloud is governed by [Google's privacy policy](https://policies.google.com/privacy).

When you use the `/dig` command, your queries are sent to the DNS resolvers that you specify in your command invocations, under their respective privacy policies. See the privacy policies for [Cloudflare's 1.1.1.1](https://developers.cloudflare.com/1.1.1.1/privacy/public-dns-resolver/), [Google Public DNS](https://developers.google.com/speed/public-dns/privacy), [Quad9](https://quad9.net/privacy/policy/), [dns0.eu](https://www.dns0.eu/privacy), [Mullvad](https://mullvad.net/en/help/privacy-policy), [AdGuard DNS](https://adguard-dns.io/en/privacy.html), [OpenDNS](https://www.opendns.com/privacy-policy/), and [Wikimedia DNS](https://meta.wikimedia.org/wiki/Wikimedia_DNS#Privacy_policy).

When you use the `/exchange` command, the data that you input (including currency codes and amounts) is sent to [Frankfurter](https://frankfurter.dev), [Wise](https://wise.com), [Revolut](https://revolut.com), [Visa](https://www.visa.com/), and [Mastercard](https://www.mastercard.com/).

When you use Valfisk's intelligence features (including translation and chat), your query is sent to [Anthropic](https://www.anthropic.com/) and processed according to Anthropic's [privacy policy](https://www.anthropic.com/legal/privacy). Translating a message sends its content to Anthropic, including messages written by others.

**Valfisk never automatically sends information that identifies you to third parties**, except for what you may explicitly include in your inputs.

## Rights to your data

We respect your rights to your personal information as outlined under the [General Data Protection Regulation (GDPR)](https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:02016R0679-20160504), regardless of your residency or citizenship. These rights include:

- Right to be informed
- Right of access
- Right to rectification
- Right to erasure
- Right to restriction of processing
- Right to data portability
- Right to object
- Right not to be subject to a decision based solely on automated processing

If you wish to exercise these rights, feel free to contact us at [valfisk@ryanccn.dev](mailto:valfisk@ryanccn.dev).
