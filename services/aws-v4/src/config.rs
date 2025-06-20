use std::fmt;

use super::constants::*;
#[cfg(not(target_arch = "wasm32"))]
use ini::Ini;
#[cfg(not(target_arch = "wasm32"))]
use log::debug;
use reqsign_core::utils::Redact;
use reqsign_core::Context;
#[cfg(not(target_arch = "wasm32"))]
use reqsign_core::{Error, Result};

/// Config for aws services.
#[derive(Clone)]
pub struct Config {
    /// `config_file` will be load from:
    ///
    /// - env value: [`AWS_CONFIG_FILE`]
    /// - default to: `~/.aws/config`
    pub config_file: String,
    /// `shared_credentials_file` will be loaded from:
    ///
    /// - env value: [`AWS_SHARED_CREDENTIALS_FILE`]
    /// - default to: `~/.aws/credentials`
    pub shared_credentials_file: String,
    /// `profile` will be loaded from:
    ///
    /// - this field if it's `is_some`
    /// - env value: [`AWS_PROFILE`]
    /// - default to: `default`
    pub profile: String,

    /// `region` will be loaded from:
    ///
    /// - this field if it's `is_some`
    /// - env value: [`AWS_REGION`]
    /// - profile config: `region`
    pub region: Option<String>,
    /// `sts_regional_endpoints` will be loaded from:
    ///
    /// - env value: [`AWS_STS_REGIONAL_ENDPOINTS`]
    /// - profile config: `sts_regional_endpoints`
    /// - default to `legacy`
    pub sts_regional_endpoints: String,
    /// `access_key_id` will be loaded from
    ///
    /// - this field if it's `is_some`
    /// - env value: [`AWS_ACCESS_KEY_ID`]
    /// - profile config: `aws_access_key_id`
    pub access_key_id: Option<String>,
    /// `secret_access_key` will be loaded from
    ///
    /// - this field if it's `is_some`
    /// - env value: [`AWS_SECRET_ACCESS_KEY`]
    /// - profile config: `aws_secret_access_key`
    pub secret_access_key: Option<String>,
    /// `session_token` will be loaded from
    ///
    /// - this field if it's `is_some`
    /// - env value: [`AWS_SESSION_TOKEN`]
    /// - profile config: `aws_session_token`
    pub session_token: Option<String>,
    /// `role_arn` value will be load from:
    ///
    /// - this field if it's `is_some`.
    /// - env value: [`AWS_ROLE_ARN`]
    /// - profile config: `role_arn`
    pub role_arn: Option<String>,
    /// `role_session_name` value will be load from:
    ///
    /// - env value: [`AWS_ROLE_SESSION_NAME`]
    /// - profile config: `role_session_name`
    /// - default to `reqsign`.
    pub role_session_name: String,
    /// `duration_seconds` value will be load from:
    ///
    /// - this field if it's `is_some`.
    /// - profile config: `duration_seconds`
    /// - default to `3600`.
    pub duration_seconds: Option<usize>,
    /// `external_id` value will be load from:
    ///
    /// - this field if it's `is_some`.
    /// - profile config: `external_id`
    pub external_id: Option<String>,
    /// `tags` value will be loaded from:
    ///
    /// - this field if it's `is_some`
    pub tags: Option<Vec<(String, String)>>,
    /// `web_identity_token_file` value will be loaded from:
    ///
    /// - this field if it's `is_some`
    /// - env value: [`AWS_WEB_IDENTITY_TOKEN_FILE`]
    /// - profile config: `web_identity_token_file`
    pub web_identity_token_file: Option<String>,
    /// `ec2_metadata_disabled` value will be loaded from:
    ///
    /// - this field
    /// - env value: [`AWS_EC2_METADATA_DISABLED`]
    pub ec2_metadata_disabled: bool,
    /// `endpoint_url` value will be loaded from:
    ///
    /// - this field
    /// - env value: [`AWS_ENDPOINT_URL`]
    pub endpoint_url: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_file: "~/.aws/config".to_string(),
            shared_credentials_file: "~/.aws/credentials".to_string(),
            profile: "default".to_string(),
            region: None,
            sts_regional_endpoints: "legacy".to_string(),
            access_key_id: None,
            secret_access_key: None,
            session_token: None,
            role_arn: None,
            role_session_name: "reqsign".to_string(),
            duration_seconds: Some(3600),
            external_id: None,
            tags: None,
            web_identity_token_file: None,
            ec2_metadata_disabled: false,
            endpoint_url: None,
        }
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("config_file", &self.config_file)
            .field("shared_credentials_file", &self.shared_credentials_file)
            .field("profile", &self.profile)
            .field("region", &self.region)
            .field("sts_regional_endpoints", &self.sts_regional_endpoints)
            .field("access_key_id", &Redact::from(&self.access_key_id))
            .field("secret_access_key", &Redact::from(&self.secret_access_key))
            .field("session_token", &Redact::from(&self.session_token))
            .field("role_arn", &self.role_arn)
            .field("role_session_name", &self.role_session_name)
            .field("duration_seconds", &self.duration_seconds)
            .field("external_id", &Redact::from(&self.external_id))
            .field("tags", &self.tags)
            .field("web_identity_token_file", &self.web_identity_token_file)
            .field("ec2_metadata_disabled", &self.ec2_metadata_disabled)
            .field("endpoint_url", &self.endpoint_url)
            .finish()
    }
}

