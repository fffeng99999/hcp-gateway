use crate::common::state;
use crate::{api, middleware};
use axum::{
    middleware::from_fn,
    routing::{any, delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::request_id::{MakeRequestUuid, SetRequestIdLayer};

pub fn create_router(app_state: Arc<state::AppState>) -> Router {
    // Protected API routes (require Auth header)
    let protected_routes = Router::new()
        // Consensus Module
        .nest(
            "/consensus",
            Router::new()
                .route("/algorithms", get(api::consensus::get_algorithms))
                .route("/config", get(api::consensus::get_config))
                .route("/select", post(api::consensus::select_algorithm))
                .route("/parameters", put(api::consensus::update_parameters)),
        )
        // Benchmark Module
        .nest(
            "/benchmarks",
            Router::new()
                .route(
                    "/",
                    get(api::benchmark::list_benchmarks).post(api::benchmark::create_benchmark),
                )
                .route(
                    "/:id",
                    get(api::benchmark::get_benchmark).delete(api::benchmark::delete_benchmark),
                )
                .route("/:id/start", post(api::benchmark::start_benchmark))
                .route("/:id/stop", post(api::benchmark::stop_benchmark))
                .route("/:id/pause", post(api::benchmark::pause_benchmark)),
        )
        // Node Module
        .nest(
            "/nodes",
            Router::new()
                .route(
                    "/",
                    get(api::node::list_nodes).post(api::node::register_node),
                )
                .route("/stats", get(api::node::get_node_stats))
                .route("/health", get(api::node::get_all_nodes_health))
                .route(
                    "/:id",
                    get(api::node::get_node).delete(api::node::remove_node),
                )
                .route("/:id/health", get(api::node::get_node_health))
                .route("/:id/fault", post(api::node::inject_fault))
                .route("/:id/recover", post(api::node::recover_node)),
        )
        // Transaction Module
        .nest(
            "/transactions",
            Router::new()
                .route("/submit", post(api::transaction::submit_transaction))
                .route("/status", get(api::transaction::get_transaction_status))
                .route("/history", get(api::transaction::get_transaction_history))
                .route("/pending", get(api::transaction::get_pending_transactions))
                .route(
                    "/confirmed",
                    get(api::transaction::get_confirmed_transactions),
                )
                .route("/query", get(api::transaction::query_transactions))
                .route("/:id", get(api::transaction::get_transaction))
                .route("/:id/cancel", post(api::transaction::cancel_transaction)),
        )
        // Block Module
        .nest(
            "/blocks",
            Router::new().route("/:height", get(api::block::get_block)),
        )
        // Performance Module (HTTP endpoints)
        .nest(
            "/performance",
            Router::new()
                .route("/metrics", get(api::performance::get_metrics))
                .route("/detailed", get(api::performance::get_detailed_metrics))
                .route("/summary", get(api::performance::get_summary))
                .route("/history", get(api::performance::get_history))
                .route(
                    "/comparison",
                    get(api::performance::get_performance_comparison),
                )
                .route("/export", post(api::performance::export_performance_data))
                .route("/data", delete(api::performance::clear_data)),
        )
        // Anti-Manipulation Module
        .nest(
            "/anti-manipulation",
            Router::new()
                .route(
                    "/config",
                    get(api::anti_manipulation::get_config)
                        .put(api::anti_manipulation::update_config),
                )
                .route("/events", get(api::anti_manipulation::get_events))
                .route(
                    "/events/:id",
                    get(api::anti_manipulation::get_event_details),
                ),
        )
        // Analysis Module
        .nest(
            "/analysis",
            Router::new()
                .route("/summary", get(api::analysis::get_summary))
                .route(
                    "/report",
                    get(api::analysis::get_report).post(api::analysis::generate_report),
                )
                .route("/trends", get(api::analysis::get_trends))
                .route("/comparison", get(api::analysis::get_comparison))
                .route("/prediction", get(api::analysis::get_prediction))
                .route("/limits/:algo", get(api::analysis::get_algo_limits))
                .route("/export", post(api::analysis::export_analysis)),
        )
        // Settings Module
        .nest(
            "/settings",
            Router::new()
                .route(
                    "/general",
                    get(api::settings::get_general).put(api::settings::update_general),
                )
                .route(
                    "/network",
                    get(api::settings::get_network).put(api::settings::update_network),
                )
                .route(
                    "/storage",
                    get(api::settings::get_storage).put(api::settings::update_storage),
                )
                .route(
                    "/security",
                    get(api::settings::get_security).put(api::settings::update_security),
                )
                .route(
                    "/notifications",
                    get(api::settings::get_notifications).put(api::settings::update_notifications),
                )
                .route(
                    "/notification",
                    get(api::settings::get_notifications).put(api::settings::update_notifications),
                )
                .route(
                    "/backup",
                    get(api::settings::get_backup).put(api::settings::update_backup),
                )
                .route("/backup/trigger", post(api::settings::trigger_backup))
                .route(
                    "/backup/validate-path",
                    post(api::settings::validate_backup_path),
                )
                .route(
                    "/users",
                    get(api::settings::get_users).post(api::settings::create_user),
                )
                .route(
                    "/users/:id",
                    put(api::settings::update_user).delete(api::settings::delete_user),
                ),
        )
        .layer(from_fn(middleware::auth_middleware));

    // Public API routes
    let api_routes = Router::new()
        .route("/auth/login", post(api::auth::login))
        .route("/performance", any(api::performance::ws_handler))
        .route("/system/stream", get(api::health::system_stream))
        .merge(protected_routes);

    Router::new()
        // Health check
        .route("/health", get(api::health::health_check))
        // API v1 (and default /api)
        .nest("/api", api_routes.clone())
        .nest("/api/v1", api_routes)
        // Middleware Stack
        .layer(
            ServiceBuilder::new()
                .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
                .layer(from_fn(middleware::propagate_request_id))
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                )
                .layer(from_fn(middleware::logging_middleware)),
        )
        .with_state(app_state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::state::AppState;
    use crate::utils::mock_data as data;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;
    use tower::Service; // Use Service trait directly

    #[tokio::test]
    async fn test_health_check() {
        let app_state = Arc::new(AppState::new(
            data::default_mock_data(),
            None,
            None,
            Arc::new(AtomicBool::new(true)),
            Arc::new(AtomicBool::new(true)),
        ));
        let mut app = create_router(app_state);

        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let response = app.call(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
