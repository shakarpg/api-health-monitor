# 🛡️ API Health Monitor (Rust)

Um monitor de saúde de APIs leve, assíncrono e profissional desenvolvido em Rust. Este projeto resolve uma dor real de empresas: **garantir que serviços críticos estejam online e ser notificado imediatamente quando algo falha.**

## 🚀 Funcionalidades

- **Assincronismo:** Utiliza `tokio` e `reqwest` para checagens de alta performance.
- **Configuração Simples:** Gerencie endpoints e intervalos via arquivo `config.yaml`.
- **Alertas Inteligentes:** Integração com Webhooks (Discord/Slack/Teams).
- **Notificações por Mudança de Estado:** Evita spam, notificando apenas quando um serviço cai ou volta a ficar online.
- **Logs Detalhados:** Sistema de logs estruturado para auditoria e debug.

## 🛠️ Tecnologias Utilizadas

- **Rust 2021 Edition**
- **Tokio:** Runtime assíncrona.
- **Reqwest:** Cliente HTTP para as checagens.
- **Serde & Serde YAML:** Serialização de configurações.
- **Anyhow:** Tratamento de erros simplificado.
- **Env Logger & Log:** Sistema de logging profissional.

## 📋 Pré-requisitos

- Rust instalado ([rustup.rs](https://rustup.rs))
- Cargo (gerenciador de pacotes do Rust)

## 🔧 Configuração

Edite o arquivo `config.yaml`:

```yaml
check_interval_seconds: 60

endpoints:
  - name: "Meu Serviço API"
    url: "https://api.exemplo.com/health"
    expected_status: 200

notifications:
  webhook_url: "SUA_WEBHOOK_URL_AQUI"
  enabled: true
```

## 🏃 Como Executar

1. Clone o repositório ou copie os arquivos.
2. Compile e execute:
   ```bash
   RUST_LOG=info cargo run
   ```

## 📦 Build para Produção

Para gerar um binário otimizado:
```bash
cargo build --release
```
O executável estará em `./target/release/api-health-monitor`.

## 🧠 Por que este projeto é profissional?

1. **Segurança de Memória:** Rust garante que não haverá *crashes* por acesso inválido à memória.
2. **Concorrência Segura:** O uso de `async/await` permite monitorar centenas de APIs com baixíssimo consumo de recursos.
3. **Tratamento de Erros:** Não usamos `unwrap()` em produção; erros de rede ou de parse são tratados de forma resiliente.
4. **Arquitetura Limpa:** Separação clara entre configuração, lógica de monitoramento e notificações.
