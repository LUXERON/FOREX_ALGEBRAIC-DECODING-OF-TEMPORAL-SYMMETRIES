use clap::{Arg, Command};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Tabs},
    Frame, Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;

#[derive(Debug, Serialize, Deserialize)]
struct RemoteSystemStatus {
    status: String,
    uptime: u64,
    active_pairs: Vec<String>,
    total_trades: u64,
    profit_loss: f64,
    correlation_opportunities: Vec<ArbitrageOpportunity>,
    system_metrics: SystemMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
struct ArbitrageOpportunity {
    primary_pair: String,
    correlated_pair: String,
    confidence: f64,
    theoretical_pips: f64,
    realistic_pips: f64,
    execution_cost: f64,
    net_expected_pips: f64,
    position_size: f64,
    time_window: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SystemMetrics {
    cpu_usage: f64,
    memory_usage: f64,
    network_latency: f64,
    database_size: u64,
    active_connections: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct TradingCommand {
    action: String,
    pair: Option<String>,
    parameters: HashMap<String, String>,
}

struct ForexCliController {
    client: Client,
    render_endpoint: String,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    current_tab: usize,
    system_status: Option<RemoteSystemStatus>,
}

impl ForexCliController {
    fn new(render_endpoint: String) -> Result<Self, Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(ForexCliController {
            client: Client::new(),
            render_endpoint,
            terminal,
            current_tab: 0,
            system_status: None,
        })
    }

    async fn fetch_system_status(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/api/status", self.render_endpoint);
        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            self.system_status = Some(response.json().await?);
        }
        
        Ok(())
    }

    async fn send_command(&self, command: TradingCommand) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}/api/command", self.render_endpoint);
        let response = self.client.post(&url).json(&command).send().await?;
        
