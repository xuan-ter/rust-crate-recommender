use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RecommendationRequest {
    pub query: String,
    pub context: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RecommendationResponse {
    pub crates: Vec<CrateInfo>,
    pub explanation: String,
}

#[derive(Debug, Serialize)]
pub struct CrateInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    pub downloads: i32,
    pub last_updated: String,
    pub score: f32,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CratesIoResponse {
    pub crates: Vec<CratesIoCrate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CratesIoCrate {
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub downloads: i32,
    pub updated_at: String,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub keywords: Vec<String>,
} 