import { createApp } from 'vue';
import App from './App.vue';
import './style.css';
import { connect } from './store.ts';
import { WsTransport, socketUrl } from './transport.ts';

async function boot(): Promise<void> {
  // The mock exists only in the dev server; a production build drops this branch and the module.
  if (import.meta.env.DEV && new URLSearchParams(window.location.search).has('mock')) {
    const { MockTransport } = await import('../dev/mock.ts');
    connect(new MockTransport());
  } else {
    connect(new WsTransport(socketUrl()));
  }
  createApp(App).mount('#app');
}

void boot();
