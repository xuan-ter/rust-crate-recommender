use std::error::Error as StdError;
use async_openai::{
    types::{
        CreateChatCompletionRequestArgs, 
        ChatCompletionRequestMessage, 
        Role,
        ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage,
        ChatCompletionRequestUserMessageContent,
    },
    Client,
    config::OpenAIConfig,
};
use reqwest::Client as ReqwestClient;
use serde_json::Value;
use crate::models::{RecommendationRequest, RecommendationResponse, CrateInfo, CratesIoResponse};
use crate::scorer::{CrateScorer, CrateMetrics};
use chrono::{DateTime, Utc, Duration};

pub struct RecommendationService {
    client: Client<OpenAIConfig>,
    crates_io_client: ReqwestClient,
    scorer: CrateScorer,
}

impl RecommendationService {
    pub fn new() -> Self {
        // 创建支持代理的 reqwest 客户端
        let http_client = ReqwestClient::builder()
            // 启用代理
            .proxy(reqwest::Proxy::http("http://127.0.0.1:7890").unwrap())
            .proxy(reqwest::Proxy::https("http://127.0.0.1:7890").unwrap())
            .build()
            .unwrap();

        // 使用自定义的 reqwest 客户端创建 OpenAI 客户端
        let client = Client::new().with_http_client(http_client.clone());

        Self {
            client,
            crates_io_client: http_client,
            scorer: CrateScorer::new(),
        }
    }

    pub async fn get_recommendations(&self, request: RecommendationRequest) -> Result<RecommendationResponse, Box<dyn StdError + Send + Sync>> {
        // 首先尝试使用 OpenAI 分析需求
        match self.analyze_requirements(&request).await {
            Ok(keywords) => {
                // 搜索 crates
                let crates = self.search_crates(&keywords).await?;
                
                // 获取每个 crate 的详细指标
                let mut evaluated_crates = Vec::new();
                for crate_info in crates {
                    if let Ok(metrics) = self.get_crate_metrics(&crate_info.name).await {
                        let score = self.scorer.calculate_total_score(&metrics);
                        let mut evaluated_crate = crate_info;
                        evaluated_crate.score = score;
                        evaluated_crates.push(evaluated_crate);
                    }
                }
                
                // 按评分排序
                evaluated_crates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
                
                Ok(RecommendationResponse {
                    crates: evaluated_crates,
                    explanation: "基于多个维度的评分，我们为您推荐以下 crates。".to_string(),
                })
            },
            Err(_) => {
                // 如果 OpenAI API 调用失败，使用本地推荐
                self.get_local_recommendations(&request).await
            }
        }
    }

