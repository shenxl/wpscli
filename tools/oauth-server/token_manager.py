#!/usr/bin/env python3
"""
WPS Token Manager - 供所有 Skills 使用的通用 Token 获取模块

使用方式：
    # 基础：获取 Token
    from token_manager import get_user_token, get_app_token

    token = get_user_token()                                       # 不指定 scope
    token = get_user_token(scope='kso.file.read,kso.file.readwrite') # 指定 scope

    # 推荐：使用 WPS 客户端
    from token_manager import get_wps_client

    client = get_wps_client(auth_type='user', scope='kso.contact.read')
    result = client.get('/v7/contacts/departments/children', params={'page_size': 50})

    # Scope 感知：请求 Token 并指定所需 scope（逗号分隔）
    from token_manager import require_token

    result = require_token(scope='kso.file.read,kso.file.readwrite',
                          skill_name='wps-app-files')
    if result['success']:
        token = result['token']
    else:
        print(f"需要重新授权: {result['auth_url']}")

Token 来源优先级：
    1. OAuth Server HTTP API（推荐，支持 scope 检查）
    2. OAuth Server 的 data/token_cache.json（本地文件）
    3. 环境变量 WPS_USER_TOKEN / WPS_APP_TOKEN
    4. 使用 refresh_token 自动刷新
"""

import os
import sys
import json
import time
import uuid
import hmac
import hashlib
import base64
import urllib.parse
from datetime import datetime
from pathlib import Path

import requests
from wps_platform import PlatformConfig, write_skill_event

# ============ 路径配置（无硬编码绝对路径）============
OAUTH_SKILL_DIR = Path(__file__).parent
OAUTH_TOKEN_CACHE = OAUTH_SKILL_DIR / 'data' / 'token_cache.json'

# auth.conf 查找（同 oauth_server.py 的策略）
def _find_auth_conf() -> Path:
    """查找 auth.conf 配置文件"""
    env_path = os.environ.get('WPS_AUTH_CONF')
    if env_path:
        p = Path(env_path)
        if p.exists():
            return p

    for candidate in [
        OAUTH_SKILL_DIR / 'auth.conf',
        OAUTH_SKILL_DIR / 'data' / 'auth.conf',
        OAUTH_SKILL_DIR.parent.parent / 'auth.conf',  # openapi/auth.conf
        OAUTH_SKILL_DIR.parent / 'auth.conf',
    ]:
        if candidate.exists():
            return candidate
    return None

AUTH_CONFIG = _find_auth_conf()
API_BASE = 'https://openapi.wps.cn'

# OAuth Server 地址（本地访问用 127.0.0.1，外网访问用 wps-mac.shenxl.com）
# 默认优先走 nginx 的 /oauth 前缀，再回退直连 8089。
_ENV_OAUTH_SERVER = os.environ.get('WPS_OAUTH_SERVER', '').strip()
_DEFAULT_OAUTH_CANDIDATES = [
    'http://127.0.0.1:80/oauth',
    'http://127.0.0.1:8089',
    'http://127.0.0.1:80',
]
if _ENV_OAUTH_SERVER:
    OAUTH_SERVER_CANDIDATES = [_ENV_OAUTH_SERVER]
else:
    OAUTH_SERVER_CANDIDATES = _DEFAULT_OAUTH_CANDIDATES
OAUTH_SERVER_URL = OAUTH_SERVER_CANDIDATES[0]

# 公网地址（用于生成用户可访问的授权 URL）
OAUTH_PUBLIC_URL = os.environ.get('WPS_OAUTH_PUBLIC_URL', 'https://wps-mac.shenxl.com/oauth')
PLATFORM_CONFIG = PlatformConfig.from_env(OAUTH_SKILL_DIR)

# WPS OpenAPI 请求稳定性参数
DEFAULT_API_TIMEOUT = float(os.environ.get('WPS_API_TIMEOUT_SECONDS', '30'))
DEFAULT_API_MAX_RETRIES = int(os.environ.get('WPS_API_MAX_RETRIES', '2'))
DEFAULT_API_RETRY_BACKOFF = float(os.environ.get('WPS_API_RETRY_BACKOFF_SECONDS', '0.8'))
DEFAULT_API_LOG_ENABLED = os.environ.get('WPS_API_LOG_ENABLED', 'true').strip().lower() in ('1', 'true', 'yes', 'on')
DEFAULT_API_LOG_PATH = Path(os.environ.get('WPS_API_LOG_PATH', str(OAUTH_SKILL_DIR / 'data' / 'wps_api_events.jsonl')))

