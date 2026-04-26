#!/bin/bash
# Quick Docker Commands Reference for Noesis API

cat << 'EOF'
╔══════════════════════════════════════════════════════════════╗
║         Noesis API - Docker Quick Reference                  ║
╚══════════════════════════════════════════════════════════════╝

📦 INITIAL SETUP
  cp .env.example .env
  # Edit .env with your settings (JWT_SECRET, passwords, etc.)

🚀 START SERVICES
  docker-compose up -d --build        # Build and start (detached)
  docker-compose up                   # Start with logs visible
  ./scripts/docker-test.sh            # Automated build & test

🛑 STOP SERVICES
  docker-compose down                 # Stop and remove containers
  docker-compose down -v              # Stop and remove volumes (clean slate)
  docker-compose stop                 # Stop without removing

🔄 RESTART SERVICES
  docker-compose restart              # Restart all services
  docker-compose restart noesis-api   # Restart only API
  docker-compose up -d --build noesis-api  # Rebuild & restart API

📊 VIEW STATUS & LOGS
  docker-compose ps                   # Show service status
  docker-compose logs -f              # Follow all logs
  docker-compose logs -f noesis-api   # Follow API logs only
  docker-compose logs --tail=50       # Last 50 lines
  docker stats                        # Resource usage

🔍 HEALTH CHECKS
  curl http://localhost:8080/health                    # API health
  docker-compose exec redis redis-cli ping             # Redis health
  docker-compose exec postgres pg_isready              # Postgres health
  docker-compose exec -T noesis-api wget -O- localhost:8080/health

🐚 ACCESS CONTAINERS
  docker-compose exec noesis-api /bin/bash             # API shell
  docker-compose exec postgres psql -U noesis_user -d noesis  # PostgreSQL
  docker-compose exec redis redis-cli                  # Redis CLI

🔧 REBUILD & CLEAN
  docker-compose build --no-cache     # Rebuild from scratch
  docker-compose up -d --force-recreate  # Recreate containers
  docker system prune -a              # Clean all unused Docker data (⚠️ careful!)
  docker volume prune                 # Remove unused volumes

💾 BACKUP & RESTORE
  # Backup PostgreSQL
  docker-compose exec postgres pg_dump -U noesis_user noesis > backup.sql
  
  # Restore PostgreSQL
  cat backup.sql | docker-compose exec -T postgres psql -U noesis_user noesis
  
  # Backup Redis
  docker-compose exec redis redis-cli SAVE
  docker cp noesis-redis:/data/dump.rdb ./redis-backup.rdb

🔍 DEBUGGING
  # Check environment variables
  docker-compose exec noesis-api env | grep -E 'REDIS|POSTGRES|DATA'
  
  # Test network connectivity
  docker-compose exec noesis-api ping redis
  docker-compose exec noesis-api nc -zv postgres 5432
  
  # View Docker Compose configuration
  docker-compose config
  
  # Check image size
  docker images noesis-api

🌐 PRODUCTION TIPS
  # Set proper secrets in .env:
  JWT_SECRET=<strong-random-value>
  POSTGRES_PASSWORD=<strong-password>
  
  # Set log level to warn/error
  RUST_LOG=warn
  
  # Remove external port bindings for internal services (in docker-compose.yml)
  # postgres: ports: - "127.0.0.1:5432:5432"  # Only localhost
  # redis: ports: - "127.0.0.1:6379:6379"     # Only localhost

📖 DOCUMENTATION
  cat DOCKER.md                              # Full deployment guide
  cat DOCKER_IMPLEMENTATION_SUMMARY.md       # Implementation details
  cat .env.example                           # Environment variables

🔗 ENDPOINTS
  API:        http://localhost:8080
  Health:     http://localhost:8080/health
  Metrics:    http://localhost:8080/metrics (if enabled)
  PostgreSQL: localhost:5432
  Redis:      localhost:6379

⚡ QUICK TEST
  # One-line health check
  curl -s http://localhost:8080/health | jq .
  
  # Full status check
  echo "API:"; curl -s localhost:8080/health | jq .status; \
  echo "Redis:"; docker-compose exec -T redis redis-cli ping; \
  echo "Postgres:"; docker-compose exec -T postgres pg_isready

EOF
