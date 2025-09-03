use std::collections::HashMap;

use saluki_env::{EnvironmentProvider, HostProvider};
use saluki_metadata;
use tracing::warn;

#[derive(Default, Clone)]
pub struct HttpHeaders {
    headers: HashMap<String, String>,
}

#[allow(dead_code)]
impl HttpHeaders {
    pub fn get<S: AsRef<str>>(&self, field: S) -> Option<&String> {
        self.headers.get(field.as_ref())
    }

    pub fn set<S: AsRef<str>>(&mut self, field: S, value: String) -> Option<String> {
        self.headers.insert(field.as_ref().to_string(), value)
    }

    pub fn unset<S: AsRef<str>>(&mut self, field: S) -> Option<String> {
        self.headers.remove(field.as_ref())
    }

    pub fn iter(&self) -> HttpHeadersIterator {
        HttpHeadersIterator {
            next: (&self.headers).iter(),
        }
    }
}
pub struct HttpHeadersIterator<'a> {
    next: <&'a HashMap<String, String> as IntoIterator>::IntoIter,
}

impl<'a> Iterator for HttpHeadersIterator<'a> {
    type Item = (&'a String, &'a String);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.next()
    }
}

// Cache execution information from datadog agent for Python checks
#[allow(dead_code)] // FIXME temporary
#[derive(Clone)]
pub struct ExecutionContext {
    pub hostname: String,
    pub http_headers: HttpHeaders,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        let http_headers = [
            (
                "User-Agent",
                format!("Datadog Agent/{}", saluki_metadata::get_app_details().version().raw()),
            ),
            ("Content-Type", "application/x-www-form-urlencoded".to_string()),
            ("Accept", "text/html, */*".to_string()),
        ];
        let http_headers = http_headers.into_iter().fold(HttpHeaders::default(), |mut h, v| {
            h.set(v.0.to_string(), v.1);
            h
        });

        Self {
            hostname: "".to_string(),
            http_headers,
        }
    }
}

impl ExecutionContext {
    pub async fn from_environment_provider<E>(environment_provider: &E) -> Self
    where
        E: EnvironmentProvider,
        <E::Host as HostProvider>::Error: std::fmt::Debug,
    {
        let default = Self::default();
        let hostname = environment_provider.host().get_hostname().await.unwrap_or_else(|e| {
            warn!("Failed to get hostname: {:?}", e);
            "".to_string()
        });

        Self { hostname, ..default }
    }
}
