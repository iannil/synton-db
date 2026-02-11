#!/bin/bash
# Copyright 2025 SYNTON-DB Team
#
# Licensed under the Apache License, Version 2.0 (the "License");
#
# SYNTON-DB 运维脚本 - 一键管理所有服务

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
GRAY='\033[0;90m'
NC='\033[0m'

# 配置
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${SCRIPT_DIR}"
COMPOSE_FILE="${PROJECT_ROOT}/docker-compose.yml"
CONFIG_FILE="${PROJECT_ROOT}/config.toml"
DATA_DIR="${PROJECT_ROOT}/data"

# 二进制文件路径
DB_SERVER_BIN="${PROJECT_ROOT}/target/release/synton-db-server"
API_SERVER_BIN="${PROJECT_ROOT}/target/release/synton-server"
MCP_BIN="${PROJECT_ROOT}/target/release/synton-mcp-server"

# PID 文件目录
PID_DIR="${PROJECT_ROOT}/.pids"
DB_SERVER_PID="${PID_DIR}/synton-db-server.pid"
API_SERVER_PID="${PID_DIR}/synton-server.pid"
WEB_SERVER_PID="${PID_DIR}/web-server.pid"

# 日志目录
LOG_DIR="${PROJECT_ROOT}/logs"

# 端口配置
REST_PORT="${SYNTON_REST_PORT:-5570}"
GRPC_PORT="${SYNTON_GRPC_PORT:-5571}"
API_SERVER_PORT="${SYNTON_API_SERVER_PORT:-5578}"
WEB_SERVER_PORT="${SYNTON_WEB_PORT:-5173}"
PROMETHEUS_PORT="${SYNTON_PROMETHEUS_PORT:-5572}"
GRAFANA_PORT="${SYNTON_GRAFANA_PORT:-5573}"

# Docker Compose 命令
if command -v docker compose &> /dev/null; then
    DOCKER_COMPOSE="docker compose"
else
    DOCKER_COMPOSE="docker-compose"
fi

# 服务列表（用于日志查看）
DOCKER_SERVICES=("synton-db" "prometheus" "grafana" "web")

# 运行模式: docker 或 local
SYNTON_MODE="${SYNTON_MODE:-local}"

# 创建必要的目录
mkdir -p "$PID_DIR"
mkdir -p "$LOG_DIR"
mkdir -p "$DATA_DIR/rocksdb"
mkdir -p "$DATA_DIR/lance"

# 打印带颜色的消息
print_info() {
    echo -e "${BLUE}i${NC} $1"
}

print_success() {
    echo -e "${GREEN}OK${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}!${NC} $1"
}

print_error() {
    echo -e "${RED}X${NC} $1"
}

print_header() {
    echo -e "\n${CYAN}================================${NC}"
    echo -e "${CYAN}  $1${NC}"
    echo -e "${CYAN}================================${NC}\n"
}

# 检查 Docker 是否安装
check_docker() {
    if ! command -v docker &> /dev/null; then
        print_error "Docker 未安装，请先安装 Docker"
        exit 1
    fi

    if ! command -v docker compose &> /dev/null; then
        if ! command -v docker-compose &> /dev/null; then
            print_error "Docker Compose 未安装，请先安装 Docker Compose"
            exit 1
        fi
    fi
}

# 检查 Rust/Cargo 是否可用
check_rust() {
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo 未安装，请先安装 Rust"
        print_info "访问 https://rustup.rs/ 安装 Rust"
        exit 1
    fi
}

# 检查是否为本地模式
is_local_mode() {
    [ "$SYNTON_MODE" = "local" ]
}

# 检查是否为 Docker 模式
is_docker_mode() {
    [ "$SYNTON_MODE" = "docker" ]
}

# 检查项目文件
check_project() {
    cd "$PROJECT_ROOT"
}

# 构建项目
build_project() {
    print_header "构建 SYNTON-DB"
    check_rust
    check_project
    print_info "使用 Cargo 构建所有服务 (启用 Candle ML feature)..."
    cargo build --release --bins --features candle
    print_success "构建完成"
}

