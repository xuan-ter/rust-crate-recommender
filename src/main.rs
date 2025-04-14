mod models;
mod services;
mod scorer;

use axum::{
    routing::{get, post},
    Router,
    extract::State,
    http::{HeaderValue, Method, StatusCode},
    response::{Json, IntoResponse},
};
use tower_http::cors::CorsLayer;
use std::sync::Arc;
use services::RecommendationService;
use models::{RecommendationRequest, RecommendationResponse};
use dotenv::dotenv;
use env_logger;

#[derive(Clone)]
struct AppState {
    recommendation_service: Arc<RecommendationService>,
}

#[tokio::main]
async fn main() {
    // 加载环境变量
    dotenv().ok();
    
    // 初始化日志
    env_logger::init();

    // 创建应用状态
    let state = AppState {
        recommendation_service: Arc::new(RecommendationService::new()),
    };

    // 配置 CORS
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([http::header::CONTENT_TYPE]);

    // 创建路由
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/recommend", post(recommend_crates))
        .layer(cors)
        .with_state(state);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

// 健康检查端点
async fn health_check() -> &'static str {
    "OK"
}

// 推荐 crates 的主要处理函数
async fn recommend_crates(
    State(state): State<AppState>,
    Json(request): Json<RecommendationRequest>,
) -> impl IntoResponse {
    match state.recommendation_service.get_recommendations(request).await {
        Ok(response) => {
            if response.crates.is_empty() {
                (
                    StatusCode::NOT_FOUND,
                    Json(RecommendationResponse {
                        crates: vec![],
                        explanation: "未找到匹配的 crates。请尝试使用不同的关键词或提供更多上下文信息。".to_string(),
                    }),
                ).into_response()
            } else {
                (StatusCode::OK, Json(response)).into_response()
            }
        }
        Err(e) => {
            eprintln!("Error getting recommendations: {}", e);
            let (status, message) = if e.to_string().contains("insufficient_quota") {
                (StatusCode::SERVICE_UNAVAILABLE, "OpenAI API 配额已用完。请稍后再试或联系管理员。")
            } else if e.to_string().contains("connection") {
                (StatusCode::BAD_GATEWAY, "无法连接到外部服务。请检查网络连接或代理设置。")
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, "服务器内部错误。请稍后重试。")
            };

            (
                status,
                Json(RecommendationResponse {
                    crates: vec![],
                    explanation: message.to_string(),
                }),
            ).into_response()
        }
    }
} 