use crate::standout::app::{
    http::{Method, RequestBuilder},
    types::{AppError, ErrorCode},
};
use serde_json::Value;
use std::collections::HashMap;

/// HTTP client for making API requests
#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
    headers: HashMap<String, String>,
}

impl ApiClient {
    /// Create a new ApiClient from connection data
    pub fn new(connection_data: &Value) -> Result<Self, AppError> {
        let base_url = connection_data
            .get("base_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError {
                code: ErrorCode::Misconfigured,
                message: "base_url not found in connection data".to_string(),
            })?
            .to_string();

        let headers_obj = connection_data
            .get("headers")
            .and_then(|v| v.as_object())
            .ok_or_else(|| AppError {
                code: ErrorCode::Misconfigured,
                message: "Headers not found in connection data".to_string(),
            })?;

        let mut headers = HashMap::new();
        for (key, value) in headers_obj {
            if let Some(header_value) = value.as_str() {
                headers.insert(key.clone(), header_value.to_string());
            }
        }

        Ok(ApiClient { base_url, headers })
    }

    /// Make a GET request and return the response body
    #[allow(dead_code)]
    pub fn get(&self, endpoint: &str) -> Result<Value, AppError> {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut request_builder = RequestBuilder::new().method(Method::Get).url(&url);

        // Add headers
        for (key, value) in &self.headers {
            request_builder = request_builder.header(key, value);
        }

        let response = request_builder.send().map_err(|_err| AppError {
            code: ErrorCode::Other,
            message: "Request failed".to_string(),
        })?;

        if response.status != 200 {
            return Err(AppError {
                code: ErrorCode::Other,
                message: format!(
                    "API request failed with status: {} - URL: {} - Response: {}",
                    response.status, url, response.body
                ),
            });
        }

        serde_json::from_str(&response.body).map_err(|_e| AppError {
            code: ErrorCode::MalformedResponse,
            message: "Invalid API response format".to_string(),
        })
    }

    /// Make a POST request with JSON body and return the response body
    #[allow(dead_code)]
    pub fn post(&self, endpoint: &str, body: &Value) -> Result<Value, AppError> {
        let body_str = serde_json::to_string(body).map_err(|e| AppError {
            code: ErrorCode::Other,
            message: format!("Failed to serialize JSON body: {}", e),
        })?;

        let url = format!("{}{}", self.base_url, endpoint);
        let mut request_builder = RequestBuilder::new().method(Method::Post).url(&url);

        // Add headers
        for (key, value) in &self.headers {
            request_builder = request_builder.header(key, value);
        }

        let response = request_builder
            .body(&body_str)
            .send()
            .map_err(|_err| AppError {
                code: ErrorCode::Other,
                message: "Request failed".to_string(),
            })?;

        if !(200..300).contains(&response.status) {
            return Err(AppError {
                code: ErrorCode::Other,
                message: format!(
                    "API request failed with status: {} - URL: {} - Response: {}",
                    response.status, url, response.body
                ),
            });
        }

        serde_json::from_str(&response.body).map_err(|_e| AppError {
            code: ErrorCode::MalformedResponse,
            message: "Invalid API response format".to_string(),
        })
    }

    /// Make a PATCH request with JSON body and return the response body
    #[allow(dead_code)]
    pub fn patch(&self, endpoint: &str, body: &Value) -> Result<Value, AppError> {
        let body_str = serde_json::to_string(body).map_err(|e| AppError {
            code: ErrorCode::Other,
            message: format!("Failed to serialize JSON body: {}", e),
        })?;

        let url = format!("{}{}", self.base_url, endpoint);
        let mut request_builder = RequestBuilder::new().method(Method::Patch).url(&url);

        // Add headers
        for (key, value) in &self.headers {
            request_builder = request_builder.header(key, value);
        }

        let response = request_builder
            .body(&body_str)
            .send()
            .map_err(|_err| AppError {
                code: ErrorCode::Other,
                message: "Request failed".to_string(),
            })?;

        if !(200..300).contains(&response.status) {
            return Err(AppError {
                code: ErrorCode::Other,
                message: format!(
                    "API request failed with status: {} - URL: {} - Response: {}",
                    response.status, url, response.body
                ),
            });
        }

        serde_json::from_str(&response.body).map_err(|_e| AppError {
            code: ErrorCode::MalformedResponse,
            message: "Invalid API response format".to_string(),
        })
    }

    /// Make a PUT request with JSON body and return the response body
    #[allow(dead_code)]
    pub fn put(&self, endpoint: &str, body: &Value) -> Result<Value, AppError> {
        let body_str = serde_json::to_string(body).map_err(|e| AppError {
            code: ErrorCode::Other,
            message: format!("Failed to serialize JSON body: {}", e),
        })?;

        let url = format!("{}{}", self.base_url, endpoint);
        let mut request_builder = RequestBuilder::new().method(Method::Put).url(&url);

        // Add headers
        for (key, value) in &self.headers {
            request_builder = request_builder.header(key, value);
        }

        let response = request_builder
            .body(&body_str)
            .send()
            .map_err(|_err| AppError {
                code: ErrorCode::Other,
                message: "Request failed".to_string(),
            })?;

        if !(200..300).contains(&response.status) {
            return Err(AppError {
                code: ErrorCode::Other,
                message: format!(
                    "API request failed with status: {} - URL: {} - Response: {}",
                    response.status, url, response.body
                ),
            });
        }

        serde_json::from_str(&response.body).map_err(|_e| AppError {
            code: ErrorCode::MalformedResponse,
            message: "Invalid API response format".to_string(),
        })
    }

    /// Make a DELETE request and return the response body
    #[allow(dead_code)]
    pub fn delete(&self, endpoint: &str) -> Result<Value, AppError> {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut request_builder = RequestBuilder::new().method(Method::Delete).url(&url);

        // Add headers
        for (key, value) in &self.headers {
            request_builder = request_builder.header(key, value);
        }

        let response = request_builder.send().map_err(|_err| AppError {
            code: ErrorCode::Other,
            message: "Request failed".to_string(),
        })?;

        if !(200..300).contains(&response.status) {
            return Err(AppError {
                code: ErrorCode::Other,
                message: format!(
                    "API request failed with status: {} - URL: {} - Response: {}",
                    response.status, url, response.body
                ),
            });
        }

        serde_json::from_str(&response.body).map_err(|_e| AppError {
            code: ErrorCode::MalformedResponse,
            message: "Invalid API response format".to_string(),
        })
    }
}
