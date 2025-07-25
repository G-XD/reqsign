use crate::{constants::*, Credential};
use async_trait::async_trait;
use reqsign_core::{Context, ProvideCredential, Result};

/// EnvCredentialProvider loads Tencent Cloud credentials from environment variables.
///
/// This provider looks for the following environment variables:
/// - `TENCENTCLOUD_SECRET_ID` or `TKE_SECRET_ID`: The Tencent Cloud secret ID
/// - `TENCENTCLOUD_SECRET_KEY` or `TKE_SECRET_KEY`: The Tencent Cloud secret key
/// - `TENCENTCLOUD_TOKEN`, `TENCENTCLOUD_SECURITY_TOKEN`, or `QCLOUD_SECRET_TOKEN`: The security token (optional)
#[derive(Debug, Default)]
pub struct EnvCredentialProvider;

impl EnvCredentialProvider {
    /// Create a new EnvCredentialProvider.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProvideCredential for EnvCredentialProvider {
    type Credential = Credential;

    async fn provide_credential(&self, ctx: &Context) -> Result<Option<Self::Credential>> {
        let envs = ctx.env_vars();

        // Try to get secret_id from multiple env vars
        let secret_id = envs
            .get(TENCENTCLOUD_SECRET_ID)
            .or_else(|| envs.get(TKE_SECRET_ID));

        // Try to get secret_key from multiple env vars
        let secret_key = envs
            .get(TENCENTCLOUD_SECRET_KEY)
            .or_else(|| envs.get(TKE_SECRET_KEY));

        match (secret_id, secret_key) {
            (Some(id), Some(key)) => {
                // Try to get security token from multiple env vars
                let security_token = envs
                    .get(TENCENTCLOUD_TOKEN)
                    .or_else(|| envs.get(TENCENTCLOUD_SECURITY_TOKEN))
                    .or_else(|| envs.get("QCLOUD_SECRET_TOKEN"))
                    .cloned();

                Ok(Some(Credential {
                    secret_id: id.clone(),
                    secret_key: key.clone(),
                    security_token,
                    expires_in: None,
                }))
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqsign_core::StaticEnv;
    use reqsign_file_read_tokio::TokioFileRead;
    use reqsign_http_send_reqwest::ReqwestHttpSend;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_env_credential_provider() -> anyhow::Result<()> {
        let envs = HashMap::from([
            (
                TENCENTCLOUD_SECRET_ID.to_string(),
                "test_secret_id".to_string(),
            ),
            (
                TENCENTCLOUD_SECRET_KEY.to_string(),
                "test_secret_key".to_string(),
            ),
        ]);

        let ctx = Context::new(TokioFileRead, ReqwestHttpSend::default()).with_env(StaticEnv {
            home_dir: None,
            envs,
        });

        let provider = EnvCredentialProvider::new();
        let cred = provider.provide_credential(&ctx).await?;
        assert!(cred.is_some());
        let cred = cred.unwrap();
        assert_eq!(cred.secret_id, "test_secret_id");
        assert_eq!(cred.secret_key, "test_secret_key");
        assert!(cred.security_token.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_env_credential_provider_with_security_token() -> anyhow::Result<()> {
        let envs = HashMap::from([
            (TKE_SECRET_ID.to_string(), "test_secret_id".to_string()),
            (TKE_SECRET_KEY.to_string(), "test_secret_key".to_string()),
            (
                TENCENTCLOUD_TOKEN.to_string(),
                "test_security_token".to_string(),
            ),
        ]);

        let ctx = Context::new(TokioFileRead, ReqwestHttpSend::default()).with_env(StaticEnv {
            home_dir: None,
            envs,
        });

        let provider = EnvCredentialProvider::new();
        let cred = provider.provide_credential(&ctx).await?;
        assert!(cred.is_some());
        let cred = cred.unwrap();
        assert_eq!(cred.secret_id, "test_secret_id");
        assert_eq!(cred.secret_key, "test_secret_key");
        assert_eq!(cred.security_token, Some("test_security_token".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_env_credential_provider_missing_credentials() -> anyhow::Result<()> {
        let ctx = Context::new(TokioFileRead, ReqwestHttpSend::default());

        let provider = EnvCredentialProvider::new();
        let cred = provider.provide_credential(&ctx).await?;
        assert!(cred.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_env_credential_provider_partial_credentials() -> anyhow::Result<()> {
        // Only secret ID
        let envs = HashMap::from([(
            TENCENTCLOUD_SECRET_ID.to_string(),
            "test_secret_id".to_string(),
        )]);

        let ctx = Context::new(TokioFileRead, ReqwestHttpSend::default()).with_env(StaticEnv {
            home_dir: None,
            envs,
        });

        let provider = EnvCredentialProvider::new();
        let cred = provider.provide_credential(&ctx).await?;
        assert!(cred.is_none());

        Ok(())
    }
}
