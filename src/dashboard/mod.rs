//! # Real-Time Forex Pattern Recognition Dashboard
//! 
//! CLI dashboard for live pattern monitoring and analysis

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Axis, Block, Borders, Chart, Dataset, Gauge, List, ListItem, Paragraph, 
        Sparkline, Table, Row, Cell, Clear
    },
    Frame, Terminal,
};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::time::interval;

use crate::core::{TimeSymmetricEngine, EngineConfig};
use crate::data::{ForexDataPoint, ForexDataManager, DataConfig, RealTimeDataFeed};
use crate::patterns::{PatternRecognizer, PatternConfig, HiddenCycle};
use crate::symmetry::TemporalSymmetry;

/// Dashboard application state
pub struct DashboardApp {
    // Core components
    engine: TimeSymmetricEngine,
    data_manager: ForexDataManager,
    pattern_recognizer: PatternRecognizer,
    data_feed: RealTimeDataFeed,
    
    // UI state
    current_tab: usize,
    should_quit: bool,
    last_update: Instant,
    
    // Data
    price_history: VecDeque<(f64, f64)>, // (timestamp, price)
    detected_cycles: Vec<HiddenCycle>,
    temporal_symmetries: Vec<TemporalSymmetry>,
    current_pair: String,
    
    // Performance metrics
    pattern_strength: f64,
    symmetry_score: f64,
    prediction_accuracy: f64,
    processing_time: Duration,
}

impl DashboardApp {
    /// Create new dashboard application
    pub async fn new() -> Result<Self> {
        let engine_config = EngineConfig::default();
        let engine = TimeSymmetricEngine::new(engine_config)?;
        
        let data_config = DataConfig::default();
        let data_manager = ForexDataManager::new(data_config)?;
        
        let pattern_config = PatternConfig::default();
        let pattern_recognizer = PatternRecognizer::new(pattern_config)?;
        
        let data_feed = RealTimeDataFeed::default().await?;
        
        Ok(Self {
            engine,
            data_manager,
            pattern_recognizer,
            data_feed,
            current_tab: 0,
            should_quit: false,
            last_update: Instant::now(),
            price_history: VecDeque::with_capacity(1000),
            detected_cycles: Vec::new(),
            temporal_symmetries: Vec::new(),
            current_pair: "EURUSD".to_string(),
            pattern_strength: 0.0,
            symmetry_score: 0.0,
            prediction_accuracy: 0.0,
            processing_time: Duration::from_millis(0),
        })
    }
    
    /// Initialize the dashboard
    pub async fn initialize(&mut self) -> Result<()> {
        self.engine.initialize().await?;
        self.load_historical_data().await?;
        Ok(())
    }
    
    /// Load historical data for analysis
    async fn load_historical_data(&mut self) -> Result<()> {
        let data_path = std::path::PathBuf::from("FOREX DATA");
        let historical_data = self.data_manager.load_data(
            &data_path, 
            &self.current_pair, 
            "1D"
        ).await?;
        
        // Initialize price history with recent data
        for (i, point) in historical_data.iter().rev().take(100).enumerate() {
            let timestamp = i as f64;
            self.price_history.push_back((timestamp, point.close));
        }
        
        // Perform initial pattern analysis
        self.update_patterns(&historical_data).await?;
        
        Ok(())
    }
    
    /// Update pattern analysis
    async fn update_patterns(&mut self, data: &[ForexDataPoint]) -> Result<()> {
        let start_time = Instant::now();
        
        // Extract temporal symmetries
        self.temporal_symmetries = self.engine.extract_temporal_symmetries(data).await?;
        
        // Detect cycles
        self.detected_cycles = self.pattern_recognizer.detect_cycles(data).await?;
        
        // Calculate metrics
        self.pattern_strength = self.calculate_pattern_strength();
        self.symmetry_score = self.calculate_symmetry_score();
        self.prediction_accuracy = self.calculate_prediction_accuracy();
        
        self.processing_time = start_time.elapsed();
        
        Ok(())
    }
    
    /// Calculate overall pattern strength
    fn calculate_pattern_strength(&self) -> f64 {
        if self.detected_cycles.is_empty() {
            return 0.0;
        }
        
        self.detected_cycles.iter()
            .map(|c| c.confidence)
            .sum::<f64>() / self.detected_cycles.len() as f64
    }
    
