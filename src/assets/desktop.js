(() => {
  if (!window.FUSIBLE_DESKTOP) return;
  const PILL = 78;
  const LABEL = 'main';
  const isMac = /Mac|iPhone|iPad/.test(navigator.platform) || /Mac OS X/.test(navigator.userAgent);
  const invoke = (cmd, args) => {
    const tauri = window.__TAURI_INTERNALS__;
    return tauri ? tauri.invoke(cmd, args) : Promise.resolve();
  };

  const style = document.createElement('style');
  style.textContent = `
    body.window-pill { --chrome-left: ${PILL + 8}px; }
    .desktop-strip { position: fixed; top: 0; left: 0; right: 0; height: var(--gutter); z-index: 60; }
    .window-controls { display: flex; align-items: center; flex: none; width: 54px; height: 28px; margin-left: 8px; }
    .project-head { flex-wrap: wrap; }
    .project-head .window-controls { order: 0; height: 20px; margin-top: 8px; margin-right: auto; }
    .project-head .btn.icon-only { order: 1; }
    .project-head .project-title { order: 2; flex: 1 0 100%; padding: 0 12px 8px; }
    .window-pill-host { top: var(--gutter); left: var(--gutter); width: ${PILL}px; padding: 0; background: none; box-shadow: none; backdrop-filter: none; -webkit-backdrop-filter: none; }
    .window-pill-host[hidden] { display: none; }
    .window-controls .traffic-group { display: flex; align-items: center; gap: 8px; }
    .window-controls .traffic { display: grid; place-items: center; width: 12px; height: 12px; padding: 0; border: 0; border-radius: 50%; cursor: pointer; }
    .window-controls .traffic.close { background: var(--traffic-close); }
    .window-controls .traffic.min { background: var(--traffic-min); }
    .window-controls .traffic.max { background: var(--traffic-max); }
    .window-controls .traffic-glyph { font-size: 11px; line-height: 1; color: var(--traffic-glyph); opacity: 0; }
    .window-controls .traffic-group:hover .traffic-glyph, .window-controls .traffic:focus-visible .traffic-glyph { opacity: 1; }
  `;
  document.head.append(style);

  const drag = (node) => node && node.setAttribute('data-tauri-drag-region', '');
  const strip = document.createElement('div');
  strip.className = 'desktop-strip';
  drag(strip);
  document.body.classList.add('desktop-chrome');
  document.body.prepend(strip);

  const controls = document.createElement('div');
  controls.className = 'window-controls';
  drag(controls);
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
    controls.append(group);
  }

  const project = document.getElementById('project');
  const head = document.querySelector('.project-head');
  const viewport = document.getElementById('viewport-container');
  const pill = document.createElement('div');
  pill.className = 'floating window-pill-host';
  drag(pill);
  if (viewport) viewport.append(pill);
  for (const node of [head, document.getElementById('project-title'), document.querySelector('.sidebar-tabs'), document.querySelector('.floating-views')]) drag(node);

  function place() {
    const inCard = Boolean(project && head && !project.hidden && getComputedStyle(project).display !== 'none');
    if (inCard) head.prepend(controls);
    else pill.append(controls);
    pill.hidden = inCard;
    document.body.classList.toggle('window-pill', !inCard);
  }
  place();
  new MutationObserver(place).observe(document.body, { attributes: true, attributeFilter: ['class'] });
  if (project) new MutationObserver(place).observe(project, { attributes: true, attributeFilter: ['hidden'] });
  matchMedia('(max-width: 900px)').addEventListener('change', place);

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
