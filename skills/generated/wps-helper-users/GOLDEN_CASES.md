# GOLDEN CASES - users

## Case 1: happy path

```bash
wpscli users --help
```

## Case 2: dry run validation

```bash
wpscli raw GET /v7/drives --auth-type app --dry-run
```

## Case 3: auth fallback

- Validate auth mode with `wpscli auth status`
- Retry with `--auth-type app` if scope/auth mismatch happens.