        Ok(response.text().await?)
    }

    fn draw_ui(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.terminal.draw(|f| {
            let size = f.size();
            
            // Create main layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),  // Header
                    Constraint::Min(0),     // Content
                    Constraint::Length(3),  // Footer
                ].as_ref())
                .split(size);

            // Header with tabs
            let tab_titles = vec!["System Status", "Arbitrage", "Trading", "Analytics", "Control"];
            let tab_spans: Vec<Spans> = tab_titles.iter().map(|t| Spans::from(vec![Span::raw(*t)])).collect();
            let tabs = Tabs::new(tab_spans)
                .block(Block::default().borders(Borders::ALL).title("Forex CLI Controller"))
                .select(self.current_tab)
                .style(Style::default().fg(Color::Cyan))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::Black));
            f.render_widget(tabs, chunks[0]);

            // Content based on selected tab
            match self.current_tab {
                0 => self.draw_system_status(f, chunks[1]),
                1 => self.draw_arbitrage_opportunities(f, chunks[1]),
                2 => self.draw_trading_panel(f, chunks[1]),
                3 => self.draw_analytics(f, chunks[1]),
                4 => self.draw_control_panel(f, chunks[1]),
                _ => {}
            }

            // Footer
            let footer = Paragraph::new("Press 'q' to quit, Tab to switch panels, Enter to execute commands")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        })?;

        Ok(())
    }

    fn draw_system_status(&self, f: &mut Frame<CrosstermBackend<io::Stdout>>, area: Rect) {
        if let Some(status) = &self.system_status {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(area);

            // System metrics
            let metrics_text = vec![
                Spans::from(vec![
                    Span::styled("Status: ", Style::default().fg(Color::Yellow)),
                    Span::styled(&status.status, Style::default().fg(Color::Green)),
                ]),
                Spans::from(vec![
                    Span::styled("Uptime: ", Style::default().fg(Color::Yellow)),
                    Span::styled(format!("{}h", status.uptime / 3600), Style::default().fg(Color::White)),
                ]),
                Spans::from(vec![
                    Span::styled("Active Pairs: ", Style::default().fg(Color::Yellow)),
                    Span::styled(status.active_pairs.len().to_string(), Style::default().fg(Color::White)),
                ]),
                Spans::from(vec![
                    Span::styled("Total Trades: ", Style::default().fg(Color::Yellow)),
                    Span::styled(status.total_trades.to_string(), Style::default().fg(Color::White)),
                ]),
                Spans::from(vec![
                    Span::styled("P&L: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("${:.2}", status.profit_loss),
                        if status.profit_loss >= 0.0 { Style::default().fg(Color::Green) } else { Style::default().fg(Color::Red) }
                    ),
                ]),
            ];

            let system_info = Paragraph::new(metrics_text)
                .block(Block::default().borders(Borders::ALL).title("System Status"));
            f.render_widget(system_info, chunks[0]);

            // Performance gauges
            let cpu_gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("CPU Usage"))
                .gauge_style(Style::default().fg(Color::Yellow))
                .percent((status.system_metrics.cpu_usage * 100.0) as u16);
            
            let memory_gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("Memory Usage"))
                .gauge_style(Style::default().fg(Color::Blue))
                .percent((status.system_metrics.memory_usage * 100.0) as u16);

            let perf_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(chunks[1]);

            f.render_widget(cpu_gauge, perf_chunks[0]);
            f.render_widget(memory_gauge, perf_chunks[1]);
        } else {
            let loading = Paragraph::new("Loading system status...")
                .block(Block::default().borders(Borders::ALL).title("System Status"));
            f.render_widget(loading, area);
        }
    }

    fn draw_arbitrage_opportunities(&self, f: &mut Frame<CrosstermBackend<io::Stdout>>, area: Rect) {
        if let Some(status) = &self.system_status {
            let opportunities: Vec<ListItem> = status.correlation_opportunities
                .iter()
                .map(|opp| {
                    ListItem::new(vec![
                        Spans::from(vec![
                            Span::styled(format!("{} ↔ {}", opp.primary_pair, opp.correlated_pair), 
                                Style::default().fg(Color::Cyan)),
                        ]),
                        Spans::from(vec![
                            Span::styled("Confidence: ", Style::default().fg(Color::Yellow)),
                            Span::styled(format!("{:.1}%", opp.confidence * 100.0), Style::default().fg(Color::White)),
                            Span::styled(" | Theoretical: ", Style::default().fg(Color::Yellow)),
                            Span::styled(format!("{:.0} pips", opp.theoretical_pips), Style::default().fg(Color::Gray)),
                        ]),
                        Spans::from(vec![
                            Span::styled("Realistic: ", Style::default().fg(Color::Green)),
                            Span::styled(format!("{:.1} pips", opp.realistic_pips), Style::default().fg(Color::Green)),
                            Span::styled(" | Net Expected: ", Style::default().fg(Color::Yellow)),
                            Span::styled(format!("{:.1} pips", opp.net_expected_pips), Style::default().fg(Color::White)),
                        ]),
                        Spans::from(vec![
                            Span::styled("Position Size: ", Style::default().fg(Color::Yellow)),
                            Span::styled(format!("${:.0}", opp.position_size), Style::default().fg(Color::White)),
                            Span::styled(" | Window: ", Style::default().fg(Color::Yellow)),
                            Span::styled(&opp.time_window, Style::default().fg(Color::White)),
                        ]),
                        Spans::from(vec![Span::styled("─".repeat(50), Style::default().fg(Color::Gray))]),
                    ])
                })
                .collect();

            let opportunities_list = List::new(opportunities)
                .block(Block::default().borders(Borders::ALL).title("Realistic Arbitrage Opportunities"));
            f.render_widget(opportunities_list, area);
        }
    }

    fn draw_trading_panel(&self, f: &mut Frame<CrosstermBackend<io::Stdout>>, area: Rect) {
        let trading_info = Paragraph::new("Trading Panel - Send commands to remote system")
            .block(Block::default().borders(Borders::ALL).title("Trading Control"));
        f.render_widget(trading_info, area);
    }

    fn draw_analytics(&self, f: &mut Frame<CrosstermBackend<io::Stdout>>, area: Rect) {
        let analytics_info = Paragraph::new("Analytics Panel - Performance metrics and historical data")
            .block(Block::default().borders(Borders::ALL).title("Analytics"));
        f.render_widget(analytics_info, area);
    }

    fn draw_control_panel(&self, f: &mut Frame<CrosstermBackend<io::Stdout>>, area: Rect) {
        let control_info = Paragraph::new("Control Panel - System configuration and deployment management")
            .block(Block::default().borders(Borders::ALL).title("System Control"));
        f.render_widget(control_info, area);
    }

    async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // Fetch latest data
            if let Err(e) = self.fetch_system_status().await {
                eprintln!("Failed to fetch system status: {}", e);
            }

            // Draw UI
            self.draw_ui()?;

            // Handle input
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Tab => {
                            self.current_tab = (self.current_tab + 1) % 5;
                        }
                        KeyCode::Enter => {
                            // Execute command based on current tab
                            self.execute_current_command().await?;
                        }
                        _ => {}
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }

    async fn execute_current_command(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self.current_tab {
            2 => {
                // Trading panel - send trading command
                let command = TradingCommand {
                    action: "get_opportunities".to_string(),
                    pair: None,
                    parameters: HashMap::new(),
                };
                let _response = self.send_command(command).await?;
            }
            4 => {
                // Control panel - system commands
                let command = TradingCommand {
                    action: "restart_analysis".to_string(),
                    pair: None,
                    parameters: HashMap::new(),
                };
                let _response = self.send_command(command).await?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl Drop for ForexCliController {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Forex CLI Controller")
        .version("1.0")
        .author("NEUNOMY")
        .about("Control and monitor remote Render forex trading system")
        .arg(
            Arg::new("endpoint")
                .short('e')
                .long("endpoint")
                .value_name("URL")
                .help("Render deployment endpoint URL")
                .required(true),
        )
        .subcommand(
            Command::new("monitor")
                .about("Start interactive monitoring dashboard")
        )
        .subcommand(
            Command::new("status")
                .about("Get current system status")
        )
        .subcommand(
            Command::new("deploy")
                .about("Deploy system to Render")
        )
        .get_matches();

    let endpoint = matches.get_one::<String>("endpoint").unwrap().to_string();

    match matches.subcommand() {
        ("monitor", _) => {
            println!("🚀 Starting Forex CLI Controller...");
            println!("📡 Connecting to: {}", endpoint);
            
            let mut controller = ForexCliController::new(endpoint)?;
            controller.run().await?;
        }
        ("status", _) => {
            let client = Client::new();
            let url = format!("{}/api/status", endpoint);
            let response = client.get(&url).send().await?;
            println!("{}", response.text().await?);
        }
        ("deploy", _) => {
            println!("🚀 Deploying to Render...");
            // This will use the Render MCP tools
        }
        _ => {
            println!("Use --help for available commands");
        }
    }

    Ok(())
}
