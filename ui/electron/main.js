const { app, BrowserWindow, shell, dialog, ipcMain } = require('electron');
const path = require('path');
const fs = require('fs');
const { spawn } = require('child_process');
const http = require('http');

// Keep references to prevent garbage collection
let mainWindow = null;
let rustBackend = null;

const isDev = !app.isPackaged;
const BACKEND_PORT = 19270;
const BACKEND_URL = `http://127.0.0.1:${BACKEND_PORT}`;

// ─── Locate the Rust binary ─────────────────────────────────
function resolveRustBinary() {
  if (isDev) {
    // In dev: use the debug binary the developer just built.
    // Prefer release if it exists, fall back to debug.
    const coreDir = path.resolve(__dirname, '..', '..', 'core');
    const releasePath = path.join(coreDir, 'target', 'release', 'ferrum');
    const debugPath = path.join(coreDir, 'target', 'debug', 'ferrum');
    if (fs.existsSync(releasePath)) return releasePath;
    if (fs.existsSync(debugPath)) return debugPath;
    return null;
  }
  // Production: look in extraResources
  const binaryName = process.platform === 'win32' ? 'ferrum.exe' : 'ferrum';
  return path.join(process.resourcesPath, 'bin', binaryName);
}

// ─── Wait for the backend to be reachable ───────────────────
async function waitForBackend(maxMs = 15000) {
  const start = Date.now();
  while (Date.now() - start < maxMs) {
    try {
      await new Promise((resolve, reject) => {
        const req = http.get(`${BACKEND_URL}/api/health`, (res) => {
          if (res.statusCode === 200) resolve(undefined);
          else reject(new Error(`status ${res.statusCode}`));
        });
        req.on('error', reject);
        req.setTimeout(800, () => req.destroy(new Error('timeout')));
      });
      return true;
    } catch {
      await new Promise((r) => setTimeout(r, 300));
    }
  }
  return false;
}

function startRustBackend() {
  const binaryPath = resolveRustBinary();
  if (!binaryPath) {
    dialog.showErrorBox(
      'Ferrum — Backend Not Found',
      `Could not find the Rust backend binary. Build it first:\n\n` +
        `  cd core && cargo build --release\n\n` +
        `Expected at:\n  core/target/release/ferrum`
    );
    console.error('[Ferrum] No backend binary found.');
    return;
  }

  console.log('[Ferrum] Starting backend:', binaryPath);

  rustBackend = spawn(binaryPath, [], {
    env: { ...process.env, RUST_LOG: process.env.RUST_LOG || 'ferrum=info,tower_http=info' },
    stdio: ['ignore', 'pipe', 'pipe'],
  });

  rustBackend.stdout.on('data', (data) => {
    process.stdout.write(`[Backend] ${data}`);
  });

  rustBackend.stderr.on('data', (data) => {
    process.stderr.write(`[Backend] ${data}`);
  });

  rustBackend.on('error', (err) => {
    console.error('[Backend] Failed to start:', err);
    dialog.showErrorBox(
      'Ferrum — Backend Error',
      `Failed to start the protection engine: ${err.message}`
    );
  });

  rustBackend.on('exit', (code) => {
    console.log('[Backend] Exited with code:', code);
  });
}

function stopRustBackend() {
  if (rustBackend) {
    console.log('[Ferrum] Stopping backend...');
    rustBackend.kill('SIGTERM');
    rustBackend = null;
  }
}