# 获取端口状态
get_port_status() {
    local port=$1
    local service_name=$2

    if command -v lsof &> /dev/null; then
        if lsof -i ":$port" -sTCP:LISTEN -t &>/dev/null; then
            local pid=$(lsof -i ":$port" -sTCP:LISTEN -t 2>/dev/null | head -1)
            echo -e "  $service_name: ${GREEN}OK${NC} (port $port, PID: ${pid:-unknown})"
        else
            echo -e "  $service_name: ${RED}X${NC} (port $port)"
        fi
    elif command -v netstat &> /dev/null; then
        if netstat -an 2>/dev/null | grep -q "\.$port.*LISTEN"; then
            echo -e "  $service_name: ${GREEN}OK${NC} (port $port)"
        else
            echo -e "  $service_name: ${RED}X${NC} (port $port)"
        fi
    else
        echo -e "  $service_name: (port $port, status unknown)"
    fi
}

# 检查服务是否在运行
is_service_running() {
    local pid_file=$1
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file" 2>/dev/null)
        if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
            return 0
        fi
    fi
    return 1
}

# 停止服务（通过 PID 文件）
stop_service_by_pid() {
    local pid_file=$1
    local service_name=$2

    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file" 2>/dev/null)
        if [ -n "$pid" ]; then
            if kill -0 "$pid" 2>/dev/null; then
                print_info "停止 $service_name (PID: $pid)..."
                kill "$pid" 2>/dev/null || true
                local count=0
                while kill -0 "$pid" 2>/dev/null && [ $count -lt 30 ]; do
                    sleep 1
                    count=$((count + 1))
                done
                if kill -0 "$pid" 2>/dev/null; then
                    kill -9 "$pid" 2>/dev/null || true
                fi
                print_success "$service_name 已停止"
            else
                print_warning "$service_name 未运行 (PID: $pid)"
            fi
        fi
        rm -f "$pid_file"
    fi
}

# 等待服务健康
wait_for_health() {
    local service=$1
    local max_wait=${2:-30}
    local count=0

    print_info "等待 $service 服务启动..."

    while [ $count -lt $max_wait ]; do
        if docker ps --format '{{.Status}}' | grep -q "healthy"; then
            print_success "$service 服务已就绪"
            return 0
        fi
        sleep 1
        count=$((count + 1))
        echo -n "."
    done

    echo ""
    print_warning "$service 服务启动超时，请检查日志"
    return 1
}

# 显示服务状态
show_status() {
    print_header "SYNTON-DB 服务状态"

    if is_local_mode; then
        echo -e "${CYAN}Mode:${NC} ${GREEN}Local${NC}"
        echo ""
        echo -e "${CYAN}Process Status:${NC}"

        # DB Server 状态
        if is_service_running "$DB_SERVER_PID"; then
            local pid=$(cat "$DB_SERVER_PID")
            echo -e "  synton-db-server: ${GREEN}Running${NC} (PID: $pid)"
        else
            echo -e "  synton-db-server: ${RED}Stopped${NC}"
        fi

        # API Server 状态
        if is_service_running "$API_SERVER_PID"; then
            local pid=$(cat "$API_SERVER_PID")
            echo -e "  synton-server:    ${GREEN}Running${NC} (PID: $pid)"
        else
            echo -e "  synton-server:    ${RED}Stopped${NC}"
        fi

        # Web Server 状态
        if is_service_running "$WEB_SERVER_PID"; then
            local pid=$(cat "$WEB_SERVER_PID")
            echo -e "  web-server:      ${GREEN}Running${NC} (PID: $pid)"
        else
            echo -e "  web-server:      ${RED}Stopped${NC}"
        fi

        echo ""
        echo -e "${CYAN}Ports:${NC}"
        get_port_status "$REST_PORT" "REST API"
        get_port_status "$GRPC_PORT" "gRPC"
        get_port_status "$API_SERVER_PORT" "API Server"
        get_port_status "$WEB_SERVER_PORT" "Web UI"
    else
        echo -e "${CYAN}Mode:${NC} ${GREEN}Docker${NC}"
        echo ""
        echo -e "${CYAN}Container Status:${NC}"
        $DOCKER_COMPOSE ps 2>/dev/null || docker ps -a --filter "name=synton"

        echo ""
        echo -e "${CYAN}Ports:${NC}"
        get_port_status "$REST_PORT" "REST API"
        get_port_status "$GRPC_PORT" "gRPC"
        get_port_status "$WEB_SERVER_PORT" "Web UI"
        get_port_status "$PROMETHEUS_PORT" "Prometheus"
        get_port_status "$GRAFANA_PORT" "Grafana"
    fi

    echo ""
    echo -e "${CYAN}Health Check:${NC}"

    # API 健康检查
    if curl -s "http://localhost:$REST_PORT/health" > /dev/null 2>&1; then
        local health_data=$(curl -s "http://localhost:$REST_PORT/health")
        local status=$(echo "$health_data" | jq -r '.status' 2>/dev/null || echo "unknown")
        local version=$(echo "$health_data" | jq -r '.version' 2>/dev/null || echo "unknown")

        if [ "$status" = "healthy" ] || [ "$status" = "ok" ]; then
            echo -e "  REST API:   ${GREEN}OK${NC} $version"
        else
            echo -e "  REST API:   ${YELLOW}!${NC} $status"
        fi
    else
        echo -e "  REST API:   ${RED}X${NC} Cannot connect"
    fi

    # API Server 健康检查 (仅本地模式)
    if is_local_mode && curl -s "http://localhost:$API_SERVER_PORT/health" > /dev/null 2>&1; then
        echo -e "  API Server: ${GREEN}OK${NC} Accessible"
    fi

    # Prometheus 健康检查 (仅 Docker 模式)
    if is_docker_mode && curl -s "http://localhost:$PROMETHEUS_PORT/-/healthy" > /dev/null 2>&1; then
        echo -e "  Prometheus: ${GREEN}OK${NC} healthy"
    fi
}