# ============ WPS 有效 Scope 白名单 ============
VALID_WPS_SCOPES = {
    'kso.user_base.read', 'kso.user_current_id.read',
    'kso.agent.readwrite',
    'kso.file.read', 'kso.file.readwrite', 'kso.file_search.readwrite', 'kso.file_link.readwrite',
    'kso.file_version.readwrite',
    'kso.file_permission.readwrite',
    'kso.deleted_file.read', 'kso.deleted_file.readwrite',
    'kso.documents.readwrite', 'kso.doclib.read', 'kso.doclib.readwrite',
    'kso.drive.readwrite', 'kso.drive_role.readwrite', 'kso.coop_files.readwrite',
    'kso.wiki.readwrite',
    'kso.dbsheet.read', 'kso.dbsheet.readwrite',
    'kso.sheets.read', 'kso.sheets.readwrite',
    'kso.airsheet.read', 'kso.airsheet.readwrite',
    'kso.calendar.read', 'kso.calendar.readwrite', 'kso.calendar_events.read', 'kso.calendar_events.readwrite',
    'kso.task.read', 'kso.task.readwrite',
    'kso.contact.read',
    'kso.meeting.read', 'kso.meeting.readwrite', 'kso.meeting_minutes.read', 'kso.meeting_minutes_content.read',
    'kso.meeting_recording.read', 'kso.meeting_recording.readwrite', 'kso.meeting_recording_content.read',
    'kso.chat.read', 'kso.chat.readwrite', 'kso.chat_message.readwrite', 'kso.chat_bookmark.readwrite',
    'kso.group.read', 'kso.group.readwrite', 'kso.component.agentspace_chat',
    'kso.airpage.read', 'kso.airpage.readwrite',
    'kso.app.read', 'kso.app.readwrite', 'kso.apps.read', 'kso.apps.readwrite',
    'kso.automation.read', 'kso.automation.readwrite',
    'kso.aidocs.readwrite', 'kso.aidocs_extract.readwrite', 'kso.docqa.readwrite',
    'kso.devhub_app.readwrite', 'kso.devhub_chat.readwrite', 'kso.devhub_session.readwrite',
    'kso.workflow_approval_define.read', 'kso.workflow_approval_define.readwrite',
    'kso.workflow_approval_instance.read', 'kso.workflow_approval_instance.readwrite',
    'kso.workflow_approval_task.readwrite',
}


def filter_valid_scopes(scope_str: str) -> str:
    """过滤掉不在白名单中的无效 scope"""
    if not scope_str:
        return scope_str
    scopes = [s.strip() for s in scope_str.replace(',', ' ').split() if s.strip()]
    return ','.join(s for s in scopes if s in VALID_WPS_SCOPES)


def classify_wps_error(payload: dict) -> dict:
    """
    统一归类 WPS API 错误，返回结构化诊断信息。

    返回字段:
      - error_type: auth|scope|rate_limit|timeout|network|not_found|conflict|validation|server|upstream|unknown
      - retryable: 是否建议自动重试
      - hint: 给调用方/用户的简短修复建议
    """
    if not isinstance(payload, dict):
        return {
            'error_type': 'unknown',
            'retryable': False,
            'hint': '响应结构异常，请检查日志。',
        }

    msg = (payload.get('msg') or '').lower()
    http_status = int(payload.get('http_status', 0) or 0)
    code = payload.get('code')

    if http_status == 429 or 'rate limit' in msg or 'too many request' in msg or '限流' in msg:
        return {'error_type': 'rate_limit', 'retryable': True, 'hint': '触发限流，请稍后重试或降低频率。'}
    if http_status in (401,) or 'unauthorized' in msg or 'invalid token' in msg or 'token' in msg and 'expired' in msg:
        return {'error_type': 'auth', 'retryable': False, 'hint': 'Token 失效，请刷新或重新授权。'}
    if http_status in (403,) or 'scope' in msg or 'permission' in msg or '权限' in msg:
        return {'error_type': 'scope', 'retryable': False, 'hint': '当前 scope 不足，请扩展授权权限。'}
    if http_status in (404,) or 'not found' in msg or '不存在' in msg:
        return {'error_type': 'not_found', 'retryable': False, 'hint': '资源不存在或已被移动，请检查 ID。'}
    if http_status in (409,) or 'conflict' in msg or '已存在' in msg:
        return {'error_type': 'conflict', 'retryable': False, 'hint': '资源冲突（可能重复创建），建议改名或先查询。'}
    if http_status in (400, 422) or 'invalid' in msg or '参数' in msg:
        return {'error_type': 'validation', 'retryable': False, 'hint': '请求参数不合法，请检查入参格式。'}
    if http_status in (500, 502, 503, 504):
        return {'error_type': 'upstream', 'retryable': True, 'hint': 'WPS 服务暂时异常，建议稍后重试。'}
    if code == -1:
        if 'timeout' in msg or 'timed out' in msg:
            return {'error_type': 'timeout', 'retryable': True, 'hint': '请求超时，建议重试或调大超时。'}
        return {'error_type': 'network', 'retryable': True, 'hint': '网络异常，请检查网络后重试。'}
    if code == -2:
        return {'error_type': 'server', 'retryable': False, 'hint': '上游返回非标准响应，请查看 raw_text。'}

    return {'error_type': 'unknown', 'retryable': False, 'hint': '未知错误，请结合 method/path/request_id 排查。'}


