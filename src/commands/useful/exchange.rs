// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::{CreateReply, serenity_prelude as serenity};

use eyre::{Result, bail};
use thousands::Separable as _;
use tokio::process::Command;

use crate::{Context, http::HTTP};

// const FIREFOX_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:150.0) Gecko/20100101 Firefox/150.0";

fn minor_unit_factor(currency: &str) -> f64 {
    match currency {
        "BIF" | "CLP" | "DJF" | "GNF" | "ISK" | "JPY" | "KMF" | "KRW" | "MGA" | "PYG" | "RWF"
        | "UGX" | "VND" | "VUV" | "XAF" | "XOF" | "XPF" => 1.0,
        _ => 100.0,
    }
}

fn format_number(n: f64) -> String {
    ((n * 100.).round() / 100.).separate_with_commas()
}

fn format_amount(amount: f64, currency: &str) -> String {
    format!("{} {currency}", format_number(amount))
}

#[derive(serde::Deserialize, Debug)]
struct FrankfurterRate {
    rate: f64,
}

#[derive(serde::Deserialize, Debug)]
struct WiseComparison {
    providers: Vec<WiseProvider>,
}

#[derive(serde::Deserialize, Debug)]
struct WiseProvider {
    alias: String,
    quotes: Vec<WiseQuote>,
}

#[derive(serde::Deserialize, Debug)]
struct WiseQuote {
    fee: f64,
    #[serde(rename = "receivedAmount")]
    received_amount: f64,
}

#[derive(serde::Deserialize, Debug)]
struct RevolutQuote {
    recipient: RevolutAmount,
}

#[derive(serde::Deserialize, Debug)]
struct RevolutAmount {
    amount: i64,
}

#[derive(Debug)]
struct WiseInfo {
    fee: f64,
    received_amount: f64,
}

#[derive(Debug)]
struct RevolutInfo {
    received_amount: f64,
}

#[derive(serde::Deserialize, Debug)]
struct VisaRate {
    #[serde(rename = "convertedAmount")]
    converted_amount: String,
}

#[derive(serde::Deserialize, Debug)]
struct MastercardResponse {
    data: MastercardData,
}

#[derive(serde::Deserialize, Debug)]
struct MastercardData {
    #[serde(rename = "crdhldBillAmt")]
    cardholder_bill_amount: String,
}

async fn fetch_frankfurter(from: &str, to: &str) -> Result<FrankfurterRate> {
    let rate = HTTP
        .get(format!("https://api.frankfurter.dev/v2/rate/{from}/{to}"))
        .send()
        .await?
        .error_for_status()?
        .json::<FrankfurterRate>()
        .await?;
    Ok(rate)
}

