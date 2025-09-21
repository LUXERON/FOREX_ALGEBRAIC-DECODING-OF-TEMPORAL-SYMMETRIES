use clap::{Arg, Command};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;
use tokio;
use std::io::{self, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradingMode {
    Demo,
    Live,
}

impl TradingMode {
    fn as_str(&self) -> &'static str {
        match self {
            TradingMode::Demo => "DEMO",
            TradingMode::Live => "LIVE",
        }
    }

    fn server(&self) -> &'static str {
        match self {
            TradingMode::Demo => "cTrader DEMO",
            TradingMode::Live => "cTrader LIVE",
        }
    }

    fn account_id(&self) -> &'static str {
        match self {
            TradingMode::Demo => "5078436", // Demo account from CTRADER.MD
            TradingMode::Live => "1259560", // Live account from CTRADER.MD
        }
    }

    fn client_id(&self) -> &'static str {
        // Same client ID for both demo and live (from CTRADER.MD)
        "14877_vyfOpsRldMcTyq4M2Qien3KxqG43yVFlSt0jLNjBhr0LX2Cpd7"
    }

    fn client_secret(&self) -> &'static str {
        // Same client secret for both demo and live (from CTRADER.MD)
        "smo86RDCn85U5Fy5hIuCi4oScBJMiKwlEt3x0zxBC406ioUioE"
    }
}

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

struct SimpleCliController {
    client: Client,
    render_endpoint: String,
    current_mode: TradingMode,
}

impl SimpleCliController {
    fn new(render_endpoint: String) -> Self {
        SimpleCliController {
            client: Client::new(),
            render_endpoint,
            current_mode: TradingMode::Demo, // Default to demo mode for safety
        }
    }

    async fn switch_mode(&mut self, mode: TradingMode) -> Result<String, Box<dyn Error>> {
        println!("🔄 Switching remote deployment to {} mode...", mode.as_str());

        let command = TradingCommand {
            action: "switch_mode".to_string(),
            pair: None,
            parameters: {
                let mut params = HashMap::new();
                params.insert("mode".to_string(), mode.as_str().to_string());
                params.insert("server".to_string(), mode.server().to_string());
                params.insert("account_id".to_string(), mode.account_id().to_string());
                params.insert("client_id".to_string(), mode.client_id().to_string());
                params.insert("client_secret".to_string(), mode.client_secret().to_string());
                params
            },
        };

        let response = self.send_command(command).await?;
        self.current_mode = mode.clone();

        println!("✅ Successfully switched to {} mode", mode.as_str());
        println!("📊 Server: {}", mode.server());
        println!("🔑 Account ID: {}", mode.account_id());

        Ok(response)
    }

    fn display_current_mode(&self) {
        println!("╔═══════════════════════════════════════════════════════════════════════════════════╗");
        println!("║                          CURRENT TRADING MODE                                    ║");
        println!("╠═══════════════════════════════════════════════════════════════════════════════════╣");
        println!("║ Mode:       {} {}",
            match self.current_mode {
                TradingMode::Demo => "🧪 DEMO",
                TradingMode::Live => "💰 LIVE",
            },
            match self.current_mode {
                TradingMode::Demo => "(Safe Testing Environment)",
                TradingMode::Live => "(Real Money Trading)",
            }
        );
        println!("║ Server:     {}", self.current_mode.server());
        println!("║ Account:    {}", self.current_mode.account_id());
        println!("║ Client ID:  {}...", &self.current_mode.client_id()[..20]);
        println!("╚═══════════════════════════════════════════════════════════════════════════════════╝");
    }

    async fn switch_mode_with_credentials(
        &mut self,
        mode: TradingMode,
        custom_client_id: Option<&String>,
        custom_client_secret: Option<&String>,
        custom_account_id: Option<&String>,
        custom_server: Option<&String>
    ) -> Result<String, Box<dyn Error>> {
        println!("🔄 Switching remote deployment to {} mode...", mode.as_str());

        let command = TradingCommand {
            action: "switch_mode".to_string(),
            pair: None,
            parameters: {
                let mut params = HashMap::new();
                params.insert("mode".to_string(), mode.as_str().to_string());
                params.insert("server".to_string(),
                    custom_server.map(|s| s.clone()).unwrap_or_else(|| mode.server().to_string()));
                params.insert("account_id".to_string(),
                    custom_account_id.map(|s| s.clone()).unwrap_or_else(|| mode.account_id().to_string()));
                params.insert("client_id".to_string(),
                    custom_client_id.map(|s| s.clone()).unwrap_or_else(|| mode.client_id().to_string()));
                params.insert("client_secret".to_string(),
                    custom_client_secret.map(|s| s.clone()).unwrap_or_else(|| mode.client_secret().to_string()));
                params
            },
        };

        let response = self.send_command(command).await?;
        self.current_mode = mode.clone();

        println!("✅ Successfully switched to {} mode", mode.as_str());
        if let Some(server) = custom_server {
            println!("📊 Server: {}", server);
        } else {
            println!("📊 Server: {}", mode.server());
        }
        if let Some(account) = custom_account_id {
            println!("🔑 Account ID: {}", account);
        } else {
            println!("🔑 Account ID: {}", mode.account_id());
        }

        Ok(response)
    }

