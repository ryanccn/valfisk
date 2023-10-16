use once_cell::sync::Lazy;
use reqwest::header;

pub static HTTP: Lazy<reqwest::Client> = Lazy::new(|| {
    let mut builder = reqwest::ClientBuilder::new();

    let mut headers = header::HeaderMap::new();

    let user_agent = format!("valfisk/{}", env!("CARGO_PKG_VERSION"));
    headers.insert(
        "user-agent",
        header::HeaderValue::from_str(&user_agent).unwrap(),
    );
    builder = builder.default_headers(headers);

    builder.build().unwrap()
});