impl Config {
    /// Load config from env.
    pub fn from_env(mut self, ctx: &Context) -> Self {
        let envs = ctx.env_vars();

        if let Some(v) = envs.get(AWS_CONFIG_FILE) {
            self.config_file = v.to_string();
        }
        if let Some(v) = envs.get(AWS_SHARED_CREDENTIALS_FILE) {
            self.shared_credentials_file = v.to_string();
        }
        if let Some(v) = envs.get(AWS_PROFILE) {
            self.profile = v.to_string();
        }
        if let Some(v) = envs.get(AWS_REGION) {
            self.region = Some(v.to_string())
        }
        if let Some(v) = envs.get(AWS_STS_REGIONAL_ENDPOINTS) {
            self.sts_regional_endpoints = v.to_string();
        }
        if let Some(v) = envs.get(AWS_ACCESS_KEY_ID) {
            self.access_key_id = Some(v.to_string())
        }
        if let Some(v) = envs.get(AWS_SECRET_ACCESS_KEY) {
            self.secret_access_key = Some(v.to_string())
        }
        if let Some(v) = envs.get(AWS_SESSION_TOKEN) {
            self.session_token = Some(v.to_string())
        }
        if let Some(v) = envs.get(AWS_ROLE_ARN) {
            self.role_arn = Some(v.to_string())
        }
        if let Some(v) = envs.get(AWS_ROLE_SESSION_NAME) {
            self.role_session_name = v.to_string();
        }
        if let Some(v) = envs.get(AWS_WEB_IDENTITY_TOKEN_FILE) {
            self.web_identity_token_file = Some(v.to_string());
        }
        if let Some(v) = envs.get(AWS_EC2_METADATA_DISABLED) {
            self.ec2_metadata_disabled = v == "true";
        }
        if let Some(v) = envs.get(AWS_ENDPOINT_URL) {
            self.endpoint_url = Some(v.to_string());
        }
        self
    }

    /// Load config from profile (and shared profile).
    ///
    /// If the env var AWS_PROFILE is set, this profile will be used,
    /// otherwise the contents of `self.profile` will be used.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn from_profile(mut self, ctx: &Context) -> Self {
        // self.profile is checked by the two load methods.
        if let Some(profile) = ctx.env_var(AWS_PROFILE) {
            self.profile = profile;
        }

        // make sure we're getting profile info from the correct place.
        // Respecting these env vars also makes it possible to unit test
        // this method.
        if let Some(config_file) = ctx.env_var(AWS_CONFIG_FILE) {
            self.config_file = config_file;
        }

        if let Some(shared_credentials_file) = ctx.env_var(AWS_SHARED_CREDENTIALS_FILE) {
            self.shared_credentials_file = shared_credentials_file;
        }

