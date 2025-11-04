# Bottom OpenTelemetry Metrics Reference

This document lists all metrics exported by Bottom when running with the `opentelemetry` feature enabled.

## System Metrics

### CPU

| Metric Name | Type | Labels | Description |
|------------|------|--------|-------------|
| `system_cpu_usage_percent` | Gauge | `cpu_id` | CPU usage percentage per core |

**Example:**
```promql
# Average CPU across all cores
avg(system_cpu_usage_percent)

# CPU usage for core 0
system_cpu_usage_percent{cpu_id="0"}
```

### Memory

| Metric Name | Type | Labels | Description |
|------------|------|--------|-------------|
| `system_memory_usage_bytes` | Gauge | - | RAM memory currently in use |
| `system_memory_total_bytes` | Gauge | - | Total RAM memory available |
| `system_swap_usage_bytes` | Gauge | - | Swap memory currently in use |
| `system_swap_total_bytes` | Gauge | - | Total swap memory available |

**Example:**
```promql
# Memory usage percentage
(system_memory_usage_bytes / system_memory_total_bytes) * 100

# Available memory
system_memory_total_bytes - system_memory_usage_bytes
```

### Network

| Metric Name | Type | Labels | Description |
|------------|------|--------|-------------|
| `system_network_rx_bytes_rate` | Gauge | `interface` | Network receive rate in bytes/sec |
| `system_network_tx_bytes_rate` | Gauge | `interface` | Network transmit rate in bytes/sec |

**Example:**
```promql
# Total network throughput
sum(system_network_rx_bytes_rate) + sum(system_network_tx_bytes_rate)

# RX rate for specific interface
system_network_rx_bytes_rate{interface="eth0"}
```

### Disk

| Metric Name | Type | Labels | Description |
|------------|------|--------|-------------|
| `system_disk_usage_bytes` | Gauge | `device`, `mount` | Disk space currently in use |
| `system_disk_total_bytes` | Gauge | `device`, `mount` | Total disk space available |

**Example:**
```promql
# Disk usage percentage
(system_disk_usage_bytes / system_disk_total_bytes) * 100

# Free disk space
system_disk_total_bytes - system_disk_usage_bytes
```

### Temperature

| Metric Name | Type | Labels | Description |
|------------|------|--------|-------------|
| `system_temperature_celsius` | Gauge | `sensor` | Temperature readings in Celsius |

**Example:**
```promql
# Average temperature across all sensors
avg(system_temperature_celsius)

# Maximum temperature
max(system_temperature_celsius)
```

## Process Metrics

| Metric Name | Type | Labels | Description |
|------------|------|--------|-------------|
| `system_process_cpu_usage_percent` | Gauge | `name`, `pid` | CPU usage percentage per process |
| `system_process_memory_usage_bytes` | Gauge | `name`, `pid` | Memory usage in bytes per process |
| `system_process_count` | Gauge | - | Total number of processes |

**Example:**
```promql
# Top 10 processes by CPU
topk(10, system_process_cpu_usage_percent)

# Top 10 processes by memory
topk(10, system_process_memory_usage_bytes)

# Total memory used by all Chrome processes
sum(system_process_memory_usage_bytes{name=~".*chrome.*"})
```

## Recording Rules

The following recording rules are pre-configured in Prometheus (see `rules/bottom_rules.yml`):

| Rule Name | Expression | Description |
|-----------|------------|-------------|
| `system_process_cpu_usage_percent:recent` | Recent process CPU metrics | Filters out stale process data (>2 min old) |
| `system_process_memory_usage_bytes:recent` | Recent process memory metrics | Filters out stale process data (>2 min old) |

**Example:**
```promql
# Query only recent process data
topk(10, system_process_cpu_usage_percent:recent)
```

## Common Queries

### System Health

```promql
# Overall system CPU usage
avg(system_cpu_usage_percent)

# Memory pressure (>80% is high)
(system_memory_usage_bytes / system_memory_total_bytes) * 100

# Disk pressure (>90% is critical)
(system_disk_usage_bytes / system_disk_total_bytes) * 100
```

### Resource Hogs

```promql
# Top CPU consumers
topk(5, system_process_cpu_usage_percent)

# Top memory consumers
topk(5, system_process_memory_usage_bytes)

# Processes using >1GB memory
system_process_memory_usage_bytes > 1073741824
```

### Network Analysis

```promql
# Total network traffic (RX + TX)
sum(system_network_rx_bytes_rate) + sum(system_network_tx_bytes_rate)

# Network traffic by interface
sum by (interface) (system_network_rx_bytes_rate + system_network_tx_bytes_rate)

# Interfaces with high RX rate (>10MB/s)
system_network_rx_bytes_rate > 10485760
```

## Alerting Examples

### Sample Prometheus Alert Rules

```yaml
groups:
  - name: bottom_alerts
    interval: 30s
    rules:
      - alert: HighCPUUsage
        expr: avg(system_cpu_usage_percent) > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage detected"
          description: "Average CPU usage is {{ $value }}%"

      - alert: HighMemoryUsage
        expr: (system_memory_usage_bytes / system_memory_total_bytes) * 100 > 90
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage detected"
          description: "Memory usage is {{ $value }}%"

      - alert: DiskAlmostFull
        expr: (system_disk_usage_bytes / system_disk_total_bytes) * 100 > 90
        for: 10m
        labels:
          severity: critical
        annotations:
          summary: "Disk {{ $labels.mount }} almost full"
          description: "Disk usage is {{ $value }}% on {{ $labels.mount }}"
```

## Label Reference

| Label | Used In | Description |
|-------|---------|-------------|
| `cpu_id` | CPU metrics | CPU core identifier (0, 1, 2, ...) |
| `interface` | Network metrics | Network interface name (eth0, wlan0, ...) |
| `device` | Disk metrics | Device name (/dev/sda1, ...) |
| `mount` | Disk metrics | Mount point (/, /home, ...) |
| `sensor` | Temperature | Temperature sensor name |
| `name` | Process metrics | Process name |
| `pid` | Process metrics | Process ID |
| `exported_job` | All | Always "bottom-system-monitor" |
| `otel_scope_name` | All | Always "bottom-system-monitor" |

## Data Retention

By default, Prometheus stores metrics for 15 days. You can adjust this in the Prometheus configuration:

```yaml
# In prometheus.yml
global:
  retention_time: 30d  # Keep data for 30 days
```

For long-term storage, consider using:
- **TimescaleDB** (see `docker-compose-timescale.yml.ko`)
- **Thanos** for multi-cluster metrics
- **Cortex** for horizontally scalable storage
