use serde::{Deserialize, Serialize};
use std::fs;
use std::time::Duration;
use tokio::time::sleep;
use reqwest::Client;
use anyhow::{Result, Context};
use log::{info, error, warn};
use chrono::Local;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Endpoint {
    name: String,
    url: String,
    expected_status: u16,
}

#[derive(Debug, Deserialize, Serialize)]
struct Notifications {
    webhook_url: String,
    enabled: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    check_interval_seconds: u64,
    endpoints: Vec<Endpoint>,
    notifications: Notifications,
}

struct Monitor {
    config: Config,
    client: Client,
}

impl Monitor {
    fn new(config: Config) -> Self {
        Self {
            config,
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Falha ao criar cliente HTTP"),
        }
    }

    async fn check_endpoint(&self, endpoint: &Endpoint) -> Result<bool> {
        match self.client.get(&endpoint.url).send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                if status == endpoint.expected_status {
                    info!("[{}] {} está UP (Status: {})", endpoint.name, endpoint.url, status);
                    Ok(true)
                } else {
                    warn!("[{}] {} está DOWN! Esperado: {}, Recebido: {}", 
                        endpoint.name, endpoint.url, endpoint.expected_status, status);
                    Ok(false)
                }
            }
            Err(e) => {
                error!("[{}] Erro ao acessar {}: {}", endpoint.name, endpoint.url, e);
                Ok(false)
            }
        }
    }

    async fn send_notification(&self, endpoint: &Endpoint, is_up: bool) -> Result<()> {
        if !self.config.notifications.enabled || self.config.notifications.webhook_url.contains("SUA_WEBHOOK_AQUI") {
            return Ok(());
        }

        let status_str = if is_up { "VOLTOU A FICAR ONLINE" } else { "ESTÁ FORA DO AR" };
        let emoji = if is_up { "✅" } else { "🚨" };
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let payload = serde_json::json!({
            "content": format!("{} **Monitor de Saúde**\n**Serviço:** {}\n**Status:** {}\n**Horário:** `{}`", 
                emoji, endpoint.name, status_str, timestamp)
        });

        self.client.post(&self.config.notifications.webhook_url)
            .json(&payload)
            .send()
            .await
            .context("Falha ao enviar notificação para o webhook")?;

        Ok(())
    }

    pub async fn run(&self) {
        info!("Iniciando Monitor de Saúde de APIs...");
        info!("Monitorando {} endpoints a cada {} segundos.", 
            self.config.endpoints.len(), self.config.check_interval_seconds);

        // Estado simples para evitar spam de notificações (apenas notifica na mudança de estado)
        let mut last_states = vec![true; self.config.endpoints.len()];

        loop {
            for (i, endpoint) in self.config.endpoints.iter().enumerate() {
                let current_state = self.check_endpoint(endpoint).await.unwrap_or(false);
                
                // Se o estado mudou, envia notificação
                if current_state != last_states[i] {
                    if let Err(e) = self.send_notification(endpoint, current_state).await {
                        error!("Erro ao enviar notificação: {}", e);
                    }
                    last_states[i] = current_state;
                }
            }

            sleep(Duration::from_secs(self.config.check_interval_seconds)).await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializa o logger (RUST_LOG=info cargo run)
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Lê o arquivo de configuração
    let config_content = fs::read_to_string("config.yaml")
        .context("Não foi possível ler o arquivo config.yaml")?;
    
    let config: Config = serde_yaml::from_str(&config_content)
        .context("Erro ao fazer parse do arquivo config.yaml")?;

    let monitor = Monitor::new(config);
    monitor.run().await;

    Ok(())
}