    /// Calculate symmetry score
    fn calculate_symmetry_score(&self) -> f64 {
        if self.temporal_symmetries.is_empty() {
            return 0.0;
        }
        
        self.temporal_symmetries.iter()
            .map(|s| s.strength)
            .sum::<f64>() / self.temporal_symmetries.len() as f64
    }
    
    /// Calculate prediction accuracy
    fn calculate_prediction_accuracy(&self) -> f64 {
        // Placeholder - would calculate based on recent predictions vs actual
        0.75 + (self.symmetry_score * 0.2)
    }
    
    /// Handle keyboard input
    pub fn handle_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Tab => {
                self.current_tab = (self.current_tab + 1) % 4;
            }
            KeyCode::Char('1') => self.current_tab = 0,
            KeyCode::Char('2') => self.current_tab = 1,
            KeyCode::Char('3') => self.current_tab = 2,
            KeyCode::Char('4') => self.current_tab = 3,
            KeyCode::Char('r') => {
                // Refresh data
                self.last_update = Instant::now();
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Check if should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
    
    /// Update with new data
    pub async fn update(&mut self) -> Result<()> {
        // Simulate new data point
        if self.last_update.elapsed() > Duration::from_secs(1) {
            self.simulate_new_data_point();
            self.last_update = Instant::now();
        }
        
        Ok(())
    }
    
    /// Simulate new data point for demo
    fn simulate_new_data_point(&mut self) {
        let timestamp = self.price_history.len() as f64;
        let last_price = self.price_history.back().map(|(_, p)| *p).unwrap_or(1.1000);
        
        // Add some realistic price movement
        let change = (timestamp * 0.1).sin() * 0.001 + 
                    (timestamp * 0.05).cos() * 0.0005;
        let new_price = last_price + change;
        
        self.price_history.push_back((timestamp, new_price));
        
        // Keep only last 100 points
        if self.price_history.len() > 100 {
            self.price_history.pop_front();
        }
    }
}

/// Render the dashboard UI
pub fn render_dashboard(f: &mut Frame, app: &DashboardApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Footer
        ])
        .split(f.area());
    
    // Render header
    render_header(f, chunks[0], app);
    
    // Render main content based on current tab
    match app.current_tab {
        0 => render_overview_tab(f, chunks[1], app),
        1 => render_patterns_tab(f, chunks[1], app),
        2 => render_symmetries_tab(f, chunks[1], app),
        3 => render_performance_tab(f, chunks[1], app),
        _ => render_overview_tab(f, chunks[1], app),
    }
    
    // Render footer
    render_footer(f, chunks[2], app);
}

/// Render header with title and tabs
fn render_header(f: &mut Frame, area: Rect, app: &DashboardApp) {
    let tabs = ["Overview", "Patterns", "Symmetries", "Performance"];
    let tab_titles: Vec<Line> = tabs.iter().enumerate().map(|(i, &tab)| {
        if i == app.current_tab {
            Line::from(Span::styled(tab, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
        } else {
            Line::from(Span::styled(tab, Style::default().fg(Color::White)))
        }
    }).collect();
    
    let header = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled("🔬 FOREX PATTERN RECONSTRUCTION DASHBOARD", 
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" | "),
            Span::styled(&app.current_pair, Style::default().fg(Color::Green)),
        ]),
        Line::from(tab_titles.into_iter().map(|line| line.spans).flatten().collect::<Vec<_>>()),
    ]))
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);
    
    f.render_widget(header, area);
}

/// Render footer with controls
fn render_footer(f: &mut Frame, area: Rect, app: &DashboardApp) {
    let footer = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled("Controls: ", Style::default().fg(Color::Yellow)),
            Span::raw("Tab/1-4: Switch tabs | R: Refresh | Q/Esc: Quit"),
        ]),
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Green)),
            Span::raw(format!("Processing: {:.2}ms | Patterns: {} | Symmetries: {}", 
                             app.processing_time.as_millis(),
                             app.detected_cycles.len(),
                             app.temporal_symmetries.len())),
        ]),
    ]))
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Left);
    
    f.render_widget(footer, area);
}

/// Render overview tab
fn render_overview_tab(f: &mut Frame, area: Rect, app: &DashboardApp) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Left side: Price chart
    render_price_chart(f, chunks[0], app);

    // Right side: Metrics
    render_metrics_panel(f, chunks[1], app);
}

