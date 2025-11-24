# OTLP Realtime Dashboard

Web-based dashboard for monitoring OTLP traces and metrics stored as Arrow IPC Streaming files produced by the Rust OTLP service.

## Features

- Runs entirely in the browser using the File System Access API or FileReader fallback
- Streams Arrow IPC data into DuckDB-wasm for fast SQL-style querying
- Visualizes metrics with Plotly.js and renders large trace lists with virtual scrolling
- Uses Web Workers to keep the UI responsive

## Getting Started

```bash
cd dashboard
npm install
npm run dev
```

Visit `http://localhost:5173` and select the OTLP output directory when prompted.

## Scripts

| Command | Description |
| --- | --- |
| `npm run dev` | Start Vite dev server with HMR |
| `npm run build` | Build production bundle |
| `npm run preview` | Preview the production build |
| `npm run test:unit` | Run Vitest unit tests with watch disabled |
| `npm run test:ui` | Launch the Vitest UI |
| `npm run test:e2e` | Run Playwright end-to-end tests (browser install required) |

## Testing

Install Playwright browsers once:

```bash
npx playwright install
```

Then execute the provided scripts. Unit and integration tests live under `tests/unit` and `tests/integration`, while E2E tests live under `tests/e2e`.

## Project Structure

```
dashboard/
  src/               # Application source (JavaScript, workers, components)
  styles/            # Global and component CSS
  tests/             # Unit, integration, and Playwright tests
  vite.config.js     # Vite configuration
  vitest.config.js   # Vitest configuration
  playwright.config.js # Playwright configuration
```

## Browser Support

- Chrome / Edge (File System Access API)
- Firefox / Safari (FileReader fallback)

Graceful degradation is provided when the File System Access API is unavailable.
