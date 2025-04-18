use std::env;
use std::str::FromStr;
use std::time::Duration;

use anyhow::Result;
use http::Request;
use http::StatusCode;
use log::debug;
use log::warn;
use percent_encoding::utf8_percent_encode;
use percent_encoding::NON_ALPHANUMERIC;
use reqsign_azure_storage::Config;
use reqsign_azure_storage::Loader;
use reqsign_azure_storage::Signer;
use reqwest::Client;

fn init_signer() -> Option<(Loader, Signer)> {
    let _ = env_logger::builder().is_test(true).try_init();
    let _ = dotenv::dotenv();

    if env::var("REQSIGN_AZURE_STORAGE_TEST").is_err()
        || env::var("REQSIGN_AZURE_STORAGE_TEST").unwrap() != "on"
    {
        return None;
    }

    let config = Config {
        account_name: Some(
            env::var("REQSIGN_AZURE_STORAGE_ACCOUNT_NAME")
                .expect("env REQSIGN_AZURE_STORAGE_ACCOUNT_NAME must set"),
        ),
        account_key: Some(
            env::var("REQSIGN_AZURE_STORAGE_ACCOUNT_KEY")
                .expect("env REQSIGN_AZURE_STORAGE_ACCOUNT_KEY must set"),
        ),
        ..Default::default()
    };

    let loader = Loader::new(config);

    Some((loader, Signer::new()))
}

#[tokio::test]
async fn test_head_blob() -> Result<()> {
    let signer = init_signer();
    if signer.is_none() {
        warn!("REQSIGN_AZURE_STORAGE_ON_TEST is not set, skipped");
        return Ok(());
    }
    let (loader, signer) = signer.unwrap();

    let url =
        &env::var("REQSIGN_AZURE_STORAGE_URL").expect("env REQSIGN_AZURE_STORAGE_URL must set");

    let mut builder = http::Request::builder();
    builder = builder.method(http::Method::HEAD);
    builder = builder.header("x-ms-version", "2023-01-03");
    builder = builder.uri(format!("{}/{}", url, "not_exist_file"));
    let req = builder.body("")?;

    let cred = loader
        .load()
        .await
        .expect("load credential must success")
        .unwrap();

    let req = {
        let (mut parts, body) = req.into_parts();
        signer
            .sign(&mut parts, &cred)
            .expect("sign request must success");
        Request::from_parts(parts, body)
    };

    debug!("signed request: {:?}", req);

    let client = Client::new();
    let resp = client
        .execute(req.try_into()?)
        .await
        .expect("request must success");

    debug!("got response: {:?}", resp);
    assert_eq!(StatusCode::NOT_FOUND, resp.status());
    Ok(())
}

#[tokio::test]
async fn test_head_object_with_encoded_characters() -> Result<()> {
    let signer = init_signer();
    if signer.is_none() {
        warn!("REQSIGN_AZURE_STORAGE_ON_TEST is not set, skipped");
        return Ok(());
    }
    let (loader, signer) = signer.unwrap();

    let url =
        &env::var("REQSIGN_AZURE_STORAGE_URL").expect("env REQSIGN_AZURE_STORAGE_URL must set");

    let mut req = http::Request::new("");
    *req.method_mut() = http::Method::HEAD;
    req.headers_mut()
        .insert("x-ms-version", "2023-01-03".parse().unwrap());
    *req.uri_mut() = http::Uri::from_str(&format!(
        "{}/{}",
        url,
        utf8_percent_encode("!@#$%^&*()_+-=;:'><,/?.txt", NON_ALPHANUMERIC)
    ))?;

    let cred = loader
        .load()
        .await
        .expect("load credential must success")
        .unwrap();

    let req = {
        let (mut parts, body) = req.into_parts();
        signer
            .sign(&mut parts, &cred)
            .expect("sign request must success");
        Request::from_parts(parts, body)
    };

    debug!("signed request: {:?}", req);

    let client = Client::new();
    let resp = client
        .execute(req.try_into()?)
        .await
        .expect("request must success");

    debug!("got response: {:?}", resp);
    assert_eq!(StatusCode::NOT_FOUND, resp.status());
    Ok(())
}