def summarize_wps_error(payload: dict) -> str:
    """将 WPS API 错误格式化为简洁可读文本。"""
    if not isinstance(payload, dict):
        return '未知错误（响应非 dict）'
    if payload.get('code') == 0:
        return ''

    error_type = payload.get('error_type', 'unknown')
    msg = payload.get('msg', '未知错误')
    http_status = payload.get('http_status')
    hint = payload.get('hint', '')
    request_id = payload.get('request_id', '')
    parts = [f'type={error_type}', f'msg={msg}']
    if http_status:
        parts.append(f'http={http_status}')
    if request_id:
        parts.append(f'request_id={request_id}')
    if hint:
        parts.append(f'hint={hint}')
    return ' | '.join(parts)


def _load_auth_config() -> dict:
    """加载 AK/SK 配置"""
    if not AUTH_CONFIG or not AUTH_CONFIG.exists():
        return {}
    config = {}
    with open(AUTH_CONFIG) as f:
        for line in f:
            if ':' in line:
                key, value = line.strip().split(':', 1)
                config[key.strip()] = value.strip()
    return config


def _load_token_cache() -> dict:
    """加载 Token 缓存文件"""
    if OAUTH_TOKEN_CACHE.exists():
        try:
            with open(OAUTH_TOKEN_CACHE, 'r', encoding='utf-8') as f:
                return json.load(f)
        except Exception:
            pass
    return {}


def _save_token_cache(data: dict):
    """保存 Token 缓存"""
    OAUTH_TOKEN_CACHE.parent.mkdir(parents=True, exist_ok=True)
    with open(OAUTH_TOKEN_CACHE, 'w', encoding='utf-8') as f:
        json.dump(data, f, ensure_ascii=False, indent=2)


# ============ OAuth Server 交互 ============

def _request_oauth_server(method: str, path: str, **kwargs) -> dict:
    """向 OAuth Server 发送请求"""
    req_path = path if str(path).startswith('/') else f'/{path}'
    for base in OAUTH_SERVER_CANDIDATES:
        try:
            url = f"{base.rstrip('/')}{req_path}"
            resp = requests.request(method, url, timeout=5, **kwargs)
            if resp.status_code == 404:
                continue
            data = resp.json()
            if isinstance(data, dict):
                return data
        except Exception:
            continue
    return {}


def require_token(scope: str = None, skill_name: str = 'unknown') -> dict:
    """
    🔑 Skill 统一入口：请求用户 Token 并指定所需 scope
    
    这是其他 Skills 获取 Token 的推荐方式。它会：
    1. 向 OAuth Server 请求 Token
    2. 自动检查 scope 是否满足
    3. 如果 scope 不足，返回授权 URL

    Args:
        scope: 所需的 OAuth scope（逗号分隔），如 'kso.file.read,kso.file.readwrite'
        skill_name: 调用方 Skill 名称（用于日志）

    Returns:
        dict: {
            'success': bool,
            'token': str or None,        # 成功时返回 token
            'scope': str,                 # 当前 token 的 scope
            'scope_covered': bool,        # scope 是否已覆盖需求
            'need_reauth': bool,          # 是否需要重新授权
            'auth_url': str or None,      # 重授权 URL（如需要）
            'missing_scopes': list,       # 缺少的 scope 列表
            'message': str,               # 提示消息
        }
    
    示例:
        result = require_token(scope='kso.file.read,kso.file.readwrite', skill_name='wps-app-files')
        if result['success']:
            token = result['token']
            # 使用 token 进行 API 调用
        elif result.get('need_reauth'):
            print(f"请在浏览器中完成授权: {result['auth_url']}")
        else:
            print(f"获取 Token 失败: {result.get('message')}")
    """
    # 优先通过 OAuth Server API
    data = _request_oauth_server('POST', '/api/token/require', json={
        'scope': scope or '',
        'skill_name': skill_name
    })

    if data.get('success'):
        write_skill_event(PLATFORM_CONFIG, {
            'skill': skill_name,
            'action': 'require_token',
            'status': 'ok',
            'scope': scope or '',
            'covered_scope': data.get('scope', ''),
        })
        return data

    if data.get('need_reauth') or data.get('auth_url'):
        write_skill_event(PLATFORM_CONFIG, {
            'skill': skill_name,
            'action': 'require_token',
            'status': 'need_reauth',
            'scope': scope or '',
            'missing_scopes': data.get('missing_scopes', []),
        })
        return {
            'success': False,
            'token': None,
            'scope_covered': data.get('scope_covered', False),
            'need_reauth': True,
            'auth_url': data.get('auth_url'),
            'missing_scopes': data.get('missing_scopes', []),
            'merged_scope': data.get('merged_scope', ''),
            'message': data.get('message', '需要重新授权以获取更多权限')
        }

    # OAuth Server 不可用，回退到本地缓存
    fallback = _fallback_get_token(scope)
    write_skill_event(PLATFORM_CONFIG, {
        'skill': skill_name,
        'action': 'require_token_fallback',
        'status': 'ok' if fallback.get('success') else 'error',
        'scope': scope or '',
        'message': fallback.get('message', ''),
    })
    return fallback