// ─── (api bridge lives in the IPC handlers section below) ──

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1280,
    height: 820,
    minWidth: 900,
    minHeight: 600,
    titleBarStyle: 'hiddenInset', // Native macOS traffic lights
    trafficLightPosition: { x: 16, y: 16 },
    backgroundColor: '#0A0B0E',
    show: false,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      // All API calls go through the IPC bridge in preload.js, so the renderer
      // never makes direct network requests. Web security stays enabled.
    },
  });

  if (isDev) {
    mainWindow.loadURL('http://localhost:5173');
  } else {
    const initialHash = process.env.MM_INITIAL_PAGE ? `#${process.env.MM_INITIAL_PAGE}` : '';
    mainWindow.loadFile(path.join(__dirname, '..', 'dist', 'index.html'), { hash: initialHash });
  }

  // Open DevTools when explicitly requested — useful for visual debugging of
  // modal/popover/menu states that are hard to trigger headlessly.
  if (process.env.MM_DEVTOOLS === '1') {
    mainWindow.webContents.openDevTools({ mode: 'detach' });
  }

  mainWindow.once('ready-to-show', () => {
    mainWindow.show();
    mainWindow.focus();
  });

  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    shell.openExternal(url);
    return { action: 'deny' };
  });

  mainWindow.on('close', (e) => {
    if (!app.isQuitting) {
      e.preventDefault();
      mainWindow.hide();
    }
  });

  mainWindow.on('closed', () => {
    mainWindow = null;
  });
}

let tray = null;

// ─── App Lifecycle ──────────────────────────────────────────

app.whenReady().then(async () => {
  startRustBackend();

  // Don't block UI on the backend; the UI shows "Engine offline" until it's up.
  waitForBackend().then((ok) => {
    if (ok) console.log('[Ferrum] Backend is online.');
    else console.warn('[Ferrum] Backend did not become ready in time.');
  });

  createWindow();

  // Create Tray icon
  const { Tray, Menu, nativeImage } = require('electron');
  
  // Load clean template PNG icon for macOS status bar
  const trayIconPath = path.join(__dirname, '..', 'assets', 'trayTemplate.png');
  let iconCanvas;
  if (fs.existsSync(trayIconPath)) {
    iconCanvas = nativeImage.createFromPath(trayIconPath).resize({ width: 18, height: 18 });
  } else {
    iconCanvas = nativeImage.createEmpty();
  }
  
  iconCanvas.setTemplateImage(true);
  
  tray = new Tray(iconCanvas);
  tray.setToolTip('Ferrum - Protection Active');
  
  const contextMenu = Menu.buildFromTemplate([
    { label: 'Open Dashboard', click: () => {
        if (mainWindow === null) createWindow();
        else { mainWindow.show(); mainWindow.focus(); }
        if (app.dock) app.dock.show();
      } 
    },
    { type: 'separator' },
    { label: 'Quit Ferrum', click: () => {
        app.isQuitting = true;
        app.quit();
      } 
    }
  ]);
  
  tray.setContextMenu(contextMenu);
  
  tray.on('click', () => {
    if (mainWindow === null) createWindow();
    else if (mainWindow.isVisible()) { mainWindow.hide(); }
    else { mainWindow.show(); mainWindow.focus(); }
  });

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
    else if (mainWindow) { mainWindow.show(); mainWindow.focus(); }
  });
});

app.on('window-all-closed', (e) => {
  // Prevent quitting, keep running in background via Tray
  e.preventDefault();
});

app.on('before-quit', () => {
  stopRustBackend();
});

// ─── IPC handlers exposed to the renderer ───────────────────

ipcMain.handle('select-folder', async () => {
  const result = await dialog.showOpenDialog(mainWindow, {
    properties: ['openDirectory'],
    title: 'Select Folder to Protect',
  });
  return result.canceled ? null : result.filePaths[0];
});

ipcMain.handle('select-file', async () => {
  const result = await dialog.showOpenDialog(mainWindow, {
    properties: ['openFile'],
    title: 'Select File to Protect',
  });
  return result.canceled ? null : result.filePaths[0];
});

ipcMain.handle('get-platform', () => process.platform);

ipcMain.handle('open-data-dir', async () => {
  const dataDir = path.join(app.getPath('userData'), 'Ferrum');
  await fs.promises.mkdir(dataDir, { recursive: true });
  shell.openPath(dataDir);
  return dataDir;
});

ipcMain.handle('open-path', async (_evt, targetPath) => {
  if (targetPath) {
    shell.openPath(targetPath);
  }
});

