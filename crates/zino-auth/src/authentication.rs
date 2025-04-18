use super::{AccessKeyId, SecretAccessKey};
use hmac::{
    Mac,
    digest::{FixedOutput, KeyInit, MacMarker, Update},
};
use http::HeaderMap;
use std::time::Duration;
use zino_core::{Map, datetime::DateTime, encoding::base64, error::Error, validation::Validation};

/// HTTP signature using HMAC.
pub struct Authentication {
    /// Service name.
    service_name: String,
    /// Access key ID.
    access_key_id: AccessKeyId,
    /// Signature.
    signature: String,
    /// HTTP method.
    method: String,
    /// Accept header value.
    accept: Option<String>,
    /// Content-MD5 header value.
    content_md5: Option<String>,
    /// Content-Type header value.
    content_type: Option<String>,
    /// Date header.
    date_header: (&'static str, DateTime),
    /// Expires.
    expires: Option<DateTime>,
    /// Canonicalized headers.
    headers: Vec<(String, String)>,
    /// Canonicalized resource.
    resource: String,
}

impl Authentication {
    /// Creates a new instance.
    #[inline]
    pub fn new(method: &str) -> Self {
        Self {
            service_name: String::new(),
            access_key_id: AccessKeyId::default(),
            signature: String::new(),
            method: method.to_ascii_uppercase(),
            accept: None,
            content_md5: None,
            content_type: None,
            date_header: ("date", DateTime::now()),
            expires: None,
            headers: Vec::new(),
            resource: String::new(),
        }
    }

    /// Sets the service name.
    #[inline]
    pub fn set_service_name(&mut self, service_name: &str) {
        self.service_name = service_name.to_ascii_uppercase();
    }

    /// Sets the access key ID.
    #[inline]
    pub fn set_access_key_id(&mut self, access_key_id: impl Into<AccessKeyId>) {
        self.access_key_id = access_key_id.into();
    }

    /// Sets the signature.
    #[inline]
    pub fn set_signature(&mut self, signature: String) {
        self.signature = signature;
    }

    /// Sets the `accept` header value.
    #[inline]
    pub fn set_accept(&mut self, accept: Option<String>) {
        self.accept = accept;
    }

    /// Sets the `content-md5` header value.
    #[inline]
    pub fn set_content_md5(&mut self, content_md5: String) {
        self.content_md5 = Some(content_md5);
    }

    /// Sets the `content-type` header value.
    #[inline]
    pub fn set_content_type(&mut self, content_type: Option<String>) {
        self.content_type = content_type;
    }

    /// Sets the header value for the date.
    #[inline]
    pub fn set_date_header(&mut self, header_name: &'static str, date: DateTime) {
        self.date_header = (header_name, date);
    }

    /// Sets the expires timestamp.
    #[inline]
    pub fn set_expires(&mut self, expires: Option<DateTime>) {
        self.expires = expires;
    }