def _fallback_get_token(scope: str = None) -> dict:
    """当 OAuth Server 不可用时，从本地缓存获取 Token"""
    now = time.time()

    # 1. 从本地缓存读取
    cache = _load_token_cache()
    user = cache.get('user', {})

    if user.get('token') and now < user.get('expires_at', 0) - 60:
        # 如果指定了 scope，检查是否覆盖
        if scope:
            # 同时支持逗号和空格分隔
            current_scope = set(s.strip() for s in user.get('scope', '').replace(',', ' ').split() if s.strip())
            required_scope = set(s.strip() for s in scope.replace(',', ' ').split() if s.strip())
            if not required_scope.issubset(current_scope):
                missing = sorted(required_scope - current_scope)
                return {
                    'success': False,
                    'token': None,
                    'scope_covered': False,
                    'need_reauth': True,
                    'auth_url': f'{OAUTH_PUBLIC_URL}/',
                    'missing_scopes': missing,
                    'message': f"当前 Token 缺少 scope: {', '.join(missing)}。"
                               f"请访问 OAuth Server 重新授权。"
                }

        return {
            'success': True,
            'token': user['token'],
            'scope': user.get('scope', ''),
            'scope_covered': True
        }

    # 2. 尝试用 refresh_token 刷新
    refresh_token = user.get('refresh_token')
    if refresh_token:
        token = _refresh_user_token_directly(refresh_token)
        if token:
            return {
                'success': True,
                'token': token,
                'scope': user.get('scope', ''),
                'scope_covered': True,
                'refreshed': True
            }

    # 3. 环境变量
    env_token = os.environ.get('WPS_USER_TOKEN')
    if env_token:
        return {
            'success': True,
            'token': env_token,
            'scope': '',
            'scope_covered': False,  # 无法确认环境变量 token 的 scope
            'source': 'env'
        }

    return {
        'success': False,
        'token': None,
        'scope_covered': False,
        'need_reauth': True,
        'auth_url': f'{OAUTH_PUBLIC_URL}/',
        'message': '无法获取用户 Token。请启动 OAuth Server 并完成授权。'
    }


def _refresh_user_token_directly(refresh_token: str) -> str:
    """直接通过 refresh_token 刷新用户 Token"""
    config = _load_auth_config()
    ak = config.get('AK')
    sk = config.get('SK')
    if not ak or not sk:
        return None

    try:
        resp = requests.post(
            f'{API_BASE}/oauth2/token',
            data=urllib.parse.urlencode({
                'grant_type': 'refresh_token',
                'client_id': ak,
                'client_secret': sk,
                'refresh_token': refresh_token
            }),
            headers={'Content-Type': 'application/x-www-form-urlencoded'}
        )
        result = resp.json()
        if 'access_token' in result:
            cache = _load_token_cache()
            old_scope = cache.get('user', {}).get('scope', '')
            cache['user'] = {
                'token': result['access_token'],
                'expires_at': time.time() + result.get('expires_in', 7200),
                'refresh_token': result.get('refresh_token', refresh_token),
                'scope': old_scope,  # 保留原 scope
                'updated_at': datetime.now().isoformat()
            }
            _save_token_cache(cache)
            return result['access_token']
    except Exception:
        pass
    return None


def _refresh_app_token_directly() -> str:
    """直接通过 API 获取应用 Token"""
    config = _load_auth_config()
    ak = config.get('AK')
    sk = config.get('SK')
    if not ak or not sk:
        return None

    try:
        resp = requests.post(
            f'{API_BASE}/oauth2/token',
            data=urllib.parse.urlencode({
                'grant_type': 'client_credentials',
                'client_id': ak,
                'client_secret': sk
            }),
            headers={'Content-Type': 'application/x-www-form-urlencoded'}
        )
        result = resp.json()
        if 'access_token' in result:
            cache = _load_token_cache()
            cache['app'] = {
                'token': result['access_token'],
                'expires_at': time.time() + result.get('expires_in', 7200),
                'updated_at': datetime.now().isoformat()
            }
            _save_token_cache(cache)
            return result['access_token']
    except Exception:
        pass
    return None


