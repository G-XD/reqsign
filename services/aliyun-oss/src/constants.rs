use percent_encoding::AsciiSet;
use percent_encoding::NON_ALPHANUMERIC;

// Env values used in aliyun services.
pub const ALIBABA_CLOUD_ACCESS_KEY_ID: &str = "ALIBABA_CLOUD_ACCESS_KEY_ID";
pub const ALIBABA_CLOUD_ACCESS_KEY_SECRET: &str = "ALIBABA_CLOUD_ACCESS_KEY_SECRET";
pub const ALIBABA_CLOUD_SECURITY_TOKEN: &str = "ALIBABA_CLOUD_SECURITY_TOKEN";
pub const ALIBABA_CLOUD_ROLE_ARN: &str = "ALIBABA_CLOUD_ROLE_ARN";
pub const ALIBABA_CLOUD_OIDC_PROVIDER_ARN: &str = "ALIBABA_CLOUD_OIDC_PROVIDER_ARN";
pub const ALIBABA_CLOUD_OIDC_TOKEN_FILE: &str = "ALIBABA_CLOUD_OIDC_TOKEN_FILE";
pub const ALIBABA_CLOUD_STS_ENDPOINT: &str = "ALIBABA_CLOUD_STS_ENDPOINT";

/// AsciiSet for UriEncode but used in query.
pub static ALIBABA_CLOUD_QUERY_ENCODE_SET: AsciiSet = NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');
