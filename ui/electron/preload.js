const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('metallic', {
  selectFolder: () => ipcRenderer.invoke('select-folder'),
  selectFile: () => ipcRenderer.invoke('select-file'),
  getPlatform: () => ipcRenderer.invoke('get-platform'),
  openDataDir: () => ipcRenderer.invoke('open-data-dir'),
  // Forward arbitrary API calls to the Rust daemon over IPC.
  // This is more reliable than fetch() from the sandboxed renderer.
  api: (method, path, body) => ipcRenderer.invoke('api-call', { method, path, body }),

  // Live event stream — opens an IPC subscription that pipes each SSE frame
  // from /api/events/stream into the renderer. Returns an unsubscribe function.
  //   onEvent(payload)       — parsed { event: "file_processed", data: {...} }
  //   onOpen()               — stream connected
  //   onError({ message })   — stream closed with error
  subscribeEvents: ({ onEvent, onOpen, onError } = {}) => {
    const handlers = {
      'sse-event': (_e, p) => onEvent && onEvent(p),
      'sse-open':  () => onOpen && onOpen(),
      'sse-error': (_e, p) => onError && onError(p),
    };
    for (const [k, h] of Object.entries(handlers)) {
      ipcRenderer.on(k, h);
    }
    return ipcRenderer.invoke('subscribe-events').then((unsub) => {
      return () => {
        for (const [k, h] of Object.entries(handlers)) {
          ipcRenderer.removeListener(k, h);
        }
        if (typeof unsub === 'function') unsub();
      };
    });
  },
});