    /// Sets the canonicalized headers.
    /// The header is matched if it has a prefix in the filter list.
    #[inline]
    pub fn set_headers(&mut self, headers: HeaderMap, filters: &[&'static str]) {
        let mut headers = headers
            .into_iter()
            .filter_map(|(name, value)| {
                name.and_then(|name| {
                    let key = name.as_str();
                    if filters.iter().any(|&s| key.starts_with(s)) {
                        value
                            .to_str()
                            .inspect_err(|err| tracing::warn!("invalid header value: {err}"))
                            .ok()
                            .map(|value| (key.to_ascii_lowercase(), value.to_owned()))
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>();
        headers.sort_by(|a, b| a.0.cmp(&b.0));
        self.headers = headers;
    }

    /// Sets the canonicalized resource.
    #[inline]
    pub fn set_resource(&mut self, path: String, query: Option<&Map>) {
        if let Some(query) = query {
            if query.is_empty() {
                self.resource = path;
            } else {
                let mut query_pairs = query.iter().collect::<Vec<_>>();
                query_pairs.sort_by(|a, b| a.0.cmp(b.0));

                let query = query_pairs
                    .iter()
                    .map(|(key, value)| format!("{key}={value}"))
                    .collect::<Vec<_>>();
                self.resource = path + "?" + &query.join("&");
            }
        } else {
            self.resource = path;
        }
    }

    /// Returns the service name.
    #[inline]
    pub fn service_name(&self) -> &str {
        self.service_name.as_str()
    }

    /// Returns the access key ID.
    #[inline]
    pub fn access_key_id(&self) -> &str {
        self.access_key_id.as_str()
    }

    /// Returns the signature.
    #[inline]
    pub fn signature(&self) -> &str {
        self.signature.as_str()
    }

    /// Returns an `authorization` header value.
    #[inline]
    pub fn authorization(&self) -> String {
        let service_name = self.service_name();
        let access_key_id = self.access_key_id();
        let signature = self.signature();
        if service_name.is_empty() {
            format!("{access_key_id}:{signature}")
        } else {
            format!("{service_name} {access_key_id}:{signature}")
        }
    }

    /// Returns the string to sign.
    pub fn string_to_sign(&self) -> String {
        let mut sign_parts = Vec::new();

        // HTTP verb
        sign_parts.push(self.method.clone());

        // Accept
        if let Some(accept) = self.accept.as_ref() {
            sign_parts.push(accept.to_owned());
        }

        // Content-MD5
        let content_md5 = self
            .content_md5
            .as_ref()
            .map(|s| s.to_owned())
            .unwrap_or_default();
        sign_parts.push(content_md5);

        // Content-Type
        let content_type = self
            .content_type
            .as_ref()
            .map(|s| s.to_owned())
            .unwrap_or_default();
        sign_parts.push(content_type);

        // Expires
        if let Some(expires) = self.expires.as_ref() {
            sign_parts.push(expires.timestamp().to_string());
        } else {
            // Date
            let date_header = &self.date_header;
            let date = if date_header.0.eq_ignore_ascii_case("date") {
                date_header.1.to_utc_string()
            } else {
                "".to_owned()
            };
            sign_parts.push(date);
        }

        // Canonicalized headers
        let headers = self
            .headers
            .iter()
            .map(|(name, values)| format!("{}:{}", name, values.trim()))
            .collect::<Vec<_>>();
        sign_parts.extend(headers);

        // Canonicalized resource
        sign_parts.push(self.resource.clone());

        sign_parts.join("\n")
    }

    /// Generates a signature with the secret access key.
    pub fn sign_with<H>(&self, secret_access_key: &SecretAccessKey) -> Result<String, Error>
    where
        H: FixedOutput + KeyInit + MacMarker + Update,
    {
        let string_to_sign = self.string_to_sign();
        let mut mac = H::new_from_slice(secret_access_key.as_ref())?;
        mac.update(string_to_sign.as_ref());
        Ok(base64::encode(mac.finalize().into_bytes()))
    }

    /// Validates the signature using the secret access key.
    pub fn validate_with<H>(&self, secret_access_key: &SecretAccessKey) -> Validation
    where
        H: FixedOutput + KeyInit + MacMarker + Update,
    {
        let mut validation = Validation::new();
        let current = DateTime::now();
        let date = self.date_header.1;
        let max_tolerance = Duration::from_secs(900);
        if date < current && date < current - max_tolerance
            || date > current && date > current + max_tolerance
        {
            validation.record("date", "untrusted date");
        }
        if let Some(expires) = self.expires {
            if current > expires {
                validation.record("expires", "valid period has expired");
            }
        }

        let signature = self.signature();
        if signature.is_empty() {
            validation.record("signature", "should be nonempty");
        } else if self
            .sign_with::<H>(secret_access_key)
            .is_ok_and(|s| s != signature)
        {
            validation.record("signature", "invalid signature");
        }
        validation
    }
}
