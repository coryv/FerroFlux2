use anyhow::{Context, Result};
use aws_credential_types::Credentials;
use aws_sigv4::http_request::{
    sign, SignableBody, SignableRequest, SigningSettings,
};
use aws_smithy_runtime_api::client::identity::Identity;
use http02::Request;

use std::time::SystemTime;

pub struct AwsSigV4Config {
    pub service: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
}

/// Signs an HTTP request using AWS Signature Version 4.
/// 
/// Returns a list of additional headers (including 'Authorization' and 'X-Amz-*')
/// that must be added to the request.
pub fn generate_sigv4_headers(
    config: &AwsSigV4Config,
    method: &str,
    url: &str,
    headers: &[(String, String)],
    body: &[u8],
) -> Result<Vec<(String, String)>> {
    let identity = Identity::new(
        Credentials::new(
            &config.access_key,
            &config.secret_key,
            None,
            None,
            "ferroflux-provider",
        ),
        None,
    );

    let mut settings = SigningSettings::default();
    // For S3, we often want to sign the payload (x-amz-content-sha256)
    if config.service == "s3" {
        settings.payload_checksum_kind = aws_sigv4::http_request::PayloadChecksumKind::XAmzSha256;
    }

    let signing_params = aws_sigv4::sign::v4::SigningParams::builder()
        .identity(&identity)
        .region(&config.region)
        .name(&config.service)
        .time(SystemTime::now())
        .settings(settings)
        .build()
        .context("Failed to build AWS signing parameters")?
        .into();



    let signable_request = SignableRequest::new(
        method,
        url,
        headers.iter().map(|(k, v)| (k.as_str(), v.as_str())),
        SignableBody::Bytes(body),
    ).context("Failed to create signable request")?;

    let (signing_instructions, _signature) = sign(signable_request, &signing_params)
        .context("Failed to sign request")?
        .into_parts();

    // We use a dummy http 0.2 request to extract the headers using the compatibility method
    let mut dummy_req = Request::builder()
        .method(method)
        .uri(url)
        .body(())
        .context("Failed to build dummy request")?;

    signing_instructions.apply_to_request_http0x(&mut dummy_req);

    let mut result_headers = Vec::new();
    for (name, value) in dummy_req.headers() {
        result_headers.push((
            name.as_str().to_string(),
            value.to_str().unwrap_or("").to_string(),
        ));
    }

    Ok(result_headers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigv4_header_generation() {
        let config = AwsSigV4Config {
            service: "s3".to_string(),
            region: "us-east-1".to_string(),
            access_key: "AKIAIOSFODNN7EXAMPLE".to_string(),
            secret_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
        };

        let headers = vec![("Host".to_string(), "examplebucket.s3.amazonaws.com".to_string())];
        let result = generate_sigv4_headers(
            &config,
            "GET",
            "https://examplebucket.s3.amazonaws.com/test.txt",
            &headers,
            &[],
        );

        assert!(result.is_ok(), "Result should be OK: {:?}", result.err());
        let signed_headers = result.unwrap();

        // Should contains Authorization and X-Amz-Date
        assert!(signed_headers.iter().any(|(k, _)| k.to_lowercase() == "authorization"));
        assert!(signed_headers.iter().any(|(k, _)| k.to_lowercase() == "x-amz-date"));
        assert!(signed_headers.iter().any(|(k, _)| k.to_lowercase() == "x-amz-content-sha256"));
    }
}
