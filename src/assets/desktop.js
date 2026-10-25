(() => {
  if (!window.FUSIBLE_DESKTOP) return;
  const STRIP = 44;
  const LABEL = 'main';
  const isMac = /Mac|iPhone|iPad/.test(navigator.platform) || /Mac OS X/.test(navigator.userAgent);
  const invoke = (cmd, args) => {
    const tauri = window.__TAURI_INTERNALS__;
    return tauri ? tauri.invoke(cmd, args) : Promise.resolve();
  };

  const style = document.createElement('style');
  style.textContent = `
    body.desktop-chrome #project, body.desktop-chrome #sidebar { padding-top: calc(var(--gutter) + ${STRIP}px); }
    body.desktop-chrome .floating-views { top: calc(var(--gutter) + ${STRIP}px); }
    .desktop-strip { position: fixed; top: 0; left: 0; right: 0; height: ${STRIP}px; z-index: 60; display: flex; align-items: center; padding: 0 16px; }
    .desktop-strip .traffic-group { display: flex; align-items: center; gap: 8px; }
    .desktop-strip .traffic { display: grid; place-items: center; width: 12px; height: 12px; padding: 0; border: 0; border-radius: 50%; cursor: pointer; }
    .desktop-strip .traffic.close { background: var(--traffic-close); }
    .desktop-strip .traffic.min { background: var(--traffic-min); }
    .desktop-strip .traffic.max { background: var(--traffic-max); }
    .desktop-strip .traffic-glyph { font-size: 11px; line-height: 1; color: var(--traffic-glyph); opacity: 0; }
    .desktop-strip .traffic-group:hover .traffic-glyph, .desktop-strip .traffic:focus-visible .traffic-glyph { opacity: 1; }
  `;
  document.head.append(style);

  const strip = document.createElement('div');
  strip.className = 'desktop-strip';
  strip.setAttribute('data-tauri-drag-region', '');
  document.body.classList.add('desktop-chrome');
  document.body.prepend(strip);

  const buttons = [
    { name: 'close', label: 'Close', glyph: '×', cmd: 'plugin:window|close' },
    { name: 'min', label: 'Minimize', glyph: '−', cmd: 'plugin:window|minimize' },
    { name: 'max', label: 'Maximize', glyph: '+', cmd: 'plugin:window|toggle_maximize' },
  ];
  if (!isMac) {
    const group = document.createElement('div');
    group.className = 'traffic-group';
    group.setAttribute('role', 'group');
    group.setAttribute('aria-label', 'Window controls');
    for (const spec of buttons) {
      const button = document.createElement('button');
      button.type = 'button';
      button.className = `traffic ${spec.name}`;
      button.title = spec.label;
      button.setAttribute('aria-label', spec.label);
      button.innerHTML = `<span class="traffic-glyph" aria-hidden="true">${spec.glyph}</span>`;
      button.addEventListener('click', () => invoke(spec.cmd, { label: LABEL }));
      group.append(button);
    }
    strip.append(group);
  }

  if (!isMac) {
    const plain = {
      n: () => invoke('open_projects', { fragment: 'new' }),
      o: () => invoke('open_projects', { fragment: 'open' }),
      ',': () => invoke('open_projects', { fragment: 'settings' }),
      q: () => invoke('plugin:window|close', { label: LABEL }),
    };
    const shifted = { o: () => invoke('open_projects', {}) };
    const editable = (node) => node instanceof Element && Boolean(node.closest('input, textarea, select, [contenteditable]:not([contenteditable="false"])'));
    document.addEventListener('keydown', (event) => {
      if (event.defaultPrevented || editable(event.target)) return;
      if (!(event.ctrlKey || event.metaKey) || event.altKey) return;
      const run = (event.shiftKey ? shifted : plain)[event.key.toLowerCase()];
      if (!run) return;
      event.preventDefault();
      run();
    });
  }

  const head = document.querySelector('.project-head');
  if (!head) return;
  const button = document.createElement('button');
  button.className = 'project-back';
  button.id = 'btn-projects';
  button.title = 'Back to projects';
  button.setAttribute('aria-label', 'Back to projects');
  button.innerHTML = '<iconify-icon icon="pixelarticons:chevron-left"></iconify-icon><span>ALL PROJECTS</span>';
  button.addEventListener('click', () => invoke('open_projects', {}));
  head.after(button);
})();
