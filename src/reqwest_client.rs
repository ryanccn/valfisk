use once_cell::sync::Lazy;
use reqwest::header;

pub static HTTP: Lazy<reqwest::Client> = Lazy::new(|| {
    let mut builder = reqwest::ClientBuilder::new();

    let mut headers = header::HeaderMap::new();
    headers.insert(
        "user-agent",
        header::HeaderValue::from_static("valfisk/0.1.0"),
    );
    builder = builder.default_headers(headers);

    builder.build().unwrap()
});
