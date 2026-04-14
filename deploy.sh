#!/bin/bash
# CarpTMS 一键部署脚本
# 支持开发环境、测试环境和生产环境部署

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的信息
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 显示帮助信息
show_help() {
    cat << EOF
CarpTMS 一键部署脚本

用法: ./deploy.sh [命令] [选项]

命令:
    dev         部署开发环境
    test        部署测试环境
    prod        部署生产环境
    stop        停止所有服务
    restart     重启所有服务
    logs        查看服务日志
    status      查看服务状态
    clean       清理所有数据和容器
    help        显示帮助信息

选项:
    -b, --build     强制重新构建镜像
    -d, --detach    后台运行（默认）
    -f, --follow    前台运行并跟踪日志

示例:
    ./deploy.sh dev              # 部署开发环境
    ./deploy.sh prod --build     # 生产环境并重新构建
    ./deploy.sh logs backend     # 查看后端日志
    ./deploy.sh status           # 查看服务状态

EOF
}

# 检查 Docker 和 Docker Compose 是否安装
check_docker() {
    if ! command -v docker &> /dev/null; then
        print_error "Docker 未安装，请先安装 Docker"
        exit 1
    fi

    if ! command -v docker-compose &> /dev/null; then
        print_error "Docker Compose 未安装，请先安装 Docker Compose"
        exit 1
    fi

    # 检查 Docker 是否运行
    if ! docker info &> /dev/null; then
        print_error "Docker 服务未运行，请启动 Docker 服务"
        exit 1
    fi

    print_success "Docker 环境检查通过"
}

# 创建环境文件
create_env_file() {
    local env_file=".env"
    
    if [ ! -f "$env_file" ]; then
        print_info "创建环境配置文件..."
        cat > "$env_file" << EOF
# CarpTMS 环境配置
# 数据库配置
DB_PASSWORD=carptms_secure_password_$(date +%s)

# Redis 配置
REDIS_PASSWORD=carptms_redis_$(date +%s)

# 后端配置
RUST_LOG=info
RUST_BACKTRACE=1

# 前端配置
NODE_ENV=production
EOF
        print_success "环境配置文件已创建: $env_file"
        print_warning "请检查并修改 .env 文件中的密码配置"
    else
        print_info "使用现有的环境配置文件"
    fi
}

# 部署开发环境
deploy_dev() {
    print_info "部署开发环境..."
    create_env_file
    
    local compose_args="-f docker-compose.yml"
    
    if [ "$BUILD" = true ]; then
        print_info "强制重新构建镜像..."
        compose_args="$compose_args --build"
    fi
    
    docker-compose $compose_args up -d
    
    print_success "开发环境部署完成！"
    print_info "前端访问: http://localhost"
    print_info "后端 API: http://localhost:8082"
    print_info "PostgreSQL: localhost:5432"
    print_info "Redis: localhost:6379"
}

# 部署测试环境
deploy_test() {
    print_info "部署测试环境..."
    create_env_file
    
    # 设置测试环境变量
    export RUST_LOG=debug
    export NODE_ENV=test
    
    docker-compose -f docker-compose.yml up -d
    
    print_success "测试环境部署完成！"
}

# 部署生产环境
deploy_prod() {
    print_info "部署生产环境..."
    
    # 检查环境变量
    if [ ! -f ".env" ]; then
        print_error "生产环境需要 .env 配置文件"
        print_info "请复制 .env.example 为 .env 并配置正确的密码"
        exit 1
    fi
    
    # 设置生产环境变量
    export RUST_LOG=warn
    export NODE_ENV=production
    
    # 强制重新构建
    docker-compose -f docker-compose.yml up -d --build
    
    print_success "生产环境部署完成！"
    print_info "请确保已配置正确的域名和 SSL 证书"
}

# 停止服务
stop_services() {
    print_info "停止所有服务..."
    docker-compose down
    print_success "所有服务已停止"
}

# 重启服务
restart_services() {
    print_info "重启所有服务..."
    docker-compose restart
    print_success "所有服务已重启"
}

# 查看日志
show_logs() {
    local service=$1
    
    if [ -z "$service" ]; then
        docker-compose logs -f
    else
        docker-compose logs -f "$service"
    fi
}

# 查看状态
show_status() {
    print_info "服务状态:"
    docker-compose ps
    
    print_info "\n容器资源使用:"
    docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}\t{{.BlockIO}}"
}

# 清理环境
clean_environment() {
    print_warning "这将删除所有容器、卷和网络！"
    read -p "确定要继续吗？(yes/no): " confirm
    
    if [ "$confirm" = "yes" ]; then
        print_info "清理环境..."
        docker-compose down -v --remove-orphans
        docker system prune -f
        print_success "环境清理完成"
    else
        print_info "取消清理操作"
    fi
}

# 主函数
main() {
    local command=$1
    shift
    
    # 解析选项
    BUILD=false
    DETACH=true
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            -b|--build)
                BUILD=true
                shift
                ;;
            -d|--detach)
                DETACH=true
                shift
                ;;
            -f|--follow)
                DETACH=false
                shift
                ;;
            *)
                break
                ;;
        esac
    done
    
    # 执行命令
    case $command in
        dev)
            check_docker
            deploy_dev
            ;;
        test)
            check_docker
            deploy_test
            ;;
        prod)
            check_docker
            deploy_prod
            ;;
        stop)
            stop_services
            ;;
        restart)
            restart_services
            ;;
        logs)
            show_logs "$1"
            ;;
        status)
            show_status
            ;;
        clean)
            clean_environment
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            print_error "未知命令: $command"
            show_help
            exit 1
            ;;
    esac
}

# 运行主函数
main "$@"


