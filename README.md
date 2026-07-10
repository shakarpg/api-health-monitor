# 🛡️ API Health Monitor (Rust)

Um monitor de saúde de APIs leve, assíncrono e profissional desenvolvido em Rust. Este projeto resolve uma dor real de empresas: **garantir que serviços críticos estejam online e ser notificado imediatamente quando algo falha.**

## 🚀 Funcionalidades

- **Assincronismo:** Utiliza `tokio` e `reqwest` para checagens de alta performance.
- **Observabilidade Avançada:** Implementação de logs estruturados e rastreamento assíncrono usando `tracing` e `tracing-subscriber`.
- **Tratamento de Erros Idiomático:** Implementação robusta usando `thiserror` para erros customizados e `anyhow` para propagação limpa.
- **Configuração Flexível:** Gerencie endpoints via `config.yaml` com suporte a variáveis de ambiente via `.env`.
- **Validação Robusta:** Validação rigorosa de URLs e códigos de status na inicialização com tipos de erro específicos.
- **Alertas Inteligentes:** Integração com Webhooks (Discord/Slack/Teams) com lógica anti-spam e tratamento de falhas de rede.
- **Testes de Qualidade:** Suite de testes unitários usando `wiremock` para simular cenários de rede.

## 📋 Pré-requisitos

- Rust instalado ([rustup.rs](https://rustup.rs))
- Cargo (gerenciador de pacotes do Rust)

## 🔧 Configuração

1. Crie o seu arquivo de configuração a partir do exemplo:
   ```bash
   cp config.example.yaml config.yaml
   ```

2. Configure suas variáveis de ambiente no arquivo `.env`:
   ```env
   DISCORD_WEBHOOK_URL=https://discord.com/api/webhooks/sua-url-aqui
   ```

3. (Opcional) Edite o `config.yaml` para adicionar os endpoints que deseja monitorar.

## 🏃 Como Executar

Para rodar o monitor localmente:
```bash
# Níveis de log suportados: trace, debug, info, warn, error
RUST_LOG=info cargo run
```

## 🧪 Como Rodar os Testes

Garantimos a qualidade do código com testes automatizados. Para executá-los:
```bash
cargo test
```

## 📦 Build para Produção

Para gerar um binário otimizado:
```bash
cargo build --release
```
O executável estará em `./target/release/api-health-monitor`.

## 🧠 Por que este projeto é profissional?

1. **Observabilidade de Nível Sênior:** O uso de `tracing` permite acompanhar o ciclo de vida de cada requisição assíncrona com contexto (campos estruturados), facilitando muito o debugging em produção comparado a logs de texto simples.
2. **Tratamento de Erros:** O uso de `thiserror` permite categorizar falhas (configuração, rede, validação), facilitando o monitoramento.
3. **Concorrência Segura:** O uso de `async/await` permite monitorar centenas de APIs com baixíssimo consumo de recursos.
4. **Resiliência:** O loop principal é projetado para sobreviver a falhas temporárias de rede sem interromper o monitoramento de outros serviços.
5. **Validação de Entrada:** O programa valida URLs e configurações antes de iniciar, utilizando um sistema de tipos rigoroso.
