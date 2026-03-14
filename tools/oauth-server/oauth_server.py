#!/usr/bin/env python3
"""
WPS 365 OAuth Token Server
通用的 OAuth 授权服务，为所有 skills 提供用户 Token

核心特性：
- 启动 HTTP 服务监听 OAuth 回调
- Token 缓存到本地文件（含 scope 信息）
- 后台线程自动刷新 Token（每小时）
- Scope 感知：当请求更大 scope 时自动触发重新授权
- 提供 HTTP API 供其他 Skills 查询 Token / 请求授权
- 端口默认 8089（可通过 nginx 反向代理到 80/443），支持 systemd / nohup 后台运行

部署方式：
  # 直接运行（前台）
  python3 oauth_server.py --no-browser

  # 后台运行（nohup）
  nohup python3 oauth_server.py --no-browser > /var/log/wps-oauth.log 2>&1 &

  # systemd 方式
  sudo systemctl start wps-oauth
"""

import os
import sys
import json
import time
import secrets
import signal
import threading
import webbrowser
import urllib.parse
import hmac
import logging
import ipaddress
from datetime import datetime
from pathlib import Path
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs, urlencode

import requests
from wps_platform import PlatformConfig, build_platform_health_snapshot

# ============ 日志 ============
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s [%(levelname)s] %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)
logger = logging.getLogger('wps-oauth')

# ============ 配置（全部通过环境变量或相对路径，无硬编码绝对路径）============
BASE_DIR = Path(__file__).parent

# auth.conf 查找顺序：环境变量 > 同目录 > 上两级目录（兼容开发和部署）
def _find_auth_conf() -> Path:
    """查找 auth.conf 配置文件"""
    # 1. 环境变量指定
    env_path = os.environ.get('WPS_AUTH_CONF')
    if env_path:
        p = Path(env_path)
        if p.exists():
            return p

    # 2. 同目录下
    local = BASE_DIR / 'auth.conf'
    if local.exists():
        return local

    # 3. data 子目录
    data_local = BASE_DIR / 'data' / 'auth.conf'
    if data_local.exists():
        return data_local

    # 4. 向上查找（兼容开发环境 skills/wps-oauth/ -> openapi/auth.conf）
    for parent in [BASE_DIR.parent.parent, BASE_DIR.parent]:
        p = parent / 'auth.conf'
        if p.exists():
            return p

    raise FileNotFoundError(
        "找不到 auth.conf 配置文件。请将 auth.conf 放在以下任一位置：\n"
        f"  1. 环境变量 WPS_AUTH_CONF 指定的路径\n"
        f"  2. {BASE_DIR / 'auth.conf'}\n"
        f"  3. {BASE_DIR / 'data' / 'auth.conf'}\n"
        "  格式：\n"
        "    AK:你的AppKey\n"
        "    SK:你的SecretKey"
    )

AUTH_CONFIG = _find_auth_conf()
TOKEN_CACHE_PATH = BASE_DIR / 'data' / 'token_cache.json'
PLATFORM_CONFIG = PlatformConfig.from_env(BASE_DIR)

# WPS OpenAPI
API_BASE = 'https://openapi.wps.cn'
OAUTH_AUTH_URL = f'{API_BASE}/oauth2/auth'
OAUTH_TOKEN_URL = f'{API_BASE}/oauth2/token'
DEFAULT_SCOPE = 'kso.user_base.read,kso.user_current_id.read'
APP_SCOPE = 'kso.chat_message.sendwrite'

# 服务器配置
# 默认 8089，通常由 nginx 在 80/443 反向代理到该端口，避免与本机 Web 服务冲突。
DEFAULT_PORT = int(os.environ.get('WPS_OAUTH_PORT', '8089'))
CALLBACK_DOMAIN = os.environ.get('WPS_OAUTH_DOMAIN', 'wps-mac.shenxl.com')
RAW_BASE_PATH = os.environ.get('WPS_OAUTH_BASE_PATH', '/oauth').strip()
if RAW_BASE_PATH in ('', '/'):
    BASE_PATH = ''
else:
    BASE_PATH = '/' + RAW_BASE_PATH.strip('/')
INTERNAL_CALLBACK_PATH = '/callback'
CALLBACK_PATH = f"{BASE_PATH}{INTERNAL_CALLBACK_PATH}"
CALLBACK_SCHEME = os.environ.get('WPS_OAUTH_SCHEME', 'https')
CALLBACK_URL = f'{CALLBACK_SCHEME}://{CALLBACK_DOMAIN}{CALLBACK_PATH}'

ADMIN_TOKEN = os.environ.get('WPS_OAUTH_ADMIN_TOKEN', '').strip()
PENDING_STATE_FILE = BASE_DIR / 'data' / 'pending_states.json'

# Token 刷新间隔（秒）
REFRESH_INTERVAL = int(os.environ.get('WPS_REFRESH_INTERVAL', '3600'))  # 1小时

# ============ WPS 有效 Scope 白名单 ============
# 只有在此集合中的 scope 才会包含在授权 URL 中，避免无效 scope 导致 invalid_scope 错误
VALID_WPS_SCOPES = {
    # 用户
    'kso.user_base.read',
    'kso.user_current_id.read',
    'kso.agent.readwrite',
    # 文件
    'kso.file.read',
    'kso.file.readwrite',
    'kso.file_search.readwrite',
    'kso.file_link.readwrite',
    'kso.file_version.readwrite',
    'kso.file_permission.readwrite',
    'kso.deleted_file.read',
    'kso.deleted_file.readwrite',
    'kso.documents.readwrite',
    'kso.doclib.read',
    'kso.doclib.readwrite',
    'kso.drive.readwrite',
    'kso.drive_role.readwrite',
    'kso.coop_files.readwrite',
    'kso.wiki.readwrite',
    # 多维表
    'kso.dbsheet.read',
    'kso.dbsheet.readwrite',
    'kso.sheets.read',
    'kso.sheets.readwrite',
    'kso.airsheet.read',
    'kso.airsheet.readwrite',
    # 日历
    'kso.calendar.read',
    'kso.calendar.readwrite',
    'kso.calendar_events.read',
    'kso.calendar_events.readwrite',
    # 任务
    'kso.task.read',
    'kso.task.readwrite',
    # 通讯录
    'kso.contact.read',
    # 会议
    'kso.meeting.read',
    'kso.meeting.readwrite',
    'kso.meeting_minutes.read',
    'kso.meeting_minutes_content.read',
    'kso.meeting_recording.read',
    'kso.meeting_recording.readwrite',
    'kso.meeting_recording_content.read',
    # 消息/群组
    'kso.chat.read',
    'kso.chat.readwrite',
    'kso.chat_message.readwrite',
    'kso.chat_message.sendwrite',
    'kso.chat_bookmark.readwrite',
    'kso.group.read',
    'kso.group.readwrite',
    'kso.component.agentspace_chat',
    # AirPage
    'kso.airpage.read',
    'kso.airpage.readwrite',
    # 应用/自动化/AI
    'kso.app.read',
    'kso.app.readwrite',
    'kso.apps.read',
    'kso.apps.readwrite',
    'kso.automation.read',
    'kso.automation.readwrite',
    'kso.aidocs.readwrite',
    'kso.aidocs_extract.readwrite',
    'kso.docqa.readwrite',
    'kso.devhub_app.readwrite',
    'kso.devhub_chat.readwrite',
    'kso.devhub_session.readwrite',
    # 审批
    'kso.workflow_approval_define.read',
    'kso.workflow_approval_define.readwrite',
    'kso.workflow_approval_instance.read',
    'kso.workflow_approval_instance.readwrite',
    'kso.workflow_approval_task.readwrite',
}


def filter_valid_scopes(scope_str: str) -> str:
    """过滤掉不在白名单中的无效 scope，返回逗号分隔的有效 scope 字符串"""
    if not scope_str:
        return scope_str
    scopes = [s.strip() for s in scope_str.replace(',', ' ').split() if s.strip()]
    valid = [s for s in scopes if s in VALID_WPS_SCOPES]
    invalid = [s for s in scopes if s not in VALID_WPS_SCOPES]
    if invalid:
        logger.warning(f"过滤无效 scope: {invalid}")
    return ','.join(valid)


def normalize_scope_string(scope_str: str) -> str:
    """将 scope 字符串规范为逗号分隔格式（不做白名单过滤）"""
    if not scope_str:
        return ''
    parts = [s.strip() for s in scope_str.replace(',', ' ').split() if s.strip()]
    return ','.join(parts)


# 确保数据目录存在
(BASE_DIR / 'data').mkdir(exist_ok=True)


class ScopeSet:
    """OAuth scope 集合，用于比较和合并 scope
    
    WPS OAuth 使用逗号分隔 scope，例如：
        kso.file.read,kso.file.readwrite,kso.calendar.read
    """

    def __init__(self, scope_str: str = None):
        self.scopes = set()
        if scope_str:
            # 同时支持逗号和空格分隔（兼容旧格式）
            parts = scope_str.replace(',', ' ').split()
            self.scopes = set(s.strip() for s in parts if s.strip())

    def __str__(self):
        """返回逗号分隔的 scope 字符串（WPS OAuth 标准格式）"""
        return ','.join(sorted(self.scopes))

    def __contains__(self, item):
        return item in self.scopes

    def __bool__(self):
        return bool(self.scopes)

    def covers(self, required: 'ScopeSet') -> bool:
        """当前 scope 是否覆盖了 required 的所有 scope"""
        if not required.scopes:
            return True
        return required.scopes.issubset(self.scopes)

    def merge(self, other: 'ScopeSet') -> 'ScopeSet':
        """合并两个 scope 集合"""
        result = ScopeSet()
        result.scopes = self.scopes | other.scopes
        return result

    @property
    def as_list(self) -> list:
        return sorted(self.scopes)