# ============ 简便函数（向后兼容）============

def get_user_token(scope: str = None) -> str:
    """
    获取用户 Token（自动从多个来源尝试）
    
    Args:
        scope: 可选，所需的 OAuth scope（逗号分隔）

    Returns:
        str: 有效的用户 Token
    
    Raises:
        Exception: 无法获取用户 Token 或 scope 不足
    """
    result = require_token(scope=scope)

    if result.get('success'):
        return result['token']

    if result.get('need_reauth'):
        auth_url = result.get('auth_url', f'{OAUTH_SERVER_URL}/')
        missing = result.get('missing_scopes', [])
        if missing:
            raise Exception(
                f"❌ 当前 Token 缺少 scope: {', '.join(missing)}\n"
                f"   请访问以下链接重新授权：\n"
                f"   {auth_url}"
            )
        raise Exception(
            f"❌ 无法获取用户 Token。请完成 OAuth 授权：\n"
            f"   {auth_url}"
        )

    raise Exception(
        "❌ 无法获取用户 Token。请先启动 OAuth Server 完成授权：\n"
        f"   OAuth Server: {OAUTH_SERVER_URL}"
    )


def refresh_user_token(refresh_token: str = None) -> str:
    """
    刷新用户 Token（优先使用 refresh_token）。

    Args:
        refresh_token: 可选，显式指定 refresh_token。为空时从缓存读取。

    Returns:
        str: 新的 access_token，失败返回 None。
    """
    token_to_use = refresh_token
    if not token_to_use:
        cache = _load_token_cache()
        token_to_use = cache.get('user', {}).get('refresh_token')

    if token_to_use:
        token = _refresh_user_token_directly(token_to_use)
        if token:
            return token

    # 回退：让 OAuth Server 负责刷新，再拉取最新 user token
    data = _request_oauth_server('GET', '/api/token/refresh?type=user')
    if data.get('user_refreshed') or data.get('success'):
        user_data = _request_oauth_server('GET', '/api/token/user')
        if user_data.get('success') and user_data.get('token'):
            return user_data['token']

    return None


def get_app_token(force: bool = False) -> str:
    """
    获取应用 Token（自动从多个来源尝试）

    Args:
        force: 是否强制刷新（忽略缓存）

    Returns:
        str: 有效的应用 Token
    
    Raises:
        Exception: 无法获取应用 Token
    """
    now = time.time()

    # 如果强制刷新，先尝试直接刷新
    if force:
        token = _refresh_app_token_directly()
        if token:
            return token

    # 1. 从 OAuth Server API 获取（支持 force 参数）
    force_param = 'true' if force else 'false'
    data = _request_oauth_server('GET', f'/api/token/app?force={force_param}')
    if data.get('success'):
        return data['token']

    # 2. 从本地缓存读取
    cache = _load_token_cache()
    app = cache.get('app', {})
    if not force and app.get('token') and now < app.get('expires_at', 0) - 60:
        return app['token']

    # 3. 环境变量
    env_token = os.environ.get('WPS_APP_TOKEN')
    if env_token and not force:
        return env_token

    # 4. 直接获取（最后尝试）
    token = _refresh_app_token_directly()
    if token:
        return token

    raise Exception("❌ 无法获取应用 Token，请检查 auth.conf 中的 AK/SK 配置")


def refresh_app_token(force: bool = False) -> str:
    """
    刷新应用 Token。

    Returns:
        str: 刷新后的 access_token，失败返回 None
    """
    if force:
        return force_refresh_app_token()
    return get_app_token(force=False)


def force_refresh_app_token() -> str:
    """
    强制刷新应用 Token（忽略缓存）。

    Returns:
        str: 刷新后的 access_token，失败返回 None
    """
    # 1. 优先通过 OAuth Server API
    data = _request_oauth_server('GET', '/api/token/app?force=true')
    if data.get('success') and data.get('token'):
        return data['token']

    refresh_data = _request_oauth_server('GET', '/api/token/refresh?type=app')
    if refresh_data.get('app_refreshed') or refresh_data.get('success'):
        token_data = _request_oauth_server('GET', '/api/token/app')
        if token_data.get('success') and token_data.get('token'):
            return token_data['token']

    # 2. 直接刷新
    token = _refresh_app_token_directly()
    return token