#[tokio::test]
async fn test_list_container_blobs() -> Result<()> {
    let signer = init_signer();
    if signer.is_none() {
        warn!("REQSIGN_AZURE_STORAGE_ON_TEST is not set, skipped");
        return Ok(());
    }
    let (loader, signer) = signer.unwrap();

    let url =
        &env::var("REQSIGN_AZURE_STORAGE_URL").expect("env REQSIGN_AZURE_STORAGE_URL must set");

    for query in [
        // Without prefix
        "restype=container&comp=list",
        // With not encoded prefix
        "restype=container&comp=list&prefix=test/path/to/dir",
        // With encoded prefix
        "restype=container&comp=list&prefix=test%2Fpath%2Fto%2Fdir",
    ] {
        let mut builder = http::Request::builder();
        builder = builder.method(http::Method::GET);
        builder = builder.uri(format!("{url}?{query}"));
        builder = builder.header("x-ms-version", "2023-01-03");
        let req = builder.body("")?;

        let cred = loader
            .load()
            .await
            .expect("load credential must success")
            .unwrap();

        let req = {
            let (mut parts, body) = req.into_parts();
            signer
                .sign(&mut parts, &cred)
                .expect("sign request must success");
            Request::from_parts(parts, body)
        };

        debug!("signed request: {:?}", req);

        let client = Client::new();
        let resp = client
            .execute(req.try_into()?)
            .await
            .expect("request must success");

        debug!("got response: {:?}", resp);
        assert_eq!(StatusCode::OK, resp.status());
    }

    Ok(())
}

#[tokio::test]
async fn test_can_head_blob_with_sas() -> Result<()> {
    let signer = init_signer();
    if signer.is_none() {
        warn!("REQSIGN_AZURE_STORAGE_ON_TEST is not set, skipped");
        return Ok(());
    }
    let (loader, signer) = signer.unwrap();

    let url =
        &env::var("REQSIGN_AZURE_STORAGE_URL").expect("env REQSIGN_AZURE_STORAGE_URL must set");

    let mut builder = http::Request::builder();
    builder = builder.method(http::Method::HEAD);
    builder = builder.header("x-ms-version", "2023-01-03");
    builder = builder.uri(format!("{}/{}", url, "not_exist_file"));
    let req = builder.body("")?;

    let cred = loader
        .load()
        .await
        .expect("load credential must success")
        .unwrap();

    let req = {
        let (mut parts, body) = req.into_parts();
        signer
            .sign_query(&mut parts, Duration::from_secs(60), &cred)
            .expect("sign request must success");
        Request::from_parts(parts, body)
    };

    println!("signed request: {:?}", req);

    let client = Client::new();
    let resp = client
        .execute(req.try_into()?)
        .await
        .expect("request must success");

    println!("got response: {:?}", resp);
    assert_eq!(StatusCode::NOT_FOUND, resp.status());
    Ok(())
}

#[tokio::test]
async fn test_can_list_container_blobs() -> Result<()> {
    // API https://learn.microsoft.com/en-us/rest/api/storageservices/list-blobs?tabs=azure-ad
    let signer = init_signer();
    if signer.is_none() {
        warn!("REQSIGN_AZURE_STORAGE_ON_TEST is not set, skipped");
        return Ok(());
    }
    let (loader, signer) = signer.unwrap();

    let url =
        &env::var("REQSIGN_AZURE_STORAGE_URL").expect("env REQSIGN_AZURE_STORAGE_URL must set");

    for query in [
        // Without prefix
        "restype=container&comp=list",
        // With not encoded prefix
        "restype=container&comp=list&prefix=test/path/to/dir",
        // With encoded prefix
        "restype=container&comp=list&prefix=test%2Fpath%2Fto%2Fdir",
    ] {
        let mut builder = http::Request::builder();
        builder = builder.method(http::Method::GET);
        builder = builder.header("x-ms-version", "2023-01-03");
        builder = builder.uri(format!("{url}?{query}"));
        let req = builder.body("")?;

        let cred = loader
            .load()
            .await
            .expect("load credential must success")
            .unwrap();

        let (mut parts, body) = req.into_parts();
        signer
            .sign_query(&mut parts, Duration::from_secs(60), &cred)
            .expect("sign request must success");
        let req = Request::from_parts(parts, body);

        let client = Client::new();
        let resp = client
            .execute(req.try_into()?)
            .await
            .expect("request must success");

        debug!("got response: {:?}", resp);
        assert_eq!(StatusCode::OK, resp.status());
    }

    Ok(())
}

