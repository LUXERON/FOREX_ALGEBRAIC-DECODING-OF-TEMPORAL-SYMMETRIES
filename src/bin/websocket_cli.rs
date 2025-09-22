//! # WebSocket CLI Controller
//!
//! Local CLI application that connects to the remote Render WebSocket API
//! for real-time forex trading control and monitoring.

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::{self, Write};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use url::Url;

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

#[derive(Parser)]
#[command(name = "forex-cli")]
#[command(about = "WebSocket CLI Controller for Forex Trading System")]
struct Cli {
    /// WebSocket server URL
    #[arg(short, long, default_value = "ws://localhost:8080/ws")]
    url: String,
    
    /// Command to execute
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Get system status
    Status,
    /// Start trading in specified mode
    Start { 
        #[arg(default_value = "DEMO")]
        mode: String 
    },
    /// Stop trading
    Stop,
    /// List available currency pairs
    Pairs,
    /// Get analysis for a currency pair
    Analyze { pair: String },
    /// Switch trading mode
    Mode { mode: String },
    /// Interactive mode with real-time updates
    Interactive,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // ASCII Art Banner
    println!("
╔═══════════════════════════════════════════════════════════════════════════════════╗
║                                                                                   ║
║    🎮 FOREX WEBSOCKET CLI CONTROLLER 🎮                                         ║
║         Local Control → Remote Render Trading System                             ║
║                                                                                   ║
╚═══════════════════════════════════════════════════════════════════════════════════╝
");

    match cli.command {
        Some(Commands::Status) => {
            execute_command(&cli.url, WSMessage::GetStatus).await?;
        }
        Some(Commands::Start { mode }) => {
            execute_command(&cli.url, WSMessage::StartTrading { mode }).await?;
        }
        Some(Commands::Stop) => {
            execute_command(&cli.url, WSMessage::StopTrading).await?;
        }
        Some(Commands::Pairs) => {
            execute_command(&cli.url, WSMessage::GetPairs).await?;
        }
        Some(Commands::Analyze { pair }) => {
            execute_command(&cli.url, WSMessage::GetAnalysis { pair }).await?;
        }
        Some(Commands::Mode { mode }) => {
            execute_command(&cli.url, WSMessage::SwitchMode { mode }).await?;
        }
        Some(Commands::Interactive) => {
            interactive_mode(&cli.url).await?;
        }
        None => {
            // Default to interactive mode
            interactive_mode(&cli.url).await?;
        }
    }

    Ok(())
}

/// Execute a single command and display the response
async fn execute_command(url: &str, command: WSMessage) -> Result<()> {
    let url = Url::parse(url)?;
    println!("🔌 Connecting to {}...", url);
    
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();
    
    // Send command
    let command_json = serde_json::to_string(&command)?;
    write.send(Message::Text(command_json)).await?;
    
    // Wait for response
    if let Some(msg) = read.next().await {
        match msg? {
            Message::Text(text) => {
                if let Ok(response) = serde_json::from_str::<WSMessage>(&text) {
                    display_response(&response);
                } else {
                    println!("📄 Raw response: {}", text);
                }
            }
            _ => println!("❓ Received non-text message"),
        }
    }
    
    Ok(())
}

/// Interactive mode with real-time updates
async fn interactive_mode(url: &str) -> Result<()> {
    let url = Url::parse(url)?;
    println!("🔌 Connecting to {}...", url);
    
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();
    
    println!("✅ Connected! Type 'help' for commands or 'quit' to exit.");
    
    // Spawn task to handle incoming messages
    let read_task = tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(response) = serde_json::from_str::<WSMessage>(&text) {
                        display_response(&response);
                    } else {
                        println!("📄 Raw: {}", text);
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    println!("❌ WebSocket error: {}", e);
                    break;
                }
            }
        }
    });
    
    // Handle user input
    loop {
        print!("forex> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        match input {
            "quit" | "exit" => break,
            "help" => {
                println!("📋 Available commands:");
                println!("  status          - Get system status");
                println!("  start [mode]    - Start trading (DEMO/LIVE)");
                println!("  stop            - Stop trading");
                println!("  pairs           - List currency pairs");
                println!("  analyze <pair>  - Analyze currency pair");
                println!("  mode <mode>     - Switch trading mode");
                println!("  help            - Show this help");
                println!("  quit            - Exit");
            }
            "status" => {
                let cmd = WSMessage::GetStatus;
                send_command(&mut write, cmd).await?;
            }
            "stop" => {
                let cmd = WSMessage::StopTrading;
                send_command(&mut write, cmd).await?;
            }
            "pairs" => {
                let cmd = WSMessage::GetPairs;
                send_command(&mut write, cmd).await?;
            }
            _ if input.starts_with("start") => {
                let parts: Vec<&str> = input.split_whitespace().collect();
                let mode = parts.get(1).unwrap_or(&"DEMO").to_string();
                let cmd = WSMessage::StartTrading { mode };
                send_command(&mut write, cmd).await?;
            }
            _ if input.starts_with("analyze") => {
                let parts: Vec<&str> = input.split_whitespace().collect();
                if let Some(pair) = parts.get(1) {
                    let cmd = WSMessage::GetAnalysis { pair: pair.to_string() };
                    send_command(&mut write, cmd).await?;
                } else {
                    println!("❌ Usage: analyze <pair>");
                }
            }
            _ if input.starts_with("mode") => {
                let parts: Vec<&str> = input.split_whitespace().collect();
                if let Some(mode) = parts.get(1) {
                    let cmd = WSMessage::SwitchMode { mode: mode.to_string() };
                    send_command(&mut write, cmd).await?;
                } else {
                    println!("❌ Usage: mode <DEMO|LIVE>");
                }
            }
            _ => {
                println!("❓ Unknown command: {}. Type 'help' for available commands.", input);
            }
        }
    }
    
    read_task.abort();
    println!("👋 Goodbye!");
    Ok(())
}

