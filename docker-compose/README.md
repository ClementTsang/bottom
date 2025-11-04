# Bottom OpenTelemetry Docker Compose Setup

This directory contains a Docker Compose setup for running an observability stack to monitor Bottom with OpenTelemetry.

## Architecture

The stack includes:

1. **OpenTelemetry Collector** - Receives metrics from Bottom via OTLP protocol
2. **Prometheus** - Scrapes and stores metrics from the OTEL Collector
3. **Grafana** - Visualizes metrics from Prometheus

```
Bottom (with --headless flag)
    ↓ (OTLP/gRPC on port 4317)
OpenTelemetry Collector
    ↓ (Prometheus scrape on port 8889)
Prometheus
    ↓ (Query on port 9090)
Grafana (accessible on port 3000)
```

## Quick Start

### 1. Start the observability stack

```bash
cd docker-compose
docker-compose up -d
```

This will start:
- OpenTelemetry Collector on ports 4317 (gRPC), 4318 (HTTP), 8889 (metrics)
- Prometheus on port 9090
- Grafana on port 3000

### 2. Build Bottom with OpenTelemetry support

```bash
cd ..
cargo build --release --features opentelemetry
```

### 3. Create a configuration file

Create a `bottom-config.toml` file:

```toml
[opentelemetry]
enabled = true
endpoint = "http://localhost:4317"
service_name = "bottom-system-monitor"
export_interval_ms = 5000

[opentelemetry.metrics]
cpu = true
memory = true
network = true
disk = true
processes = true
temperature = true
gpu = true
```

### 4. Run Bottom in headless mode

```bash
./target/release/btm --config bottom-config.toml --headless
```

Or without config file:

```bash
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
./target/release/btm --headless
```

### 5. Access the dashboards

- **Prometheus**: http://localhost:9090
- **Grafana**: http://localhost:3000 (username: `admin`, password: `admin`)

## Configuration Files

### otel-collector-config.yml

Configures the OpenTelemetry Collector to:
- Receive OTLP data on ports 4317 (gRPC) and 4318 (HTTP)
- Export metrics in Prometheus format on port 9090
- Debug log all received data

### prometheus.yml

Configures Prometheus to:
- Scrape metrics from the OTEL Collector every 10 seconds
- Load alerting rules from `rules/bottom_rules.yml`

### rules/bottom_rules.yml

Contains Prometheus recording rules for Bottom metrics, including:
- Recent process CPU usage metrics
- Recent process memory usage metrics

## Viewing Metrics in Prometheus

1. Go to http://localhost:9090
2. Click on "Graph"
3. Try these example queries:

```promql
# CPU usage by core
system_cpu_usage_percent

# Memory usage
system_memory_usage_bytes

# Network RX/TX
system_network_rx_bytes
system_network_tx_bytes

# Disk usage
system_disk_usage_bytes

# Top processes by CPU
topk(10, system_process_cpu_usage_percent)

# Top processes by memory
topk(10, system_process_memory_usage_bytes)
```

## Grafana Configuration

Grafana is automatically configured with:
- **Prometheus data source** (http://prometheus:9090) - pre-configured
- **Bottom System Overview dashboard** - pre-loaded

To access:
1. Go to http://localhost:3000 (username: `admin`, password: `admin`)
2. Navigate to Dashboards → Browse → "Bottom System Overview"

The dashboard includes:
- CPU usage by core
- Memory usage (RAM/Swap)
- Network traffic
- Disk usage
- Top 10 processes by CPU
- Top 10 processes by Memory

## Stopping the Stack

```bash
docker-compose down
```

To also remove volumes:

```bash
docker-compose down -v
```

## Troubleshooting

### Bottom not sending metrics

Check the OTEL Collector logs:
```bash
docker-compose logs -f otel-collector
```

You should see messages about receiving metrics.

### Prometheus not scraping

1. Check Prometheus targets at http://localhost:9090/targets
2. The `otel-collector` target should be UP

### No data in Grafana

1. Verify Prometheus data source is configured correctly
2. Check that Prometheus has data by querying directly
3. Ensure your time range in Grafana includes when Bottom was running

## Advanced Configuration

### Using with TimescaleDB (optional)

A TimescaleDB configuration file is available as `docker-compose-timescale.yml.ko` for long-term storage of metrics. Rename it to include it in your stack.

### Custom Prometheus Rules

Edit `rules/bottom_rules.yml` to add custom recording or alerting rules.

### OTEL Collector Sampling

Edit `otel-collector-config.yml` to adjust the batch processor settings for different performance characteristics.
