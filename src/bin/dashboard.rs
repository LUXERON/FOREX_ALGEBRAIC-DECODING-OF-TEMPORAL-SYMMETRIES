//! # Real-Time Forex Pattern Recognition Dashboard
//! 
//! CLI application for live pattern monitoring and analysis

use anyhow::Result;
use clap::{Arg, Command};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};
use tokio::time::interval;

use forex_pattern_reconstruction::dashboard::{DashboardApp, render_dashboard};

/// ASCII Art Banner
const BANNER: &str = r#"
╔═══════════════════════════════════════════════════════════════════════════════╗
║                                                                               ║
║    ███████╗ ██████╗ ██████╗ ███████╗██╗  ██╗    ██████╗  █████╗ ████████╗    ║
║    ██╔════╝██╔═══██╗██╔══██╗██╔════╝╚██╗██╔╝    ██╔══██╗██╔══██╗╚══██╔══╝    ║
║    █████╗  ██║   ██║██████╔╝█████╗   ╚███╔╝     ██████╔╝███████║   ██║       ║
║    ██╔══╝  ██║   ██║██╔══██╗██╔══╝   ██╔██╗     ██╔═══╝ ██╔══██║   ██║       ║
║    ██║     ╚██████╔╝██║  ██║███████╗██╔╝ ██╗    ██║     ██║  ██║   ██║       ║
║    ╚═╝      ╚═════╝ ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝    ╚═╝     ╚═╝  ╚═╝   ╚═╝       ║
║                                                                               ║
║           ██████╗ ███████╗ ██████╗ ██████╗ ███╗   ██╗███████╗████████╗        ║
║           ██╔══██╗██╔════╝██╔════╝██╔═══██╗████╗  ██║██╔════╝╚══██╔══╝        ║
║           ██████╔╝█████╗  ██║     ██║   ██║██╔██╗ ██║███████╗   ██║           ║
║           ██╔══██╗██╔══╝  ██║     ██║   ██║██║╚██╗██║╚════██║   ██║           ║
║           ██║  ██║███████╗╚██████╗╚██████╔╝██║ ╚████║███████║   ██║           ║
║           ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝╚══════╝   ╚═╝           ║
║                                                                               ║
║                    🔬 TIME-SYMMETRIC PATTERN RECOGNITION 🔬                   ║
║                         Real-Time Dashboard v1.0.0                           ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
"#;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let matches = Command::new("forex-pattern-dashboard")
        .version("1.0.0")
        .author("NEUNOMY - CURILEXA ALPHA")
        .about("Real-time forex pattern recognition dashboard")
        .arg(
            Arg::new("pair")
                .short('p')
                .long("pair")
                .value_name("PAIR")
                .help("Currency pair to analyze (default: EURUSD)")
                .default_value("EURUSD")
        )
        .arg(
            Arg::new("data-dir")
                .short('d')
                .long("data-dir")
                .value_name("DIR")
                .help("Directory containing forex data")
                .default_value("FOREX DATA")
        )
        .arg(
            Arg::new("update-interval")
                .short('u')
                .long("update-interval")
                .value_name("MS")
                .help("Update interval in milliseconds")
                .default_value("1000")
        )
        .get_matches();

    // Display banner
    println!("{}", BANNER);
    println!("🚀 Initializing Time-Symmetric Pattern Recognition Engine...");
    println!("📊 Loading historical forex data...");
    println!("🔬 Preparing real-time analysis dashboard...");
    println!();
    
    // Initialize dashboard
    let mut app = DashboardApp::new().await?;
    app.initialize().await?;
    
    println!("✅ Dashboard initialized successfully!");
    println!("🎯 Press any key to start the real-time dashboard...");
    
    // Wait for user input
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    // Run the dashboard
    run_dashboard(app).await?;
    
    Ok(())
}

/// Run the main dashboard loop
async fn run_dashboard(mut app: DashboardApp) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create update interval
    let mut update_interval = interval(Duration::from_millis(100));
    let mut last_tick = Instant::now();
    
    // Main loop
    loop {
        // Handle events
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_input(key.code)?;
                    
                    if app.should_quit() {
                        break;
                    }
                }
            }
        }
        
        // Update app state
        if last_tick.elapsed() >= Duration::from_millis(1000) {
            app.update().await?;
            last_tick = Instant::now();
        }
        
        // Render UI
        terminal.draw(|f| render_dashboard(f, &app))?;
        
        // Wait for next tick
        update_interval.tick().await;
    }
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    println!("🎯 Dashboard closed. Thank you for using FOREX PATTERN RECONSTRUCTION!");
    
    Ok(())
}

/// Display startup information
fn display_startup_info() {
    println!("📋 DASHBOARD FEATURES:");
    println!("   🔄 Real-time pattern recognition");
    println!("   📊 Temporal symmetry detection");
    println!("   📈 Live price charts");
    println!("   🎯 Performance metrics");
    println!("   ⚡ Sub-second analysis updates");
    println!();
    println!("🎮 CONTROLS:");
    println!("   Tab/1-4: Switch between tabs");
    println!("   R: Refresh data");
    println!("   Q/Esc: Quit dashboard");
    println!();
    println!("📊 TABS:");
    println!("   1. Overview: Price charts and key metrics");
    println!("   2. Patterns: Detected cycles and pattern strength");
    println!("   3. Symmetries: Temporal symmetries and visualization");
    println!("   4. Performance: System performance and history");
    println!();
}
