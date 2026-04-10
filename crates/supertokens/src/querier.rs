use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock, RwLock};

use reqwest::Client;
use serde_json::Value;

use crate::constants::{
    API_KEY_HEADER, API_VERSION, API_VERSION_HEADER, RATE_LIMIT_STATUS_CODE, SUPPORTED_CDI_VERSIONS,
};
use crate::error::{raise_general_exception, SuperTokensError};
use crate::types::config::{Host, NetworkInterceptor, NetworkInterceptorRequest};
use crate::user_context::{internal_keys, CoreCallCache, UserContext};
use crate::utils::{get_timestamp_ms, is_4xx_error, is_5xx_error};

/// Global querier state shared across all instances.
struct QuerierGlobal {
    hosts: Vec<Host>,
    api_key: Option<String>,
    network_interceptor: Option<NetworkInterceptor>,
    disable_cache: bool,
    last_tried_index: AtomicUsize,
    api_version: RwLock<Option<String>>,
    global_cache_tag: RwLock<u64>,
    init_called: AtomicBool,
    client: Client,
}

static GLOBAL: OnceLock<Arc<QuerierGlobal>> = OnceLock::new();

/// HTTP client for communicating with the SuperTokens Core service.
///
/// Handles round-robin host selection, retries, rate-limit backoff,
/// API version negotiation, and response caching.
#[derive(Clone)]
pub struct Querier {
    hosts: Vec<Host>,
    rid_to_core: Option<String>,
}

impl Querier {
    /// Initialize the global querier state. Must be called once during SDK init.
    pub fn init(
        hosts: Vec<Host>,
        api_key: Option<String>,
        network_interceptor: Option<NetworkInterceptor>,
        disable_cache: bool,
    ) {
        let _ = GLOBAL.set(Arc::new(QuerierGlobal {
            hosts,
            api_key,
            network_interceptor,
            disable_cache,
            last_tried_index: AtomicUsize::new(0),
            api_version: RwLock::new(None),
            global_cache_tag: RwLock::new(get_timestamp_ms()),
            init_called: AtomicBool::new(true),
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }));
    }

    /// Get a Querier instance. Optionally specify a recipe ID to include in Core requests.
    pub fn get_instance(rid_to_core: Option<String>) -> Result<Self, SuperTokensError> {
        let global = GLOBAL.get().ok_or_else(|| {
            raise_general_exception("Querier not initialized. Call Supertokens::init() first.")
        })?;

        if !global.init_called.load(Ordering::Relaxed) {
            return Err(raise_general_exception("Querier not initialized"));
        }

        Ok(Self {
            hosts: global.hosts.clone(),
            rid_to_core,
        })
    }

