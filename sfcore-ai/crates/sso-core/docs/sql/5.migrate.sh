#!/bin/bash

# ============================================================================
# SSO Fullstack Rust - Database Migration Script
# File: migrate.sh
# Description: Helper script to run PostgreSQL migrations
# ============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-sso_rust_db}"
DB_USER="${DB_USER:-postgres}"
DB_PASSWORD="${DB_PASSWORD:-postgres}"

# Migration files directory
MIGRATIONS_DIR="./migrations"

echo -e "${GREEN}===========================================${NC}"
echo -e "${GREEN}SSO Rust - Database Migration Script${NC}"
echo -e "${GREEN}===========================================${NC}"
echo ""

# Function to check if PostgreSQL is running
check_postgres() {
    echo -e "${YELLOW}Checking PostgreSQL connection...${NC}"
    if PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d postgres -c '\q' 2>/dev/null; then
        echo -e "${GREEN}✓ PostgreSQL is running${NC}"
    else
        echo -e "${RED}✗ Cannot connect to PostgreSQL${NC}"
        echo -e "${RED}Please check your database configuration${NC}"
        exit 1
    fi
}

# Function to create database if not exists
create_database() {
    echo -e "${YELLOW}Checking if database exists...${NC}"
    if PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -lqt | cut -d \| -f 1 | grep -qw $DB_NAME; then
        echo -e "${GREEN}✓ Database '$DB_NAME' already exists${NC}"
    else
        echo -e "${YELLOW}Creating database '$DB_NAME'...${NC}"
        PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d postgres -c "CREATE DATABASE $DB_NAME;"
        echo -e "${GREEN}✓ Database created successfully${NC}"
    fi
}

# Function to run migration file
run_migration() {
    local file=$1
    local filename=$(basename "$file")
    
    echo -e "${YELLOW}Running migration: $filename${NC}"
    if PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -f "$file" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ $filename completed successfully${NC}"
    else
        echo -e "${RED}✗ $filename failed${NC}"
        exit 1
    fi
}

# Function to run all migrations
run_migrations() {
    echo -e "${YELLOW}Running migrations...${NC}"
    echo ""
    
    # Migration files in order
    local migrations=(
        "001_initial_schema.sql"
        "002_authentication_tables.sql"
        "003_audit_security_tables.sql"
    )
    
    for migration in "${migrations[@]}"; do
        if [ -f "$migration" ]; then
            run_migration "$migration"
        else
            echo -e "${RED}✗ Migration file not found: $migration${NC}"
            exit 1
        fi
    done
    
    echo ""
    echo -e "${GREEN}✓ All migrations completed successfully${NC}"
}

# Function to seed database
seed_database() {
    echo ""
    echo -e "${YELLOW}Seeding database with dummy data...${NC}"
    
    if [ -f "004_seed_data.sql" ]; then
        run_migration "004_seed_data.sql"
        echo -e "${GREEN}✓ Database seeded successfully${NC}"
    else
        echo -e "${YELLOW}⚠ Seed file not found, skipping...${NC}"
    fi
}

# Function to rollback (drop all tables)
rollback() {
    echo -e "${RED}WARNING: This will drop all tables!${NC}"
    read -p "Are you sure you want to rollback? (yes/no): " confirm
    
    if [ "$confirm" = "yes" ]; then
        echo -e "${YELLOW}Rolling back database...${NC}"
        
        PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME << EOF
DROP SCHEMA public CASCADE;
CREATE SCHEMA public;
GRANT ALL ON SCHEMA public TO $DB_USER;
GRANT ALL ON SCHEMA public TO public;
EOF
        
        echo -e "${GREEN}✓ Rollback completed${NC}"
    else
        echo -e "${YELLOW}Rollback cancelled${NC}"
    fi
}

# Function to show database info
show_info() {
    echo ""
    echo -e "${GREEN}Database Information:${NC}"
    echo "  Host: $DB_HOST"
    echo "  Port: $DB_PORT"
    echo "  Database: $DB_NAME"
    echo "  User: $DB_USER"
    echo ""
    
    echo -e "${GREEN}Table Statistics:${NC}"
    PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME << 'EOF'
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size,
    n_live_tup AS rows
FROM pg_stat_user_tables
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
EOF
}

# Main script logic
case "${1:-}" in
    up)
        check_postgres
        create_database
        run_migrations
        ;;
    seed)
        check_postgres
        seed_database
        ;;
    fresh)
        check_postgres
        create_database
        run_migrations
        seed_database
        show_info
        ;;
    rollback)
        check_postgres
        rollback
        ;;
    info)
        check_postgres
        show_info
        ;;
    *)
        echo "Usage: $0 {up|seed|fresh|rollback|info}"
        echo ""
        echo "Commands:"
        echo "  up       - Run migrations only"
        echo "  seed     - Seed database with dummy data"
        echo "  fresh    - Drop all tables, run migrations, and seed"
        echo "  rollback - Drop all tables (destructive!)"
        echo "  info     - Show database information"
        echo ""
        echo "Environment variables:"
        echo "  DB_HOST     - Database host (default: localhost)"
        echo "  DB_PORT     - Database port (default: 5432)"
        echo "  DB_NAME     - Database name (default: sso_rust_db)"
        echo "  DB_USER     - Database user (default: postgres)"
        echo "  DB_PASSWORD - Database password (default: postgres)"
        echo ""
        echo "Example:"
        echo "  DB_HOST=localhost DB_NAME=mydb $0 fresh"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}===========================================${NC}"
echo -e "${GREEN}Migration script completed!${NC}"
echo -e "${GREEN}===========================================${NC}"