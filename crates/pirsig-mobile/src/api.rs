//! Pirsig's REST API, on the same origin as the page (see `desktop/README.md`).

use gloo_net::http::{Request, Response};
use serde::{de::DeserializeOwned, Serialize};

use crate::model::Page;

/// What went wrong, in words to show.
pub type Error = String;

/// The words for an error answer: the API's own message when it sent one.
async fn fail(res: Response) -> Error {
    let status = res.status();
    let text = res.text().await.unwrap_or_default();
    serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|v| {
            ["description", "message", "error"]
                .into_iter()
                .find_map(|k| v.get(k).and_then(|m| m.as_str()).map(str::to_string))
        })
        .unwrap_or_else(|| {
            if text.is_empty() {
                format!("{status} {}", res.status_text())
            } else {
                text
            }
        })
}

async fn read<T: DeserializeOwned>(res: Result<Response, gloo_net::Error>) -> Result<T, Error> {
    let res = res.map_err(|e| e.to_string())?;
    if !res.ok() {
        return Err(fail(res).await);
    }
    res.json().await.map_err(|e| e.to_string())
}

/// `GET` a JSON answer.
///
/// # Errors
/// The words to show when the request fails or the API refuses it.
pub async fn get<T: DeserializeOwned>(path: &str) -> Result<T, Error> {
    read(
        Request::get(path)
            .header("Accept", "application/json")
            .send()
            .await,
    )
    .await
}

/// `POST` a JSON body, read the JSON answer.
///
/// # Errors
/// The words to show when the request fails or the API refuses it.
pub async fn post<B: Serialize, T: DeserializeOwned>(path: &str, body: &B) -> Result<T, Error> {
    let req = Request::post(path).json(body).map_err(|e| e.to_string())?;
    read(req.send().await).await
}

/// `PATCH` a JSON body, read the JSON answer.
///
/// # Errors
/// The words to show when the request fails or the API refuses it.
pub async fn patch<B: Serialize, T: DeserializeOwned>(path: &str, body: &B) -> Result<T, Error> {
    let req = Request::patch(path).json(body).map_err(|e| e.to_string())?;
    read(req.send().await).await
}

/// Every item of a paginated list, page by page.
///
/// # Errors
/// The words to show when the request fails or the API refuses it.
pub async fn all<T: DeserializeOwned>(path: &str) -> Result<Vec<T>, Error> {
    let sep = if path.contains('?') { '&' } else { '?' };
    let mut items = Vec::new();
    for page in 1.. {
        let res: Page<T> = get(&format!("{path}{sep}page={page}&page_size=100")).await?;
        items.extend(res.items);
        if page >= res.total_pages {
            break;
        }
    }
    Ok(items)
}
