# 🛡️ API Health Monitor (Rust)

Um monitor de saúde de APIs leve, assíncrono e profissional desenvolvido em Rust. Este projeto resolve uma dor real de empresas: **garantir que serviços críticos estejam online e ser notificado imediatamente quando algo falha.**

## 🚀 Funcionalidades

- **Assincronismo:** Utiliza `tokio` e `reqwest` para checagens de alta performance.
- **Configuração Flexível:** Gerencie endpoints via `config.yaml` com suporte a variáveis de ambiente via `.env`.
- **Validação Robusta:** Validação rigorosa de URLs e códigos de status na inicialização.
- **Alertas Inteligentes:** Integração com Webhooks (Discord/Slack/Teams) com lógica anti-spam.
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

1. **Segurança de Memória:** Rust garante que não haverá *crashes* por acesso inválido à memória.
2. **Concorrência Segura:** O uso de `async/await` permite monitorar centenas de APIs com baixíssimo consumo de recursos.
3. **Validação de Entrada:** O programa valida URLs e configurações antes de iniciar, evitando erros em tempo de execução.
4. **Tratamento de Erros:** Uso de `anyhow` para propagação limpa de erros e logs detalhados.
5. **Segredos Protegidos:** Uso de `.env` e `config.example.yaml` para evitar vazamento de chaves sensíveis no controle de versão.