/// Send a command via WebSocket
async fn send_command(write: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>, command: WSMessage) -> Result<()> {
    let command_json = serde_json::to_string(&command)?;
    write.send(Message::Text(command_json)).await?;
    Ok(())
}

/// Display a WebSocket response in a user-friendly format
fn display_response(response: &WSMessage) {
    match response {
        WSMessage::Status { active, mode, pairs_count, uptime } => {
            println!("📊 System Status:");
            println!("   Active: {}", if *active { "✅ YES" } else { "❌ NO" });
            println!("   Mode: {}", mode);
            println!("   Pairs: {}", pairs_count);
            println!("   Uptime: {}", uptime);
        }
        WSMessage::TradingStarted { mode } => {
            println!("🚀 Trading started in {} mode", mode);
        }
        WSMessage::TradingStopped => {
            println!("⏹️  Trading stopped");
        }
        WSMessage::PairsList { pairs } => {
            println!("💱 Available Currency Pairs ({}):", pairs.len());
            for (i, pair) in pairs.iter().enumerate() {
                if i % 5 == 0 { println!(); }
                print!("  {:8}", pair);
            }
            println!();
        }
        WSMessage::Analysis { pair, correlation, trend, recommendation } => {
            println!("📈 Analysis for {}:", pair);
            println!("   Correlation: {:.3}", correlation);
            println!("   Trend: {}", trend);
            println!("   Recommendation: {}", recommendation);
        }
        WSMessage::ModeChanged { new_mode } => {
            println!("🔄 Mode changed to: {}", new_mode);
        }
        WSMessage::PriceUpdate { pair, price, timestamp } => {
            println!("💰 {} @ {:.5} ({})", pair, price, timestamp);
        }
        WSMessage::TradeExecuted { pair, action, price, profit } => {
            println!("⚡ Trade: {} {} @ {:.5} | Profit: {:.2}", action, pair, price, profit);
        }
        WSMessage::Error { message } => {
            println!("❌ Error: {}", message);
        }
        _ => {
            println!("📄 Response: {:?}", response);
        }
    }
}
