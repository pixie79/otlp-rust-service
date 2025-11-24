import { App } from './app.js';

const bootstrap = async () => {
  const root = document.getElementById('app');
  if (!root) {
    throw new Error('Missing #app root element');
  }

  const app = new App(root);
  await app.initialize();

  window.addEventListener('beforeunload', () => app.destroy(), { once: true });
};

bootstrap().catch((error) => {
  console.error('Failed to bootstrap dashboard', error);
  const root = document.getElementById('app');
  if (root) {
    root.innerHTML = `
      <div class="panel error-state">
        <h1>Dashboard failed to load</h1>
        <p>${error.message}</p>
      </div>
    `;
  }
});
