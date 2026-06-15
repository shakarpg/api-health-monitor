use serde::{Deserialize, Serialize};
use std::fs;
use std::time::Duration;
use tokio::time::sleep;
use reqwest::Client;
use anyhow::{Result, Context, anyhow};
use log::{info, error, warn};
use chrono::Local;
use url::Url;
use dotenvy::dotenv;
use thiserror::Error;

/// Erros customizados para o Monitor de Saúde
#[derive(Error, Debug)]
pub enum MonitorError {
    #[error("Intervalo de checagem inválido: {0}. Deve ser maior que 0.")]
    InvalidInterval(u64),
    #[error("Status esperado inválido para {name}: {status}. Deve estar entre 100 e 599.")]
    InvalidStatus { name: String, status: u16 },
    #[error("URL inválida para {name}: {source}")]
    InvalidUrl { name: String, source: url::ParseError },
    #[error("Configuração de notificação incompleta: Webhook URL é obrigatória quando ativada.")]
    MissingWebhook,
    #[error("Falha na requisição HTTP para {name}: {source}")]
    HttpRequestError { name: String, source: reqwest::Error },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Endpoint {
    name: String,
    url: String,
    expected_status: u16,
}

#[derive(Debug, Deserialize, Serialize)]
struct Notifications {
    webhook_url: Option<String>,
    enabled: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    check_interval_seconds: u64,
    endpoints: Vec<Endpoint>,
    notifications: Notifications,
}

impl Config {
    /// Valida as configurações do monitor
    fn validate(&self) -> Result<(), MonitorError> {
        if self.check_interval_seconds == 0 {
            return Err(MonitorError::InvalidInterval(self.check_interval_seconds));
        }

        for endpoint in &self.endpoints {
            if endpoint.expected_status < 100 || endpoint.expected_status > 599 {
                return Err(MonitorError::InvalidStatus { 
                    name: endpoint.name.clone(), 
                    status: endpoint.expected_status 
                });
            }
            Url::parse(&endpoint.url).map_err(|e| MonitorError::InvalidUrl { 
                name: endpoint.name.clone(), 
                source: e 
            })?;
        }

        if self.notifications.enabled && self.notifications.webhook_url.is_none() {
            return Err(MonitorError::MissingWebhook);
        }

        Ok(())
    }
}

struct Monitor {
    config: Config,
    client: Client,
}

impl Monitor {
    fn new(config: Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .context("Falha ao criar cliente HTTP")?;

        Ok(Self {
            config,
            client,
        })
    }

    /// Verifica o status de um endpoint específico
    async fn check_endpoint(&self, endpoint: &Endpoint) -> Result<bool, MonitorError> {
        let response = self.client.get(&endpoint.url).send().await
            .map_err(|e| MonitorError::HttpRequestError { 
                name: endpoint.name.clone(), 
                source: e 
            })?;

        let status = response.status().as_u16();
        if status == endpoint.expected_status {
            info!("[{}] {} está UP (Status: {})", endpoint.name, endpoint.url, status);
            Ok(true)
        } else {
            warn!("[{}] {} está DOWN! Esperado: {}, Recebido: {}", 
                endpoint.name, endpoint.url, endpoint.expected_status, status);
            Ok(false)
        }
    }

    /// Envia notificação via webhook se habilitado
    async fn send_notification(&self, endpoint: &Endpoint, is_up: bool) -> Result<()> {
        if !self.config.notifications.enabled {
            return Ok(());
        }

        // Uso idiomático de Option para obter a URL do webhook
        let webhook_url = self.config.notifications.webhook_url.as_ref()
            .context("Webhook URL não configurada, mas notificações estão ativadas")?;

        let status_str = if is_up { "VOLTOU A FICAR ONLINE" } else { "ESTÁ FORA DO AR" };
        let emoji = if is_up { "✅" } else { "🚨" };
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let payload = serde_json::json!({
            "content": format!("{} **Monitor de Saúde**\n**Serviço:** {}\n**Status:** {}\n**Horário:** `{}`", 
                emoji, endpoint.name, status_str, timestamp)
        });

        self.client.post(webhook_url)
            .json(&payload)
            .send()
            .await
            .context("Falha ao enviar notificação para o webhook")?;

        Ok(())
    }

    /// Loop principal de execução do monitor
    pub async fn run(&self) {
        info!("Iniciando Monitor de Saúde de APIs...");
        info!("Monitorando {} endpoints a cada {} segundos.", 
            self.config.endpoints.len(), self.config.check_interval_seconds);

        let mut last_states = vec![true; self.config.endpoints.len()];

        loop {
            for (i, endpoint) in self.config.endpoints.iter().enumerate() {
                // Tratamento de erro robusto ao verificar endpoint
                match self.check_endpoint(endpoint).await {
                    Ok(current_state) => {
                        if current_state != last_states[i] {
                            if let Err(e) = self.send_notification(endpoint, current_state).await {
                                error!("Erro ao enviar notificação para {}: {}", endpoint.name, e);
                            }
                            last_states[i] = current_state;
                        }
                    }
                    Err(e) => {
                        error!("Erro crítico ao monitorar {}: {}", endpoint.name, e);
                        // Em caso de erro de rede, assumimos que o estado pode ter mudado para DOWN
                        if last_states[i] {
                            let _ = self.send_notification(endpoint, false).await;
                            last_states[i] = false;
                        }
                    }
                }
            }

            sleep(Duration::from_secs(self.config.check_interval_seconds)).await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializa o logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Carrega .env se existir
    dotenv().ok();

    // Lê o arquivo de configuração
    let config_content = fs::read_to_string("config.yaml")
        .context("Não foi possível ler o arquivo config.yaml. Certifique-se de criá-lo a partir do config.example.yaml")?;
    
    let mut config: Config = serde_yaml::from_str(&config_content)
        .context("Erro ao fazer parse do arquivo config.yaml")?;

    // Sobrescreve webhook se houver variável de ambiente
    if let Ok(env_webhook) = std::env::var("DISCORD_WEBHOOK_URL") {
        if !env_webhook.is_empty() && env_webhook != "${DISCORD_WEBHOOK_URL}" {
            config.notifications.webhook_url = Some(env_webhook);
        }
    }

    // Valida configuração com o novo sistema de erros
    config.validate().map_err(|e| anyhow!("Erro de configuração: {}", e))?;

    let monitor = Monitor::new(config)?;
    monitor.run().await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_check_endpoint_up() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let config = Config {
            check_interval_seconds: 60,
            endpoints: vec![],
            notifications: Notifications { enabled: false, webhook_url: None },
        };
        let monitor = Monitor::new(config).unwrap();
        
        let endpoint = Endpoint {
            name: "Test".to_string(),
            url: format!("{}/health", server.uri()),
            expected_status: 200,
        };

        let result = monitor.check_endpoint(&endpoint).await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_check_endpoint_down() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let config = Config {
            check_interval_seconds: 60,
            endpoints: vec![],
            notifications: Notifications { enabled: false, webhook_url: None },
        };
        let monitor = Monitor::new(config).unwrap();
        
        let endpoint = Endpoint {
            name: "Test".to_string(),
            url: format!("{}/health", server.uri()),
            expected_status: 200,
        };

        let result = monitor.check_endpoint(&endpoint).await.unwrap();
        assert!(!result);
    }
}