/// This test must run on azure vm with imds enabled,
#[tokio::test]
async fn test_head_blob_with_ldms() -> Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    let _ = dotenv::dotenv();

    if env::var("REQSIGN_AZURE_STORAGE_TEST").is_err()
        || env::var("REQSIGN_AZURE_STORAGE_TEST").unwrap() != "on"
        || env::var("REQSIGN_AZURE_STORAGE_CRED").is_err()
        || env::var("REQSIGN_AZURE_STORAGE_CRED").unwrap() != "imds"
    {
        return Ok(());
    }

    let config = Config {
        ..Default::default()
    };
    let loader = Loader::new(config);
    let cred = loader
        .load()
        .await
        .expect("load credential must success")
        .unwrap();

    let url =
        &env::var("REQSIGN_AZURE_STORAGE_URL").expect("env REQSIGN_AZURE_STORAGE_URL must set");

    let req = http::Request::builder()
        .method(http::Method::HEAD)
        .header("x-ms-version", "2023-01-03")
        .uri(format!("{}/{}", url, "not_exist_file"))
        .body("")?;

    let (mut parts, body) = req.into_parts();
    Signer::new()
        .sign(&mut parts, &cred)
        .expect("sign request must success");
    let req = Request::from_parts(parts, body);

    println!("signed request: {:?}", req);

    let client = Client::new();
    let resp = client
        .execute(req.try_into()?)
        .await
        .expect("request must success");

    assert_eq!(StatusCode::NOT_FOUND, resp.status());

    Ok(())
}

/// This test must run on azure vm with imds enabled
#[tokio::test]
async fn test_can_list_container_blobs_with_ldms() -> Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    let _ = dotenv::dotenv();

    if env::var("REQSIGN_AZURE_STORAGE_TEST").is_err()
        || env::var("REQSIGN_AZURE_STORAGE_TEST").unwrap() != "on"
        || env::var("REQSIGN_AZURE_STORAGE_CRED").is_err()
        || env::var("REQSIGN_AZURE_STORAGE_CRED").unwrap() != "imds"
    {
        return Ok(());
    }

    let config = Config {
        ..Default::default()
    };
    let loader = Loader::new(config);
    let cred = loader
        .load()
        .await
        .expect("load credential must success")
        .unwrap();

    let url =
        &env::var("REQSIGN_AZURE_STORAGE_URL").expect("env REQSIGN_AZURE_STORAGE_URL must set");

    for query in [
        // Without prefix
        "restype=container&comp=list",
        // With not encoded prefix
        "restype=container&comp=list&prefix=test/path/to/dir",
        // With encoded prefix
        "restype=container&comp=list&prefix=test%2Fpath%2Fto%2Fdir",
    ] {
        let mut builder = http::Request::builder();
        builder = builder.method(http::Method::GET);
        builder = builder.header("x-ms-version", "2023-01-03");
        builder = builder.uri(format!("{url}?{query}"));
        let req = builder.body("")?;

        let (mut parts, body) = req.into_parts();
        Signer::new()
            .sign(&mut parts, &cred)
            .expect("sign request must success");
        let req = Request::from_parts(parts, body);

        let client = Client::new();
        let resp = client
            .execute(req.try_into()?)
            .await
            .expect("request must success");

        debug!("got response: {:?}", resp);
        assert_eq!(StatusCode::OK, resp.status());
    }

    Ok(())
}