        // Ignore all errors happened internally.
        let _ = self.load_via_profile_config_file(ctx).await.map_err(|err| {
            debug!("load_via_profile_config_file failed: {err:?}");
        });

        let _ = self
            .load_via_profile_shared_credentials_file(ctx)
            .await
            .map_err(|err| debug!("load_via_profile_shared_credentials_file failed: {err:?}"));

        self
    }

    /// Only the following fields will exist in shared_credentials_file:
    ///
    /// - `aws_access_key_id`
    /// - `aws_secret_access_key`
    /// - `aws_session_token`
    #[cfg(not(target_arch = "wasm32"))]
    async fn load_via_profile_shared_credentials_file(&mut self, ctx: &Context) -> Result<()> {
        let path = ctx
            .expand_home_dir(&self.shared_credentials_file)
            .ok_or_else(|| Error::config_invalid("expand homedir failed"))?;

        let content = ctx.file_read(&path).await.map_err(|e| {
            Error::config_invalid("failed to read shared credentials file").with_source(e)
        })?;
        let conf = Ini::load_from_str(&String::from_utf8_lossy(&content)).map_err(|e| {
            Error::config_invalid("failed to parse shared credentials file")
                .with_source(anyhow::Error::new(e))
        })?;

        let props = conf.section(Some(&self.profile)).ok_or_else(|| {
            Error::config_invalid(format!("section {} is not found", self.profile))
        })?;

        if let Some(v) = props.get("aws_access_key_id") {
            self.access_key_id = Some(v.to_string())
        }
        if let Some(v) = props.get("aws_secret_access_key") {
            self.secret_access_key = Some(v.to_string())
        }
        if let Some(v) = props.get("aws_session_token") {
            self.session_token = Some(v.to_string())
        }

        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn load_via_profile_config_file(&mut self, ctx: &Context) -> Result<()> {
        let path = ctx
            .expand_home_dir(&self.config_file)
            .ok_or_else(|| Error::config_invalid("expand homedir failed"))?;

        let content = ctx
            .file_read(&path)
            .await
            .map_err(|e| Error::config_invalid("failed to read config file").with_source(e))?;
        let conf = Ini::load_from_str(&String::from_utf8_lossy(&content)).map_err(|e| {
            Error::config_invalid("failed to parse config file").with_source(anyhow::Error::new(e))
        })?;

        let section = match self.profile.as_str() {
            "default" => "default".to_string(),
            x => format!("profile {x}"),
        };
        let props = conf.section(Some(section)).ok_or_else(|| {
            Error::config_invalid(format!("section {} is not found", self.profile))
        })?;

        if let Some(v) = props.get("region") {
            self.region = Some(v.to_string())
        }
        if let Some(v) = props.get("sts_regional_endpoints") {
            self.sts_regional_endpoints = v.to_string();
        }
        if let Some(v) = props.get("aws_access_key_id") {
            self.access_key_id = Some(v.to_string())
        }
        if let Some(v) = props.get("aws_secret_access_key") {
            self.secret_access_key = Some(v.to_string())
        }
        if let Some(v) = props.get("aws_session_token") {
            self.session_token = Some(v.to_string())
        }
        if let Some(v) = props.get("role_arn") {
            self.role_arn = Some(v.to_string())
        }
        if let Some(v) = props.get("role_session_name") {
            self.role_session_name = v.to_string()
        }
        if let Some(v) = props.get("duration_seconds") {
            self.duration_seconds = Some(v.to_string().parse::<usize>().unwrap())
        }
        if let Some(v) = props.get("web_identity_token_file") {
            self.web_identity_token_file = Some(v.to_string())
        }
        if let Some(v) = props.get("endpoint_url") {
            self.endpoint_url = Some(v.to_string())
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use reqsign_core::StaticEnv;
    use reqsign_file_read_tokio::TokioFileRead;
    use reqsign_http_send_reqwest::ReqwestHttpSend;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[tokio::test]
    #[cfg(not(target_arch = "wasm32"))]
    async fn test_config_from_profile_shared_credentials() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        // Create a dummy credentials file to test against
        let tmp_dir = tempdir()?;
        let file_path = tmp_dir.path().join("credentials");
        let mut tmp_file = File::create(&file_path)?;
        writeln!(tmp_file, "[default]")?;
        writeln!(tmp_file, "aws_access_key_id = DEFAULTACCESSKEYID")?;
        writeln!(tmp_file, "aws_secret_access_key = DEFAULTSECRETACCESSKEY")?;
        writeln!(tmp_file, "aws_session_token = DEFAULTSESSIONTOKEN")?;
        writeln!(tmp_file)?;
        writeln!(tmp_file, "[profile1]")?;
        writeln!(tmp_file, "aws_access_key_id = PROFILE1ACCESSKEYID")?;
        writeln!(tmp_file, "aws_secret_access_key = PROFILE1SECRETACCESSKEY")?;
        writeln!(tmp_file, "aws_session_token = PROFILE1SESSIONTOKEN")?;

        let context = Context::new(TokioFileRead, ReqwestHttpSend::default());
        let context = context.with_env(StaticEnv {
            home_dir: None,
            envs: HashMap::from_iter([
                (AWS_PROFILE.to_string(), "profile1".to_string()),
                (
                    AWS_SHARED_CREDENTIALS_FILE.to_string(),
                    file_path.to_str().unwrap().to_owned(),
                ),
            ]),
        });

        let config = Config::default().from_profile(&context).await;

        assert_eq!(config.profile, "profile1".to_owned());
        assert_eq!(config.access_key_id, Some("PROFILE1ACCESSKEYID".to_owned()));
        assert_eq!(
            config.secret_access_key,
            Some("PROFILE1SECRETACCESSKEY".to_owned())
        );
        assert_eq!(
            config.session_token,
            Some("PROFILE1SESSIONTOKEN".to_owned())
        );

        Ok(())
    }

    #[tokio::test]
    #[cfg(not(target_arch = "wasm32"))]
    async fn test_config_from_profile_config() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        // Create a dummy credentials file to test against
        let tmp_dir = tempdir()?;
        let file_path = tmp_dir.path().join("config");
        let mut tmp_file = File::create(&file_path)?;
        writeln!(tmp_file, "[default]")?;
        writeln!(tmp_file, "aws_access_key_id = DEFAULTACCESSKEYID")?;
        writeln!(tmp_file, "aws_secret_access_key = DEFAULTSECRETACCESSKEY")?;
        writeln!(tmp_file, "aws_session_token = DEFAULTSESSIONTOKEN")?;
        writeln!(tmp_file)?;
        writeln!(tmp_file, "[profile profile1]")?;
        writeln!(tmp_file, "aws_access_key_id = PROFILE1ACCESSKEYID")?;
        writeln!(tmp_file, "aws_secret_access_key = PROFILE1SECRETACCESSKEY")?;
        writeln!(tmp_file, "aws_session_token = PROFILE1SESSIONTOKEN")?;
        writeln!(tmp_file, "endpoint_url = http://localhost:8080")?;

        let context = Context::new(TokioFileRead, ReqwestHttpSend::default());
        let context = context.with_env(StaticEnv {
            home_dir: None,
            envs: HashMap::from_iter([
                (AWS_PROFILE.to_string(), "profile1".to_string()),
                (
                    AWS_CONFIG_FILE.to_string(),
                    file_path.to_str().unwrap().to_owned(),
                ),
            ]),
        });

        let config = Config::default().from_profile(&context).await;

        assert_eq!(config.profile, "profile1".to_owned());
        assert_eq!(config.access_key_id, Some("PROFILE1ACCESSKEYID".to_owned()));
        assert_eq!(
            config.secret_access_key,
            Some("PROFILE1SECRETACCESSKEY".to_owned())
        );
        assert_eq!(
            config.session_token,
            Some("PROFILE1SESSIONTOKEN".to_owned())
        );
        assert_eq!(
            config.endpoint_url,
            Some("http://localhost:8080".to_owned())
        );

        Ok(())
    }
}
