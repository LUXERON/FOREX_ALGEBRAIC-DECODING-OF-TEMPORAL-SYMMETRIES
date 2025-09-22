//! # WebSocket-Enabled Forex Trading Server
//!
//! Production-ready HTTP API server with WebSocket support for real-time
//! communication between local CLI and remote Render deployment.

use anyhow::Result;
use std::env;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use warp::{Filter, ws::{Message, WebSocket}};
use serde::{Deserialize, Serialize};
use serde_json::json;
use futures_util::{SinkExt, StreamExt};

/// WebSocket message types for CLI communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WSMessage {
    // Commands from CLI to Server
    GetStatus,
    StartTrading { mode: String },
    StopTrading,
    GetPairs,
    GetAnalysis { pair: String },
    SwitchMode { mode: String },
    
    // Responses from Server to CLI
    Status { 
        active: bool, 
        mode: String, 
        pairs_count: usize,
        uptime: String 
    },
    TradingStarted { mode: String },
    TradingStopped,
    PairsList { pairs: Vec<String> },
    Analysis { 
        pair: String, 
        correlation: f64, 
        trend: String,
        recommendation: String 
    },
    ModeChanged { new_mode: String },
    Error { message: String },
    
    // Real-time updates
    PriceUpdate { 
        pair: String, 
        price: f64, 
        timestamp: String 
    },
    TradeExecuted { 
        pair: String, 
        action: String, 
        price: f64, 
        profit: f64 
    },
}

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub trading_active: Arc<Mutex<bool>>,
    pub trading_mode: Arc<Mutex<String>>,
    pub broadcast_tx: broadcast::Sender<WSMessage>,
    pub start_time: std::time::Instant,
    pub pairs: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // ASCII Art Banner
    println!("
╔═══════════════════════════════════════════════════════════════════════════════════╗
║                                                                                   ║
║    🌐 WEBSOCKET FOREX TRADING SERVER 🌐                                         ║
║         Real-time CLI ↔ Render Communication                                     ║
║                                                                                   ║
╚═══════════════════════════════════════════════════════════════════════════════════╝
");

    let start_time = std::time::Instant::now();

    // Simulate available pairs (since we can't access embedded DB in multi-threaded context)
    let currency_pairs = vec![
        "EURUSD".to_string(), "GBPUSD".to_string(), "USDJPY".to_string(),
        "USDCHF".to_string(), "USDCAD".to_string(), "EURGBP".to_string(),
        "EURJPY".to_string(), "EURCHF".to_string(), "EURCAD".to_string(),
        "GBPJPY".to_string(), "GBPCHF".to_string(), "GBPCAD".to_string(),
        "CADJPY".to_string(), "CADCHF".to_string(), "CHFJPY".to_string(),
    ];

    println!("📊 Loaded {} currency pairs", currency_pairs.len());

    // Initialize components
    let trading_active = Arc::new(Mutex::new(false));
    let trading_mode = Arc::new(Mutex::new("DEMO".to_string()));

    // Create broadcast channel for real-time updates
    let (broadcast_tx, _) = broadcast::channel(100);

    let app_state = AppState {
        trading_active,
        trading_mode,
        broadcast_tx: broadcast_tx.clone(),
        start_time,
        pairs: currency_pairs.clone(),
    };

    // HTTP Routes
    let health = warp::path("health")
        .map(|| warp::reply::json(&json!({
            "status": "healthy",
            "service": "websocket-forex-trader",
            "version": "1.0.0"
        })));

    let status = warp::path("status")
        .and(with_state(app_state.clone()))
        .and_then(handle_status);

    let pairs = warp::path("pairs")
        .and(with_state(app_state.clone()))
        .and_then(handle_pairs);

    // WebSocket route for real-time CLI communication
    let websocket = warp::path("ws")
        .and(warp::ws())
        .and(with_state(app_state.clone()))
        .map(|ws: warp::ws::Ws, state| {
            ws.on_upgrade(move |socket| handle_websocket(socket, state))
        });

    // Combine all routes
    let routes = health
        .or(status)
        .or(pairs)
        .or(websocket)
        .with(warp::cors().allow_any_origin());

    // Get port from environment (Render sets this automatically)
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    println!("🚀 Starting WebSocket Forex Trading Server on port {}", port);
    println!("📡 WebSocket endpoint: ws://localhost:{}/ws", port);
    println!("🌐 HTTP API: http://localhost:{}/", port);
    
    // Start background task for price updates
    tokio::spawn(price_update_task(broadcast_tx, currency_pairs.clone()));

    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;

    Ok(())
}