    fn global() -> &'static Arc<QuerierGlobal> {
        GLOBAL.get().expect("Querier not initialized")
    }

    /// Negotiate the API version with the SuperTokens Core.
    pub async fn get_api_version(
        &self,
        _user_context: &mut UserContext,
    ) -> Result<String, SuperTokensError> {
        // Check cache first
        {
            let cached = Self::global().api_version.read().unwrap();
            if let Some(ref v) = *cached {
                return Ok(v.clone());
            }
        }

        let response = self
            .send_request_helper(
                &crate::normalised_url_path::NormalisedURLPath::new(API_VERSION)?,
                "get",
                |url, headers| {
                    let global = Self::global();
                    let client = global.client.clone();
                    Box::pin(async move {
                        let resp = client
                            .get(&url)
                            .headers(to_reqwest_headers(&headers))
                            .send()
                            .await?;
                        response_to_value(resp).await
                    })
                },
                self.hosts.len(),
                &mut HashMap::new(),
            )
            .await?;

        let core_versions: Vec<String> = response
            .get("versions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let core_refs: Vec<&str> = core_versions.iter().map(|s| s.as_str()).collect();
        let api_version = crate::utils::find_max_version(SUPPORTED_CDI_VERSIONS, &core_refs)
            .ok_or_else(|| SuperTokensError::IncompatibleCdiVersion {
                sdk_versions: SUPPORTED_CDI_VERSIONS.join(", "),
                core_versions: core_versions.join(", "),
            })?;

        // Cache result
        {
            let mut cached = Self::global().api_version.write().unwrap();
            *cached = Some(api_version.clone());
        }

        Ok(api_version)
    }

    /// Send a GET request to the Core. Supports response caching.
    pub async fn send_get_request(
        &self,
        path: &crate::normalised_url_path::NormalisedURLPath,
        params: Option<HashMap<String, String>>,
        user_context: &mut UserContext,
    ) -> Result<Value, SuperTokensError> {
        let api_version = self.get_api_version(user_context).await?;
        let global = Self::global();

        let mut headers = self.build_headers(&api_version);

        // Build cache key
        let cache_key = if !global.disable_cache {
            let mut sorted_params: Vec<_> = params
                .as_ref()
                .map(|p| p.iter().collect::<Vec<_>>())
                .unwrap_or_default();
            sorted_params.sort_by_key(|(k, _)| (*k).clone());
            let mut sorted_headers: Vec<_> = headers.iter().collect::<Vec<_>>();
            sorted_headers.sort_by_key(|(k, _)| (*k).clone());
            Some(format!(
                "{}:{}:{}",
                path.get_as_string_dangerous(),
                sorted_params
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("&"),
                sorted_headers
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("&")
            ))
        } else {
            None
        };

        // Check cache
        if let Some(ref key) = cache_key {
            if let Some(cached) = self.get_from_cache(key, user_context) {
                return Ok(cached);
            }
        }

        // Apply network interceptor
        if let Some(ref interceptor) = global.network_interceptor {
            let req = NetworkInterceptorRequest {
                url: String::new(), // filled by send_request_helper
                method: "get".to_string(),
                headers: headers.clone(),
                params: params.clone(),
                body: None,
            };
            let modified = interceptor(req).await;
            headers = modified.headers;
        }

        let params_clone = params.clone();
        let headers_clone = headers.clone();

        let response = self
            .send_request_helper(
                path,
                "get",
                move |url, mut h| {
                    h.extend(headers_clone.clone());
                    let params = params_clone.clone();
                    let global = Self::global();
                    let client = global.client.clone();
                    Box::pin(async move {
                        let mut req = client.get(&url).headers(to_reqwest_headers(&h));
                        if let Some(p) = params {
                            req = req.query(
                                &p.iter()
                                    .map(|(k, v)| (k.as_str(), v.as_str()))
                                    .collect::<Vec<_>>(),
                            );
                        }
                        let resp = req.send().await?;
                        response_to_value(resp).await
                    })
                },
                self.hosts.len(),
                &mut HashMap::new(),
            )
            .await?;

        // Store in cache
        if let Some(ref key) = cache_key {
            self.store_in_cache(key, &response, user_context);
        }

        Ok(response)
    }

    /// Send a POST request to the Core. Invalidates cache.
    pub async fn send_post_request(
        &self,
        path: &crate::normalised_url_path::NormalisedURLPath,
        data: Option<Value>,
        user_context: &mut UserContext,
    ) -> Result<Value, SuperTokensError> {
        self.invalidate_core_call_cache(user_context, true);

        let api_version = self.get_api_version(user_context).await?;
        let global = Self::global();
        let mut headers = self.build_headers(&api_version);
        headers.insert(
            "content-type".to_string(),
            "application/json; charset=utf-8".to_string(),
        );

        if let Some(ref interceptor) = global.network_interceptor {
            let req = NetworkInterceptorRequest {
                url: String::new(),
                method: "post".to_string(),
                headers: headers.clone(),
                params: None,
                body: data.clone(),
            };
            let modified = interceptor(req).await;
            headers = modified.headers;
        }

        let data_clone = data.clone();
        let headers_clone = headers.clone();

        self.send_request_helper(
            path,
            "post",
            move |url, mut h| {
                h.extend(headers_clone.clone());
                let body = data_clone.clone();
                let global = Self::global();
                let client = global.client.clone();
                Box::pin(async move {
                    let mut req = client.post(&url).headers(to_reqwest_headers(&h));
                    if let Some(b) = body {
                        req = req.json(&b);
                    }
                    let resp = req.send().await?;
                    response_to_value(resp).await
                })
            },
            self.hosts.len(),
            &mut HashMap::new(),
        )
        .await
    }

    /// Send a DELETE request to the Core. Invalidates cache.
    pub async fn send_delete_request(
        &self,
        path: &crate::normalised_url_path::NormalisedURLPath,
        params: Option<HashMap<String, String>>,
        user_context: &mut UserContext,
    ) -> Result<Value, SuperTokensError> {
        self.invalidate_core_call_cache(user_context, true);

        let api_version = self.get_api_version(user_context).await?;
        let global = Self::global();
        let mut headers = self.build_headers(&api_version);

        if let Some(ref interceptor) = global.network_interceptor {
            let req = NetworkInterceptorRequest {
                url: String::new(),
                method: "delete".to_string(),
                headers: headers.clone(),
                params: params.clone(),
                body: None,
            };
            let modified = interceptor(req).await;
            headers = modified.headers;
        }

        let params_clone = params.clone();
        let headers_clone = headers.clone();

        self.send_request_helper(
            path,
            "delete",
            move |url, mut h| {
                h.extend(headers_clone.clone());
                let params = params_clone.clone();
                let global = Self::global();
                let client = global.client.clone();
                Box::pin(async move {
                    let mut req = client.delete(&url).headers(to_reqwest_headers(&h));
                    if let Some(p) = params {
                        req = req.query(
                            &p.iter()
                                .map(|(k, v)| (k.as_str(), v.as_str()))
                                .collect::<Vec<_>>(),
                        );
                    }
                    let resp = req.send().await?;
                    response_to_value(resp).await
                })
            },
            self.hosts.len(),
            &mut HashMap::new(),
        )
        .await
    }

    /// Send a PUT request to the Core. Invalidates cache.
    pub async fn send_put_request(
        &self,
        path: &crate::normalised_url_path::NormalisedURLPath,
        data: Option<Value>,
        params: Option<HashMap<String, String>>,
        user_context: &mut UserContext,
    ) -> Result<Value, SuperTokensError> {
        self.invalidate_core_call_cache(user_context, true);

        let api_version = self.get_api_version(user_context).await?;
        let global = Self::global();
        let mut headers = self.build_headers(&api_version);
        headers.insert(
            "content-type".to_string(),
            "application/json; charset=utf-8".to_string(),
        );

        if let Some(ref interceptor) = global.network_interceptor {
            let req = NetworkInterceptorRequest {
                url: String::new(),
                method: "put".to_string(),
                headers: headers.clone(),
                params: params.clone(),
                body: data.clone(),
            };
            let modified = interceptor(req).await;
            headers = modified.headers;
        }

        let data_clone = data.clone();
        let params_clone = params.clone();
        let headers_clone = headers.clone();

        self.send_request_helper(
            path,
            "put",
            move |url, mut h| {
                h.extend(headers_clone.clone());
                let body = data_clone.clone();
                let params = params_clone.clone();
                let global = Self::global();
                let client = global.client.clone();
                Box::pin(async move {
                    let mut req = client.put(&url).headers(to_reqwest_headers(&h));
                    if let Some(p) = params {
                        req = req.query(
                            &p.iter()
                                .map(|(k, v)| (k.as_str(), v.as_str()))
                                .collect::<Vec<_>>(),
                        );
                    }
                    if let Some(b) = body {
                        req = req.json(&b);
                    }
                    let resp = req.send().await?;
                    response_to_value(resp).await
                })
            },
            self.hosts.len(),
            &mut HashMap::new(),
        )
        .await
    }

    /// Get all possible Core URLs for a given path.
    pub fn get_all_core_urls_for_path(
        &self,
        path: &crate::normalised_url_path::NormalisedURLPath,
    ) -> Vec<String> {
        self.hosts
            .iter()
            .map(|host| {
                let base = host.domain.get_as_string_dangerous();
                let bp = host.base_path.get_as_string_dangerous();
                let p = path.get_as_string_dangerous();
                format!("{}{}{}", base, bp, p)
            })
            .collect()
    }

    /// Invalidate the per-request Core call cache.
    pub fn invalidate_core_call_cache(&self, user_context: &mut UserContext, upd_global_tag: bool) {
        user_context.remove(internal_keys::CORE_CALL_CACHE);

        if upd_global_tag {
            let keep_alive = user_context
                .get::<bool>(internal_keys::KEEP_CACHE_ALIVE)
                .copied()
                .unwrap_or(false);
            if !keep_alive {
                if let Some(global) = GLOBAL.get() {
                    let mut tag = global.global_cache_tag.write().unwrap();
                    *tag = get_timestamp_ms();
                }
            }
        }
    }

    // ---------- Private helpers ----------

    fn build_headers(&self, api_version: &str) -> HashMap<String, String> {
        let global = Self::global();
        let mut headers = HashMap::new();
        headers.insert(API_VERSION_HEADER.to_string(), api_version.to_string());
        if let Some(ref key) = global.api_key {
            headers.insert(API_KEY_HEADER.to_string(), key.clone());
        }
        if let Some(ref rid) = self.rid_to_core {
            headers.insert(crate::constants::RID_KEY_HEADER.to_string(), rid.clone());
        }
        headers
    }

    fn get_from_cache(&self, key: &str, user_context: &UserContext) -> Option<Value> {
        let cache = user_context.get::<CoreCallCache>(internal_keys::CORE_CALL_CACHE)?;
        cache.get(key).cloned()
    }

    fn store_in_cache(&self, key: &str, value: &Value, user_context: &mut UserContext) {
        if Self::global().disable_cache {
            return;
        }
        let mut cache = user_context
            .get::<CoreCallCache>(internal_keys::CORE_CALL_CACHE)
            .cloned()
            .unwrap_or_default();
        cache.insert(key.to_string(), value.clone());
        user_context.insert(internal_keys::CORE_CALL_CACHE, cache);
    }

    /// Core retry/round-robin logic.
    fn send_request_helper<'a, F>(
        &'a self,
        path: &'a crate::normalised_url_path::NormalisedURLPath,
        _method: &'a str,
        http_function: F,
        no_of_tries: usize,
        retry_info_map: &'a mut HashMap<String, u32>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, SuperTokensError>> + Send + 'a>,
    >
    where
        F: Fn(
                String,
                HashMap<String, String>,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<(Value, u16), reqwest::Error>> + Send>,
            > + Clone
            + Send
            + Sync
            + 'a,
    {
        Box::pin(async move {
            if no_of_tries == 0 {
                return Err(raise_general_exception(
                    "All SuperTokens Core hosts failed. Please ensure the Core is running.",
                ));
            }

            let global = Self::global();
            let current_index =
                global.last_tried_index.fetch_add(1, Ordering::Relaxed) % self.hosts.len();
            let host = &self.hosts[current_index];

            let url = format!(
                "{}{}{}",
                host.domain.get_as_string_dangerous(),
                host.base_path.get_as_string_dangerous(),
                path.get_as_string_dangerous()
            );

            let max_retries: u32 = 5;
            let attempts = retry_info_map.entry(url.clone()).or_insert(0);

            let headers = HashMap::new();

            match http_function(url.clone(), headers).await {
                Ok((body, status)) => {
                    if status == RATE_LIMIT_STATUS_CODE {
                        *attempts += 1;
                        if *attempts >= max_retries {
                            return Err(SuperTokensError::Querier {
                                message: format!(
                                    "Rate limit exceeded after {} retries for {}",
                                    max_retries, url
                                ),
                                status_code: status,
                                response_text: Some(body.to_string()),
                            });
                        }
                        let delay_ms = 10 + (*attempts as u64) * 250;
                        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                        return self
                            .send_request_helper(
                                path,
                                _method,
                                http_function,
                                no_of_tries,
                                retry_info_map,
                            )
                            .await;
                    }

                    if is_4xx_error(status) || is_5xx_error(status) {
                        let msg = body
                            .get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown error")
                            .to_string();
                        return Err(SuperTokensError::Querier {
                            message: msg,
                            status_code: status,
                            response_text: Some(body.to_string()),
                        });
                    }

                    Ok(body)
                }
                Err(e) => {
                    if e.is_connect() || e.is_timeout() {
                        // Retry on connection/timeout errors
                        self.send_request_helper(
                            path,
                            _method,
                            http_function,
                            no_of_tries - 1,
                            retry_info_map,
                        )
                        .await
                    } else {
                        Err(SuperTokensError::Network(e))
                    }
                }
            }
        }) // end Box::pin
    }

    /// Reset global state (testing only).
    #[cfg(test)]
    pub fn reset() {
        // OnceLock doesn't support reset, but in tests we can use a different approach
    }
}

// ---------- Helper functions ----------

fn to_reqwest_headers(headers: &HashMap<String, String>) -> reqwest::header::HeaderMap {
    let mut map = reqwest::header::HeaderMap::new();
    for (key, value) in headers {
        if let (Ok(name), Ok(val)) = (
            reqwest::header::HeaderName::from_bytes(key.as_bytes()),
            reqwest::header::HeaderValue::from_str(value),
        ) {
            map.insert(name, val);
        }
    }
    map
}

async fn response_to_value(resp: reqwest::Response) -> Result<(Value, u16), reqwest::Error> {
    let status = resp.status().as_u16();
    let text = resp.text().await?;
    let body = serde_json::from_str::<Value>(&text)
        .unwrap_or_else(|_| serde_json::json!({ "_text": text }));
    Ok((body, status))
}
