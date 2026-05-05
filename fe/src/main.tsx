import { createRoot } from 'react-dom/client';
import './index.css';
import './styles/xterm.css';
import App from './App.tsx';

// Global error handler to prevent uncaught errors from crashing Electron
window.onerror = (message, source, lineno, colno, error) => {
  console.error('[Global Error]', { message, source, lineno, colno, error })
  return false
}

window.onunhandledrejection = (event) => {
  console.error('[Unhandled Promise Rejection]', event.reason)
}

createRoot(document.getElementById('root')!).render(<App />);
