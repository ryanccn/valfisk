<!--
SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>

SPDX-License-Identifier: AGPL-3.0-only
-->

# Privacy

Last updated: Jan 15, 2026

## Data we collect

We automatically collect data for moderation purposes. All messages sent have their content, author ID, and attachments stored for a period of **1 day** before they are deleted; edits and deletes are logged to a channel that is configured per-guild. Messages that receive enough reactions to be posted to starboards will have their message IDs stored for **30 days** in order to relate the original message to the message on the starboard.

Guild-level configurations, such as those set using the `/config` and `/autoreply` commands, are stored indefinitely and associated with the guild ID.

When users set reminders, the content of the reminder and the user's ID are stored until the reminder is completed (i.e. sent to the user after the specified duration has elapsed).

When you interact with Valfisk's intelligence features, your messages and generated responses to your messages will be stored temporarily within a window of **5 minutes** in order to construct a continuous conversational context.

## Data we share with third parties

Valfisk protects servers it is installed in with [Google Safe Browsing](https://safebrowsing.google.com/). If a sent link is considered suspicious (by matching with a list of hash prefixes), the hash prefix will be sent to Google in order to obtain a full list of URL hashes. **The links that you send are never sent to Google.**

When you use the `/lighthouse` command, the URL that is being tested will be sent to Google. The usage of APIs provided by Google Cloud are governed by [Google's privacy policy](https://policies.google.com/privacy).

When you use the `/dig` command, your queries will be sent to the DNS resolvers that you specify in your command invocations, under their respective privacy policies. See the privacy policies for [Cloudflare's 1.1.1.1](https://developers.cloudflare.com/1.1.1.1/privacy/public-dns-resolver/), [Google Public DNS](https://developers.google.com/speed/public-dns/privacy), [Quad9](https://quad9.net/privacy/policy/), [dns0.eu](https://www.dns0.eu/privacy), [Mullvad](https://mullvad.net/en/help/privacy-policy), [AdGuard DNS](https://adguard-dns.io/en/privacy.html), [OpenDNS](https://www.opendns.com/privacy-policy/), and [Wikimedia DNS](https://meta.wikimedia.org/wiki/Wikimedia_DNS#Privacy_policy).

When you use Valfisk's intelligence features (including translation and chat), your query will be sent to [OpenRouter](https://openrouter.ai/) and [Anthropic](https://www.anthropic.com/), according to [OpenRouter's](https://openrouter.ai/privacy) and [Anthropic's](https://www.anthropic.com/legal/privacy) privacy policies.

## Rights to your data

We respect your rights to your personal information as outlined under [General Data Protection Regulation](https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:02016R0679-20160504), regardless of your residency or citizenship. These rights include:

- Right to be informed
- Right of access
- Right to rectification
- Right to restriction of processing
- Right to data portability
- Right to object
- Right not be subject to a decision based solely on automated processing

If you wish to exercise these rights, feel free to [contact us](mailto:hello@ryanccn.dev).
