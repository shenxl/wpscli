#!/bin/bash
# WPS OAuth Token Server 启动脚本
#
# 使用方式：
#   ./start.sh              # 前台运行
#   ./start.sh --daemon     # 后台运行（nohup）
#   ./start.sh --stop       # 停止后台进程
#   ./start.sh --status     # 查看 Token 状态
#   ./start.sh --logs       # 查看后台日志

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PIDFILE="$SCRIPT_DIR/data/oauth_server.pid"
LOGFILE="$SCRIPT_DIR/data/oauth_server.log"
PYTHON="${PYTHON:-python3}"
OAUTH_PORT="${WPS_OAUTH_PORT:-8089}"

find_running_pid() {
    # 优先按监听端口识别真实服务进程，避免误匹配当前 shell 命令
    local pid cmd
    pid="$(lsof -ti tcp:"$OAUTH_PORT" -sTCP:LISTEN 2>/dev/null | head -n 1)"
    if [ -n "$pid" ]; then
        cmd="$(ps -p "$pid" -o command= 2>/dev/null)"
        if [ -n "$cmd" ] && [[ "$cmd" == *"oauth_server.py"* ]]; then
            echo "$pid"
            return
        fi
    fi

    # 兜底：匹配 Python 启动命令
    pgrep -f "[Pp]ython.*oauth_server.py --no-browser" 2>/dev/null | head -n 1
}