/// Render patterns tab
fn render_patterns_tab(f: &mut Frame, area: Rect, app: &DashboardApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Top: Detected cycles
    render_cycles_list(f, chunks[0], app);

    // Bottom: Pattern strength over time
    render_pattern_strength_chart(f, chunks[1], app);
}

/// Render symmetries tab
fn render_symmetries_tab(f: &mut Frame, area: Rect, app: &DashboardApp) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left: Temporal symmetries
    render_symmetries_list(f, chunks[0], app);

    // Right: Symmetry visualization
    render_symmetry_chart(f, chunks[1], app);
}

/// Render performance tab
fn render_performance_tab(f: &mut Frame, area: Rect, app: &DashboardApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Performance gauges
            Constraint::Min(0),     // Performance history
        ])
        .split(area);

    // Top: Performance gauges
    render_performance_gauges(f, chunks[0], app);

    // Bottom: Performance history
    render_performance_history(f, chunks[1], app);
}

/// Render price chart
fn render_price_chart(f: &mut Frame, area: Rect, app: &DashboardApp) {
    let price_data: Vec<(f64, f64)> = app.price_history.iter().cloned().collect();

    if price_data.is_empty() {
        let placeholder = Paragraph::new("Loading price data...")
            .block(Block::default().title("Price Chart").borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(placeholder, area);
        return;
    }

    let min_price = price_data.iter().map(|(_, p)| *p).fold(f64::INFINITY, f64::min);
    let max_price = price_data.iter().map(|(_, p)| *p).fold(f64::NEG_INFINITY, f64::max);
    let price_range = max_price - min_price;

    let datasets = vec![
        Dataset::default()
            .name(app.current_pair.as_str())
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Cyan))
            .data(&price_data)
    ];

    let chart = Chart::new(datasets)
        .block(Block::default().title("Real-Time Price Chart").borders(Borders::ALL))
        .x_axis(
            Axis::default()
                .title("Time")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, price_data.len() as f64])
        )
        .y_axis(
            Axis::default()
                .title("Price")
                .style(Style::default().fg(Color::Gray))
                .bounds([min_price - price_range * 0.1, max_price + price_range * 0.1])
        );

    f.render_widget(chart, area);
}

/// Render metrics panel
fn render_metrics_panel(f: &mut Frame, area: Rect, app: &DashboardApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Pattern strength
            Constraint::Length(3),  // Symmetry score
            Constraint::Length(3),  // Prediction accuracy
            Constraint::Min(0),     // Additional info
        ])
        .split(area);

    // Pattern strength gauge
    let pattern_gauge = Gauge::default()
        .block(Block::default().title("Pattern Strength").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green))
        .percent((app.pattern_strength * 100.0) as u16);
    f.render_widget(pattern_gauge, chunks[0]);

    // Symmetry score gauge
    let symmetry_gauge = Gauge::default()
        .block(Block::default().title("Symmetry Score").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Blue))
        .percent((app.symmetry_score * 100.0) as u16);
    f.render_widget(symmetry_gauge, chunks[1]);

    // Prediction accuracy gauge
    let accuracy_gauge = Gauge::default()
        .block(Block::default().title("Prediction Accuracy").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Yellow))
        .percent((app.prediction_accuracy * 100.0) as u16);
    f.render_widget(accuracy_gauge, chunks[2]);

    // Additional info
    let info_text = vec![
        Line::from(vec![
            Span::styled("Cycles Detected: ", Style::default().fg(Color::White)),
            Span::styled(app.detected_cycles.len().to_string(), Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("Symmetries Found: ", Style::default().fg(Color::White)),
            Span::styled(app.temporal_symmetries.len().to_string(), Style::default().fg(Color::Blue)),
        ]),
        Line::from(vec![
            Span::styled("Processing Time: ", Style::default().fg(Color::White)),
            Span::styled(format!("{:.2}ms", app.processing_time.as_millis()), Style::default().fg(Color::Yellow)),
        ]),
    ];

    let info = Paragraph::new(Text::from(info_text))
        .block(Block::default().title("Analysis Info").borders(Borders::ALL))
        .alignment(Alignment::Left);
    f.render_widget(info, chunks[3]);
}