# 启动服务
cmd_start() {
    print_header "Starting SYNTON-DB Services"

    if is_local_mode; then
        cmd_start_local
    else
        cmd_start_docker
    fi
}

# 本地模式启动服务
cmd_start_local() {
    print_info "本地模式启动服务..."

    check_project

    # 检查是否已有服务在运行
    local db_started=false
    local web_started=false

    if is_service_running "$DB_SERVER_PID"; then
        print_warning "synton-db-server already running"
        db_started=true
    else
        # 检查二进制文件
        if [ ! -f "$DB_SERVER_BIN" ]; then
            print_warning "二进制文件不存在，开始构建..."
            build_project
        fi

        print_info "启动 synton-db-server..."

        # 启动 DB Server
        nohup "$DB_SERVER_BIN" \
            --config "${CONFIG_FILE}" \
            --rest-port "$REST_PORT" \
            --grpc-port "$GRPC_PORT" \
            --log-level "info" \
            > "$LOG_DIR/db-server.log" 2>&1 &

        local db_pid=$!
        echo "$db_pid" > "$DB_SERVER_PID"

        print_success "synton-db-server 已启动 (PID: $db_pid)"
        db_started=true
    fi

    # 启动 Web 服务器
    if is_service_running "$WEB_SERVER_PID"; then
        print_warning "web-server already running"
    else
        if command -v npm &> /dev/null; then
            print_info "启动 web-server..."

            cd "$PROJECT_ROOT/web"
            nohup npm run dev > "$LOG_DIR/web-server.log" 2>&1 &
            local web_pid=$!
            echo "$web_pid" > "$WEB_SERVER_PID"

            print_success "web-server 已启动 (PID: $web_pid)"
            cd "$PROJECT_ROOT"
            web_started=true
        else
            print_warning "npm not installed, skipping web-server"
        fi
    fi

    # 等待服务就绪
    print_info "等待服务启动..."
    local count=0
    while [ $count -lt 30 ]; do
        if curl -s "http://localhost:$REST_PORT/health" > /dev/null 2>&1; then
            break
        fi
        sleep 1
        count=$((count + 1))
        echo -n "."
    done
    echo ""

    print_success "服务已启动"
    echo ""
    echo -e "  REST API:       ${BLUE}http://localhost:$REST_PORT${NC}"
    echo -e "  gRPC:           ${BLUE}localhost:$GRPC_PORT${NC}"
    if $web_started; then
        echo -e "  Web UI:         ${BLUE}http://localhost:$WEB_SERVER_PORT${NC}"
    fi
    echo -e "  Log Directory:   ${BLUE}${LOG_DIR}${NC}"
    echo ""
    echo -e "${CYAN}Tips:${NC}"
    echo -e "  - Use '$0 logs' to view logs"
    echo -e "  - Use '$0 stop' to stop services"
    echo -e "  - MCP server runs directly from clients like Claude Code"
}

