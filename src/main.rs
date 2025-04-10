mod models;
mod services;

use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::State,
    http::{Method, StatusCode},
};
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use dotenv::dotenv;
use env_logger;

use crate::models::{RecommendationRequest, RecommendationResponse};
use crate::services::RecommendationService;

// 应用状态
struct AppState {
    recommendation_service: Arc<RecommendationService>,
}

#[tokio::main]
async fn main() {
    // 加载环境变量
    dotenv().ok();
    
    // 初始化日志
    env_logger::init();

    // 创建推荐服务
    let recommendation_service = Arc::new(RecommendationService::new());

    // 创建应用状态
    let app_state = Arc::new(AppState {
        recommendation_service,
    });

    // 配置 CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    // 创建路由
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/recommend", post(recommend_crates))
        .layer(cors)
        .with_state(app_state);

    // 启动服务器
    let addr = "127.0.0.1:3000";
    println!("Server running on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// 健康检查端点
async fn health_check() -> &'static str {
    "OK"
}

// 推荐 crates 的主要处理函数
#[axum::debug_handler]
async fn recommend_crates(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RecommendationRequest>,
) -> Result<Json<RecommendationResponse>, (StatusCode, String)> {
    match state.recommendation_service.get_recommendations(request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            let error_message = if e.to_string().contains("insufficient_quota") {
                "OpenAI API 配额已用完。请稍后再试或联系管理员。".to_string()
            } else if e.to_string().contains("connection") {
                "无法连接到 OpenAI API。请检查网络连接或代理设置。".to_string()
            } else {
                format!("发生错误: {}", e)
            };
            
            eprintln!("Error getting recommendations: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_message))
        }
    }
} 