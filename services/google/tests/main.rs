use std::env;
use std::time::Duration;

use http::StatusCode;
use log::debug;
use log::warn;
use reqsign_core::{Context, Result, Signer};
use reqsign_file_read_tokio::TokioFileRead;
use reqsign_google::{Credential, RequestSigner, StaticCredentialProvider};
use reqsign_http_send_reqwest::ReqwestHttpSend;
use reqwest::Client;

async fn init_signer() -> Option<(Context, Signer<Credential>)> {
    let _ = env_logger::builder().is_test(true).try_init();
    let _ = dotenv::dotenv();

    if env::var("REQSIGN_GOOGLE_TEST").is_err() || env::var("REQSIGN_GOOGLE_TEST").unwrap() != "on"
    {
        return None;
    }

    let credential_content =
        env::var("REQSIGN_GOOGLE_CREDENTIAL").expect("env REQSIGN_GOOGLE_CREDENTIAL must be set");
    let scope = env::var("REQSIGN_GOOGLE_CLOUD_STORAGE_SCOPE")
        .expect("env REQSIGN_GOOGLE_CLOUD_STORAGE_SCOPE must be set");

    let loader = StaticCredentialProvider::from_base64(credential_content)
        .expect("credential must be valid base64")
        .with_scope(&scope);
    let builder = RequestSigner::new("storage").with_scope(&scope);

    let ctx = Context::new(TokioFileRead, ReqwestHttpSend::default());
    let signer = Signer::new(ctx.clone(), loader, builder);
    Some((ctx, signer))
}

async fn init_signer_for_signed_url() -> Option<(Context, Signer<Credential>)> {
    let _ = env_logger::builder().is_test(true).try_init();
    let _ = dotenv::dotenv();

    if env::var("REQSIGN_GOOGLE_TEST").is_err() || env::var("REQSIGN_GOOGLE_TEST").unwrap() != "on"
    {
        return None;
    }

    let credential_content =
        env::var("REQSIGN_GOOGLE_CREDENTIAL").expect("env REQSIGN_GOOGLE_CREDENTIAL must be set");

    // Don't set scope for signed URL generation
    let loader = StaticCredentialProvider::from_base64(credential_content)
        .expect("credential must be valid base64");
    let builder = RequestSigner::new("storage");

    let ctx = Context::new(TokioFileRead, ReqwestHttpSend::default());
    let signer = Signer::new(ctx.clone(), loader, builder);
    Some((ctx, signer))
}

#[tokio::test]
async fn test_get_object() -> Result<()> {
    let Some((_ctx, signer)) = init_signer().await else {
        warn!("REQSIGN_GOOGLE_TEST is not set, skipped");
        return Ok(());
    };

    let url = &env::var("REQSIGN_GOOGLE_CLOUD_STORAGE_URL")
        .expect("env REQSIGN_GOOGLE_CLOUD_STORAGE_URL must set");

    let mut builder = http::Request::builder();
    builder = builder.method(http::Method::GET);
    builder = builder.uri(format!("{}/o/{}", url, "not_exist_file"));
    let req = builder.body("").map_err(|e| {
        reqsign_core::Error::unexpected("failed to build HTTP request").with_source(e)
    })?;

    let (mut parts, body) = req.into_parts();
    signer
        .sign(&mut parts, None)
        .await
        .expect("sign request must success");
    let req = http::Request::from_parts(parts, body);

    debug!("signed request: {req:?}");

    let client = Client::new();
    let resp = client
        .execute(req.try_into().map_err(|e| {
            reqsign_core::Error::unexpected("failed to convert request").with_source(e)
        })?)
        .await
        .expect("request must succeed");

    debug!("got response: {resp:?}");
    assert_eq!(StatusCode::NOT_FOUND, resp.status());
    Ok(())
}

#[tokio::test]
async fn test_list_objects() -> Result<()> {
    let Some((_ctx, signer)) = init_signer().await else {
        warn!("REQSIGN_GOOGLE_TEST is not set, skipped");
        return Ok(());
    };

    let url = &env::var("REQSIGN_GOOGLE_CLOUD_STORAGE_URL")
        .expect("env REQSIGN_GOOGLE_CLOUD_STORAGE_URL must set");

    let mut builder = http::Request::builder();
    builder = builder.method(http::Method::GET);
    builder = builder.uri(format!("{url}/o"));
    let req = builder.body("").map_err(|e| {
        reqsign_core::Error::unexpected("failed to build HTTP request").with_source(e)
    })?;

    let (mut parts, body) = req.into_parts();
    signer
        .sign(&mut parts, None)
        .await
        .expect("sign request must success");
    let req = http::Request::from_parts(parts, body);

    debug!("signed request: {req:?}");

    let client = Client::new();
    let resp = client
        .execute(req.try_into().map_err(|e| {
            reqsign_core::Error::unexpected("failed to convert request").with_source(e)
        })?)
        .await
        .expect("request must succeed");

    debug!("got response: {resp:?}");
    assert_eq!(StatusCode::OK, resp.status());
    Ok(())
}

#[tokio::test]
async fn test_get_object_with_query() -> Result<()> {
    let Some((_ctx, signer)) = init_signer_for_signed_url().await else {
        warn!("REQSIGN_GOOGLE_TEST is not set, skipped");
        return Ok(());
    };

    let url = &env::var("REQSIGN_GOOGLE_CLOUD_STORAGE_URL")
        .expect("env REQSIGN_GOOGLE_CLOUD_STORAGE_URL must set");

    let mut builder = http::Request::builder();
    builder = builder.method(http::Method::GET);
    builder = builder.uri(format!(
        "{}/{}",
        url.replace("storage/v1/b/", ""),
        "not_exist_file"
    ));
    let req = builder.body("").map_err(|e| {
        reqsign_core::Error::unexpected("failed to build HTTP request").with_source(e)
    })?;

    let (mut parts, body) = req.into_parts();
    signer
        .sign(&mut parts, Some(Duration::from_secs(3600)))
        .await
        .expect("sign request must success");
    let req = http::Request::from_parts(parts, body);

    debug!("signed request: {req:?}");

    let client = Client::new();
    let resp = client
        .execute(req.try_into().map_err(|e| {
            reqsign_core::Error::unexpected("failed to convert request").with_source(e)
        })?)
        .await
        .expect("request must succeed");

    let code = resp.status();
    debug!("got response: {resp:?}");
    debug!(
        "got body: {}",
        resp.text()
            .await
            .map_err(
                |e| reqsign_core::Error::unexpected("failed to read response body").with_source(e)
            )?
    );
    assert_eq!(StatusCode::NOT_FOUND, code);
    Ok(())
}