async fn fetch_wise(from: &str, to: &str, amount: f64) -> Result<Option<WiseInfo>> {
    let resp = HTTP
        .get("https://wise.com/gateway/v4/comparisons")
        .query(&[
            ("sourceCurrency", from),
            ("targetCurrency", to),
            ("sendAmount", &amount.to_string()),
            ("sourceCountry", "US"),
            ("filter", "POPULAR"),
            ("includeWise", "true"),
            ("numberOfProviders", "3"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json::<WiseComparison>()
        .await?;

    let result = resp
        .providers
        .into_iter()
        .find(|p| p.alias == "wise")
        .and_then(|p| {
            p.quotes.into_iter().next().map(|q| WiseInfo {
                fee: q.fee,
                received_amount: q.received_amount,
            })
        });

    Ok(result)
}

#[expect(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
async fn fetch_revolut(from: &str, to: &str, amount: f64) -> Result<RevolutInfo> {
    let amount_minor = (amount * minor_unit_factor(from)).round() as i64;

    let resp = HTTP
        .get("https://www.revolut.com/api/exchange/quote")
        .query(&[
            ("amount", amount_minor.to_string().as_str()),
            ("country", "US"),
            ("fromCurrency", from),
            ("isRecipientAmount", "false"),
            ("toCurrency", to),
        ])
        .header("Accept-Language", "en")
        .send()
        .await?
        .error_for_status()?
        .json::<RevolutQuote>()
        .await?;

    Ok(RevolutInfo {
        received_amount: resp.recipient.amount as f64 / minor_unit_factor(to),
    })
}

async fn fetch_visa(from: &str, to: &str, amount: f64) -> Result<f64> {
    let today = chrono::Utc::now().format("%m/%d/%Y").to_string();

    let args = vec![
        "--compressed".to_owned(),
        "--impersonate".to_owned(),
        "chrome145".to_owned(),
        "--url-query".to_owned(),
        format!("amount={amount}"),
        "--url-query".to_owned(),
        "fee=0".to_owned(),
        "--url-query".to_owned(),
        format!("utcConvertedDate={today}"),
        "--url-query".to_owned(),
        format!("exchangedate={today}"),
        "--url-query".to_owned(),
        format!("fromCurr={to}"),
        "--url-query".to_owned(),
        format!("toCurr={from}"),
        "https://usa.visa.com/cmsapi/fx/rates".to_owned(),
    ];

    let output = Command::new("curl-impersonate")
        .args(&args)
        .output()
        .await?;

    if !output.status.success() {
        bail!("fetching Visa exchange rate failed");
    }

    let data = serde_json::from_slice::<VisaRate>(&output.stdout)?;
    Ok(data.converted_amount.replace(',', "").parse::<f64>()?)
}

async fn fetch_mastercard(from: &str, to: &str, amount: f64) -> Result<f64> {
    let args = vec![
        "--compressed".to_owned(),
        "--impersonate".to_owned(),
        "chrome145".to_owned(),
        "--url-query".to_owned(),
        "exchange_date=0000-00-00".to_owned(),
        "--url-query".to_owned(),
        format!("transaction_currency={from}"),
        "--url-query".to_owned(),
        format!("cardholder_billing_currency={to}"),
        "--url-query".to_owned(),
        "bank_fee=0".to_owned(),
        "--url-query".to_owned(),
        format!("transaction_amount={amount}"),
        "https://www.mastercard.com/marketingservices/public/mccom-services/currency-conversions/conversion-rates".to_owned()
    ];

    let output = Command::new("curl-impersonate")
        .args(&args)
        .output()
        .await?;

    if !output.status.success() {
        bail!("fetching Mastercard exchange rate failed");
    }

    let data = serde_json::from_slice::<MastercardResponse>(&output.stdout)?;
    Ok(data.data.cardholder_bill_amount.parse::<f64>()?)
}

/// Convert between currencies
#[tracing::instrument(skip(ctx), fields(ctx.channel = ctx.channel_id().get(), ctx.author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    install_context = "Guild | User",
    interaction_context = "Guild | BotDm | PrivateChannel"
)]
pub async fn exchange(
    ctx: Context<'_>,
    #[description = "Source currency code (e.g. USD)"] from: String,
    #[description = "Target currency code (e.g. EUR)"] to: String,

    #[description = "Amount to convert (default: 1)"]
    #[min = 0]
    amount: Option<f64>,
) -> Result<()> {
    let from = from.trim().to_uppercase();
    let to = to.trim().to_uppercase();
    let amount = amount.unwrap_or(1.0);

    if !amount.is_finite() || amount <= 0.0 {
        ctx.send(
            CreateReply::default()
                .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                .components(&[serenity::CreateComponent::Container(
                    serenity::CreateContainer::new(vec![
                        serenity::CreateContainerComponent::TextDisplay(
                            serenity::CreateTextDisplay::new(
                                "### Invalid amount\nThe amount must be a positive number.",
                            ),
                        ),
                    ])
                    .accent_color(0xffd43b),
                )]),
        )
        .await?;
        return Ok(());
    }

    if from == to {
        ctx.send(
            CreateReply::default()
                .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                .components(&[serenity::CreateComponent::Container(
                    serenity::CreateContainer::new(vec![
                        serenity::CreateContainerComponent::TextDisplay(
                            serenity::CreateTextDisplay::new(
                                "### Invalid currencies\nThe source and target currencies must be different.",
                            ),
                        ),
                    ])
                    .accent_color(0xffd43b),
                )]),
        )
        .await?;
        return Ok(());
    }

    ctx.defer().await?;

    let available_emojis = ctx.http().get_application_emojis().await?;
    let emoji_prefix = |name: &str| {
        available_emojis
            .iter()
            .find(|e| e.name == name)
            .map_or_else(String::new, |e| format!("{e} "))
    };

    let Ok(frankfurter_result) = fetch_frankfurter(&from, &to).await else {
        ctx.send(
            CreateReply::default()
                .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                .components(&[serenity::CreateComponent::Container(
                    serenity::CreateContainer::new(vec![
                        serenity::CreateContainerComponent::TextDisplay(
                            serenity::CreateTextDisplay::new(
                                "### Exchange rate unavailable\nThe specified currency codes may be invalid or unsupported.",
                            ),
                        ),
                    ])
                    .accent_color(0xff6b6b),
                )]),
        )
        .await?;
        return Ok(());
    };

    let (wise_result, revolut_result, visa_result, mastercard_result) = tokio::join!(
        fetch_wise(&from, &to, amount),
        fetch_revolut(&from, &to, amount),
        fetch_visa(&from, &to, amount),
        fetch_mastercard(&from, &to, amount),
    );

    let converted = amount * frankfurter_result.rate;

    let mut components = vec![
        serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(format!(
            "### **{}** = **{}**\n1 {} = {} {}",
            format_amount(amount, &from),
            format_amount(converted, &to),
            from,
            format_number(frankfurter_result.rate),
            to,
        ))),
        serenity::CreateContainerComponent::Separator(
            serenity::CreateSeparator::new().divider(false),
        ),
    ];

    let mut source_links: Vec<&str> = vec!["[Frankfurter](https://frankfurter.dev)"];

    if let Ok(Some(wise)) = &wise_result {
        components.push(serenity::CreateContainerComponent::TextDisplay(
            serenity::CreateTextDisplay::new(format!(
                "{}**Wise**\n{}\n-# Fee: {:.2} {}",
                emoji_prefix("wise"),
                format_amount(wise.received_amount, &to),
                wise.fee,
                from,
            )),
        ));
        source_links.push("[Wise](https://wise.com)");
    }

    if let Ok(revolut) = &revolut_result {
        components.push(serenity::CreateContainerComponent::TextDisplay(
            serenity::CreateTextDisplay::new(format!(
                "{}**Revolut**\n{}",
                emoji_prefix("revolut"),
                format_amount(revolut.received_amount, &to),
            )),
        ));
        source_links.push("[Revolut](https://revolut.com)");
    }

    if let Ok(visa) = &visa_result {
        components.push(serenity::CreateContainerComponent::TextDisplay(
            serenity::CreateTextDisplay::new(format!(
                "{}**Visa**\n{}",
                emoji_prefix("visa"),
                format_amount(*visa, &to)
            )),
        ));
        source_links.push("[Visa](https://www.visa.com/)");
    }

    if let Ok(mastercard) = &mastercard_result {
        components.push(serenity::CreateContainerComponent::TextDisplay(
            serenity::CreateTextDisplay::new(format!(
                "{}**Mastercard**\n{}",
                emoji_prefix("mastercard"),
                format_amount(*mastercard, &to),
            )),
        ));
        source_links.push("[Mastercard](https://www.mastercard.com/)");
    }

    components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new().divider(false),
    ));

    components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(format!(
            "-# {} \u{00b7} {}",
            serenity::FormattedTimestamp::now(),
            source_links.join(" \u{00b7} "),
        )),
    ));

    ctx.send(
        CreateReply::default()
            .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
            .allowed_mentions(serenity::CreateAllowedMentions::new())
            .components(&[serenity::CreateComponent::Container(
                serenity::CreateContainer::new(components).accent_color(0x4ade80),
            )]),
    )
    .await?;

    Ok(())
}