case "${1:-}" in
    --daemon|-d)
        # 后台运行
        if [ -f "$PIDFILE" ] && kill -0 "$(cat "$PIDFILE")" 2>/dev/null; then
            echo "⚠️  OAuth Server 已经在运行 (PID: $(cat "$PIDFILE"))"
            echo "   使用 $0 --stop 先停止"
            exit 1
        fi
        if [ ! -f "$PIDFILE" ]; then
            RUNNING_PID="$(find_running_pid)"
            if [ -n "$RUNNING_PID" ] && kill -0 "$RUNNING_PID" 2>/dev/null; then
                echo "$RUNNING_PID" > "$PIDFILE"
                echo "⚠️  检测到已运行进程 (PID: $RUNNING_PID)，已自动恢复 PID 文件"
                exit 0
            fi
        fi

        echo "🚀 启动 OAuth Token Server (后台模式)..."
        if [ -f "$LOGFILE" ]; then
            echo "" >> "$LOGFILE"
        fi
        echo "===== $(date '+%Y-%m-%d %H:%M:%S') start.sh --daemon =====" >> "$LOGFILE"
        nohup "$PYTHON" "$SCRIPT_DIR/oauth_server.py" --no-browser >> "$LOGFILE" 2>&1 &
        echo $! > "$PIDFILE"
        sleep 1

        if kill -0 "$(cat "$PIDFILE")" 2>/dev/null; then
            echo "✅ 启动成功！"
            echo "   PID: $(cat "$PIDFILE")"
            echo "   日志: $LOGFILE"
            echo "   状态: $0 --status"
            echo "   停止: $0 --stop"
        else
            echo "❌ 启动失败，请检查日志: $LOGFILE"
            cat "$LOGFILE"
            exit 1
        fi
        ;;

    --stop|-k)
        # 停止后台进程
        if [ -f "$PIDFILE" ]; then
            PID=$(cat "$PIDFILE")
            if kill -0 "$PID" 2>/dev/null; then
                echo "🛑 停止 OAuth Server (PID: $PID)..."
                kill "$PID"
                sleep 2
                if kill -0 "$PID" 2>/dev/null; then
                    echo "   强制终止..."
                    kill -9 "$PID"
                fi
                rm -f "$PIDFILE"
                echo "✅ 已停止"
                EXTRA_PID="$(find_running_pid)"
                if [ -n "$EXTRA_PID" ] && [ "$EXTRA_PID" != "$PID" ] && kill -0 "$EXTRA_PID" 2>/dev/null; then
                    echo "   检测到额外实例 (PID: $EXTRA_PID)，正在停止..."
                    kill "$EXTRA_PID" 2>/dev/null
                    sleep 2
                    if kill -0 "$EXTRA_PID" 2>/dev/null; then
                        echo "   强制终止额外实例..."
                        kill -9 "$EXTRA_PID" 2>/dev/null
                    fi
                fi
            else
                echo "⚠️  进程不存在 (PID: $PID)，清理 PID 文件"
                rm -f "$PIDFILE"
                RUNNING_PID="$(find_running_pid)"
                if [ -n "$RUNNING_PID" ] && kill -0 "$RUNNING_PID" 2>/dev/null; then
                    echo "   检测到存活进程 (PID: $RUNNING_PID)，正在停止..."
                    kill "$RUNNING_PID" 2>/dev/null
                    sleep 2
                    if kill -0 "$RUNNING_PID" 2>/dev/null; then
                        echo "   强制终止..."
                        kill -9 "$RUNNING_PID" 2>/dev/null
                    fi
                    echo "✅ 已停止"
                fi
            fi
        else
            echo "⚠️  没有找到 PID 文件，尝试查找进程..."
            PIDS=$(pgrep -f "oauth_server.py" 2>/dev/null)
            if [ -n "$PIDS" ]; then
                echo "   找到进程: $PIDS"
                kill $PIDS 2>/dev/null
                echo "✅ 已发送停止信号"
            else
                echo "   未找到运行中的 OAuth Server"
            fi
        fi
        ;;

    --restart|-r)
        # 重启
        "$0" --stop
        sleep 1
        "$0" --daemon
        ;;

    --status|-s)
        # 查看状态
        "$PYTHON" "$SCRIPT_DIR/oauth_server.py" --status
        echo ""
        if [ -f "$PIDFILE" ] && kill -0 "$(cat "$PIDFILE")" 2>/dev/null; then
            echo "🟢 服务运行中 (PID: $(cat "$PIDFILE"))"
        else
            RUNNING_PID="$(find_running_pid)"
            if [ -n "$RUNNING_PID" ] && kill -0 "$RUNNING_PID" 2>/dev/null; then
                echo "$RUNNING_PID" > "$PIDFILE"
                echo "🟢 服务运行中 (PID: $RUNNING_PID) [已自动修复 PID 文件]"
            else
                echo "🔴 服务未运行"
            fi
        fi
        ;;

    --logs|-l)
        # 查看日志
        if [ -f "$LOGFILE" ]; then
            tail -f "$LOGFILE"
        else
            echo "⚠️  日志文件不存在: $LOGFILE"
        fi
        ;;

    --help|-h)
        echo "WPS OAuth Token Server 启动脚本"
        echo ""
        echo "用法:"
        echo "  $0               前台运行"
        echo "  $0 --daemon|-d   后台运行（nohup）"
        echo "  $0 --stop|-k     停止后台进程"
        echo "  $0 --restart|-r  重启后台进程"
        echo "  $0 --status|-s   查看 Token 和服务状态"
        echo "  $0 --logs|-l     查看后台日志（tail -f）"
        echo "  $0 --help|-h     显示此帮助"
        echo ""
        echo "环境变量:"
        echo "  WPS_OAUTH_PORT       监听端口 (默认: 8089)"
        echo "  WPS_OAUTH_DOMAIN     回调域名 (默认: wps-mac.shenxl.com)"
        echo "  WPS_AUTH_CONF        auth.conf 路径"
        echo "  WPS_REFRESH_INTERVAL 刷新间隔秒数 (默认: 3600)"
        echo "  PYTHON               Python 解释器 (默认: python3)"
        ;;

    *)
        # 前台运行
        echo "🚀 启动 OAuth Token Server (前台模式)..."
        echo "   按 Ctrl+C 停止"
        exec "$PYTHON" "$SCRIPT_DIR/oauth_server.py" --no-browser "$@"
        ;;
esac
