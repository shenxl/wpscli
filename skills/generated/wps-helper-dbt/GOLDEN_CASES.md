# GOLDEN CASES - dbt

## Case 1: happy path

```bash
wpscli dbt --help
```

## Case 2: dry run validation

```bash
wpscli raw GET /v7/drives --auth-type user --dry-run
```

## Case 3: auth fallback

- Validate auth mode with `wpscli auth status`
- Retry with `--auth-type user` if scope/auth mismatch happens.