# Docker 模式启动服务
cmd_start_docker() {
    print_info "Docker 模式启动服务..."

    check_docker
    check_project

    print_info "启动所有容器..."
    $DOCKER_COMPOSE up -d

    echo ""
    wait_for_health "synton-db"

    echo ""
    print_success "所有服务已启动"
    echo ""
    echo -e "  REST API:    ${BLUE}http://localhost:$REST_PORT${NC}"
    echo -e "  gRPC:        ${BLUE}localhost:$GRPC_PORT${NC}"
    echo -e "  Web UI:      ${BLUE}http://localhost:$WEB_SERVER_PORT${NC}"
    echo -e "  Prometheus:  ${BLUE}http://localhost:$PROMETHEUS_PORT${NC}"
    echo -e "  Grafana:     ${BLUE}http://localhost:$GRAFANA_PORT${NC} (admin/admin)"
    echo ""
    echo -e "${CYAN}Tips:${NC}"
    echo -e "  - MCP server runs directly from clients like Claude Code"
}

# 停止服务
cmd_stop() {
    print_header "停止 SYNTON-DB 服务"

    if is_local_mode; then
        cmd_stop_local
    else
        cmd_stop_docker
    fi
}

# 本地模式停止服务
cmd_stop_local() {
    print_info "本地模式停止服务..."

    check_project

    local stopped=0

    if is_service_running "$DB_SERVER_PID"; then
        stop_service_by_pid "$DB_SERVER_PID" "synton-db-server"
        stopped=1
    fi

    if is_service_running "$API_SERVER_PID"; then
        stop_service_by_pid "$API_SERVER_PID" "synton-server"
        stopped=1
    fi

    if is_service_running "$WEB_SERVER_PID"; then
        stop_service_by_pid "$WEB_SERVER_PID" "web-server"
        stopped=1
    fi

    if [ $stopped -eq 0 ]; then
        print_warning "没有运行中的服务"
    else
        print_success "所有服务已停止"
    fi
}

# Docker 模式停止服务
cmd_stop_docker() {
    print_info "Docker 模式停止服务..."

    check_docker
    check_project

    print_info "停止所有容器..."
    $DOCKER_COMPOSE down --remove-orphans

    print_success "所有服务已停止"
}

# 重启服务
cmd_restart() {
    print_header "重启 SYNTON-DB 服务"

    if is_local_mode; then
        cmd_stop_local
        sleep 2
        cmd_start_local
    else
        cmd_restart_docker
    fi
}

# Docker 模式重启服务
cmd_restart_docker() {
    print_info "Docker 模式重启服务..."

    check_docker
    check_project

    print_info "重启所有容器..."
    $DOCKER_COMPOSE restart

    echo ""
    wait_for_health "synton-db"

    print_success "所有服务已重启"
}

# 查看日志
cmd_logs() {
    local service=${1:-""}

    if is_local_mode; then
        cmd_logs_local "$service"
    else
        cmd_logs_docker "$service"
    fi
}

# 本地模式查看日志
cmd_logs_local() {
    local service=${1:-""}

    check_project

    if [ -z "$service" ]; then
        print_header "SYNTON-DB 日志"
        print_info "显示所有服务日志 (Ctrl+C 退出)..."
        echo ""

        if [ -f "$LOG_DIR/db-server.log" ]; then
            echo -e "${CYAN}=== DB Server Logs ===${NC}"
            tail -f "$LOG_DIR/db-server.log" &
            local tail_pid=$!
        fi

        if [ -f "$LOG_DIR/api-server.log" ]; then
            echo -e "${CYAN}=== API Server Logs ===${NC}"
            tail -f "$LOG_DIR/api-server.log" &
        fi

        if [ -f "$LOG_DIR/web-server.log" ]; then
            echo -e "${CYAN}=== Web Server Logs ===${NC}"
            tail -f "$LOG_DIR/web-server.log" &
        fi

        trap "kill $tail_pid 2>/dev/null || true; exit 0" INT TERM
        wait
    else
        print_header "SYNTON-DB 日志 - $service"
        print_info "显示 $service 服务日志 (Ctrl+C 退出)..."
        echo ""

        case "$service" in
            db|db-server|synton-db)
                if [ -f "$LOG_DIR/db-server.log" ]; then
                    tail -f "$LOG_DIR/db-server.log"
                else
                    print_error "日志文件不存在: $LOG_DIR/db-server.log"
                fi
                ;;
            api|api-server|synton-server)
                if [ -f "$LOG_DIR/api-server.log" ]; then
                    tail -f "$LOG_DIR/api-server.log"
                else
                    print_error "日志文件不存在: $LOG_DIR/api-server.log"
                fi
                ;;
            web|web-server)
                if [ -f "$LOG_DIR/web-server.log" ]; then
                    tail -f "$LOG_DIR/web-server.log"
                else
                    print_error "日志文件不存在: $LOG_DIR/web-server.log"
                fi
                ;;
            *)
                print_error "未知服务: $service"
                print_info "可用服务: db-server, api-server, web-server"
                ;;
        esac
    fi
}

