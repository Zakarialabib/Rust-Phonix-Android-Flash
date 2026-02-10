import { render } from 'solid-js/web';
import App from './App';
import './styles.css';

const root = document.getElementById('root');

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    'Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?',
  );
}

// Disable context menu in production for native feel
if (!import.meta.env.DEV) {
  document.addEventListener('contextmenu', (e) => e.preventDefault());
}

import { AppProvider } from './context/AppContext';

render(
  () => (
    <AppProvider>
      <App />
    </AppProvider>
  ),
  root!
);
