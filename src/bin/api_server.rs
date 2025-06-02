use axum::{
    routing::{get, post, put},
    Router,
    http::Method,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::{CorsLayer, Any},
    trace::TraceLayer,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{info, error};
use tracing_subscriber;

use fluxdefense::api::{
    handlers::{
        AppState, health_check, get_system_status, get_threat_metrics, get_network_metrics,
        get_system_metrics, get_system_resources, get_security_events, get_security_event,
        get_network_connections, get_dns_queries, get_threat_detections, get_malware_signatures,
        get_event_logs, get_live_events, get_settings, update_settings, get_security_settings,
        update_security_settings,
    },
    websocket::{websocket_handler, populate_mock_data},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting FluxDefense API Server...");

    // Create application state
    let state = Arc::new(AppState::new());
    
    // Populate initial mock data
    populate_mock_data(Arc::clone(&state));
    
    info!("Populated initial mock data");

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any)
        .allow_origin(Any);

    // Build the application router
    let app = Router::new()
        // Health check
        .route("/api/health", get(health_check))
        
        // Dashboard overview
        .route("/api/dashboard/status", get(get_system_status))
        .route("/api/dashboard/threats", get(get_threat_metrics))
        .route("/api/dashboard/network", get(get_network_metrics))
        
        // System monitoring
        .route("/api/system/metrics", get(get_system_metrics))
        .route("/api/system/resources", get(get_system_resources))
        
        // Security events
        .route("/api/security/events", get(get_security_events))
        .route("/api/security/events/:id", get(get_security_event))
        
        // Network monitoring
        .route("/api/network/connections", get(get_network_connections))
        .route("/api/network/dns", get(get_dns_queries))
        
        // Threat detection
        .route("/api/threats/detections", get(get_threat_detections))
        .route("/api/threats/signatures", get(get_malware_signatures))
        
        // Event logs
        .route("/api/logs/events", get(get_event_logs))
        
        // Live monitoring
        .route("/api/live/events", get(get_live_events))
        .route("/api/live/ws", get(websocket_handler))
        
        // Settings
        .route("/api/settings", get(get_settings).put(update_settings))
        .route("/api/settings/security", get(get_security_settings).put(update_security_settings))
        
        // Static file serving for the web dashboard
        .fallback_service(tower_http::services::ServeDir::new("web-dashboard/dist"))
        
        // Add middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors)
        )
        .with_state(state);

    // Start the server
    let port = std::env::var("PORT").unwrap_or_else(|_| "3177".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    info!("Starting server on {}", addr);
    
    let listener = TcpListener::bind(&addr).await?;
    
    info!("FluxDefense API Server running on http://{}", addr);
    info!("Dashboard available at http://{}", addr);
    info!("API endpoints available at http://{}/api/", addr);
    info!("WebSocket endpoint: ws://{}/api/live/ws", addr);
    
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        let state = Arc::new(AppState::new());
        let app = Router::new()
            .route("/api/health", get(health_check))
            .with_state(state);

        let request = Request::builder()
            .uri("/api/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_system_metrics() {
        let state = Arc::new(AppState::new());
        let app = Router::new()
            .route("/api/system/metrics", get(get_system_metrics))
            .with_state(state);

        let request = Request::builder()
            .uri("/api/system/metrics")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_security_events() {
        let state = Arc::new(AppState::new());
        populate_mock_data(Arc::clone(&state));
        
        let app = Router::new()
            .route("/api/security/events", get(get_security_events))
            .with_state(state);

        let request = Request::builder()
            .uri("/api/security/events")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}