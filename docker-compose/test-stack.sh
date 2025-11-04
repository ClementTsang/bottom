#!/bin/bash
# Test script to verify the observability stack is running correctly

set -e

echo "🔍 Testing Bottom OpenTelemetry Stack..."
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test OTEL Collector gRPC endpoint
echo -n "Testing OTEL Collector gRPC (port 4317)... "
if nc -zv localhost 4317 2>&1 | grep -q "succeeded\|open"; then
    echo -e "${GREEN}✓ OK${NC}"
else
    echo -e "${RED}✗ FAILED${NC}"
    exit 1
fi

# Test OTEL Collector HTTP endpoint
echo -n "Testing OTEL Collector HTTP (port 4318)... "
if nc -zv localhost 4318 2>&1 | grep -q "succeeded\|open"; then
    echo -e "${GREEN}✓ OK${NC}"
else
    echo -e "${RED}✗ FAILED${NC}"
    exit 1
fi

# Test OTEL Collector metrics endpoint
echo -n "Testing OTEL Collector metrics (port 8889)... "
if curl -s http://localhost:8889/metrics > /dev/null; then
    echo -e "${GREEN}✓ OK${NC}"
else
    echo -e "${RED}✗ FAILED${NC}"
    exit 1
fi

# Test Prometheus
echo -n "Testing Prometheus (port 9090)... "
if curl -s http://localhost:9090/-/healthy | grep -q "Prometheus"; then
    echo -e "${GREEN}✓ OK${NC}"
else
    echo -e "${RED}✗ FAILED${NC}"
    exit 1
fi

# Test Prometheus targets
echo -n "Testing Prometheus targets... "
TARGETS=$(curl -s http://localhost:9090/api/v1/targets | grep -o '"health":"up"' | wc -l)
if [ "$TARGETS" -gt 0 ]; then
    echo -e "${GREEN}✓ OK${NC} (${TARGETS} targets up)"
else
    echo -e "${YELLOW}⚠ WARNING${NC} (no targets up yet - this is normal if just started)"
fi

# Test Grafana
echo -n "Testing Grafana (port 3000)... "
if curl -s http://localhost:3000/api/health | grep -q "ok"; then
    echo -e "${GREEN}✓ OK${NC}"
else
    echo -e "${RED}✗ FAILED${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}✓ All tests passed!${NC}"
echo ""
echo "📊 Access points:"
echo "   - Prometheus: http://localhost:9090"
echo "   - Grafana: http://localhost:3000 (admin/admin)"
echo "   - OTEL Collector metrics: http://localhost:8889/metrics"
echo ""
echo "💡 Next steps:"
echo "   1. Build bottom with: cargo build --release --features opentelemetry"
echo "   2. Run in headless mode: ./target/release/btm --headless"
echo "   3. Check metrics in Prometheus: http://localhost:9090/graph"
