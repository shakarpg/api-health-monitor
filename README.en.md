# 🛡️ API Health Monitor (Rust)

A lightweight, asynchronous, and professional API health monitor developed in Rust. This project addresses a real business pain point: **ensuring critical services are online and being notified immediately when something fails.**

## ✨ Features

- **Asynchronous Operations:** Utilizes `tokio` and `reqwest` for high-performance checks.
- **Advanced Observability:** Implements structured logging and asynchronous tracing using `tracing` and `tracing-subscriber`.
- **Prometheus Metrics:** Exposes latency and error rate metrics per endpoint via a `/metrics` endpoint compatible with Prometheus.
- **Idiomatic Error Handling:** Robust implementation using `thiserror` for custom errors and `anyhow` for clean propagation.
- **Flexible Configuration:** Manage endpoints via `config.yaml` with environment variable support via `.env`.
- **Robust Validation:** Strict validation of URLs and status codes at startup with specific error types.
- **Smart Alerts:** Integration with Webhooks (Discord/Slack/Teams) with anti-spam logic and network failure handling.
- **Quality Tests:** Unit test suite using `wiremock` to simulate network scenarios.

## 📋 Prerequisites

- Rust installed ([rustup.rs](https://rustup.rs))
- Cargo (Rust's package manager)

## 🔧 Configuration

1. Create your configuration file from the example:
   ```bash
   cp config.example.yaml config.yaml
   ```

2. Configure your environment variables in the `.env` file:
   ```env
   DISCORD_WEBHOOK_URL=https://discord.com/api/webhooks/your-url-here
   ```

3. (Optional) Edit `config.yaml` to add the endpoints you want to monitor and the `metrics_port` (default: 9090).

## 🏃 How to Run

To run the monitor locally:
```bash
# Supported log levels: trace, debug, info, warn, error
RUST_LOG=info cargo run
```

The metrics server will be available at `http://localhost:9090/metrics` (or your configured port).

## 🧪 How to Run Tests

We ensure code quality with automated tests. To run them:
```bash
cargo test
```

## 📦 Build for Production

To generate an optimized binary:
```bash
cargo build --release
```
The executable will be in `./target/release/api-health-monitor`.

## 🧠 Why is this project professional?

1. **Senior-Level Observability:** The use of `tracing` allows monitoring the lifecycle of each asynchronous request with context (structured fields), greatly facilitating debugging in production compared to simple text logs.
2. **Production Metrics:** Exposing metrics in Prometheus format allows integration with monitoring tools like Grafana, providing real-time visibility into API health and performance.
3. **Error Handling:** The use of `thiserror` allows categorizing failures (configuration, network, validation), facilitating monitoring.
4. **Safe Concurrency:** The use of `async/await` allows monitoring hundreds of APIs with very low resource consumption.
5. **Resilience:** The main loop is designed to survive temporary network failures without interrupting the monitoring of other services.
6. **Input Validation:** The program validates URLs and configurations before starting, using a rigorous type system.