class TokenStore:
    """Token 存储管理（含 scope 信息）"""

    def __init__(self, cache_path: Path = TOKEN_CACHE_PATH):
        self.cache_path = cache_path
        self._lock = threading.Lock()
        self._data = self._load()

    def _load(self) -> dict:
        """从文件加载 Token 缓存"""
        if self.cache_path.exists():
            try:
                with open(self.cache_path, 'r', encoding='utf-8') as f:
                    data = json.load(f)
                    if 'app' not in data:
                        data['app'] = {'token': None, 'expires_at': 0}
                    if 'user' not in data:
                        data['user'] = {'token': None, 'expires_at': 0, 'refresh_token': None, 'scope': ''}
                    # 确保 user 有 scope 字段
                    if 'scope' not in data.get('user', {}):
                        data['user']['scope'] = ''
                    return data
            except Exception as e:
                logger.warning(f"加载 Token 缓存失败: {e}")
        return {
            'app': {'token': None, 'expires_at': 0},
            'user': {'token': None, 'expires_at': 0, 'refresh_token': None, 'scope': ''},
            'last_refresh': None,
            'last_auth': None
        }

    def _save(self):
        """保存 Token 缓存到文件"""
        try:
            self.cache_path.parent.mkdir(parents=True, exist_ok=True)
            with open(self.cache_path, 'w', encoding='utf-8') as f:
                json.dump(self._data, f, ensure_ascii=False, indent=2)
        except Exception as e:
            logger.error(f"保存 Token 缓存失败: {e}")

    def get_user_token(self) -> str:
        """获取用户 Token（如果未过期）"""
        with self._lock:
            user = self._data.get('user', {})
            if user.get('token') and time.time() < user.get('expires_at', 0) - 60:
                return user['token']
            return None

    def get_user_scope(self) -> ScopeSet:
        """获取当前用户 Token 的 scope"""
        with self._lock:
            return ScopeSet(self._data.get('user', {}).get('scope', ''))

    def get_refresh_token(self) -> str:
        """获取 refresh_token"""
        with self._lock:
            return self._data.get('user', {}).get('refresh_token')

    def get_app_token(self) -> str:
        """获取应用 Token（如果未过期）"""
        with self._lock:
            app = self._data.get('app', {})
            if app.get('token') and time.time() < app.get('expires_at', 0) - 60:
                return app['token']
            return None

    def set_user_token(self, access_token: str, expires_in: int,
                       refresh_token: str = None, scope: str = None):
        """设置用户 Token（含 scope 信息）"""
        with self._lock:
            old_scope = self._data.get('user', {}).get('scope', '')
            self._data['user'] = {
                'token': access_token,
                'expires_at': time.time() + expires_in,
                'refresh_token': refresh_token or self._data.get('user', {}).get('refresh_token'),
                'scope': scope if scope is not None else old_scope,
                'updated_at': datetime.now().isoformat()
            }
            self._data['last_auth'] = datetime.now().isoformat()
            self._save()

    def set_app_token(self, access_token: str, expires_in: int):
        """设置应用 Token"""
        with self._lock:
            self._data['app'] = {
                'token': access_token,
                'expires_at': time.time() + expires_in,
                'updated_at': datetime.now().isoformat()
            }
            self._save()

    def update_refresh_time(self):
        """更新最后刷新时间"""
        with self._lock:
            self._data['last_refresh'] = datetime.now().isoformat()
            self._save()

    def get_status(self) -> dict:
        """获取 Token 状态（含 scope 信息）"""
        with self._lock:
            user = self._data.get('user', {})
            app = self._data.get('app', {})
            now = time.time()

            user_valid = bool(user.get('token') and now < user.get('expires_at', 0))
            app_valid = bool(app.get('token') and now < app.get('expires_at', 0))

            user_ttl = max(0, user.get('expires_at', 0) - now) if user_valid else 0
            app_ttl = max(0, app.get('expires_at', 0) - now) if app_valid else 0

            return {
                'user': {
                    'valid': user_valid,
                    'has_refresh_token': bool(user.get('refresh_token')),
                    'ttl_seconds': int(user_ttl),
                    'ttl_human': self._format_ttl(user_ttl),
                    'scope': user.get('scope', ''),
                    'scope_list': ScopeSet(user.get('scope', '')).as_list,
                    'scope_missing': bool(user_valid and not ScopeSet(user.get('scope', '')).as_list),
                    'updated_at': user.get('updated_at')
                },
                'app': {
                    'valid': app_valid,
                    'ttl_seconds': int(app_ttl),
                    'ttl_human': self._format_ttl(app_ttl),
                    'updated_at': app.get('updated_at')
                },
                'last_refresh': self._data.get('last_refresh'),
                'last_auth': self._data.get('last_auth')
            }

    def _format_ttl(self, seconds: float) -> str:
        if seconds <= 0:
            return '已过期'
        hours = int(seconds // 3600)
        minutes = int((seconds % 3600) // 60)
        if hours > 0:
            return f'{hours}小时{minutes}分钟'
        return f'{minutes}分钟'




def _load_pending_states() -> dict:
    if not PENDING_STATE_FILE.exists():
        return {}
    try:
        with open(PENDING_STATE_FILE, 'r', encoding='utf-8') as f:
            data = json.load(f)
            return data if isinstance(data, dict) else {}
    except Exception:
        return {}


def _save_pending_states(states: dict):
    try:
        PENDING_STATE_FILE.parent.mkdir(parents=True, exist_ok=True)
        with open(PENDING_STATE_FILE, 'w', encoding='utf-8') as f:
            json.dump(states, f, ensure_ascii=False, indent=2)
    except Exception as e:
        logger.warning(f"保存 pending states 失败: {e}")


def _is_retryable_exchange_error(result: dict) -> bool:
    if not isinstance(result, dict):
        return False
    code = str(result.get('code', ''))
    msg = str(result.get('msg', '')).lower()
    debug = result.get('debug') if isinstance(result.get('debug'), dict) else {}
    cause = str(debug.get('cause', '')).lower()
    return code == '50000006' or 'server_error' in msg or 'exec fail' in cause

class WPSAuth:
    """WPS OAuth 认证管理（支持 scope 跟踪和扩展）"""

    def __init__(self, token_store: TokenStore):
        self.token_store = token_store
        self.ak, self.sk = self._load_auth()
        self._pending_states = _load_pending_states()  # state -> {created_at, scope}

    def _load_auth(self) -> tuple:
        """加载 AK/SK"""
        config = {}
        with open(AUTH_CONFIG) as f:
            for line in f:
                if ':' in line:
                    key, value = line.strip().split(':', 1)
                    config[key.strip()] = value.strip()

        ak = config.get('AK')
        sk = config.get('SK')
        if not ak or not sk:
            raise ValueError(f"认证配置缺少 AK 或 SK，请检查: {AUTH_CONFIG}")
        logger.info(f"已加载认证配置: {AUTH_CONFIG}")
        return ak, sk

    def check_scope_coverage(self, required_scope: str) -> dict:
        """
        检查当前 Token 的 scope 是否满足需求
        
        Returns:
            {
                'covered': bool,           # 是否已覆盖
                'current_scope': str,       # 当前 scope
                'required_scope': str,      # 需要的 scope
                'missing_scopes': list,     # 缺少的 scope
                'merged_scope': str,        # 合并后的完整 scope
                'need_reauth': bool,        # 是否需要重新授权
                'auth_url': str or None     # 如需重授权，返回授权 URL
            }
        """
        current = self.token_store.get_user_scope()
        required = ScopeSet(required_scope)

        if current.covers(required):
            return {
                'covered': True,
                'current_scope': str(current),
                'required_scope': str(required),
                'missing_scopes': [],
                'merged_scope': str(current),
                'need_reauth': False,
                'auth_url': None
            }

        # 需要更大的 scope
        merged = current.merge(required)
        missing = sorted(required.scopes - current.scopes)
        auth_url, state = self.get_authorize_url(str(merged))

        return {
            'covered': False,
            'current_scope': str(current),
            'required_scope': str(required),
            'missing_scopes': missing,
            'merged_scope': str(merged),
            'need_reauth': True,
            'auth_url': auth_url,
            'state': state
        }

    def get_authorize_url(self, scope: str = None) -> tuple:
        """生成 OAuth 授权 URL，返回 (url, state)"""
        state = secrets.token_urlsafe(16)

        # 清理过期 state（10分钟）
        expired = [k for k, v in self._pending_states.items()
                   if time.time() - v['created_at'] > 600]
        for k in expired:
            del self._pending_states[k]
        if expired:
            _save_pending_states(self._pending_states)

        # 确保 scope 使用逗号分隔（WPS OAuth 标准格式），并过滤无效 scope
        scope_str = normalize_scope_string(scope or DEFAULT_SCOPE)
        # 过滤无效 scope，防止 invalid_scope 错误
        scope_str = filter_valid_scopes(scope_str) or DEFAULT_SCOPE
        # 记录实际发起授权使用的 scope，供 callback 持久化
        self._pending_states[state] = {
            'created_at': time.time(),
            'scope': scope_str
        }
        _save_pending_states(self._pending_states)

        # 使用 urlencode 统一参数编码，避免手工拼接带来的编码不一致。
        query = urlencode(
            {
                'response_type': 'code',
                'client_id': self.ak,
                'redirect_uri': CALLBACK_URL,
                'scope': scope_str,
                'state': state
            },
            quote_via=urllib.parse.quote,
            safe=''
        )
        url = f"{OAUTH_AUTH_URL}?{query}"
        return url, state

    def get_pending_scope(self, state: str) -> str:
        """获取 state 对应的 scope"""
        pending = self._pending_states.get(state, {})
        return pending.get('scope', '')

    def exchange_code(self, code: str, state: str = None) -> dict:
        """用授权码换取 Token"""
        # 获取这次授权请求的 scope
        scope = None
        state_known = False
        code_hint = (code[:8] + '...') if code else ''
        if state:
            pending = self._pending_states.get(state)
            if pending:
                scope = pending.get('scope')
                state_known = True
                del self._pending_states[state]
                _save_pending_states(self._pending_states)
                logger.info(f"OAuth callback 命中 pending state: {state[:8]}..., scope={scope}")
            else:
                logger.warning(f"OAuth callback state 不存在或已过期: {state[:8]}...，将保留已有 scope")
        else:
            logger.warning(f"OAuth callback 缺少 state 参数，code={code_hint}")

        # 安全保护：授权码回调必须命中本地 pending state。
        # 否则这次回调不是由当前服务发起（例如复用旧链接、跨重启、手工拼接 state），
        # 继续调用 token 端点会产生误导性 500 错误并污染排查。
        if not state_known:
            logger.error(
                "❌ 回调 state 未命中 pending 记录，拒绝执行授权码换 token。"
                f"state_present={bool(state)}, code={code_hint}"
            )
            return {
                'success': False,
                'error': {
                    'code': 'invalid_or_expired_state',
                    'msg': '回调 state 不存在、已过期或不属于当前进程',
                    'tip': '请通过 /api/auth/start 重新生成授权链接，并在 10 分钟内完成授权回调'
                }
            }

        data = {
            'grant_type': 'authorization_code',
            'client_id': self.ak,
            'client_secret': self.sk,
            'code': code,
            'redirect_uri': CALLBACK_URL
        }

        # 注意：authorization_code 是一次性凭证，任何重试都可能导致“第一次已消费、第二次 invalid_grant”。
        # 因此授权码换 token 必须严格单次请求；若失败，要求重新走 /api/auth/start 申请新 code。
        result = None
        resp = None
        try:
            resp = requests.post(
                OAUTH_TOKEN_URL,
                data=urlencode(data),
                headers={'Content-Type': 'application/x-www-form-urlencoded'},
                timeout=30
            )
            result = resp.json()
        except Exception as e:
            logger.error(f"❌ 授权码换取 Token 请求异常（不重试，避免重复消费授权码）: {e}")
            return {
                'success': False,
                'error': {
                    'code': 'token_exchange_request_error',
                    'msg': '授权码换取 Token 请求异常',
                    'debug': {'cause': str(e)},
                    'tip': '请通过 /api/auth/start 重新生成授权链接后重试'
                }
            }

        if 'access_token' in result:
            # 优先使用 OAuth 返回的 scope；若返回缺失则回退到 pending state。
            # 若 state 未命中且返回也无 scope，拒绝本次写入，避免出现“授权成功但 scope 丢失”。
            response_scope = normalize_scope_string(result.get('scope', '')) if isinstance(result.get('scope'), str) else ''
            previous_scope = str(self.token_store.get_user_scope())
            if not response_scope and not state_known:
                logger.error(
                    "❌ 授权成功但 scope 缺失，且 state 未命中 pending 记录，拒绝写入以避免假成功。"
                    f"state_present={bool(state)}, state_known={state_known}, "
                    f"previous_scope_present={bool(previous_scope)}"
                )
                return {
                    'success': False,
                    'error': {
                        'code': 'scope_missing_state_miss',
                        'msg': '授权返回未携带 scope，且 state 未命中本地待授权记录',
                        'tip': '请勿复用旧 state；请通过 /api/auth/start 重新生成授权链接并一次性完成回调'
                    }
                }
            if not response_scope and not scope and not previous_scope:
                logger.error(
                    "❌ 授权成功但 scope 为空，且无法从 state/缓存回退。"
                    f"state_present={bool(state)}, state_known={state_known}, "
                    "请使用 /api/auth/start 生成新链接后重试。"
                )
                return {
                    'success': False,
                    'error': {
                        'code': 'empty_scope_without_fallback',
                        'msg': '授权返回 scope 为空，且 state 未命中 pending 记录',
                        'tip': '请勿复用旧 state 或手工拼接授权链接，请通过 /api/auth/start 重新发起授权'
                    }
                }
            effective_scope = response_scope or scope or previous_scope or None

            self.token_store.set_user_token(
                access_token=result['access_token'],
                expires_in=result.get('expires_in', 7200),
                refresh_token=result.get('refresh_token'),
                scope=effective_scope  # None 时保留旧 scope
            )
            saved_scope = str(self.token_store.get_user_scope())
            if not response_scope:
                logger.warning(
                    "授权返回 scope 为空，已回退保存 scope。"
                    f"state_present={bool(state)}, state_known={state_known}, pending_scope={'yes' if scope else 'no'}, previous_scope={'yes' if previous_scope else 'no'}"
                )
            if response_scope and scope and response_scope != scope:
                logger.warning(f"授权返回 scope 与请求 scope 不一致: requested={scope}, returned={response_scope}")
            logger.info(f"✅ 用户授权成功！scope={saved_scope}, 有效期={result.get('expires_in', 7200)}s")
            return {
                'success': True,
                'expires_in': result.get('expires_in', 7200),
                'scope': saved_scope
            }
        else:
            if isinstance(result, dict) and str(result.get('code', '')) == '40100009':
                logger.error(
                    "❌ 授权码已失效或已被使用（invalid_grant）: "
                    f"code={code_hint}, state_present={bool(state)}, redirect_uri={CALLBACK_URL}, result={result}"
                )
                return {
                    'success': False,
                    'error': {
                        'code': 'invalid_grant',
                        'msg': '授权码无效、已过期或已被使用',
                        'debug': result.get('debug', {}),
                        'tip': '请勿重复提交同一个回调；通过 /api/auth/start 重新生成新链接并在 10 分钟内一次性完成授权'
                    }
                }
            logger.error(
                "❌ 授权码换取 Token 失败: "
                f"http_status={getattr(resp, 'status_code', 'unknown')}, "
                f"code={code_hint}, state_present={bool(state)}, result={result}, redirect_uri={CALLBACK_URL}"
            )
            return {'success': False, 'error': result}

    def refresh_user_token(self) -> bool:
        """刷新用户 Token（保留原 scope）"""
        refresh_token = self.token_store.get_refresh_token()
        if not refresh_token:
            logger.warning("没有 refresh_token，无法刷新")
            return False

        data = {
            'grant_type': 'refresh_token',
            'client_id': self.ak,
            'client_secret': self.sk,
            'refresh_token': refresh_token
        }

        try:
            logger.debug(f"正在刷新用户 Token（refresh_token: {refresh_token[:20]}...）")
            resp = requests.post(
                OAUTH_TOKEN_URL,
                data=urlencode(data),
                headers={'Content-Type': 'application/x-www-form-urlencoded'},
                timeout=30
            )

            result = resp.json()

            if 'access_token' in result:
                expires_in = result.get('expires_in', 7200)
                new_refresh = result.get('refresh_token', refresh_token)
                # 刷新时保留原 scope（scope=None 表示不修改）
                self.token_store.set_user_token(
                    access_token=result['access_token'],
                    expires_in=expires_in,
                    refresh_token=new_refresh,
                    scope=None  # 保留已有 scope
                )
                self.token_store.update_refresh_time()
                logger.info(f"✅ 用户 Token 刷新成功！有效期: {expires_in}s "
                           f"(~{expires_in // 60}分钟), "
                           f"refresh_token {'已更新' if new_refresh != refresh_token else '未变化'}")
                return True
            else:
                error_code = result.get('error', 'unknown')
                error_desc = result.get('error_description', str(result))
                logger.error(f"❌ 用户 Token 刷新失败: error={error_code}, desc={error_desc}")
                # 如果是 refresh_token 失效，清理旧数据提示重新授权
                if error_code in ('invalid_grant', 'invalid_token'):
                    logger.error("❌ Refresh Token 已失效，需要用户重新授权")
                return False
        except requests.exceptions.Timeout:
            logger.error("❌ 用户 Token 刷新超时（30s）")
            return False
        except requests.exceptions.ConnectionError as e:
            logger.error(f"❌ 用户 Token 刷新网络连接失败: {e}")
            return False
        except Exception as e:
            logger.error(f"❌ 用户 Token 刷新异常: {e}", exc_info=True)
            return False

    def refresh_app_token(self) -> bool:
        """获取/刷新应用 Token"""
        data = {
            'grant_type': 'client_credentials',
            'client_id': self.ak,
            'client_secret': self.sk
        }

        try:
            resp = requests.post(
                OAUTH_TOKEN_URL,
                data=urlencode(data),
                headers={'Content-Type': 'application/x-www-form-urlencoded'}
            )

            result = resp.json()

            if 'access_token' in result:
                self.token_store.set_app_token(
                    access_token=result['access_token'],
                    expires_in=result.get('expires_in', 7200)
                )
                return True
            else:
                logger.error(f"❌ 应用 Token 获取失败: {result}")
                return False
        except Exception as e:
            logger.error(f"❌ 应用 Token 获取异常: {e}")
            return False


class TokenRefresher(threading.Thread):
    """后台 Token 自动刷新线程
    
    刷新策略：
    - 启动后立即检查并刷新过期/即将过期的 Token
    - 根据 Token 过期时间智能调度下次刷新（过期前 10 分钟）
    - 刷新失败时短间隔重试（5 分钟），最多连续重试 5 次
    - 完整的异常处理，防止线程静默崩溃
    """

    RETRY_INTERVAL = 300       # 失败后 5 分钟重试
    REFRESH_BEFORE = 600       # 提前 10 分钟刷新
    MAX_RETRY = 5              # 连续失败最大重试次数
    MIN_SLEEP = 60             # 最小检查间隔 60 秒
    MAX_SLEEP = 3600           # 最大检查间隔 1 小时

    def __init__(self, auth: WPSAuth, interval: int = REFRESH_INTERVAL):
        super().__init__(daemon=True)
        self.auth = auth
        self.interval = interval
        self._stop_event = threading.Event()
        self._consecutive_failures = 0

    def _get_next_wait(self) -> float:
        """根据 Token 过期时间计算下次检查等待时间"""
        try:
            user = self.auth.token_store._data.get('user', {})
            app = self.auth.token_store._data.get('app', {})
            now = time.time()

            # 计算最近的过期时间
            soonest_expiry = float('inf')

            user_expires = user.get('expires_at', 0)
            if user.get('token') and user_expires > now:
                soonest_expiry = min(soonest_expiry, user_expires)

            app_expires = app.get('expires_at', 0)
            if app.get('token') and app_expires > now:
                soonest_expiry = min(soonest_expiry, app_expires)

            if soonest_expiry == float('inf'):
                # Token 都已过期或不存在，使用重试间隔
                return self.RETRY_INTERVAL

            # 在过期前 REFRESH_BEFORE 秒刷新
            wait = soonest_expiry - now - self.REFRESH_BEFORE
            return max(self.MIN_SLEEP, min(wait, self.MAX_SLEEP))
        except Exception:
            return self.interval

    def run(self):
        logger.info(f"🔄 Token 自动刷新已启动（基准间隔: {self.interval // 60} 分钟，智能调度）")

        # 启动后立即做一次检查（不等待）
        self._do_refresh_cycle()

        while not self._stop_event.is_set():
            wait_time = self._get_next_wait()

            # 如果有连续失败，使用更短的重试间隔
            if self._consecutive_failures > 0:
                wait_time = min(wait_time, self.RETRY_INTERVAL)
                logger.info(f"⏰ 刷新失败重试 ({self._consecutive_failures}/{self.MAX_RETRY})，"
                           f"{int(wait_time)}秒后重试...")
            else:
                logger.info(f"⏰ 下次刷新检查: {int(wait_time)}秒后")

            self._stop_event.wait(wait_time)
            if self._stop_event.is_set():
                break

            self._do_refresh_cycle()

    def _do_refresh_cycle(self):
        """执行一轮 Token 刷新，包含完整的异常处理"""
        try:
            now = time.time()
            user = self.auth.token_store._data.get('user', {})
            app = self.auth.token_store._data.get('app', {})

            user_refreshed = False
            app_refreshed = False

            # 刷新用户 Token：已过期或即将过期（10分钟内）
            if self.auth.token_store.get_refresh_token():
                user_expires = user.get('expires_at', 0)
                if user_expires - now < self.REFRESH_BEFORE:
                    logger.info("⏰ 刷新用户 Token（已过期或即将过期）...")
                    user_refreshed = self.auth.refresh_user_token()
                    if user_refreshed:
                        logger.info("✅ 用户 Token 刷新成功")
                    else:
                        logger.warning("⚠️ 用户 Token 刷新失败")

            # 刷新应用 Token：已过期或即将过期
            app_expires = app.get('expires_at', 0)
            if app_expires - now < self.REFRESH_BEFORE:
                logger.info("⏰ 刷新应用 Token（已过期或即将过期）...")
                app_refreshed = self.auth.refresh_app_token()
                if app_refreshed:
                    logger.info("✅ 应用 Token 刷新成功")
                else:
                    logger.warning("⚠️ 应用 Token 刷新失败")

            # 跟踪连续失败次数
            needs_user = (self.auth.token_store.get_refresh_token() and
                         user.get('expires_at', 0) - now < self.REFRESH_BEFORE)
            needs_app = app.get('expires_at', 0) - now < self.REFRESH_BEFORE

            if (needs_user and not user_refreshed) or (needs_app and not app_refreshed):
                self._consecutive_failures += 1
                if self._consecutive_failures >= self.MAX_RETRY:
                    logger.error(f"❌ Token 刷新连续失败 {self._consecutive_failures} 次，"
                               f"将在 {self.interval}秒后继续尝试")
                    self._consecutive_failures = 0  # 重置，但使用标准间隔
            else:
                self._consecutive_failures = 0

        except Exception as e:
            self._consecutive_failures += 1
            logger.error(f"❌ Token 刷新周期异常 (第{self._consecutive_failures}次): {e}", exc_info=True)

    def stop(self):
        self._stop_event.set()


class OAuthHandler(BaseHTTPRequestHandler):
    """HTTP 请求处理器"""

    auth: WPSAuth = None
    token_store: TokenStore = None

    def log_message(self, format, *args):
        logger.debug(f"[{self.client_address[0]}] {args[0]}")

    def _send_json(self, data: dict, status: int = 200):
        self.send_response(status)
        self.send_header('Content-Type', 'application/json; charset=utf-8')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type, Authorization')
        self.end_headers()
        self.wfile.write(json.dumps(data, ensure_ascii=False, indent=2).encode('utf-8'))

    def _send_html(self, html: str, status: int = 200):
        self.send_response(status)
        self.send_header('Content-Type', 'text/html; charset=utf-8')
        self.end_headers()
        self.wfile.write(html.encode('utf-8'))

    def do_OPTIONS(self):
        self.send_response(204)
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type, Authorization')
        self.end_headers()


    def _get_client_ip(self) -> str:
        xff = (self.headers.get('X-Forwarded-For') or '').split(',')[0].strip()
        return xff or (self.client_address[0] if self.client_address else '')

    def _is_loopback_client(self) -> bool:
        ip = self._get_client_ip()
        try:
            return ipaddress.ip_address(ip).is_loopback
        except Exception:
            return False

    def _authorized_admin_request(self) -> tuple:
        # Two-layer protection for sensitive endpoints:
        # 1) loopback source only 2) optional bearer token when configured
        if not self._is_loopback_client():
            return False, 'forbidden_remote'
        if ADMIN_TOKEN:
            auth = self.headers.get('Authorization', '')
            provided = auth[7:].strip() if auth.lower().startswith('bearer ') else ''
            if not provided or not hmac.compare_digest(provided, ADMIN_TOKEN):
                return False, 'unauthorized_token'
        return True, ''

    def _normalize_path(self, path: str) -> str:
        # 支持通过 /oauth 前缀统一接入，同时兼容无前缀本地调试。
        if BASE_PATH and path.startswith(BASE_PATH + '/'):
            normalized = path[len(BASE_PATH):]
            return normalized or '/'
        if BASE_PATH and path == BASE_PATH:
            return '/'
        return path

    def _with_base(self, path: str) -> str:
        if not BASE_PATH:
            return path
        if path == '/':
            return f"{BASE_PATH}/"
        return f"{BASE_PATH}{path}"

    def do_GET(self):
        parsed = urlparse(self.path)
        path = self._normalize_path(parsed.path)
        params = parse_qs(parsed.query)

        routes = {
            '/': self._handle_index,
            INTERNAL_CALLBACK_PATH: lambda: self._handle_callback(params),
            '/api/token/status': self._handle_token_status,
            '/api/dashboard': lambda: self._handle_dashboard_api(params),
            '/api/token/user': lambda: self._handle_get_user_token(params),
            '/api/token/app': self._handle_get_app_token,
            '/api/token/refresh': self._handle_refresh,
            '/api/auth/start': lambda: self._handle_start_auth(params),
            '/api/scope/check': lambda: self._handle_scope_check(params),
            '/dashboard': lambda: self._handle_dashboard(params),
            '/health': self._handle_health,
        }

        handler = routes.get(path)
        if handler:
            handler()
        else:
            self._send_json({'error': 'Not Found'}, 404)

    def do_POST(self):
        parsed = urlparse(self.path)
        path = self._normalize_path(parsed.path)

        content_length = int(self.headers.get('Content-Length', 0))
        raw = b''
        body = {}
        if content_length > 0:
            raw = self.rfile.read(content_length)
            try:
                body = json.loads(raw.decode('utf-8'))
            except:
                pass

        if path == '/oauth2/token':
            form = parse_qs(raw.decode('utf-8')) if raw else {}
            self._handle_oauth2_token(form)
        elif path == '/api/auth/start':
            self._handle_start_auth_post(body)
        elif path == '/api/token/refresh':
            self._handle_refresh()
        elif path == '/api/token/set':
            self._handle_set_token(body)
        elif path == '/api/token/require':
            self._handle_require_token(body)
        else:
            self._send_json({'error': 'Not Found'}, 404)

    def _handle_oauth2_token(self, form: dict):
        """
        兼容 OAuth 风格的 /oauth2/token 入口：
        - grant_type=client_credentials -> App Token（AK/SK）
        - grant_type=refresh_token -> User Token（refresh_token）
        """
        grant_type = (form.get('grant_type', [''])[0] or '').strip()
        refresh_token = (form.get('refresh_token', [''])[0] or '').strip()

        if grant_type == 'client_credentials':
            ok = self.auth.refresh_app_token()
            token = self.token_store.get_app_token() if ok else None
            if token:
                status = self.token_store.get_status()
                self._send_json({
                    'access_token': token,
                    'token_type': 'app',
                    'expires_in': max(0, int(status.get('app', {}).get('ttl_seconds', 7200))),
                    'scope': APP_SCOPE,
                })
                return
            self._send_json({'error': 'app_token_refresh_failed'}, 500)
            return

        if grant_type == 'refresh_token':
            # 优先使用调用方传入 refresh_token，避免依赖本地缓存中的旧值
            if refresh_token:
                data = {
                    'grant_type': 'refresh_token',
                    'client_id': self.auth.ak,
                    'client_secret': self.auth.sk,
                    'refresh_token': refresh_token,
                }
                try:
                    resp = requests.post(
                        OAUTH_TOKEN_URL,
                        data=urlencode(data),
                        headers={'Content-Type': 'application/x-www-form-urlencoded'},
                        timeout=30
                    )
                    result = resp.json()
                except Exception as e:
                    self._send_json({'error': 'refresh_failed', 'detail': str(e)}, 500)
                    return

                if 'access_token' in result:
                    self.token_store.set_user_token(
                        access_token=result['access_token'],
                        expires_in=result.get('expires_in', 7200),
                        refresh_token=result.get('refresh_token', refresh_token),
                        scope=None,  # 保留用户已有 scope
                    )
                    user_scope = str(self.token_store.get_user_scope()) or DEFAULT_SCOPE
                    self._send_json({
                        'access_token': result['access_token'],
                        'token_type': 'user',
                        'expires_in': result.get('expires_in', 7200),
                        'scope': user_scope,
                    })
                    return

                self._send_json({'error': 'invalid_refresh_token', 'detail': result}, 401)
                return

            # 未传 refresh_token 时，回退使用缓存中的 refresh_token
            if self.auth.refresh_user_token():
                token = self.token_store.get_user_token()
                status = self.token_store.get_status()
                self._send_json({
                    'access_token': token,
                    'token_type': 'user',
                    'expires_in': max(0, int(status.get('user', {}).get('ttl_seconds', 7200))),
                    'scope': str(self.token_store.get_user_scope()) or DEFAULT_SCOPE,
                })
                return
            self._send_json({'error': 'refresh_failed_no_refresh_token'}, 401)
            return

        self._send_json({'error': 'unsupported_grant_type'}, 400)

    # ============ Skill 入口 API ============

    def _handle_require_token(self, body: dict):
        """
        🔑 Skill 统一入口：请求用户 Token 并指定所需 scope
        
        POST /api/token/require
        {
            "scope": "kso.file.read,kso.file.readwrite",
            "skill_name": "wps-app-files"  // 可选，用于日志
        }
        
        返回:
        - 如果当前 Token 已覆盖所需 scope → 直接返回 token
        - 如果需要更多 scope → 返回 auth_url，需用户重新授权
        """
        required_scope = body.get('scope', '')
        skill_name = body.get('skill_name', 'unknown')

        if not required_scope:
            # 不指定 scope，直接返回当前 token
            token = self.token_store.get_user_token()
            if token:
                self._send_json({
                    'success': True,
                    'token': token,
                    'scope': str(self.token_store.get_user_scope()),
                    'scope_covered': True
                })
            else:
                # 尝试刷新
                if self.auth.refresh_user_token():
                    token = self.token_store.get_user_token()
                    if token:
                        self._send_json({
                            'success': True,
                            'token': token,
                            'scope': str(self.token_store.get_user_scope()),
                            'scope_covered': True,
                            'refreshed': True
                        })
                        return
                self._send_json({
                    'success': False,
                    'error': 'Token 不可用，请先完成 OAuth 授权',
                    'auth_url': f'{CALLBACK_SCHEME}://{CALLBACK_DOMAIN}/'
                }, 401)
            return

        # 检查 scope 覆盖
        check = self.auth.check_scope_coverage(required_scope)

        if check['covered']:
            # scope 已覆盖，返回 token
            token = self.token_store.get_user_token()
            if token:
                logger.info(f"[{skill_name}] Token scope 已覆盖: {required_scope}")
                self._send_json({
                    'success': True,
                    'token': token,
                    'scope': check['current_scope'],
                    'scope_covered': True
                })
            else:
                # Token 过期但 scope 满足，尝试刷新
                if self.auth.refresh_user_token():
                    token = self.token_store.get_user_token()
                    if token:
                        self._send_json({
                            'success': True,
                            'token': token,
                            'scope': check['current_scope'],
                            'scope_covered': True,
                            'refreshed': True
                        })
                        return
                self._send_json({
                    'success': False,
                    'error': 'Token 已过期，刷新失败',
                    'need_reauth': True,
                    'auth_url': check.get('auth_url')
                }, 401)
        else:
            # scope 不足，需要重新授权
            logger.info(
                f"[{skill_name}] 需要扩展 scope: "
                f"当前={check['current_scope']}, "
                f"需要={check['required_scope']}, "
                f"缺少={check['missing_scopes']}"
            )
            self._send_json({
                'success': False,
                'scope_covered': False,
                'current_scope': check['current_scope'],
                'required_scope': check['required_scope'],
                'missing_scopes': check['missing_scopes'],
                'merged_scope': check['merged_scope'],
                'need_reauth': True,
                'auth_url': check['auth_url'],
                'message': f"当前 Token 缺少以下 scope: {', '.join(check['missing_scopes'])}。"
                           f"请访问 auth_url 重新授权以获取完整权限。"
            }, 403)

    def _handle_scope_check(self, params: dict):
        """
        GET /api/scope/check?scope=kso.file.read,kso.file.readwrite
        检查当前 Token 的 scope 是否覆盖所需的 scope
        """
        required_scope = params.get('scope', [''])[0]
        if not required_scope:
            self._send_json({'error': 'Missing scope parameter'}, 400)
            return

        check = self.auth.check_scope_coverage(required_scope)
        self._send_json(check)

    def _handle_get_user_token(self, params: dict):
        """
        GET /api/token/user[?scope=xxx]
        获取用户 Token，可选指定所需 scope
        """
        required_scope = params.get('scope', [''])[0]

        # 如果指定了 scope，先检查覆盖
        if required_scope:
            check = self.auth.check_scope_coverage(required_scope)
            if not check['covered']:
                self._send_json({
                    'success': False,
                    'error': f"当前 Token scope 不足，缺少: {', '.join(check['missing_scopes'])}",
                    'scope_covered': False,
                    'current_scope': check['current_scope'],
                    'missing_scopes': check['missing_scopes'],
                    'need_reauth': True,
                    'auth_url': check['auth_url'],
                    'merged_scope': check['merged_scope']
                }, 403)
                return

        token = self.token_store.get_user_token()
        if token:
            self._send_json({
                'success': True,
                'token': token,
                'scope': str(self.token_store.get_user_scope())
            })
        else:
            # 尝试刷新
            if self.auth.refresh_user_token():
                token = self.token_store.get_user_token()
                if token:
                    self._send_json({
                        'success': True,
                        'token': token,
                        'scope': str(self.token_store.get_user_scope()),
                        'refreshed': True
                    })
                    return
            self._send_json({
                'success': False,
                'error': 'Token 不可用，请重新授权'
            }, 401)

    def _handle_start_auth(self, params: dict):
        """GET /api/auth/start?scope=xxx"""
        scope = params.get('scope', [None])[0]
        merge_flag = str(params.get('merge', ['false'])[0]).strip().lower() in ('1', 'true', 'yes', 'on')
        # 默认按传入 scope 精确授权；仅在 merge=true 时合并旧 scope
        if scope and merge_flag:
            current = self.token_store.get_user_scope()
            required = ScopeSet(scope)
            merged = current.merge(required)
            scope = str(merged)
        elif scope:
            scope = normalize_scope_string(scope)

        url, state = self.auth.get_authorize_url(scope)
        self._send_json({
            'auth_url': url,
            'state': state,
            'scope': scope,
            'merge': merge_flag,
            'callback_url': CALLBACK_URL
        })

    def _handle_start_auth_post(self, body: dict):
        """POST /api/auth/start"""
        scope = body.get('scope')
        merge_flag = bool(body.get('merge', False))
        if scope and merge_flag:
            current = self.token_store.get_user_scope()
            required = ScopeSet(scope)
            merged = current.merge(required)
            scope = str(merged)
        elif scope:
            scope = normalize_scope_string(scope)

        url, state = self.auth.get_authorize_url(scope)
        self._send_json({
            'auth_url': url,
            'state': state,
            'scope': scope,
            'merge': merge_flag,
            'callback_url': CALLBACK_URL
        })

    # ============ 基础 API ============

    def _handle_index(self):
        """首页 - 状态和授权入口"""
        base_home = self._with_base('/')
        auth_start_api = self._with_base('/api/auth/start')
        refresh_api = self._with_base('/api/token/refresh')
        status = self.token_store.get_status()
        scope_missing = status['user'].get('scope_missing', False)
        if status['user']['valid'] and scope_missing:
            user_status = '⚠️ 异常（scope 为空）'
        else:
            user_status = '✅ 有效' if status['user']['valid'] else '❌ 无效/未授权'
        app_status = '✅ 有效' if status['app']['valid'] else '❌ 无效'
        scope_display = status['user'].get('scope', '') or '（未授权）'
        if scope_missing:
            scope_display = '（scope 丢失，请重新授权）'
        scope_list_html = ''.join(
            f'<span style="background:#252547;padding:2px 8px;border-radius:4px;margin:2px;font-size:12px;">{s}</span>'
            for s in status['user'].get('scope_list', [])
        ) or ('<span style="color:#ffb020;">scope 为空（需重新授权）</span>' if scope_missing else '<span style="color:#666;">无</span>')

        html = f'''<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>WPS OAuth Token Server</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #0f0f23;
            color: #e8e8e8;
            min-height: 100vh;
            display: flex;
            justify-content: center;
            align-items: center;
            padding: 20px;
        }}
        .container {{ max-width: 640px; width: 100%; }}
        h1 {{ font-size: 24px; margin-bottom: 8px; color: #fff; }}
        .subtitle {{ color: #888; margin-bottom: 30px; font-size: 14px; }}
        .card {{
            background: #1a1a2e;
            border: 1px solid #2a2a4a;
            border-radius: 12px;
            padding: 24px;
            margin-bottom: 16px;
        }}
        .card h3 {{ font-size: 16px; margin-bottom: 16px; color: #a0a0ff; }}
        .status-row {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 8px 0;
            border-bottom: 1px solid #252547;
        }}
        .status-row:last-child {{ border-bottom: none; }}
        .status-label {{ color: #888; font-size: 14px; }}
        .status-value {{ font-size: 14px; }}
        .scope-tags {{ display: flex; flex-wrap: wrap; gap: 4px; justify-content: flex-end; }}
        .btn {{
            display: inline-block;
            padding: 12px 24px;
            border: none;
            border-radius: 8px;
            font-size: 15px;
            cursor: pointer;
            text-decoration: none;
            transition: all 0.2s;
            margin: 4px;
        }}
        .btn-primary {{ background: #6366f1; color: #fff; }}
        .btn-primary:hover {{ background: #4f46e5; }}
        .btn-success {{ background: #22c55e; color: #fff; }}
        .btn-success:hover {{ background: #16a34a; }}
        .btn-secondary {{ background: #374151; color: #e8e8e8; }}
        .btn-secondary:hover {{ background: #4b5563; }}
        .actions {{ display: flex; gap: 8px; flex-wrap: wrap; margin-top: 16px; }}
        .scope-input {{
            width: 100%;
            padding: 10px 14px;
            background: #252547;
            border: 1px solid #3a3a5a;
            border-radius: 8px;
            color: #e8e8e8;
            font-size: 14px;
            margin-bottom: 12px;
        }}
        .scope-input:focus {{ outline: none; border-color: #6366f1; }}
        .hint {{ color: #666; font-size: 12px; margin-top: 8px; }}
        .api-list {{ margin-top: 16px; }}
        .api-item {{
            padding: 6px 0;
            font-size: 13px;
            font-family: monospace;
            color: #a0a0b0;
        }}
        .api-item code {{
            color: #ff6c37;
            background: #252547;
            padding: 2px 6px;
            border-radius: 4px;
        }}
        .api-item .new-badge {{
            background: #6366f1;
            color: white;
            padding: 1px 6px;
            border-radius: 4px;
            font-size: 10px;
            margin-left: 4px;
        }}
        #result {{
            margin-top: 12px;
            padding: 12px;
            background: #252547;
            border-radius: 8px;
            font-size: 13px;
            display: none;
            white-space: pre-wrap;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🔑 WPS OAuth Token Server</h1>
        <p class="subtitle">为 Skills 提供统一的用户授权和 Token 管理（Scope 感知）</p>

        <div class="card">
            <h3>📊 Token 状态</h3>
            <div class="status-row">
                <span class="status-label">用户 Token</span>
                <span class="status-value">{user_status}</span>
            </div>
            <div class="status-row">
                <span class="status-label">用户 Token 剩余</span>
                <span class="status-value">{status['user']['ttl_human']}</span>
            </div>
            <div class="status-row">
                <span class="status-label">Refresh Token</span>
                <span class="status-value">{'✅ 有' if status['user']['has_refresh_token'] else '❌ 无'}</span>
            </div>
            <div class="status-row">
                <span class="status-label">当前 Scope</span>
                <span class="status-value"><div class="scope-tags">{scope_list_html}</div></span>
            </div>
            <div class="status-row">
                <span class="status-label">应用 Token</span>
                <span class="status-value">{app_status}</span>
            </div>
            <div class="status-row">
                <span class="status-label">最后授权</span>
                <span class="status-value">{status.get('last_auth') or '-'}</span>
            </div>
            <div class="status-row">
                <span class="status-label">最后刷新</span>
                <span class="status-value">{status.get('last_refresh') or '-'}</span>
            </div>
        </div>

        <div class="card">
            <h3>🔐 用户授权</h3>
            <input type="text" class="scope-input" id="scope"
                   value="{scope_display if status['user']['valid'] and not scope_missing else DEFAULT_SCOPE}"
                   placeholder="输入 OAuth scope（多个用逗号分隔，如 kso.file.read,kso.calendar.read）">
            <div class="actions">
                <button class="btn btn-primary" onclick="startAuth()">🚀 开始 OAuth 授权</button>
                <button class="btn btn-success" onclick="refreshToken()">🔄 手动刷新 Token</button>
                <button class="btn btn-secondary" onclick="checkStatus()">📊 刷新状态</button>
            </div>
            <div id="result"></div>
            <p class="hint">回调地址: <code>{CALLBACK_URL}</code></p>
            <p class="hint">💡 新的 scope 会自动合并到已有 scope 中，扩展而非替换</p>
        </div>

        <div class="card">
            <h3>📡 API 接口</h3>
            <div class="api-list">
                <div class="api-item"><code>POST /api/token/require</code> Skill 统一入口（指定 scope 获取 token）<span class="new-badge">推荐</span></div>
                <div class="api-item"><code>GET /api/scope/check?scope=xxx</code> 检查 scope 是否覆盖 <span class="new-badge">NEW</span></div>
                <div class="api-item"><code>GET /dashboard</code> 平台可观测面板（HTML） <span class="new-badge">NEW</span></div>
                <div class="api-item"><code>GET /api/dashboard?hours=24</code> 平台可观测快照（JSON） <span class="new-badge">NEW</span></div>
                <div class="api-item"><code>GET /api/token/status</code> 查询 Token 状态（含 scope）</div>
                <div class="api-item"><code>GET /api/token/user[?scope=xxx]</code> 获取用户 Token</div>
                <div class="api-item"><code>GET /api/token/app</code> 获取应用 Token</div>
                <div class="api-item"><code>GET /api/token/refresh</code> 手动刷新 Token</div>
                <div class="api-item"><code>GET /api/auth/start?scope=xxx</code> 开始 OAuth 授权</div>
                <div class="api-item"><code>GET /health</code> 健康检查</div>
            </div>
        </div>
    </div>

    <script>
        function startAuth() {{
            const scope = document.getElementById('scope').value.trim();
            fetch('{auth_start_api}?scope=' + encodeURIComponent(scope))
                .then(r => r.json())
                .then(data => {{
                    if (data.auth_url) {{
                        window.open(data.auth_url, '_blank', 'width=600,height=700');
                        showResult('⏳ 已打开授权页面（scope: ' + (data.scope || scope) + '）\\n请在浏览器中完成授权...');
                    }} else {{
                        showResult('❌ ' + (data.error || '未知错误'));
                    }}
                }})
                .catch(e => showResult('❌ 请求失败: ' + e.message));
        }}

        function refreshToken() {{
            fetch('{refresh_api}')
                .then(r => r.json())
                .then(data => {{
                    showResult(data.success ? '✅ Token 刷新成功！' : '❌ 刷新失败: ' + JSON.stringify(data));
                    setTimeout(() => location.reload(), 1500);
                }})
                .catch(e => showResult('❌ 请求失败: ' + e.message));
        }}

        function checkStatus() {{ location.reload(); }}

        function showResult(msg) {{
            const el = document.getElementById('result');
            el.style.display = 'block';
            el.textContent = msg;
        }}
    </script>
</body>
</html>'''
        self._send_html(html)

    def _handle_callback(self, params: dict):
        """处理 OAuth 回调"""
        base_home = self._with_base('/')
        code = params.get('code', [None])[0]
        state = params.get('state', [None])[0]
        error = params.get('error', [None])[0]
        logger.info(
            "收到 OAuth 回调: "
            f"has_code={bool(code)}, has_state={bool(state)}, has_error={bool(error)}"
        )

        if error:
            self._send_html(f'''<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>授权失败</title>
<style>body{{font-family:sans-serif;padding:40px;background:#0f0f23;color:#e8e8e8;text-align:center;}}
h2{{color:#ef4444;}}.msg{{background:#1a1a2e;padding:20px;border-radius:8px;margin:20px auto;max-width:400px;}}</style></head>
<body><h2>❌ 授权失败</h2><div class="msg">{error}: {params.get('error_description', [''])[0]}</div>
<p><a href="{base_home}" style="color:#6366f1;">返回首页</a></p></body></html>''', 400)
            return

        if not code:
            self._send_html('<h2>Missing authorization code</h2>', 400)
            return

        result = self.auth.exchange_code(code, state)
        scope = result.get('scope', '')

        if result.get('success'):
            self._send_html(f'''<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>授权成功</title>
<style>body{{font-family:sans-serif;padding:40px;background:#0f0f23;color:#e8e8e8;text-align:center;}}
h2{{color:#22c55e;}}.msg{{background:#1a1a2e;padding:20px;border-radius:8px;margin:20px auto;max-width:500px;}}
.scope-tag{{background:#252547;padding:2px 8px;border-radius:4px;margin:2px;font-size:12px;display:inline-block;}}</style></head>
<body>
<h2>✅ 授权成功！</h2>
<div class="msg">
    <p>Token 有效期: {result['expires_in']}秒</p>
    <p>自动刷新: 已启用（每 {REFRESH_INTERVAL // 60} 分钟）</p>
    <p>授权 Scope:</p>
    <p>{''.join(f'<span class="scope-tag">{s}</span>' for s in ScopeSet(scope).as_list) or '默认'}</p>
</div>
<p style="color:#888;margin-top:20px;">此窗口可以关闭了</p>
<script>
    if (window.opener) window.opener.postMessage({{type:'oauth_success',scope:'{scope}'}}, '*');
    setTimeout(() => window.close(), 3000);
</script>
</body></html>''')
        else:
            error_msg = json.dumps(result.get('error', {}), ensure_ascii=False, indent=2)
            self._send_html(f'''<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>授权失败</title>
<style>body{{font-family:sans-serif;padding:40px;background:#0f0f23;color:#e8e8e8;text-align:center;}}
h2{{color:#ef4444;}}pre{{background:#1a1a2e;padding:20px;border-radius:8px;text-align:left;margin:20px auto;max-width:500px;overflow:auto;}}</style></head>
<body>
<h2>❌ 授权码换取 Token 失败</h2>
<pre>{error_msg}</pre>
<p>回调地址: <code>{CALLBACK_URL}</code></p>
<p><a href="{base_home}" style="color:#6366f1;">返回首页重试</a></p>
</body></html>''', 400)

    def _handle_token_status(self):
        self._send_json(self.token_store.get_status())

    def _handle_dashboard_api(self, params: dict):
        lookback = params.get('hours', ['24'])[0]
        try:
            lookback_hours = max(1, min(168, int(lookback)))
        except Exception:
            lookback_hours = 24
        token_status = self.token_store.get_status()
        snapshot = build_platform_health_snapshot(
            config=PLATFORM_CONFIG,
            token_status=token_status,
            lookback_hours=lookback_hours,
        )
        http_status = 200 if snapshot.get('overall') != 'unhealthy' else 503
        self._send_json(snapshot, http_status)

    def _handle_dashboard(self, params: dict):
        dashboard_api = self._with_base('/api/dashboard')
        health_url = self._with_base('/health')
        home_url = self._with_base('/')
        lookback = params.get('hours', ['24'])[0]
        try:
            lookback_hours = max(1, min(168, int(lookback)))
        except Exception:
            lookback_hours = 24

        token_status = self.token_store.get_status()
        snapshot = build_platform_health_snapshot(
            config=PLATFORM_CONFIG,
            token_status=token_status,
            lookback_hours=lookback_hours,
        )
        overall = snapshot.get('overall', 'unknown')
        color = '#22c55e' if overall == 'healthy' else ('#eab308' if overall == 'degraded' else '#ef4444')
        icon = '🟢' if overall == 'healthy' else ('🟡' if overall == 'degraded' else '🔴')
        api = snapshot.get('api', {})
        token = snapshot.get('token', {})
        top_paths = api.get('top_paths', [])
        top_errors = api.get('top_error_types', [])
        latest_events = snapshot.get('skill_events', {}).get('latest', [])

        top_paths_html = ''.join(
            f"<div class='row'><span>{i['path']}</span><strong>{i['count']}</strong></div>" for i in top_paths
        ) or "<div class='empty'>无失败接口</div>"
        top_errors_html = ''.join(
            f"<div class='row'><span>{i['error_type']}</span><strong>{i['count']}</strong></div>" for i in top_errors
        ) or "<div class='empty'>无错误类型</div>"
        latest_events_html = ''.join(
            f"<div class='event'><span>{e.get('time','')}</span><code>{e.get('skill','unknown')}</code><em>{e.get('action','')}</em></div>"
            for e in latest_events[-10:]
        ) or "<div class='empty'>暂无技能事件</div>"

        html = f'''<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>WPS Platform Dashboard</title>
  <meta http-equiv="refresh" content="30">
  <style>
    * {{ box-sizing: border-box; }}
    body {{ margin: 0; padding: 24px; background:#0f0f23; color:#e8e8e8; font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif; }}
    .container {{ max-width: 980px; margin: 0 auto; }}
    .title {{ display:flex; justify-content:space-between; align-items:center; margin-bottom: 18px; }}
    .badge {{ border:2px solid {color}; color:{color}; border-radius:999px; padding:6px 16px; font-weight:600; }}
    .grid {{ display:grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 12px; }}
    .card {{ background:#1a1a2e; border:1px solid #2a2a4a; border-radius:12px; padding:16px; }}
    .card h3 {{ margin:0 0 12px 0; color:#a0a0ff; font-size:15px; }}
    .metric {{ display:flex; justify-content:space-between; padding:8px 0; border-bottom:1px solid #252547; font-size:14px; }}
    .metric:last-child {{ border-bottom:none; }}
    .row {{ display:flex; justify-content:space-between; padding:6px 0; border-bottom:1px dashed #252547; gap: 10px; }}
    .row:last-child {{ border-bottom:none; }}
    .row span {{ overflow:hidden; text-overflow:ellipsis; white-space:nowrap; max-width:220px; color:#b5b5c8; }}
    .row strong {{ color:#fff; }}
    .event {{ display:flex; gap:8px; font-size:12px; padding:6px 0; border-bottom:1px dashed #252547; }}
    .event:last-child {{ border-bottom:none; }}
    .event span {{ color:#8e8ea8; min-width:130px; }}
    .event code {{ color:#ffb86c; background:#252547; padding:1px 4px; border-radius:4px; }}
    .event em {{ color:#cbd5e1; font-style:normal; }}
    .empty {{ color:#666; font-size:13px; padding:8px 0; }}
    .links {{ margin-top: 12px; color:#666; font-size:12px; }}
    .links a {{ color:#6366f1; text-decoration:none; }}
  </style>
</head>
<body>
  <div class="container">
    <div class="title">
      <h1>📊 WPS Platform Dashboard</h1>
      <div class="badge">{icon} {overall.upper()}</div>
    </div>
    <div class="grid">
      <div class="card">
        <h3>Token 健康</h3>
        <div class="metric"><span>用户 Token</span><strong>{'✅' if token.get('user_ok') else '❌'}</strong></div>
        <div class="metric"><span>应用 Token</span><strong>{'✅' if token.get('app_ok') else '❌'}</strong></div>
        <div class="metric"><span>Refresh Token</span><strong>{'✅' if token.get('has_refresh_token') else '❌'}</strong></div>
        <div class="metric"><span>User TTL</span><strong>{token.get('user_ttl') or '-'}</strong></div>
        <div class="metric"><span>App TTL</span><strong>{token.get('app_ttl') or '-'}</strong></div>
        <div class="metric"><span>Scope 数</span><strong>{token.get('scope_count', 0)}</strong></div>
      </div>
      <div class="card">
        <h3>API 失败概览（{lookback_hours}h）</h3>
        <div class="metric"><span>请求总数</span><strong>{api.get('total', 0)}</strong></div>
        <div class="metric"><span>失败数</span><strong>{api.get('failed', 0)}</strong></div>
        <div class="metric"><span>失败率</span><strong>{api.get('failure_rate', 0)}%</strong></div>
      </div>
      <div class="card">
        <h3>失败接口 Top</h3>
        {top_paths_html}
      </div>
      <div class="card">
        <h3>错误类型 Top</h3>
        {top_errors_html}
      </div>
      <div class="card" style="grid-column: 1 / -1;">
        <h3>最近技能事件</h3>
        {latest_events_html}
      </div>
    </div>
    <div class="links">
      <a href="{dashboard_api}?hours={lookback_hours}">API JSON</a> ·
      <a href="{health_url}">Health</a> ·
      <a href="{home_url}">OAuth 首页</a>
    </div>
  </div>
</body>
</html>'''
        self._send_html(html)

    def _handle_get_app_token(self):
        params = parse_qs(urlparse(self.path).query)
        force = params.get('force', ['false'])[0].lower() in ('true', '1')

        if force or not self.token_store.get_app_token():
            self.auth.refresh_app_token()

        token = self.token_store.get_app_token()

        if token:
            self._send_json({'success': True, 'token': token})
        else:
            self._send_json({'success': False, 'error': '应用 Token 获取失败'}, 500)

    def _handle_refresh(self):
        params = parse_qs(urlparse(self.path).query)
        refresh_type = params.get('type', [''])[0].lower()

        user_ok = False
        app_ok = False

        if refresh_type == 'user':
            user_ok = self.auth.refresh_user_token()
            app_ok = True  # 不刷新 app，视为成功
        elif refresh_type == 'app':
            app_ok = self.auth.refresh_app_token()
            user_ok = True  # 不刷新 user，视为成功
        else:
            # 默认行为：都刷新
            user_ok = self.auth.refresh_user_token()
            app_ok = self.auth.refresh_app_token()

        self._send_json({
            'success': user_ok or app_ok,
            'user_refreshed': user_ok,
            'app_refreshed': app_ok
        })

    def _handle_set_token(self, body: dict):
        ok, reason = self._authorized_admin_request()
        if not ok:
            code = 403 if reason == 'forbidden_remote' else 401
            self._send_json({'error': reason}, code)
            return

        access_token = (body.get('access_token') or '').strip()
        refresh_token = body.get('refresh_token') or None
        expires_in = body.get('expires_in', 7200)
        scope = body.get('scope')

        if not access_token:
            self._send_json({'error': 'Missing access_token'}, 400)
            return

        try:
            expires_in = int(expires_in)
            if expires_in <= 0:
                raise ValueError('expires_in must be positive')
        except Exception:
            self._send_json({'error': 'Invalid expires_in'}, 400)
            return

        # Empty scope should not erase existing scope.
        if isinstance(scope, str):
            scope = normalize_scope_string(scope)
            scope = scope if scope else None

        self.token_store.set_user_token(access_token, expires_in, refresh_token, scope)
        self._send_json({'success': True})


    def _handle_health(self):
        """
        GET /health — 健康检查
        浏览器访问 → 可视化 HTML 页面
        API 调用（Accept: application/json）→ JSON 响应
        """
        home_url = self._with_base('/')
        token_status_url = self._with_base('/api/token/status')
        health_url = self._with_base('/health')
        accept = self.headers.get('Accept', '')
        status = self.token_store.get_status()

        now = datetime.now()
        user_ok = status['user']['valid']
        app_ok = status['app']['valid']
        has_refresh = status['user']['has_refresh_token']
        overall = 'healthy' if (user_ok and app_ok and has_refresh) else (
            'degraded' if (user_ok or app_ok) else 'unhealthy'
        )

        # API 调用返回 JSON
        if 'application/json' in accept and 'text/html' not in accept:
            self._send_json({
                'status': overall,
                'time': now.isoformat(),
                'checks': {
                    'user_token': {'ok': user_ok, 'ttl': status['user']['ttl_human']},
                    'app_token': {'ok': app_ok, 'ttl': status['app']['ttl_human']},
                    'refresh_token': {'ok': has_refresh},
                },
                'scope_count': len(status['user'].get('scope_list', [])),
                'last_refresh': status.get('last_refresh'),
                'last_auth': status.get('last_auth'),
            }, 200 if overall != 'unhealthy' else 503)
            return

        # 浏览器访问返回 HTML
        def _check_icon(ok):
            return '✅' if ok else '❌'

        overall_color = '#22c55e' if overall == 'healthy' else (
            '#eab308' if overall == 'degraded' else '#ef4444'
        )
        overall_icon = '🟢' if overall == 'healthy' else (
            '🟡' if overall == 'degraded' else '🔴'
        )

        scope_count = len(status['user'].get('scope_list', []))

        html = f'''<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>WPS OAuth - Health Check</title>
    <meta http-equiv="refresh" content="30">
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #0f0f23;
            color: #e8e8e8;
            min-height: 100vh;
            display: flex;
            justify-content: center;
            align-items: center;
            padding: 20px;
        }}
        .container {{ max-width: 560px; width: 100%; }}
        .header {{
            text-align: center;
            margin-bottom: 28px;
        }}
        .header h1 {{
            font-size: 22px;
            color: #fff;
            margin-bottom: 6px;
        }}
        .overall {{
            display: inline-block;
            padding: 6px 20px;
            border-radius: 20px;
            font-size: 16px;
            font-weight: 600;
            color: {overall_color};
            border: 2px solid {overall_color};
            margin-top: 8px;
        }}
        .card {{
            background: #1a1a2e;
            border: 1px solid #2a2a4a;
            border-radius: 12px;
            padding: 20px;
            margin-bottom: 14px;
        }}
        .card h3 {{
            font-size: 15px;
            margin-bottom: 14px;
            color: #a0a0ff;
        }}
        .check-row {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 10px 0;
            border-bottom: 1px solid #252547;
        }}
        .check-row:last-child {{ border-bottom: none; }}
        .check-label {{ color: #b0b0c0; font-size: 14px; }}
        .check-value {{
            font-size: 14px;
            font-weight: 500;
        }}
        .check-ok {{ color: #22c55e; }}
        .check-fail {{ color: #ef4444; }}
        .check-warn {{ color: #eab308; }}
        .meta {{
            text-align: center;
            color: #555;
            font-size: 12px;
            margin-top: 14px;
        }}
        .meta a {{ color: #6366f1; text-decoration: none; }}
        .meta a:hover {{ text-decoration: underline; }}
        .btn-row {{
            display: flex;
            gap: 8px;
            justify-content: center;
            margin-top: 16px;
        }}
        .btn {{
            padding: 8px 18px;
            border: none;
            border-radius: 8px;
            font-size: 13px;
            cursor: pointer;
            text-decoration: none;
            transition: all 0.2s;
        }}
        .btn-refresh {{ background: #374151; color: #e8e8e8; }}
        .btn-refresh:hover {{ background: #4b5563; }}
        .btn-home {{ background: #6366f1; color: #fff; }}
        .btn-home:hover {{ background: #4f46e5; }}
        .scope-tag {{
            display: inline-block;
            background: #252547;
            padding: 2px 7px;
            border-radius: 4px;
            font-size: 11px;
            margin: 2px;
            color: #a0a0c0;
        }}
        .scope-section {{
            max-height: 120px;
            overflow-y: auto;
            padding: 8px 0;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🔑 WPS OAuth Health Check</h1>
            <div class="overall">{overall_icon} {overall.upper()}</div>
        </div>

        <div class="card">
            <h3>🏥 服务检查项</h3>
            <div class="check-row">
                <span class="check-label">用户 Token</span>
                <span class="check-value {'check-ok' if user_ok else 'check-fail'}">{_check_icon(user_ok)} {'有效' if user_ok else '无效/过期'}</span>
            </div>
            <div class="check-row">
                <span class="check-label">Token 剩余时间</span>
                <span class="check-value {'check-ok' if status['user']['ttl_seconds'] > 600 else ('check-warn' if user_ok else 'check-fail')}">{status['user']['ttl_human']}</span>
            </div>
            <div class="check-row">
                <span class="check-label">Refresh Token</span>
                <span class="check-value {'check-ok' if has_refresh else 'check-fail'}">{_check_icon(has_refresh)} {'可用' if has_refresh else '缺失'}</span>
            </div>
            <div class="check-row">
                <span class="check-label">应用 Token</span>
                <span class="check-value {'check-ok' if app_ok else 'check-fail'}">{_check_icon(app_ok)} {'有效' if app_ok else '无效/过期'}</span>
            </div>
            <div class="check-row">
                <span class="check-label">OAuth Scope</span>
                <span class="check-value {'check-ok' if scope_count > 0 else 'check-fail'}">{scope_count} 个</span>
            </div>
        </div>

        <div class="card">
            <h3>⏱ 时间信息</h3>
            <div class="check-row">
                <span class="check-label">当前时间</span>
                <span class="check-value">{now.strftime('%Y-%m-%d %H:%M:%S')}</span>
            </div>
            <div class="check-row">
                <span class="check-label">最后授权</span>
                <span class="check-value">{status.get('last_auth', '-') or '-'}</span>
            </div>
            <div class="check-row">
                <span class="check-label">最后刷新</span>
                <span class="check-value">{status.get('last_refresh', '-') or '-'}</span>
            </div>
        </div>

        <div class="card">
            <h3>🔐 当前 Scope</h3>
            <div class="scope-section">
                {''.join(f'<span class="scope-tag">{s}</span>' for s in status['user'].get('scope_list', [])) or '<span style="color:#666;">未授权</span>'}
            </div>
        </div>

        <div class="btn-row">
            <button class="btn btn-refresh" onclick="location.reload()">🔄 刷新</button>
            <a href="{home_url}" class="btn btn-home">🏠 管理首页</a>
        </div>

        <p class="meta">
            自动刷新间隔: 30秒 · 端口: {DEFAULT_PORT} ·
            <a href="{token_status_url}">API Status</a> ·
            <a href="{health_url}">JSON Health</a>
        </p>
    </div>
</body>
</html>'''
        self._send_html(html)


def run_server(port: int = DEFAULT_PORT, no_browser: bool = False):
    """启动 OAuth 服务器"""
    token_store = TokenStore()
    auth = WPSAuth(token_store)

    OAuthHandler.auth = auth
    OAuthHandler.token_store = token_store

    # 首次获取应用 Token
    logger.info("🔑 获取应用 Token...")
    if auth.refresh_app_token():
        logger.info("✅ 应用 Token 获取成功")
    else:
        logger.warning("⚠️ 应用 Token 获取失败，稍后自动重试")

    # 检查并刷新用户 Token（启动时立即处理，带重试）
    status = token_store.get_status()
    if status['user']['valid']:
        ttl = status['user']['ttl_seconds']
        if status['user'].get('scope_missing'):
            logger.warning(
                "⚠️ 用户 Token 有效但 scope 为空（缓存异常），建议立即通过 /api/auth/start 重新授权。"
                f" ttl={status['user']['ttl_human']}"
            )
        else:
            logger.info(f"✅ 用户 Token 有效，剩余: {status['user']['ttl_human']}，scope: {status['user']['scope']}")
        # 如果剩余时间不足 10 分钟，立即刷新
        if ttl < 600 and token_store.get_refresh_token():
            logger.info("⏰ 用户 Token 即将过期，主动刷新...")
            if auth.refresh_user_token():
                logger.info("✅ 用户 Token 刷新成功")
            else:
                logger.warning("⚠️ 用户 Token 刷新失败")
    elif token_store.get_refresh_token():
        logger.info("🔄 用户 Token 已过期，尝试刷新...")
        max_retries = 3
        for attempt in range(1, max_retries + 1):
            if auth.refresh_user_token():
                logger.info(f"✅ 用户 Token 刷新成功（第 {attempt} 次尝试）")
                break
            else:
                logger.warning(f"⚠️ 用户 Token 刷新失败（第 {attempt}/{max_retries} 次）")
                if attempt < max_retries:
                    time.sleep(3)  # 短暂等待后重试
        else:
            logger.error("❌ 启动时用户 Token 刷新全部失败，将由自动刷新线程继续重试")
    else:
        logger.info("⚠️  用户 Token 未设置，请通过浏览器完成 OAuth 授权")

    # 启动自动刷新线程（在初始刷新之后启动，避免竞争）
    refresher = TokenRefresher(auth)
    refresher.start()

    # 启动 HTTP 服务器
    server = HTTPServer(('0.0.0.0', port), OAuthHandler)

    def signal_handler(sig, frame):
        logger.info("\n🛑 正在关闭服务器...")
        refresher.stop()
        server.shutdown()
        sys.exit(0)

    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)

    logger.info(f"\n{'='*50}")
    logger.info(f"🚀 OAuth Token Server 已启动")
    logger.info(f"   监听端口: {port}")
    logger.info(f"   回调地址: {CALLBACK_URL}")
    logger.info(f"   Token 缓存: {TOKEN_CACHE_PATH}")
    logger.info(f"   认证配置: {AUTH_CONFIG}")
    logger.info(f"   自动刷新: 每 {REFRESH_INTERVAL // 60} 分钟")
    logger.info(f"{'='*50}\n")

    if not no_browser:
        webbrowser.open(f'http://localhost:{port}')

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        refresher.stop()
        server.server_close()
        logger.info("👋 服务器已关闭")


if __name__ == '__main__':
    import argparse

    parser = argparse.ArgumentParser(description='WPS OAuth Token Server')
    parser.add_argument('--port', '-p', type=int, default=DEFAULT_PORT,
                        help=f'服务器端口 (默认: {DEFAULT_PORT})')
    parser.add_argument('--no-browser', action='store_true',
                        help='不自动打开浏览器')
    parser.add_argument('--status', action='store_true',
                        help='仅查看 Token 状态')
    parser.add_argument('--refresh', action='store_true',
                        help='仅刷新 Token（不启动服务器）')

    args = parser.parse_args()

    if args.status:
        store = TokenStore()
        status = store.get_status()
        print("\n📊 Token 状态:")
        print(f"   用户 Token: {'✅ 有效' if status['user']['valid'] else '❌ 无效'}")
        print(f"   剩余时间: {status['user']['ttl_human']}")
        print(f"   Scope: {status['user']['scope'] or '（无）'}")
        print(f"   Scope 列表: {status['user']['scope_list']}")
        print(f"   Refresh Token: {'✅ 有' if status['user']['has_refresh_token'] else '❌ 无'}")
        print(f"   应用 Token: {'✅ 有效' if status['app']['valid'] else '❌ 无效'}")
        print(f"   最后授权: {status.get('last_auth') or '-'}")
        print(f"   最后刷新: {status.get('last_refresh') or '-'}")
    elif args.refresh:
        store = TokenStore()
        auth = WPSAuth(store)
        print("🔄 刷新 Token...")
        user_ok = auth.refresh_user_token()
        app_ok = auth.refresh_app_token()
        print(f"   用户 Token: {'✅ 成功' if user_ok else '❌ 失败'}")
        print(f"   应用 Token: {'✅ 成功' if app_ok else '❌ 失败'}")
        # 刷新后显示状态
        if user_ok or app_ok:
            status = store.get_status()
            if user_ok:
                print(f"   用户 Token 有效期: {status['user']['ttl_human']}")
            if app_ok:
                print(f"   应用 Token 有效期: {status['app']['ttl_human']}")
    else:
        run_server(port=args.port, no_browser=args.no_browser)