def get_token_status() -> dict:
    """获取 Token 状态"""
    # 优先从 OAuth Server 获取
    data = _request_oauth_server('GET', '/api/token/status')
    if data and 'user' in data:
        return {
            'user_token_valid': data['user']['valid'],
            'user_token_ttl': data['user']['ttl_seconds'],
            'user_scope': data['user'].get('scope', ''),
            'user_scope_list': data['user'].get('scope_list', []),
            'scopes': data['user'].get('scope_list', []),
            'has_refresh_token': data['user']['has_refresh_token'],
            'app_token_valid': data['app']['valid'],
            'app_token_ttl': data['app']['ttl_seconds'],
        }

    # 回退到本地缓存
    cache = _load_token_cache()
    user = cache.get('user', {})
    app = cache.get('app', {})
    now = time.time()

    user_valid = bool(user.get('token') and now < user.get('expires_at', 0))
    app_valid = bool(app.get('token') and now < app.get('expires_at', 0))

    return {
        'user_token_valid': user_valid,
        'user_token_ttl': max(0, int(user.get('expires_at', 0) - now)) if user_valid else 0,
        'user_scope': user.get('scope', ''),
        'user_scope_list': sorted(set(s.strip() for s in user.get('scope', '').replace(',', ' ').split() if s.strip())) if user.get('scope') else [],
        'scopes': sorted(set(s.strip() for s in user.get('scope', '').replace(',', ' ').split() if s.strip())) if user.get('scope') else [],
        'has_refresh_token': bool(user.get('refresh_token')),
        'app_token_valid': app_valid,
        'app_token_ttl': max(0, int(app.get('expires_at', 0) - now)) if app_valid else 0,
    }


# ============ WPS API 客户端 ============