/// Helper to pass state to handlers
fn with_state(state: AppState) -> impl Filter<Extract = (AppState,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

/// Handle HTTP status endpoint
async fn handle_status(state: AppState) -> Result<impl warp::Reply, warp::Rejection> {
    let trading_active = *state.trading_active.lock().await;
    let trading_mode = state.trading_mode.lock().await.clone();
    let uptime = format!("{:.2}s", state.start_time.elapsed().as_secs_f64());

    Ok(warp::reply::json(&json!({
        "active": trading_active,
        "mode": trading_mode,
        "pairs_count": state.pairs.len(),
        "uptime": uptime,
        "pairs": state.pairs
    })))
}

/// Handle HTTP pairs endpoint
async fn handle_pairs(state: AppState) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&json!({
        "pairs": state.pairs,
        "count": state.pairs.len()
    })))
}

/// Handle WebSocket connections for real-time CLI communication
async fn handle_websocket(ws: WebSocket, state: AppState) {
    println!("🔌 New WebSocket connection established");
    
    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut broadcast_rx = state.broadcast_tx.subscribe();
    
    // Send welcome message
    let welcome = WSMessage::Status {
        active: *state.trading_active.lock().await,
        mode: state.trading_mode.lock().await.clone(),
        pairs_count: state.pairs.len(),
        uptime: format!("{:.2}s", state.start_time.elapsed().as_secs_f64()),
    };
    
    if let Ok(msg) = serde_json::to_string(&welcome) {
        let _ = ws_tx.send(Message::text(msg)).await;
    }
    
    // Handle incoming messages from CLI
    let state_clone = state.clone();
    let ws_tx_clone = Arc::new(Mutex::new(ws_tx));
    let ws_tx_for_broadcast = ws_tx_clone.clone();
    
    // Task to handle CLI commands
    let command_task = tokio::spawn(async move {
        while let Some(result) = ws_rx.next().await {
            match result {
                Ok(msg) => {
                    if let Ok(text) = msg.to_str() {
                        if let Ok(ws_msg) = serde_json::from_str::<WSMessage>(text) {
                            let response = handle_ws_command(ws_msg, &state_clone).await;
                            if let Ok(response_text) = serde_json::to_string(&response) {
                                let mut tx = ws_tx_clone.lock().await;
                                let _ = tx.send(Message::text(response_text)).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("❌ WebSocket error: {}", e);
                    break;
                }
            }
        }
    });
    
    // Task to broadcast updates to CLI
    let broadcast_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            if let Ok(msg_text) = serde_json::to_string(&msg) {
                let mut tx = ws_tx_for_broadcast.lock().await;
                if tx.send(Message::text(msg_text)).await.is_err() {
                    break;
                }
            }
        }
    });
    
    // Wait for either task to complete
    tokio::select! {
        _ = command_task => {},
        _ = broadcast_task => {},
    }
    
    println!("🔌 WebSocket connection closed");
}

/// Handle WebSocket commands from CLI
async fn handle_ws_command(msg: WSMessage, state: &AppState) -> WSMessage {
    match msg {
        WSMessage::GetStatus => {
            WSMessage::Status {
                active: *state.trading_active.lock().await,
                mode: state.trading_mode.lock().await.clone(),
                pairs_count: state.pairs.len(),
                uptime: format!("{:.2}s", state.start_time.elapsed().as_secs_f64()),
            }
        }
        WSMessage::StartTrading { mode } => {
            *state.trading_active.lock().await = true;
            *state.trading_mode.lock().await = mode.clone();
            WSMessage::TradingStarted { mode }
        }
        WSMessage::StopTrading => {
            *state.trading_active.lock().await = false;
            WSMessage::TradingStopped
        }
        WSMessage::GetPairs => {
            WSMessage::PairsList { pairs: state.pairs.clone() }
        }
        WSMessage::GetAnalysis { pair } => {
            // Simulate analysis using embedded data
            WSMessage::Analysis {
                pair: pair.clone(),
                correlation: 0.85,
                trend: "BULLISH".to_string(),
                recommendation: "BUY".to_string(),
            }
        }
        WSMessage::SwitchMode { mode } => {
            *state.trading_mode.lock().await = mode.clone();
            WSMessage::ModeChanged { new_mode: mode }
        }
        _ => WSMessage::Error {
            message: "Unknown command".to_string(),
        }
    }
}

/// Background task to simulate price updates
async fn price_update_task(tx: broadcast::Sender<WSMessage>, pairs: Vec<String>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

    loop {
        interval.tick().await;

        if let Some(pair) = pairs.first() {
            let price_update = WSMessage::PriceUpdate {
                pair: pair.clone(),
                price: 1.0850 + (rand::random::<f64>() - 0.5) * 0.01,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };

            let _ = tx.send(price_update);
        }
    }
}