# Docker 模式查看日志
cmd_logs_docker() {
    local service=${1:-""}

    check_docker
    check_project

    if [ -z "$service" ]; then
        print_header "SYNTON-DB 日志"
        print_info "显示所有服务日志 (Ctrl+C 退出)..."
        echo ""
        $DOCKER_COMPOSE logs -f
    else
        print_header "SYNTON-DB 日志 - $service"
        print_info "显示 $service 服务日志 (Ctrl+C 退出)..."
        echo ""
        $DOCKER_COMPOSE logs -f "$service"
    fi
}

# 重新构建
cmd_rebuild() {
    print_header "重新构建 SYNTON-DB"

    if is_local_mode; then
        cmd_rebuild_local "$@"
    else
        cmd_rebuild_docker "$@"
    fi
}

# 本地模式重新构建
cmd_rebuild_local() {
    print_info "本地模式重新构建..."

    check_project

    if is_service_running "$DB_SERVER_PID" || is_service_running "$API_SERVER_PID" || is_service_running "$WEB_SERVER_PID"; then
        print_info "停止运行中的服务..."
        cmd_stop_local
    fi

    build_project

    echo ""
    print_info "是否需要重启服务? [y/N]"
    read -r response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        cmd_start_local
    fi
}

# Docker 模式重新构建
cmd_rebuild_docker() {
    print_info "Docker 模式重新构建..."

    check_docker
    check_project

    print_info "构建 Docker 镜像..."
    $DOCKER_COMPOSE build --no-cache "$@"

    print_success "构建完成"

    echo ""
    print_info "是否需要重启服务? [y/N]"
    read -r response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        cmd_restart_docker
    fi
}

# 清理数据
cmd_clean() {
    print_header "清理 SYNTON-DB 数据"

    if is_local_mode; then
        cmd_clean_local
    else
        cmd_clean_docker
    fi
}