// Generic API bridge — fetches from the local Rust daemon and returns parsed JSON.
// The renderer calls window.metallic.api(method, path, body) where path is "/foo" —
// we prepend the /api prefix the daemon expects.
ipcMain.handle('api-call', async (_evt, { method, path, body }) => {
  const url = `${BACKEND_URL}/api${path}`;
  const init = { method };
  if (body !== undefined && body !== null) {
    init.headers = { 'Content-Type': 'application/json' };
    init.body = JSON.stringify(body);
  }
  let res;
  try {
    res = await fetch(url, init);
  } catch (e) {
    throw new Error(`Backend unreachable: ${e.message}`);
  }
  const text = await res.text();
  let data;
  try { data = text ? JSON.parse(text) : null; } catch { data = text; }
  if (!res.ok) {
    const msg = (data && data.error) || `HTTP ${res.status}`;
    throw new Error(msg);
  }
  return data;
});

// SSE bridge — forwards the engine event stream to the renderer one event at a
// time so the live-wired "How It Works" page can animate without buffering.
// The renderer calls window.metallic.subscribeEvents() and gets back an
// unsubscribe function. We open a streaming fetch to /api/events/stream, parse
// SSE frames in memory, and push each event as an IPC message.
const sseSubscribers = new Map();
ipcMain.handle('subscribe-events', async (event) => {
  const senderId = event.sender.id;
  // If this sender already has an active subscription, close it first so we
  // don't leave dangling fetches when the renderer remounts the page.
  const existing = sseSubscribers.get(senderId);
  if (existing) {
    try { existing.controller.abort(); } catch {}
    sseSubscribers.delete(senderId);
  }

  const controller = new AbortController();
  const sender = event.sender;

  // The fetch runs in the main process Node.js. The response body is a
  // ReadableStream<Uint8Array>; we read it frame by frame and forward each
  // parsed SSE event as 'sse-event' to the renderer.
  (async () => {
    try {
      const res = await fetch(`${BACKEND_URL}/api/events/stream`, {
        signal: controller.signal,
        headers: { Accept: 'text/event-stream' },
      });
      if (!res.ok || !res.body) {
        sender.send('sse-error', { message: `SSE HTTP ${res.status}` });
        return;
      }
      sender.send('sse-open', {});
      const reader = res.body.getReader();
      const decoder = new TextDecoder();
      let buffer = '';
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        buffer += decoder.decode(value, { stream: true });
        // SSE frames are delimited by a blank line (\r\n\r\n or \n\n).
        let idx;
        while ((idx = buffer.search(/\r\n\r\n|\n\n/)) !== -1) {
          const frame = buffer.slice(0, idx);
          buffer = buffer.slice(idx + (buffer.startsWith('\r\n\r\n', idx) ? 4 : 2));
          const ev = parseSseFrame(frame);
          if (ev) sender.send('sse-event', ev);
        }
      }
    } catch (e) {
      if (e.name !== 'AbortError') {
        try { sender.send('sse-error', { message: e.message }); } catch {}
      }
    } finally {
      sseSubscribers.delete(senderId);
    }
  })();

  sseSubscribers.set(senderId, { controller });

  return () => {
    try { controller.abort(); } catch {}
    sseSubscribers.delete(senderId);
  };
});

ipcMain.on('sse-unsubscribe', (event) => {
  const sub = sseSubscribers.get(event.sender.id);
  if (sub) {
    try { sub.controller.abort(); } catch {}
    sseSubscribers.delete(event.sender.id);
  }
});

// Parse one SSE frame into { event, data } or null if it's a comment/heartbeat.
function parseSseFrame(frame) {
  let event = 'message';
  const dataLines = [];
  for (const line of frame.split(/\r?\n/)) {
    if (!line) continue;
    if (line.startsWith(':')) continue;        // comment / heartbeat
    const colon = line.indexOf(':');
    if (colon === -1) continue;
    const field = line.slice(0, colon);
    let value = line.slice(colon + 1);
    if (value.startsWith(' ')) value = value.slice(1);
    if (field === 'event') event = value;
    else if (field === 'data') dataLines.push(value);
    // ignore id:, retry:, etc.
  }
  if (dataLines.length === 0) return null;
  const data = dataLines.join('\n');
  // Try to parse JSON so the renderer doesn't have to.
  try { return { event, data: JSON.parse(data) }; }
  catch { return { event, data }; }
}