    async fn analyze_requirements(&self, request: &RecommendationRequest) -> Result<String, Box<dyn StdError + Send + Sync>> {
        let messages = vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessage {
                    content: Some("You are a Rust expert. Analyze the user's requirements and provide a detailed description of what kind of crates would be suitable.".to_string()),
                    name: None,
                    role: Role::System,
                }
            ),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: Some(ChatCompletionRequestUserMessageContent::Text(
                        format!(
                            "User request: {}\nContext: {}",
                            request.query,
                            request.context.as_deref().unwrap_or("")
                        )
                    )),
                    name: None,
                    role: Role::User,
                }
            ),
        ];

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-3.5-turbo")
            .messages(messages)
            .build()?;

        let response = self.client.chat().create(request).await?;
        Ok(response.choices[0].message.content.clone().unwrap_or_default())
    }

    async fn search_crates(&self, analysis: &str) -> Result<Vec<CrateInfo>, Box<dyn StdError + Send + Sync>> {
        // 从 crates.io API 搜索 crates
        let response = self.crates_io_client
            .get("https://crates.io/api/v1/crates")
            .header("User-Agent", "rust-crate-recommender/0.1.0")
            .query(&[("q", analysis)])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to search crates: {}", response.status()).into());
        }

        let response = response.json::<CratesIoResponse>().await?;

        Ok(response.crates.into_iter().map(|c| CrateInfo {
            name: c.name,
            description: c.description.unwrap_or_default(),
            version: c.version,
            downloads: c.downloads,
            last_updated: c.updated_at,
            score: 0.0, // 将在评估阶段计算
            repository: c.repository,
            documentation: c.documentation,
            keywords: c.keywords,
        }).collect())
    }

    async fn get_crate_metrics(&self, crate_name: &str) -> Result<CrateMetrics, Box<dyn StdError + Send + Sync>> {
        // 从 crates.io API 获取 crate 详细信息
        let response = self.crates_io_client
            .get(&format!("https://crates.io/api/v1/crates/{}", crate_name))
            .header("User-Agent", "rust-crate-recommender/0.1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to get crate info: {}", response.status()).into());
        }

        let crate_data: Value = response.json().await?;
        
        // 从 GitHub API 获取仓库信息
        let repo_url = crate_data["crate"]["repository"].as_str().unwrap_or("");
        let (last_commit, issue_response_time) = if !repo_url.is_empty() {
            self.get_github_metrics(repo_url).await.unwrap_or((None, None))
        } else {
            (None, None)
        };

        // 构建 CrateMetrics
        Ok(CrateMetrics {
            last_commit,
            issue_response_time,
            releases_per_year: self.calculate_releases_per_year(&crate_data),
            readme_length: crate_data["crate"]["readme"].as_str().map_or(0, |r| r.len()),
            has_docs_rs: crate_data["crate"]["documentation"].as_str().map_or(false, |d| d.contains("docs.rs")),
            has_examples: crate_data["crate"]["examples"].as_array().map_or(false, |e| !e.is_empty()),
            cargo_audit_passed: true, // 需要实现 cargo-audit 检查
            rustsec_vulnerabilities: 0, // 需要实现 RustSec 检查
            dependent_count: crate_data["crate"]["dependent_count"].as_i64().unwrap_or(0) as usize,
            recent_downloads: crate_data["crate"]["recent_downloads"].as_i64().unwrap_or(0) as usize,
            total_downloads: crate_data["crate"]["downloads"].as_i64().unwrap_or(0) as usize,
            license: crate_data["crate"]["license"].as_str().unwrap_or("").to_string(),
        })
    }

    async fn get_github_metrics(&self, repo_url: &str) -> Result<(Option<DateTime<Utc>>, Option<Duration>), Box<dyn StdError + Send + Sync>> {
        // 这里需要实现 GitHub API 调用
        // 1. 获取最近提交时间
        // 2. 获取 issue 响应时间
        // 暂时返回空值
        Ok((None, None))
    }

    fn calculate_releases_per_year(&self, crate_data: &Value) -> f32 {
        // 从版本历史计算每年发布次数
        if let Some(versions) = crate_data["versions"].as_array() {
            let total_releases = versions.len() as f32;
            let years = 1.0; // 假设为一年
            total_releases / years
        } else {
            0.0
        }
    }

    // 添加本地推荐功能
    async fn get_local_recommendations(&self, request: &RecommendationRequest) -> Result<RecommendationResponse, Box<dyn StdError + Send + Sync>> {
        // 根据关键词匹配本地推荐的 crates
        let keywords = request.query.to_lowercase();
        let mut recommended_crates = Vec::new();

        // 定义一些常用的 crates 及其关键词
        let common_crates = vec![
            ("serde", "json,serialization,deserialization,data"),
            ("tokio", "async,concurrent,async-await,runtime"),
            ("axum", "web,http,api,server,framework"),
            ("reqwest", "http,client,request,api"),
            ("chrono", "date,time,datetime,timezone"),
            ("clap", "cli,command-line,argument,parser"),
            ("sqlx", "sql,database,postgres,mysql"),
            ("tracing", "logging,debug,diagnostics"),
            ("anyhow", "error,handling,result"),
            ("futures", "async,stream,future"),
            ("rand", "random,number,generator"),
            ("regex", "regular,expression,pattern,matching"),
            ("serde_json", "json,serialization"),
            ("async-trait", "async,trait,await"),
            ("thiserror", "error,handling,derive"),
            ("uuid", "unique,identifier,id"),
            ("env_logger", "logging,environment"),
            ("dotenv", "environment,configuration,env"),
            ("cargo", "package,manager,dependency"),
            ("cargo-edit", "cargo,add,remove,upgrade"),
        ];

        // 根据关键词匹配 crates
        for (name, crate_keywords) in common_crates {
            if keywords.contains(&name.to_string()) || 
               crate_keywords.split(',').any(|k| keywords.contains(k)) {
                // 获取 crate 信息
                if let Ok(crate_info) = self.get_crate_info(name).await {
                    recommended_crates.push(crate_info);
                }
            }
        }

        // 如果没有找到匹配的 crates，返回一些通用的推荐
        if recommended_crates.is_empty() {
            recommended_crates = vec![
                self.get_crate_info("serde").await?,
                self.get_crate_info("tokio").await?,
                self.get_crate_info("anyhow").await?,
            ];
        }

        // 为每个 crate 分配一个基于关键词匹配度的分数
        for crate_info in &mut recommended_crates {
            let match_score = if keywords.contains(&crate_info.name.to_string()) {
                0.9
            } else if crate_info.description.to_lowercase().contains(&keywords) {
                0.7
            } else {
                0.5
            };
            crate_info.score = match_score;
        }

        // 按分数排序
        recommended_crates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(RecommendationResponse {
            crates: recommended_crates,
            explanation: "由于 OpenAI API 不可用，这是基于关键词匹配的本地推荐结果。".to_string(),
        })
    }

    // 获取单个 crate 的信息
    async fn get_crate_info(&self, name: &str) -> Result<CrateInfo, Box<dyn StdError + Send + Sync>> {
        let url = format!("https://crates.io/api/v1/crates/{}", name);
        let response = self.crates_io_client
            .get(&url)
            .header("User-Agent", "rust-crate-recommender/0.1.0")
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to get crate info: {}", response.status()).into());
        }
        
        let crate_data: Value = response.json().await?;
        
        let crate_info = crate_data["crate"].as_object()
            .ok_or_else(|| format!("Invalid crate data for {}", name))?;
        
        Ok(CrateInfo {
            name: name.to_string(),
            description: crate_info["description"].as_str().unwrap_or("").to_string(),
            version: crate_info["newest_version"].as_str().unwrap_or("").to_string(),
            downloads: crate_info["downloads"].as_i64().unwrap_or(0) as i32,
            last_updated: crate_info["updated_at"].as_str().unwrap_or("").to_string(),
            score: 0.0,
            repository: crate_info["repository"].as_str().map(|s| s.to_string()),
            documentation: crate_info["documentation"].as_str().map(|s| s.to_string()),
            keywords: vec![],
        })
    }
} 