class WPSClient:
    """
    通用的 WPS API 客户端
    自动处理认证、签名和 scope 检查
    
    用法：
        client = WPSClient(auth_type='user', scope='kso.file.read,kso.file.readwrite')
        result = client.get('/v7/contacts/departments/children', params={'page_size': 50})
        result = client.post('/v7/drives/{drive_id}/files/{parent_id}/create', body={...})
    """

    def __init__(self, auth_type: str = 'user', scope: str = None):
        """
        初始化客户端
        
        Args:
            auth_type: 'user' 或 'app'
            scope: 所需的 OAuth scope（逗号分隔，仅对 user 类型有效）
        """
        self.auth_type = auth_type
        self.scope = scope
        config = _load_auth_config()
        self.ak = config.get('AK', '')
        self.sk = config.get('SK', '')
        self.timeout_seconds = DEFAULT_API_TIMEOUT
        self.max_retries = max(0, DEFAULT_API_MAX_RETRIES)
        self.retry_backoff_seconds = max(0.1, DEFAULT_API_RETRY_BACKOFF)
        self.log_enabled = DEFAULT_API_LOG_ENABLED
        self.log_path = DEFAULT_API_LOG_PATH
        retry_mutating = os.environ.get('WPS_API_RETRY_MUTATING', 'false').strip().lower()
        self.retry_mutating = retry_mutating in ('1', 'true', 'yes', 'on')
        auto_idempotency = os.environ.get('WPS_AUTO_IDEMPOTENCY', 'true').strip().lower()
        self.auto_idempotency = auto_idempotency in ('1', 'true', 'yes', 'on')

    def _get_token(self) -> str:
        if self.auth_type == 'user':
            return get_user_token(scope=self.scope)
        else:
            return get_app_token()

    def _build_headers(self, method: str, path: str) -> dict:
        """构建请求头（含签名）"""
        token = self._get_token()
        date_str = datetime.utcnow().strftime('%a, %d %b %Y %H:%M:%S GMT')

        headers = {
            'Content-Type': 'application/json',
            'Authorization': f'Bearer {token}',
            'X-Kso-Date': date_str,
        }

        if self.sk:
            parsed = urllib.parse.urlparse(path)
            sign_path = parsed.path
            if parsed.query:
                sign_path += '?' + parsed.query

            string_to_sign = f"{method.upper()}\n{sign_path}\n{date_str}"
            signature = hmac.new(
                self.sk.encode('utf-8'),
                string_to_sign.encode('utf-8'),
                hashlib.sha1
            ).digest()
            headers['X-Kso-Authorization'] = 'KSO-1 ' + base64.b64encode(signature).decode('utf-8')

        return headers

    def _can_retry_method(self, method: str) -> bool:
        method = method.upper()
        if method in ('GET', 'HEAD', 'OPTIONS'):
            return True
        if method in ('POST', 'PUT', 'PATCH', 'DELETE'):
            return self.retry_mutating
        return False

    def _generate_request_id(self) -> str:
        return uuid.uuid4().hex

    def _generate_idempotency_key(self, method: str, path: str, body: dict = None) -> str:
        raw = json.dumps(body or {}, ensure_ascii=False, sort_keys=True, separators=(',', ':'))
        digest = hashlib.sha256(f'{method.upper()}|{path}|{raw}'.encode('utf-8')).hexdigest()[:24]
        return f'wps-{int(time.time())}-{digest}'

    def _log_api_event(self, event: dict):
        if not self.log_enabled:
            return
        try:
            self.log_path.parent.mkdir(parents=True, exist_ok=True)
            with open(self.log_path, 'a', encoding='utf-8') as f:
                f.write(json.dumps(event, ensure_ascii=False) + '\n')
        except Exception:
            pass

    def _normalize_response(self, resp: requests.Response) -> dict:
        try:
            payload = resp.json()
        except Exception:
            text = (resp.text or '')[:500]
            return {
                'code': -2,
                'msg': f'WPS API 返回非 JSON 响应 (HTTP {resp.status_code})',
                'http_status': resp.status_code,
                'raw_text': text,
            }

        if isinstance(payload, dict):
            payload.setdefault('http_status', resp.status_code)
            return payload

        return {
            'code': -2,
            'msg': 'WPS API 响应 JSON 结构异常（非对象）',
            'http_status': resp.status_code,
            'data': payload,
        }

    def request(self, method: str, path: str, params: dict = None, body: dict = None,
                idempotency_key: str = None) -> dict:
        """
        发送 API 请求
        
        Args:
            method: HTTP 方法
            path: API 路径（如 /v7/contacts/departments/children）
            params: URL 查询参数
            body: 请求体
        
        Returns:
            dict: API 响应
        """
        url = f'{API_BASE}{path}'
        if params:
            url += '?' + urllib.parse.urlencode(params, doseq=True)

        headers = self._build_headers(method, url.replace(API_BASE, ''))

        method_upper = method.upper()
        if method_upper not in ('GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS'):
            raise ValueError(f'不支持的请求方法: {method}')

        retry_enabled = self._can_retry_method(method_upper)
        max_attempts = 1 + (self.max_retries if retry_enabled else 0)
        last_error = None
        request_id = self._generate_request_id()
        headers['X-Request-Id'] = request_id
        if method_upper in ('POST', 'PUT', 'PATCH', 'DELETE'):
            if not idempotency_key and self.auto_idempotency:
                idempotency_key = self._generate_idempotency_key(method_upper, path, body)
            if idempotency_key:
                headers['X-Idempotency-Key'] = idempotency_key

        for attempt in range(1, max_attempts + 1):
            try:
                started_at = time.time()
                resp = requests.request(
                    method_upper,
                    url,
                    headers=headers,
                    json=body,
                    timeout=self.timeout_seconds
                )
                latency_ms = int((time.time() - started_at) * 1000)
                data = self._normalize_response(resp)
            except requests.RequestException as e:
                last_error = str(e)
                if attempt < max_attempts:
                    sleep_seconds = self.retry_backoff_seconds * (2 ** (attempt - 1))
                    time.sleep(sleep_seconds)
                    continue
                return {
                    'code': -1,
                    'msg': f'WPS API 请求异常: {e}',
                    'method': method_upper,
                    'path': path,
                    'request_id': request_id,
                }

            http_status = resp.status_code
            is_retryable_http = http_status in (429, 500, 502, 503, 504)
            if is_retryable_http and attempt < max_attempts:
                sleep_seconds = self.retry_backoff_seconds * (2 ** (attempt - 1))
                time.sleep(sleep_seconds)
                continue

            # 统一补充调试信息，便于跨技能排障
            if isinstance(data, dict):
                data.setdefault('method', method_upper)
                data.setdefault('path', path)
                data.setdefault('request_id', request_id)
                if idempotency_key:
                    data.setdefault('idempotency_key', idempotency_key)
                if data.get('code') != 0 or http_status >= 400:
                    classification = classify_wps_error(data)
                    data.update(classification)

            self._log_api_event({
                'time': datetime.utcnow().isoformat() + 'Z',
                'method': method_upper,
                'path': path,
                'http_status': http_status,
                'code': data.get('code') if isinstance(data, dict) else None,
                'error_type': data.get('error_type') if isinstance(data, dict) else None,
                'retryable': data.get('retryable') if isinstance(data, dict) else None,
                'attempt': attempt,
                'max_attempts': max_attempts,
                'latency_ms': latency_ms,
                'request_id': request_id,
                'idempotency_key': idempotency_key,
            })

            time.sleep(0.1)  # 限流保护
            return data

        return {
            'code': -1,
            'msg': f'WPS API 请求失败: {last_error or "未知错误"}',
            'method': method_upper,
            'path': path,
            'request_id': request_id,
            **classify_wps_error({'code': -1, 'msg': f'WPS API 请求失败: {last_error or "未知错误"}'}),
        }

    def get(self, path: str, params: dict = None) -> dict:
        return self.request('GET', path, params=params)

    def post(self, path: str, body: dict = None, params: dict = None, idempotency_key: str = None) -> dict:
        return self.request('POST', path, params=params, body=body, idempotency_key=idempotency_key)

    def put(self, path: str, body: dict = None, params: dict = None, idempotency_key: str = None) -> dict:
        return self.request('PUT', path, params=params, body=body, idempotency_key=idempotency_key)

    def delete(self, path: str, params: dict = None, idempotency_key: str = None) -> dict:
        return self.request('DELETE', path, params=params, idempotency_key=idempotency_key)


