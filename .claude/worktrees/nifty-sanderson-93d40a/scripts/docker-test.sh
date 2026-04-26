#!/bin/bash
# Docker Build and Test Script for Noesis API

set -e

echo "🐳 Noesis API - Docker Build & Test Script"
echo "=========================================="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}❌ Docker daemon is not running${NC}"
    echo "Please start Docker Desktop and try again"
    exit 1
fi

# Check if .env exists, create from example if not
if [ ! -f .env ]; then
    echo -e "${YELLOW}⚠️  .env file not found, copying from .env.example${NC}"
    cp .env.example .env
    echo -e "${GREEN}✓ Created .env file${NC}"
fi

# Step 1: Build the Docker image
echo ""
echo -e "${YELLOW}Step 1: Building Docker image...${NC}"
echo "This may take 10-15 minutes on first build"
docker build -t noesis-api:latest .
echo -e "${GREEN}✓ Docker image built successfully${NC}"

# Step 2: Check image size
echo ""
echo -e "${YELLOW}Step 2: Checking image size...${NC}"
IMAGE_SIZE=$(docker images noesis-api:latest --format "{{.Size}}")
echo "Image size: $IMAGE_SIZE"

# Step 3: Start services with docker-compose
echo ""
echo -e "${YELLOW}Step 3: Starting services with docker-compose...${NC}"
docker-compose up -d
echo -e "${GREEN}✓ Services started${NC}"

# Step 4: Wait for services to be healthy
echo ""
echo -e "${YELLOW}Step 4: Waiting for services to be healthy...${NC}"
echo "This may take 30-60 seconds..."

# Wait for postgres
echo -n "Waiting for PostgreSQL... "
for i in {1..30}; do
    if docker-compose exec -T postgres pg_isready -U noesis_user -d noesis > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
        break
    fi
    sleep 2
    echo -n "."
done

# Wait for redis
echo -n "Waiting for Redis... "
for i in {1..30}; do
    if docker-compose exec -T redis redis-cli ping > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
        break
    fi
    sleep 2
    echo -n "."
done

# Wait for API
echo -n "Waiting for Noesis API... "
for i in {1..60}; do
    if curl -f http://localhost:8080/health > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
        break
    fi
    sleep 2
    echo -n "."
done

# Step 5: Test API health endpoint
echo ""
echo -e "${YELLOW}Step 5: Testing API health endpoint...${NC}"
HEALTH_RESPONSE=$(curl -s http://localhost:8080/health)
echo "Health response: $HEALTH_RESPONSE"

if echo "$HEALTH_RESPONSE" | grep -q "healthy"; then
    echo -e "${GREEN}✓ API is healthy${NC}"
else
    echo -e "${RED}❌ API health check failed${NC}"
    echo "Checking logs..."
    docker-compose logs --tail=50 noesis-api
    exit 1
fi

# Step 6: Show service status
echo ""
echo -e "${YELLOW}Step 6: Service Status${NC}"
docker-compose ps

# Step 7: Show logs
echo ""
echo -e "${YELLOW}Step 7: Recent logs (last 20 lines)${NC}"
docker-compose logs --tail=20 noesis-api

# Success summary
echo ""
echo -e "${GREEN}=========================================="
echo "✅ Docker deployment successful!"
echo "==========================================${NC}"
echo ""
echo "Services running:"
echo "  - Noesis API:  http://localhost:8080"
echo "  - PostgreSQL:  localhost:5432"
echo "  - Redis:       localhost:6379"
echo ""
echo "Useful commands:"
echo "  - View logs:   docker-compose logs -f noesis-api"
echo "  - Stop all:    docker-compose down"
echo "  - Restart API: docker-compose restart noesis-api"
echo "  - Shell:       docker-compose exec noesis-api /bin/bash"
echo ""
echo "Test the API:"
echo "  curl http://localhost:8080/health"
echo ""
