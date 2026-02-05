# SYNTON-DB E2E Tests

End-to-end testing framework using Playwright.

## Setup

```bash
cd e2e
npm install
npx playwright install
```

## Running Tests

```bash
# Run all tests
npm test

# Run with headed mode (see browser)
npm run test:headed

# Debug tests
npm run test:debug

# View test report
npm run test:report
```

## Test Scenarios

| Test | Description |
|------|-------------|
| `node-operations.spec.ts` | Create, retrieve, list, delete nodes |
| `edge-operations.spec.ts` | Create edges, verify graph structure |
| `query-operations.spec.ts` | Execute PaQL queries, verify results |
| `stats.spec.ts` | Health checks, database statistics |

## Configuration

Environment variables:
- `BASE_URL` - Server URL (default: `http://127.0.0.1:8080`)
- `SERVER_PATH` - Path to server binary (default: `./target/release/synton-db-server`)
- `CONFIG_PATH` - Path to test config (default: `./e2e/test-config.toml`)

## Test Data

Test data is stored in `./test-data/` during tests and cleaned up after each test.