def get_wps_client(auth_type: str = 'user', scope: str = None) -> WPSClient:
    """
    获取 WPS API 客户端实例
    
    Args:
        auth_type: 'user' 或 'app'
        scope: 所需的 OAuth scope（逗号分隔，仅对 user 类型有效）
    
    Returns:
        WPSClient: 已配置好认证的客户端
    
    示例:
        # 无 scope（使用当前 token 的 scope）
        client = get_wps_client()

        # 指定 scope（逗号分隔）
        client = get_wps_client(scope='kso.file.read,kso.file.readwrite')
        
        # 应用级别
        client = get_wps_client(auth_type='app')
    """
    return WPSClient(auth_type=auth_type, scope=scope)


# ============ CLI ============
if __name__ == '__main__':
    import argparse

    parser = argparse.ArgumentParser(description='WPS Token Manager')
    parser.add_argument('--status', action='store_true', help='查看 Token 状态')
    parser.add_argument('--require', metavar='SCOPE', help='请求指定 scope 的 Token')
    parser.add_argument('--check-scope', metavar='SCOPE', help='检查 scope 覆盖情况')
    parser.add_argument('--user-token', action='store_true', help='(兼容) 获取用户 Token')
    parser.add_argument('--app-token', action='store_true', help='(兼容) 获取应用 Token')

    subparsers = parser.add_subparsers(dest='action')
    user_parser = subparsers.add_parser('user-token', help='User Token 操作')
    user_parser.add_argument('--refresh', action='store_true', help='刷新 User Token')
    user_parser.add_argument('--scope', help='指定 scope')

    app_parser = subparsers.add_parser('app-token', help='App Token 操作')
    app_parser.add_argument('--force-refresh', action='store_true', help='强制刷新 App Token')

    args = parser.parse_args()

    if args.status:
        status = get_token_status()
        print("\n📊 Token 状态:")
        print(f"   用户 Token: {'✅ 有效' if status['user_token_valid'] else '❌ 无效'}")
        if status['user_token_valid']:
            ttl = status['user_token_ttl']
            print(f"   剩余时间: {ttl // 3600}小时{(ttl % 3600) // 60}分钟")
        print(f"   Scope: {status.get('user_scope') or '（无）'}")
        print(f"   Scope 列表: {status.get('user_scope_list', [])}")
        print(f"   Refresh Token: {'✅ 有' if status['has_refresh_token'] else '❌ 无'}")
        print(f"   应用 Token: {'✅ 有效' if status['app_token_valid'] else '❌ 无效'}")
    elif args.action == 'user-token':
        try:
            if args.refresh:
                token = refresh_user_token()
            else:
                token = get_user_token(scope=args.scope)
            if not token:
                raise Exception('刷新 User Token 失败')
            print(token)
        except Exception as e:
            print(f"❌ {e}", file=sys.stderr)
            sys.exit(1)
    elif args.action == 'app-token':
        try:
            token = force_refresh_app_token() if args.force_refresh else get_app_token()
            if not token:
                raise Exception('获取 App Token 失败')
            print(token)
        except Exception as e:
            print(f"❌ {e}", file=sys.stderr)
            sys.exit(1)
    elif args.user_token:
        try:
            token = get_user_token()
            print(token)
        except Exception as e:
            print(f"❌ {e}", file=sys.stderr)
            sys.exit(1)
    elif args.app_token:
        try:
            token = get_app_token()
            print(token)
        except Exception as e:
            print(f"❌ {e}", file=sys.stderr)
            sys.exit(1)
    elif args.require:
        result = require_token(scope=args.require, skill_name='cli')
        print(json.dumps(result, ensure_ascii=False, indent=2))
        if not result['success']:
            sys.exit(1)
    elif args.check_scope:
        data = _request_oauth_server('GET', f'/api/scope/check?scope={urllib.parse.quote(args.check_scope)}')
        if data:
            print(json.dumps(data, ensure_ascii=False, indent=2))
        else:
            print("❌ OAuth Server 不可用", file=sys.stderr)
            sys.exit(1)
    else:
        parser.print_help()