# 本地模式清理数据
cmd_clean_local() {
    print_warning "此操作将删除所有本地数据，包括："
    echo "  - 数据库数据 (RocksDB)"
    echo "  - 向量索引 (Lance)"
    echo "  - 日志文件"
    echo ""

    read -p "确认删除? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "已取消"
        return
    fi

    cmd_stop_local

    print_info "删除数据目录..."
    rm -rf "${DATA_DIR:?}/rocksdb"/*
    rm -rf "${DATA_DIR:?}/lance"/*

    print_info "清理日志文件..."
    rm -rf "${LOG_DIR:?}"/*

    print_success "清理完成"
}

# Docker 模式清理数据
cmd_clean_docker() {
    print_warning "此操作将删除所有 Docker 数据，包括："
    echo "  - 数据库数据 (RocksDB)"
    echo "  - 向量索引 (Lance)"
    echo "  - Prometheus 监控数据"
    echo "  - Grafana 配置和仪表板"
    echo ""

    read -p "确认删除? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "已取消"
        return
    fi

    check_docker
    check_project

    print_info "删除卷和数据..."
    $DOCKER_COMPOSE down -v

    docker volume rm synton-data synton-prometheus-data synton-grafana-data 2>/dev/null || true

    print_success "清理完成"
}

# 清理缓存
cmd_purge() {
    print_header "清理未使用的资源"

    if is_local_mode; then
        print_info "本地模式清理 Cargo 缓存..."
        print_warning "这将清理 Cargo 构建缓存，释放磁盘空间"
        echo ""
        read -p "确认清理? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_info "已取消"
            return
        fi

        cargo clean
        print_success "清理完成"
    else
        check_docker

        print_info "删除未使用的容器、网络、镜像..."
        docker system prune -a

        print_success "清理完成"
    fi
}

# 进入容器
cmd_shell() {
    local service=${1:-synton-db}

    if is_local_mode; then
        print_error "本地模式不支持进入容器 shell"
        print_info "如需调试，请直接查看日志或使用调试器"
        return 1
    fi

    check_docker
    check_project

    print_info "进入 $service 容器 shell..."
    $DOCKER_COMPOSE exec "$service" /bin/sh
}

# 查看 API 统计
cmd_stats() {
    print_header "SYNTON-DB 统计信息"

    if curl -s "http://localhost:$REST_PORT/health" > /dev/null 2>&1; then
        curl -s "http://localhost:$REST_PORT/stats" | jq .
    else
        print_error "无法连接到 API (端口 $REST_PORT)"
        return 1
    fi
}

# 测试 API
cmd_test() {
    print_header "测试 SYNTON-DB API"

    print_info "测试健康检查..."
    if curl -s "http://localhost:$REST_PORT/health" | jq .; then
        echo ""
    fi

    print_info "测试统计信息..."
    if curl -s "http://localhost:$REST_PORT/stats" | jq .; then
        echo ""
    fi

    print_info "测试节点查询..."
    if curl -s "http://localhost:$REST_PORT/nodes" | jq .; then
        echo ""
    fi

    print_success "API 测试完成"
}

# MCP 服务器管理 (调试用)
cmd_mcp_start() {
    print_header "启动 MCP 服务器"

    check_project

    if ! curl -s "http://localhost:$REST_PORT/health" > /dev/null 2>&1; then
        print_error "SYNTON-DB API 未运行，请先启动服务: $0 start"
        return 1
    fi

    if [ ! -f "$MCP_BIN" ]; then
        print_info "MCP 服务器未构建，开始构建..."
        cargo build --release -p synton-mcp-server
    fi

    print_info "启动 MCP 服务器 (stdio 模式)..."
    print_info "端点: http://localhost:$REST_PORT"
    print_info "注意: MCP 服务器通常由客户端 (如 Claude Code) 直接启动"
    print_info "此命令仅用于调试测试，按 Ctrl+C 停止"
    echo ""

    exec "$MCP_BIN" --endpoint "http://localhost:$REST_PORT"
}

# 显示帮助
cmd_help() {
    cat << 'HELP_EOF'
================================
  SYNTON-DB Operations Script
================================

Usage:
  $0 <command> [options]

Commands:
  start       Start all services
  stop        Stop all services
  restart     Restart all services
  status      Show service status
  logs        View service logs [service_name]
  rebuild     Rebuild services
  clean       Clean all data
  purge       Purge unused resources
  shell       Enter container shell (Docker only)
  stats       Show database statistics
  test        Test API endpoints
  mcp         Start MCP server (debug only)
  help        Show this help

Description:
  - Run mode controlled by SYNTON_MODE env var (default: local)
  - Local mode (local): Directly run compiled binaries
  - Docker mode (docker): Use Docker Compose to run services
  - Web UI (React + Vite) starts automatically in both modes
  - MCP server uses stdio, runs directly from clients like Claude Code

Environment Variables:
  SYNTON_MODE              Run mode: local or docker (default: local)
  SYNTON_REST_PORT         REST API port (default: 5570)
  SYNTON_GRPC_PORT         gRPC port (default: 5571)
  SYNTON_WEB_PORT          Web UI port (default: 5173)

Examples:
  $0 start                      # Start services (local mode)
  $0 status                     # Show service status
  $0 logs db-server             # View specific service logs (local)
  $0 logs synton-db             # View specific service logs (Docker)
  $0 logs web                   # View web service logs (all modes)
  $0 mcp                        # Manual MCP test (debug only)
  SYNTON_MODE=docker $0 start   # Use Docker mode
================================
HELP_EOF
}

# 主函数
main() {
    local command="${1:-help}"

    case "$command" in
        start|up)
            cmd_start
            ;;
        stop|down)
            cmd_stop
            ;;
        restart)
            cmd_restart
            ;;
        status|ps)
            show_status
            ;;
        logs|log)
            cmd_logs "$2"
            ;;
        rebuild|build)
            cmd_rebuild "$@"
            ;;
        clean)
            cmd_clean
            ;;
        purge)
            cmd_purge
            ;;
        shell|sh)
            cmd_shell "$2"
            ;;
        stats)
            cmd_stats
            ;;
        test)
            cmd_test
            ;;
        mcp)
            cmd_mcp_start
            ;;
        help|--help|-h)
            cmd_help
            ;;
        *)
            print_error "未知命令: $command"
            echo ""
            cmd_help
            exit 1
            ;;
    esac
}

# 运行主函数
main "$@"
