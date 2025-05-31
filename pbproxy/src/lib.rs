use spin_sdk::http::{IntoResponse, Method, Params, Request, Response, Router};
use spin_sdk::http_component;

const LESMA_BASE_URL: &str = "https://lesma.eu";

async fn handle_get(_: Request, params: Params) -> anyhow::Result<Response> {
    let id = params.get("id").unwrap_or_default();
    let url = format!("{}/{}?raw", LESMA_BASE_URL, id);
    let request = Request::builder().method(Method::Get).uri(url).build();

    let resp: Response = spin_sdk::http::send(request).await?;
    let body_bytes = resp.body();
    let body_str = String::from_utf8(body_bytes.to_vec())?;
    let status = *resp.status();

    if status == 200 {
        Ok(Response::new(status, body_str.as_str()))
    } else {
        Ok(Response::new(status, "Error from lesma.eu".to_string()))
    }
}

async fn handle_post(req: Request, _: Params) -> anyhow::Result<Response> {
    let body_bytes = req.body();
    let body_str = String::from_utf8(body_bytes.to_vec())?;
    let payload = urlencoding::encode(body_str.as_str());
    let body = format!("lesma={}", payload);
    let url = format!("{}?expire=24", LESMA_BASE_URL);
    let request = Request::builder()
        .method(Method::Post)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept", "text/plain")
        // lesma.eu seems to look at this instead of the Accept header
        .header("User-Agent", "curl/7.68.0")
        .body(body)
        .uri(url)
        .build();

    let resp: Response = spin_sdk::http::send(request).await?;
    let body_bytes = resp.body();
    let body_str = String::from_utf8(body_bytes.to_vec())?;
    let status = *resp.status();

    if status == 303 {
        let url = url::Url::parse(body_str.as_str())?;
        let id = url
            .path()
            .strip_prefix('/')
            .ok_or_else(|| anyhow::anyhow!("Failed to parse URL returned from lesma.eu"))?;

        Ok(Response::new(status, id))
    } else {
        Ok(Response::new(status, "Error from lesma.eu".to_string()))
    }
}

#[http_component]
fn handle_weather_data_aggregator_pbproxy(req: Request) -> anyhow::Result<impl IntoResponse> {
    let mut router = Router::default();

    router.get_async("/pbproxy/:id", handle_get);
    router.post_async("/pbproxy", handle_post);

    Ok(router.handle(req))
}
