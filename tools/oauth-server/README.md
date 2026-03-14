# OAuth Daemon (Extracted)

This directory hosts the extracted long-running OAuth service from `wpsskill/wps-oauth`.

It is designed to provide:

- user/app token HTTP endpoints for `wpscli`
- background token refresh
- scope-aware token acquisition

## Files

- `oauth_server.py`: HTTP OAuth daemon
- `token_manager.py`: shared Python token client helpers
- `wps_fault_report.py`: fault report utility
- `wps_platform.py`: platform observability helpers
- `start.sh`: daemon lifecycle script
- `wps-oauth.service`: systemd unit template

## Quick Start

```bash
cd tools/oauth-server
python3 oauth_server.py --no-browser
```

Or run in daemon mode:

```bash
cd tools/oauth-server
./start.sh --daemon
```

Then point `wpscli` to this service:

```bash
wpscli auth setup --oauth-server http://127.0.0.1:8089
wpscli auth status
```

`wpscli` will automatically call `/api/token/user` or `/api/token/app` when `oauth_server` is configured.

## Sensitive Files

Create local secrets at runtime (do not commit):

- `auth.conf`
- `data/token_cache.json`
- `data/*.log`