/// This test must run on azure vm with imds enabled,
#[tokio::test]
async fn test_head_blob_with_client_secret() -> Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    let _ = dotenv::dotenv();

    if env::var("REQSIGN_AZURE_STORAGE_TEST").is_err()
        || env::var("REQSIGN_AZURE_STORAGE_TEST").unwrap() != "on"
    {
        warn!("REQSIGN_AZURE_STORAGE_ON_TEST is not set, skipped");
        return Ok(());
    }

    if env::var("REQSIGN_AZURE_STORAGE_CLIENT_SECRET")
        .unwrap_or_default()
        .is_empty()
    {
        warn!("REQSIGN_AZURE_STORAGE_CLIENT_SECRET is not set, skipped");
        return Ok(());
    }

    let config = Config::default().from_env();

    assert!(config.client_secret.is_some());
    assert!(config.tenant_id.is_some());
    assert!(config.client_id.is_some());
    assert!(config.authority_host.is_some());
    assert!(config.account_key.is_none());

    let loader = Loader::new(config);

    let cred = loader
        .load()
        .await
        .expect("load credential must success")
        .unwrap();

    let url =
        &env::var("REQSIGN_AZURE_STORAGE_URL").expect("env REQSIGN_AZURE_STORAGE_URL must set");

    let req = http::Request::builder()
        .method(http::Method::HEAD)
        .header("x-ms-version", "2023-01-03")
        .uri(format!("{}/{}", url, "not_exist_file"))
        .body("")?;

    let (mut parts, body) = req.into_parts();
    Signer::new()
        .sign(&mut parts, &cred)
        .expect("sign request must success");
    let req = Request::from_parts(parts, body);

    println!("signed request: {:?}", req);

    let client = Client::new();
    let resp = client
        .execute(req.try_into()?)
        .await
        .expect("request must success");

    assert_eq!(StatusCode::NOT_FOUND, resp.status());

    Ok(())
}

/// This test must run on azure vm with imds enabled
#[tokio::test]
async fn test_can_list_container_blobs_client_secret() -> Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    let _ = dotenv::dotenv();

    if env::var("REQSIGN_AZURE_STORAGE_TEST").is_err()
        || env::var("REQSIGN_AZURE_STORAGE_TEST").unwrap() != "on"
    {
        warn!("REQSIGN_AZURE_STORAGE_ON_TEST is not set, skipped");
        return Ok(());
    }

    if env::var("REQSIGN_AZURE_STORAGE_CLIENT_SECRET")
        .unwrap_or_default()
        .is_empty()
    {
        warn!("REQSIGN_AZURE_STORAGE_CLIENT_SECRET is not set, skipped");
        return Ok(());
    }

    let config = Config::default().from_env();

    assert!(config.client_secret.is_some());
    assert!(config.tenant_id.is_some());
    assert!(config.client_id.is_some());
    assert!(config.authority_host.is_some());
    assert!(config.account_key.is_none());

    let loader = Loader::new(config);

    let cred = loader
        .load()
        .await
        .expect("load credential must success")
        .unwrap();

    let url =
        &env::var("REQSIGN_AZURE_STORAGE_URL").expect("env REQSIGN_AZURE_STORAGE_URL must set");
    for query in [
        // Without prefix
        "restype=container&comp=list",
        // With not encoded prefix
        "restype=container&comp=list&prefix=test/path/to/dir",
        // With encoded prefix
        "restype=container&comp=list&prefix=test%2Fpath%2Fto%2Fdir",
    ] {
        let mut builder = http::Request::builder();
        builder = builder.method(http::Method::GET);
        builder = builder.header("x-ms-version", "2023-01-03");
        builder = builder.uri(format!("{url}?{query}"));
        let req = builder.body("")?;

        let (mut parts, body) = req.into_parts();
        Signer::new()
            .sign(&mut parts, &cred)
            .expect("sign request must success");
        let req = Request::from_parts(parts, body);

        let client = Client::new();
        let resp = client
            .execute(req.try_into()?)
            .await
            .expect("request must success");
        let stat = resp.status();
        debug!("got response: {:?}", resp);
        debug!("{}", resp.text().await?);

        assert_eq!(StatusCode::OK, stat);
    }

    Ok(())
}