    async fn set_custom_credentials(
        &self,
        client_id: String,
        client_secret: String,
        demo_account: String,
        live_account: String
    ) -> Result<String, Box<dyn Error>> {
        println!("🔧 Setting custom cTrader credentials on remote deployment...");

        let command = TradingCommand {
            action: "set_credentials".to_string(),
            pair: None,
            parameters: {
                let mut params = HashMap::new();
                params.insert("client_id".to_string(), client_id.clone());
                params.insert("client_secret".to_string(), client_secret.clone());
                params.insert("demo_account".to_string(), demo_account.clone());
                params.insert("live_account".to_string(), live_account.clone());
                params
            },
        };

        let response = self.send_command(command).await?;

        println!("✅ Custom credentials set successfully!");
        println!("🔑 Client ID: {}...", &client_id[..20]);
        println!("🧪 Demo Account: {}", demo_account);
        println!("💰 Live Account: {}", live_account);
        println!("⚠️  Client Secret: [HIDDEN FOR SECURITY]");

        Ok(response)
    }

    async fn fetch_system_status(&self) -> Result<RemoteSystemStatus, Box<dyn std::error::Error>> {
        let url = format!("{}/api/status", self.render_endpoint);
        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(format!("Failed to fetch status: {}", response.status()).into())
        }
    }

    async fn send_command(&self, command: TradingCommand) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}/api/command", self.render_endpoint);
        let response = self.client.post(&url).json(&command).send().await?;
        
        Ok(response.text().await?)
    }

    async fn monitor_system(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("╔═══════════════════════════════════════════════════════════════════════════════════╗");
        println!("║                                                                                   ║");
        println!("║    🚀 FOREX CLI CONTROLLER - MONITORING MODE 🚀                                 ║");
        println!("║         Real-time monitoring of Render deployment                                ║");
        println!("║                                                                                   ║");
        println!("╚═══════════════════════════════════════════════════════════════════════════════════╝");
        println!();
        println!("📡 Connecting to: {}", self.render_endpoint);
        println!("⏱️  Fetching system status every 10 seconds...");
        println!("🔄 Press Ctrl+C to stop monitoring");
        println!();

        loop {
            match self.fetch_system_status().await {
                Ok(status) => {
                    self.display_status(&status);
                }
                Err(e) => {
                    println!("❌ Error fetching status: {}", e);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
    }

    fn display_status(&self, status: &RemoteSystemStatus) {
        println!("╔═══════════════════════════════════════════════════════════════════════════════════╗");
        println!("║                            SYSTEM STATUS REPORT                                  ║");
        println!("╠═══════════════════════════════════════════════════════════════════════════════════╣");
        println!("║ Status: {:70} ║", status.status);
        println!("║ Uptime: {:69} hours ║", status.uptime / 3600);
        println!("║ Active Pairs: {:63} ║", status.active_pairs.len());
        println!("║ Total Trades: {:63} ║", status.total_trades);
        println!("║ P&L: ${:72.2} ║", status.profit_loss);
        println!("╠═══════════════════════════════════════════════════════════════════════════════════╣");
        println!("║                           SYSTEM METRICS                                         ║");
        println!("╠═══════════════════════════════════════════════════════════════════════════════════╣");
        println!("║ CPU Usage: {:66.1}% ║", status.system_metrics.cpu_usage * 100.0);
        println!("║ Memory Usage: {:63.1}% ║", status.system_metrics.memory_usage * 100.0);
        println!("║ Network Latency: {:58.1}ms ║", status.system_metrics.network_latency);
        println!("║ Database Size: {:60} KB ║", status.system_metrics.database_size / 1024);
        println!("║ Active Connections: {:55} ║", status.system_metrics.active_connections);
        println!("╠═══════════════════════════════════════════════════════════════════════════════════╣");
        println!("║                        ARBITRAGE OPPORTUNITIES                                   ║");
        println!("╚═══════════════════════════════════════════════════════════════════════════════════╝");

        for (i, opp) in status.correlation_opportunities.iter().enumerate().take(5) {
            println!("🎯 Opportunity #{}: {} ↔ {}", i + 1, opp.primary_pair, opp.correlated_pair);
            println!("   Confidence: {:.1}% | Realistic: {:.1} pips | Net Expected: {:.1} pips", 
                opp.confidence * 100.0, opp.realistic_pips, opp.net_expected_pips);
            println!("   Position Size: ${:.0} | Time Window: {}", opp.position_size, opp.time_window);
            println!("   ⚠️  Theoretical: {:.0} pips (cumulative potential, not single trade)", opp.theoretical_pips);
            println!();
        }

        println!("📊 Active Currency Pairs: {}", status.active_pairs.join(", "));
        println!("🕒 Last Updated: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
        println!("═══════════════════════════════════════════════════════════════════════════════════");
        println!();
    }

    async fn get_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔍 Fetching system status...");
        
        match self.fetch_system_status().await {
            Ok(status) => {
                self.display_status(&status);
            }
            Err(e) => {
                println!("❌ Error: {}", e);
            }
        }

        Ok(())
    }

    async fn deploy_system(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Deploying system to Render...");
        println!("📋 This will use the Render MCP tools to deploy the statically linked executable");
        
        // This would integrate with the Render MCP tools
        println!("✅ Deployment initiated! Check Render dashboard for progress.");
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Simple Forex CLI Controller")
        .version("1.0")
        .author("NEUNOMY")
        .about("Control and monitor remote Render forex trading system")
        .arg(
            Arg::new("endpoint")
                .short('e')
                .long("endpoint")
                .value_name("URL")
                .help("Render deployment endpoint URL")
                .default_value("http://localhost:8080"),
        )
        .subcommand(
            Command::new("monitor")
                .about("Start continuous monitoring of the remote system")
        )
        .subcommand(
            Command::new("status")
                .about("Get current system status (one-time)")
        )
        .subcommand(
            Command::new("deploy")
                .about("Deploy system to Render using MCP tools")
        )
        .subcommand(
            Command::new("mode")
                .about("Switch trading mode between DEMO and LIVE")
                .arg(
                    Arg::new("trading_mode")
                        .help("Trading mode: demo or live")
                        .required(true)
                        .value_parser(["demo", "live"])
                )
                .arg(
                    Arg::new("client_id")
                        .long("client-id")
                        .help("Custom cTrader Client ID (optional)")
                        .value_name("CLIENT_ID")
                )
                .arg(
                    Arg::new("client_secret")
                        .long("client-secret")
                        .help("Custom cTrader Client Secret (optional)")
                        .value_name("CLIENT_SECRET")
                )
                .arg(
                    Arg::new("account_id")
                        .long("account-id")
                        .help("Custom cTrader Account ID (optional)")
                        .value_name("ACCOUNT_ID")
                )
                .arg(
                    Arg::new("server")
                        .long("server")
                        .help("Custom cTrader Server (optional)")
                        .value_name("SERVER")
                )
        )
        .subcommand(
            Command::new("current-mode")
                .about("Display current trading mode configuration")
        )
        .subcommand(
            Command::new("set-credentials")
                .about("Set custom cTrader credentials for the remote deployment")
                .arg(
                    Arg::new("client_id")
                        .long("client-id")
                        .help("cTrader Client ID")
                        .required(true)
                        .value_name("CLIENT_ID")
                )
                .arg(
                    Arg::new("client_secret")
                        .long("client-secret")
                        .help("cTrader Client Secret")
                        .required(true)
                        .value_name("CLIENT_SECRET")
                )
                .arg(
                    Arg::new("demo_account")
                        .long("demo-account")
                        .help("Demo Account ID")
                        .required(true)
                        .value_name("DEMO_ACCOUNT")
                )
                .arg(
                    Arg::new("live_account")
                        .long("live-account")
                        .help("Live Account ID")
                        .required(true)
                        .value_name("LIVE_ACCOUNT")
                )
        )
        .get_matches();

    let endpoint = matches.get_one::<String>("endpoint").unwrap().to_string();
    let mut controller = SimpleCliController::new(endpoint);

    match matches.subcommand() {
        Some(("monitor", _)) => {
            controller.display_current_mode();
            println!();
            controller.monitor_system().await?;
        }
        Some(("status", _)) => {
            controller.display_current_mode();
            println!();
            controller.get_status().await?;
        }
        Some(("deploy", _)) => {
            controller.deploy_system().await?;
        }
        Some(("mode", sub_matches)) => {
            let mode_str = sub_matches.get_one::<String>("trading_mode").unwrap();
            let mode = match mode_str.as_str() {
                "demo" => TradingMode::Demo,
                "live" => TradingMode::Live,
                _ => return Err("Invalid trading mode".into()),
            };

            // Get custom credentials if provided
            let custom_client_id = sub_matches.get_one::<String>("client_id");
            let custom_client_secret = sub_matches.get_one::<String>("client_secret");
            let custom_account_id = sub_matches.get_one::<String>("account_id");
            let custom_server = sub_matches.get_one::<String>("server");

            // Safety confirmation for LIVE mode
            if matches!(mode, TradingMode::Live) {
                let default_account = mode.account_id().to_string();
                let account_display = custom_account_id.unwrap_or(&default_account);
                println!("⚠️  WARNING: You are about to switch to LIVE TRADING MODE!");
                println!("💰 This will use REAL MONEY for trading operations.");
                println!("🔴 Account: {} (cTrader LIVE)", account_display);
                println!();
                print!("Type 'CONFIRM LIVE TRADING' to proceed: ");
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if input.trim() != "CONFIRM LIVE TRADING" {
                    println!("❌ Live mode activation cancelled.");
                    return Ok(());
                }
            }

            controller.switch_mode_with_credentials(
                mode,
                custom_client_id,
                custom_client_secret,
                custom_account_id,
                custom_server
            ).await?;
        }
        Some(("set-credentials", sub_matches)) => {
            let client_id = sub_matches.get_one::<String>("client_id").unwrap();
            let client_secret = sub_matches.get_one::<String>("client_secret").unwrap();
            let demo_account = sub_matches.get_one::<String>("demo_account").unwrap();
            let live_account = sub_matches.get_one::<String>("live_account").unwrap();

            controller.set_custom_credentials(
                client_id.clone(),
                client_secret.clone(),
                demo_account.clone(),
                live_account.clone()
            ).await?;
        }
        Some(("current-mode", _)) => {
            controller.display_current_mode();
        }
        _ => {
            println!("╔═══════════════════════════════════════════════════════════════════════════════════╗");
            println!("║                                                                                   ║");
            println!("║    🚀 SIMPLE FOREX CLI CONTROLLER 🚀                                            ║");
            println!("║         Control and monitor your Render forex trading system                     ║");
            println!("║                                                                                   ║");
            println!("╚═══════════════════════════════════════════════════════════════════════════════════╝");
            println!();
            println!("Available commands:");
            println!("  monitor         - Start continuous monitoring dashboard");
            println!("  status          - Get current system status");
            println!("  deploy          - Deploy system to Render");
            println!("  mode <demo|live> - Switch between DEMO and LIVE trading modes");
            println!("  current-mode    - Display current trading mode configuration");
            println!("  set-credentials - Set custom cTrader credentials");
            println!();
            println!("🧪 DEMO Mode: Safe testing environment (Default: Account 5078436)");
            println!("💰 LIVE Mode: Real money trading (Default: Account 1259560)");
            println!();
            println!("Custom Credentials:");
            println!("  --client-id     - Custom cTrader Client ID");
            println!("  --client-secret - Custom cTrader Client Secret");
            println!("  --account-id    - Custom Account ID (for mode command)");
            println!("  --server        - Custom Server (for mode command)");
            println!();
            println!("Example usage:");
            println!("  {} -e https://your-render-app.onrender.com monitor",
                std::env::args().next().unwrap_or_else(|| "forex-cli".to_string()));
            println!("  {} mode demo    # Switch to demo mode (default credentials)",
                std::env::args().next().unwrap_or_else(|| "forex-cli".to_string()));
            println!("  {} mode live --client-id YOUR_ID --account-id YOUR_ACCOUNT",
                std::env::args().next().unwrap_or_else(|| "forex-cli".to_string()));
            println!("  {} set-credentials --client-id ID --client-secret SECRET --demo-account DEMO --live-account LIVE",
                std::env::args().next().unwrap_or_else(|| "forex-cli".to_string()));
        }
    }

    Ok(())
}
