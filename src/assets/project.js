(() => {
  const state = { project: null, board: null, boards: {}, health: {}, runs: [], activeRun: null, item: null };
  const STATUS_TEXT = { running: 'RUNNING', ok: 'OK', fuse_blown: 'FUSE BLOWN', error: 'FAILED' };
  const GROUPS = [
    ['boards', 'board', 'BOARDS', 'check', 'Check this board'],
    ['firmware', 'firmware', 'FIRMWARE', 'build', 'Build this firmware'],
    ['bodies', 'body', 'BODIES', 'body', 'Build this body'],
  ];

  const byId = (id) => document.getElementById(id);
  const icon = (name) => `<iconify-icon icon="pixelarticons:${name}"></iconify-icon>`;

  async function getJson(url) {
    const response = await fetch(url, { cache: 'no-store' });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
  }

  function itemsOf(kind) {
    const group = GROUPS.find(([, k]) => k === kind);
    return (group && state.project && state.project[group[0]]) || [];
  }

  function findItem(kind, name) {
    return itemsOf(kind).find((item) => item.name === name);
  }

  function healthOf(board) {
    return state.health[board] || { fuse_blown: false, violations: 0, known: false };
  }

  function subLabel(kind, item) {
    if (kind === 'board') {
      const h = healthOf(item.name);
      return h.fuse_blown ? `${h.violations} VIOLATION${h.violations === 1 ? '' : 'S'}` : 'FUSE INTACT';
    }
    if (kind === 'firmware') return `→ ${item.board || 'board'}`;
    if (item.kind === 'authored') return item.exists === false ? 'MISSING' : 'AUTHORED';
    return 'GENERATED';
  }

  function dotClass(kind, item) {
    if (kind === 'board') return healthOf(item.name).fuse_blown ? 'blown' : 'ok';
    return item.kind === 'authored' ? 'authored' : 'generated';
  }

  function rowHtml(kind, item, action, hint) {
    const key = `${kind}:${item.name}`;
    const run = `<button class="tree-run" data-action="${action}" data-target="${escapeHtml(item.name)}" title="${hint}" aria-label="${hint}">${icon('play')}</button>`;
    return `<div class="tree-item${state.item === key ? ' selected' : ''}" data-kind="${kind}" data-name="${escapeHtml(item.name)}" tabindex="0">
      <span class="tree-dot ${dotClass(kind, item)}"></span>
      <span class="tree-name">${escapeHtml(item.name)}</span>
      <span class="tree-sub">${escapeHtml(subLabel(kind, item))}</span>${run}</div>`;
  }

  function renderTree() {
    const nav = byId('project');
    const project = state.project;
    const count = GROUPS.reduce((sum, [list]) => sum + project[list].length, 0);
    const shown = count > 1 || project.firmware.length || project.bodies.length;
    nav.hidden = !shown;
    document.body.classList.toggle('has-project', !!shown);
    byId('project-title').textContent = (project.name || 'PROJECT').toUpperCase();
    byId('project-tree').innerHTML = GROUPS.filter(([list]) => project[list].length)
      .map(([list, kind, title, action, hint]) => `<div class="tree-group">${title}</div>${project[list]
        .map((item) => rowHtml(kind, item, action, hint)).join('')}`)
      .join('');
  }

  function showBoard(name) {
    state.board = name;
    resetInspector();
    const data = state.boards[name];
    if (data) updateStudio(data);
    else getJson(`/api/circuit?board=${encodeURIComponent(name)}`).then(updateStudio).catch(() => {});
  }

  function boardOfItem(kind, item) {
    if (kind === 'board') return item.name;
    return item.board || state.project.boards[0].name || '';
  }

  function applyBody(kind, item) {
    if (!window.Tscircuit) return;
    window.Tscircuit.setMechanicalBody(kind === 'body' ? item.name : null, kind === 'body' ? item.kind : '');
  }

  function select(kind, name) {
    const item = findItem(kind, name);
    if (!item) return;
    state.item = `${kind}:${name}`;
    const board = boardOfItem(kind, item);
    if (board !== state.board) showBoard(board);
    if (kind === 'firmware') window.FirmwareView.show(name);
    applyBody(kind, item);
    if (kind === 'body') switchView('mechanical');
    renderTree();
    try {
      history.replaceState(null, '', `?item=${encodeURIComponent(state.item)}`);
    } catch (err) {}
  }

  function noteHealth(board, data) {
    state.health[board] = { fuse_blown: !!data.fuse_blown, violations: (data.diagnostics || []).length, known: true };
    if (state.project) renderTree();
  }

  function getRun(id) {
    let run = state.runs.find((r) => r.id === id);
    if (!run) {
      run = { id, label: `run ${id}`, status: 'running', lines: [] };
      state.runs.push(run);
    }
    return run;
  }

  function renderRunList() {
    const busy = state.runs.some((r) => r.status === 'running');
    document.querySelectorAll('#runs-toolbar .btn').forEach((btn) => { btn.disabled = busy; });
    byId('run-list').innerHTML = state.runs.slice().reverse().map((run) => `
      <div class="run-item${run.id === state.activeRun ? ' selected' : ''}" data-run="${run.id}">
        <span class="run-status ${run.status}">${STATUS_TEXT[run.status] || run.status}</span>
        <span class="run-label">${escapeHtml(run.label)}</span>
      </div>`).join('') || '<div class="empty-state"><span>NO RUNS YET. PRESS ▶ ON AN ITEM.</span></div>';
  }

  function renderLog() {
    const run = state.runs.find((r) => r.id === state.activeRun);
    byId('run-log').textContent = run ? run.lines.join('\n') : '';
    byId('run-log').scrollTop = byId('run-log').scrollHeight;
  }

  function appendLog(id, line) {
    const run = getRun(id);
    run.lines.push(line);
    if (id !== state.activeRun) return;
    const log = byId('run-log');
    const atEnd = log.scrollTop + log.clientHeight >= log.scrollHeight - 24;
    log.append(`${log.textContent ? '\n' : ''}${line}`);
    if (atEnd) log.scrollTop = log.scrollHeight;
  }

  function finishRun(id, status) {
    getRun(id).status = status;
    renderRunList();
    window.FirmwareView.onRun();
  }

  async function startRun(action, target) {
    const response = await fetch('/api/run', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(target ? { action, target } : { action }),
    });
    const body = await response.json().catch(() => ({}));
    openSidebar('runs');
    if (!response.ok) {
      byId('run-log').textContent = (body.error || `HTTP ${response.status}`).toUpperCase();
      return;
    }
    const run = getRun(body.run);
    run.label = target ? `${action} ${target}` : `${action} all`;
    state.activeRun = run.id;
    renderRunList();
    renderLog();
  }

  function onTreeClick(event) {
    const runButton = event.target.closest('.tree-run');
    if (runButton) {
      event.stopPropagation();
      startRun(runButton.dataset.action, runButton.dataset.target);
      return;
    }
    const row = event.target.closest('.tree-item');
    if (row) select(row.dataset.kind, row.dataset.name);
  }

  function bindEvents() {
    byId('project-tree').addEventListener('click', onTreeClick);
    byId('project-tree').addEventListener('keydown', (event) => {
      const row = event.target.closest('.tree-item');
      if (row && (event.key === 'Enter' || event.key === ' ')) {
        event.preventDefault();
        select(row.dataset.kind, row.dataset.name);
      }
    });
    byId('runs-toolbar').addEventListener('click', (event) => {
      const btn = event.target.closest('.btn');
      if (btn && !btn.disabled) startRun(btn.dataset.action);
    });
    byId('run-list').addEventListener('click', (event) => {
      const row = event.target.closest('.run-item');
      if (!row) return;
      state.activeRun = Number(row.dataset.run);
      renderRunList();
      renderLog();
    });
  }

  function initialItem() {
    const wanted = new URLSearchParams(location.search).get('item') || '';
    const [kind, ...rest] = wanted.split(':');
    const name = rest.join(':');
    if (findItem(kind, name)) return [kind, name];
    const first = state.project.boards[0];
    return ['board', first.name];
  }

  async function init() {
    try {
      state.project = await getJson('/api/project');
      state.runs = await getJson('/api/runs');
    } catch (err) {
      return;
    }
    state.project.boards.forEach((b) => { b.name = b.name || ''; });
    state.activeRun = state.runs.length ? state.runs[state.runs.length - 1].id : null;
    bindEvents();
    state.board = state.project.boards[0] ? state.project.boards[0].name : '';
    const [kind, name] = state.project.boards.length ? initialItem() : ['board', ''];
    state.item = `${kind}:${name}`;
    renderTree();
    renderRunList();
    renderLog();
    if (kind !== 'board' || name !== state.board) select(kind, name);
  }

  function handle(data) {
    if (data.event === 'log') return appendLog(data.run, data.line), true;
    if (data.event === 'run') return finishRun(data.run, data.status), true;
    const board = data.board || '';
    state.boards[board] = data;
    noteHealth(board, data);
    window.FirmwareView.onBoard(board);
    if (state.board === null) state.board = board;
    return board !== state.board;
  }

  window.addEventListener('tscircuit-ready', () => {
    const [kind, ...rest] = (state.item || '').split(':');
    const item = findItem(kind, rest.join(':'));
    if (item) applyBody(kind, item);
  });

  window.Project = { init, handle, board: () => state.board || '' };
  window.Project.runs = () => state.runs;
  window.Project.startRun = startRun;
})();
