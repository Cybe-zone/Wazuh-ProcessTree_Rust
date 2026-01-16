use crate::models::*;
use gloo::net::http::Request;
use wasm_bindgen::JsValue;

#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            base_url: String::from(""),
        }
    }

    pub async fn check_session(&self) -> Result<bool, String> {
        let response = Request::get("/api/auth/check")
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.ok() {
            return Ok(false);
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))?;

        Ok(json["data"]["authenticated"].as_bool().unwrap_or(false))
    }

    pub async fn analyze_process(&self, request: AnalysisRequest) -> Result<AnalysisData, String> {
        let response = Request::post("/api/process/analyze")
            .header("Content-Type", "application/json")
            .json(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.ok() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Analysis failed: {}", error_text));
        }

        let result: AnalysisResponse = response
            .json()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))?;

        if result.success {
            result.data.ok_or_else(|| "No data in response".to_string())
        } else {
            Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }

    pub async fn get_classifications(&self) -> Result<Vec<ProcessClassification>, String> {
        let response = Request::get("/api/classifications")
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.ok() {
            return Err("Failed to load classifications".to_string());
        }

        let result: ClassificationsResponse = response
            .json()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))?;

        if result.success {
            result.data.ok_or_else(|| "No data in response".to_string())
        } else {
            Err("Failed to load classifications".to_string())
        }
    }

    pub async fn logout(&self) -> Result<(), String> {
        Request::post("/api/auth/logout")
            .send()
            .await
            .map_err(|e| format!("Logout failed: {}", e))?;
        Ok(())
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}
