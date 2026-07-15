use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use reqwest::Client;
use anyhow::{Result, Context, anyhow};
use chrono::Local;
use url::Url;
use dotenvy::dotenv;
use thiserror::Error;
use tracing::{info, error, warn, instrument};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use metrics::{counter, histogram};
use metrics_exporter_prometheus::PrometheusBuilder;
use axum::{routing::get, Router};
use std::net::SocketAddr;

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
    #[serde(default = "default_metrics_port")]
    metrics_port: u16,
}

fn default_metrics_port() -> u16 {
    9090
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
    #[instrument(skip(self, endpoint), fields(endpoint_name = %endpoint.name))]
    async fn check_endpoint(&self, endpoint: &Endpoint) -> Result<bool, MonitorError> {
        info!(url = %endpoint.url, "Iniciando checagem de endpoint");
        
        let start = Instant::now();
        let response_result = self.client.get(&endpoint.url).send().await;
        let duration = start.elapsed();

        // Registra a latência da requisição
        histogram!("api_check_duration_seconds", duration.as_secs_f64(), "endpoint" => endpoint.name.clone());

        let response = match response_result {
            Ok(res) => res,
            Err(e) => {
                counter!("api_check_errors_total", 1, "endpoint" => endpoint.name.clone(), "reason" => "request_failed");
                return Err(MonitorError::HttpRequestError { 
                    name: endpoint.name.clone(), 
                    source: e 
                });
            }
        };

        let status = response.status().as_u16();
        let is_up = status == endpoint.expected_status;

        if is_up {
            info!(status = status, "Endpoint está UP");
            counter!("api_check_success_total", 1, "endpoint" => endpoint.name.clone());
            Ok(true)
        } else {
            warn!(status = status, expected = endpoint.expected_status, "Endpoint está DOWN!");
            counter!("api_check_errors_total", 1, "endpoint" => endpoint.name.clone(), "reason" => "wrong_status");
            Ok(false)
        }
    }

    /// Envia notificação via webhook se habilitado
    #[instrument(skip(self, endpoint, is_up), fields(endpoint_name = %endpoint.name))]
    async fn send_notification(&self, endpoint: &Endpoint, is_up: bool) -> Result<()> {
        if !self.config.notifications.enabled {
            return Ok(());
        }

        let webhook_url = self.config.notifications.webhook_url.as_ref()
            .context("Webhook URL não configurada, mas notificações estão ativadas")?;

        let status_str = if is_up { "VOLTOU A FICAR ONLINE" } else { "ESTÁ FORA DO AR" };
        let emoji = if is_up { "✅" } else { "🚨" };
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let payload = serde_json::json!({
            "content": format!("{} **Monitor de Saúde**\n**Serviço:** {}\n**Status:** {}\n**Horário:** `{}`", 
                emoji, endpoint.name, status_str, timestamp)
        });

        info!("Enviando notificação para o webhook");
        self.client.post(webhook_url)
            .json(&payload)
            .send()
            .await
            .context("Falha ao enviar notificação para o webhook")?;

        Ok(())
    }

    /// Loop principal de execução do monitor
    pub async fn run(&self) {
        info!(
            endpoint_count = self.config.endpoints.len(),
            interval = self.config.check_interval_seconds,
            metrics_port = self.config.metrics_port,
            "Iniciando Monitor de Saúde de APIs..."
        );

        let mut last_states = vec![true; self.config.endpoints.len()];

        loop {
            for (i, endpoint) in self.config.endpoints.iter().enumerate() {
                match self.check_endpoint(endpoint).await {
                    Ok(current_state) => {
                        if current_state != last_states[i] {
                            if let Err(e) = self.send_notification(endpoint, current_state).await {
                                error!(error = %e, "Erro ao enviar notificação");
                            }
                            last_states[i] = current_state;
                        }
                    }
                    Err(e) => {
                        error!(error = %e, "Erro crítico ao monitorar endpoint");
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

async fn start_metrics_server(port: u16) -> Result<()> {
    let builder = PrometheusBuilder::new();
    let handle = builder
        .install_recorder()
        .context("Falha ao instalar o gravador Prometheus")?;

    let app = Router::new().route("/metrics", get(move || async move { handle.render() }));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Servidor de métricas ouvindo em http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializa o tracing com suporte a variáveis de ambiente (RUST_LOG)
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

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

    // Valida configuração
    config.validate().map_err(|e| anyhow!("Erro de configuração: {}", e))?;

    let metrics_port = config.metrics_port;
    let monitor = Monitor::new(config)?;

    // Inicia o servidor de métricas em uma tarefa separada
    tokio::spawn(async move {
        if let Err(e) = start_metrics_server(metrics_port).await {
            error!(error = %e, "Falha ao iniciar o servidor de métricas");
        }
    });

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
            metrics_port: 9090,
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
}