/// Render cycles list
fn render_cycles_list(f: &mut Frame, area: Rect, app: &DashboardApp) {
    let items: Vec<ListItem> = app.detected_cycles.iter().map(|cycle| {
        ListItem::new(Line::from(vec![
            Span::styled(format!("Period: {:.1}d", cycle.period), Style::default().fg(Color::White)),
            Span::raw(" | "),
            Span::styled(format!("Confidence: {:.2}", cycle.confidence), Style::default().fg(Color::Green)),
            Span::raw(" | "),
            Span::styled(format!("Amplitude: {:.3}", cycle.amplitude), Style::default().fg(Color::Yellow)),
        ]))
    }).collect();

    let list = List::new(items)
        .block(Block::default().title("Detected Cycles").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, area);
}

/// Render pattern strength chart
fn render_pattern_strength_chart(f: &mut Frame, area: Rect, _app: &DashboardApp) {
    // Generate sample pattern strength data
    let strength_data: Vec<u64> = (0..50).map(|i| {
        ((i as f64 * 0.1).sin() * 30.0 + 50.0) as u64
    }).collect();

    let sparkline = Sparkline::default()
        .block(Block::default().title("Pattern Strength Over Time").borders(Borders::ALL))
        .data(&strength_data)
        .style(Style::default().fg(Color::Green));

    f.render_widget(sparkline, area);
}

/// Render symmetries list
fn render_symmetries_list(f: &mut Frame, area: Rect, app: &DashboardApp) {
    let items: Vec<ListItem> = app.temporal_symmetries.iter().map(|symmetry| {
        ListItem::new(Line::from(vec![
            Span::styled(format!("Type: {}", symmetry.symmetry_type), Style::default().fg(Color::White)),
            Span::raw(" | "),
            Span::styled(format!("Strength: {:.3}", symmetry.strength), Style::default().fg(Color::Blue)),
            Span::raw(" | "),
            Span::styled(format!("Confidence: {:.2}", symmetry.confidence), Style::default().fg(Color::Cyan)),
        ]))
    }).collect();

    let list = List::new(items)
        .block(Block::default().title("Temporal Symmetries").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, area);
}

/// Render symmetry chart
fn render_symmetry_chart(f: &mut Frame, area: Rect, _app: &DashboardApp) {
    // Generate sample symmetry visualization data
    let symmetry_data: Vec<(f64, f64)> = (0..100).map(|i| {
        let x = i as f64;
        let y = (x * 0.1).sin() * 50.0 + 50.0;
        (x, y)
    }).collect();

    let datasets = vec![
        Dataset::default()
            .name("Symmetry Pattern")
            .marker(symbols::Marker::Dot)
            .style(Style::default().fg(Color::Blue))
            .data(&symmetry_data)
    ];

    let chart = Chart::new(datasets)
        .block(Block::default().title("Symmetry Visualization").borders(Borders::ALL))
        .x_axis(
            Axis::default()
                .title("Time")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, 100.0])
        )
        .y_axis(
            Axis::default()
                .title("Symmetry Strength")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, 100.0])
        );

    f.render_widget(chart, area);
}

/// Render performance gauges
fn render_performance_gauges(f: &mut Frame, area: Rect, app: &DashboardApp) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(area);

    // Pattern recognition performance
    let pattern_perf = Gauge::default()
        .block(Block::default().title("Pattern Recognition").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green))
        .percent((app.pattern_strength * 100.0) as u16);
    f.render_widget(pattern_perf, chunks[0]);

    // Symmetry detection performance
    let symmetry_perf = Gauge::default()
        .block(Block::default().title("Symmetry Detection").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Blue))
        .percent((app.symmetry_score * 100.0) as u16);
    f.render_widget(symmetry_perf, chunks[1]);

    // Overall system performance
    let overall_perf = Gauge::default()
        .block(Block::default().title("Overall Performance").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Yellow))
        .percent(((app.pattern_strength + app.symmetry_score) * 50.0) as u16);
    f.render_widget(overall_perf, chunks[2]);
}

/// Render performance history
fn render_performance_history(f: &mut Frame, area: Rect, _app: &DashboardApp) {
    // Generate sample performance history data
    let perf_data: Vec<u64> = (0..100).map(|i| {
        ((i as f64 * 0.05).sin() * 20.0 + 70.0) as u64
    }).collect();

    let sparkline = Sparkline::default()
        .block(Block::default().title("Performance History").borders(Borders::ALL))
        .data(&perf_data)
        .style(Style::default().fg(Color::Cyan));

    f.render_widget(sparkline, area);
}
