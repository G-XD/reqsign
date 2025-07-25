use crate::{constants::*, Credential};
use async_trait::async_trait;
use reqsign_core::{Context, ProvideCredential, Result};

/// EnvCredentialProvider loads Oracle Cloud credentials from environment variables.
///
/// This provider looks for the following environment variables:
/// - `OCI_USER`: The Oracle Cloud user ID
/// - `OCI_TENANCY`: The Oracle Cloud tenancy ID
/// - `OCI_KEY_FILE`: The path to the private key file
/// - `OCI_FINGERPRINT`: The fingerprint of the key
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

        let user = envs.get(ORACLE_USER);
        let tenancy = envs.get(ORACLE_TENANCY);
        let key_file = envs.get(ORACLE_KEY_FILE);
        let fingerprint = envs.get(ORACLE_FINGERPRINT);

        match (user, tenancy, key_file, fingerprint) {
            (Some(user), Some(tenancy), Some(key_file), Some(fingerprint)) => {
                // Expand key file path if it starts with ~
                let expanded_key_file = if key_file.starts_with('~') {
                    ctx.expand_home_dir(key_file).ok_or_else(|| {
                        reqsign_core::Error::unexpected("Failed to expand home directory")
                    })?
                } else {
                    key_file.clone()
                };

                Ok(Some(Credential {
                    user: user.clone(),
                    tenancy: tenancy.clone(),
                    key_file: expanded_key_file,
                    fingerprint: fingerprint.clone(),
                    expires_in: Some(
                        reqsign_core::time::now()
                            + chrono::TimeDelta::try_minutes(10).expect("in bounds"),
                    ),
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
            (ORACLE_USER.to_string(), "test_user".to_string()),
            (ORACLE_TENANCY.to_string(), "test_tenancy".to_string()),
            (ORACLE_KEY_FILE.to_string(), "/path/to/key".to_string()),
            (
                ORACLE_FINGERPRINT.to_string(),
                "test_fingerprint".to_string(),
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
        assert_eq!(cred.user, "test_user");
        assert_eq!(cred.tenancy, "test_tenancy");
        assert_eq!(cred.key_file, "/path/to/key");
        assert_eq!(cred.fingerprint, "test_fingerprint");

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
        // Only user and tenancy
        let envs = HashMap::from([
            (ORACLE_USER.to_string(), "test_user".to_string()),
            (ORACLE_TENANCY.to_string(), "test_tenancy".to_string()),
        ]);

        let ctx = Context::new(TokioFileRead, ReqwestHttpSend::default()).with_env(StaticEnv {
            home_dir: None,
            envs,
        });

        let provider = EnvCredentialProvider::new();
        let cred = provider.provide_credential(&ctx).await?;
        assert!(cred.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_env_credential_provider_with_home_expansion() -> anyhow::Result<()> {
        let envs = HashMap::from([
            (ORACLE_USER.to_string(), "test_user".to_string()),
            (ORACLE_TENANCY.to_string(), "test_tenancy".to_string()),
            (ORACLE_KEY_FILE.to_string(), "~/key.pem".to_string()),
            (
                ORACLE_FINGERPRINT.to_string(),
                "test_fingerprint".to_string(),
            ),
        ]);

        let ctx = Context::new(TokioFileRead, ReqwestHttpSend::default()).with_env(StaticEnv {
            home_dir: Some("/home/user".into()),
            envs,
        });

        let provider = EnvCredentialProvider::new();
        let cred = provider.provide_credential(&ctx).await?;
        assert!(cred.is_some());
        let cred = cred.unwrap();
        assert_eq!(cred.key_file, "/home/user/key.pem");

        Ok(())
    }
}
