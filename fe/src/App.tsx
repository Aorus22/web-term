import { useState, useCallback, useRef } from 'react';
import { Terminal, useTerminal } from '@wterm/react';
import '@wterm/react/css';
import './App.css';

function App() {
  const { ref, write, focus } = useTerminal();
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const wsRef = useRef<WebSocket | null>(null);

  // Form state
  const [host, setHost] = useState('');
  const [port, setPort] = useState('22');
  const [user, setUser] = useState('');
  const [password, setPassword] = useState('');

  const handleConnect = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      setError(null);

      const portNum = parseInt(port, 10) || 22;

      // Create WebSocket to Vite dev proxy (which proxies to Go backend).
      const socket = new WebSocket(
        `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/ws`
      );
      wsRef.current = socket;

      socket.binaryType = 'arraybuffer';

      socket.onopen = () => {
        // Send connect message with SSH params.
        const connectMsg = JSON.stringify({
          type: 'connect',
          host,
          port: portNum,
          user,
          password,
        });
        socket.send(connectMsg);
      };

      socket.onmessage = (event) => {
        if (typeof event.data === 'string') {
          // JSON control message from server.
          try {
            const msg = JSON.parse(event.data);
            if (msg.type === 'connected') {
              setConnected(true);
              // Focus terminal after connection.
              setTimeout(() => focus(), 100);
            } else if (msg.type === 'error') {
              setError(msg.message || 'Connection error');
              socket.close();
            }
          } catch {
            // Ignore malformed JSON.
          }
        } else {
          // Binary frame — SSH output data.
          const data =
            event.data instanceof ArrayBuffer
              ? new Uint8Array(event.data)
              : event.data;
          write(data);
        }
      };

      socket.onclose = () => {
        setConnected(false);
        wsRef.current = null;
      };

      socket.onerror = () => {
        setError('WebSocket connection failed');
        setConnected(false);
        wsRef.current = null;
      };
    },
    [host, port, user, password, write, focus]
  );

  // Terminal user input → WebSocket binary frame.
  const handleData = useCallback(
    (data: string) => {
      const socket = wsRef.current;
      if (socket && socket.readyState === WebSocket.OPEN) {
        socket.send(new TextEncoder().encode(data));
      }
    },
    []
  );

  // Terminal resize → WebSocket JSON control message.
  const handleResize = useCallback(
    (cols: number, rows: number) => {
      const socket = wsRef.current;
      if (socket && socket.readyState === WebSocket.OPEN) {
        socket.send(JSON.stringify({ type: 'resize', cols, rows }));
      }
    },
    []
  );

  // Disconnect handler.
  const handleDisconnect = useCallback(() => {
    const socket = wsRef.current;
    if (socket) {
      socket.close();
      wsRef.current = null;
    }
    setConnected(false);
  }, []);

  if (connected) {
    return (
      <div className="terminal-container">
        <div className="terminal-header">
          <span className="terminal-title">
            {user}@{host}
          </span>
          <button className="disconnect-btn" onClick={handleDisconnect}>
            Disconnect
          </button>
        </div>
        <div className="terminal-wrapper">
          <Terminal
            ref={ref}
            autoResize
            theme="solarized-dark"
            cursorBlink
            onData={handleData}
            onResize={handleResize}
          />
        </div>
      </div>
    );
  }

  return (
    <div className="connect-container">
      <div className="connect-card">
        <h1 className="connect-title">WebTerm</h1>
        <p className="connect-subtitle">SSH from your browser</p>

        {error && <div className="connect-error">{error}</div>}

        <form onSubmit={handleConnect} className="connect-form">
          <div className="form-group">
            <label htmlFor="host">Host</label>
            <input
              id="host"
              type="text"
              placeholder="192.168.1.1"
              value={host}
              onChange={(e) => setHost(e.target.value)}
              required
              autoFocus
            />
          </div>

          <div className="form-row">
            <div className="form-group">
              <label htmlFor="port">Port</label>
              <input
                id="port"
                type="number"
                placeholder="22"
                value={port}
                onChange={(e) => setPort(e.target.value)}
                min={1}
                max={65535}
              />
            </div>
            <div className="form-group">
              <label htmlFor="user">Username</label>
              <input
                id="user"
                type="text"
                placeholder="root"
                value={user}
                onChange={(e) => setUser(e.target.value)}
                required
              />
            </div>
          </div>

          <div className="form-group">
            <label htmlFor="password">Password</label>
            <input
              id="password"
              type="password"
              placeholder="••••••••"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
            />
          </div>

          <button type="submit" className="connect-btn">
            Connect
          </button>
        </form>
      </div>
    </div>
  );
}

export default App;
