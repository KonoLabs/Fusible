(() => {
  const MAX_LINES = 2000;
  const MAX_ORPHAN_RUNS = 8;
  const state = { project: null, board: null, boards: {}, health: {}, runs: [], orphans: {}, activeRun: null, item: null };
  const STATUS_TEXT = { running: 'RUNNING', ok: 'OK', fuse_blown: 'FUSE BLOWN', error: 'FAILED' };
  const GROUPS = [
    ['boards', 'board', 'BOARDS', 'check', 'Check this board'],
    ['firmware', 'firmware', 'FIRMWARE', 'build', 'Build this firmware'],
    ['bodies', 'body', 'BODIES', 'body', 'Build this body'],
  ];

  const byId = (id) => document.getElementById(id);
  const icon = (name) => `<iconify-icon icon="pixelarticons:${name}"></iconify-icon>`;
  const isBusy = () => state.runs.some((run) => run.status === 'running');

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

  function firstBoardName() {
    const first = state.project && state.project.boards[0];
    return (first && first.name) || '';
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
    const selected = state.item === key;
    const name = escapeHtml(item.name);
    return `<div class="tree-item${selected ? ' selected' : ''}" data-kind="${kind}" data-name="${name}">
      <button class="tree-select" ${selected ? 'aria-current="true"' : ''}>
        <span class="tree-dot ${dotClass(kind, item)}"></span>
        <span class="tree-name">${name}</span>
        <span class="tree-sub">${escapeHtml(subLabel(kind, item))}</span>
      </button>
      <button class="tree-run" data-action="${action}" data-target="${name}" title="${hint}" aria-label="${hint}: ${name}">${icon('play')}</button>
    </div>`;
  }

  function renderTree() {
    const nav = byId('project');
    const project = state.project;
    const count = GROUPS.reduce((sum, [list]) => sum + project[list].length, 0);
    const shown = count > 0;
    nav.hidden = !shown;
    document.body.classList.toggle('has-project', !!shown);
    byId('project-title').textContent = (project.name || 'PROJECT').toUpperCase();
    byId('project-tree').innerHTML = GROUPS.filter(([list]) => project[list].length)
      .map(([list, kind, title, action, hint]) => `<div class="tree-group">${title}</div>${project[list]
        .map((item) => rowHtml(kind, item, action, hint)).join('')}`)
      .join('') + bomRowHtml();
  }

  function bomRowHtml() {
    const selected = state.item === 'bom:';
    return `<div class="tree-group">COSTS</div><div class="tree-item${selected ? ' selected' : ''}" data-kind="bom" data-name="">
      <button class="tree-select"${selected ? ' aria-current="true"' : ''}>
        <span class="tree-dot authored"></span><span class="tree-name">PARTS &amp; COSTS</span><span class="tree-sub">BOM</span>
      </button></div>`;
  }

  function showBoard(name) {
    state.board = name;
    resetInspector();
    const data = state.boards[name];
    if (data) {
      updateStudio(data);
      return;
    }
    getJson(`/api/circuit?board=${encodeURIComponent(name)}`)
      .then((fresh) => {
        if (state.board === name) updateStudio(fresh);
      })
      .catch((err) => {
        console.warn('board load failed', err);
        if (state.board === name) showStatus(`BOARD ${name.toUpperCase()} UNAVAILABLE: ${String(err.message || err).toUpperCase()}`);
      });
  }

  function boardOfItem(kind, item) {
    return kind === 'board' ? item.name : item.board || firstBoardName();
  }

  function applyBody(kind, item) {
    if (!window.Tscircuit) return;
    window.Tscircuit.setMechanicalBody(kind === 'body' ? item.name : null, kind === 'body' ? item.kind : '');
  }

  function rememberItem() {
    try {
      const url = new URL(location.href);
      url.searchParams.set('item', state.item);
      history.replaceState(null, '', url);
    } catch (err) {
      console.warn('cannot record the selection in the URL', err);
    }
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
    else if (kind === 'board' && currentView === 'mechanical') switchView('pcb');
    renderTree();
    rememberItem();
  }

  function noteHealth(board, data) {
    state.health[board] = { fuse_blown: !!data.fuse_blown, violations: (data.diagnostics || []).length, known: true };
    if (state.project) renderTree();
  }

  function adopt(run) {
    state.runs.push(run);
    (state.orphans[run.id] || []).forEach((line) => addLine(run, line));
    delete state.orphans[run.id];
    return run;
  }

  function knownRun(id, label) {
    return state.runs.find((r) => r.id === id) || (label ? adopt({ id, label, status: 'running', lines: [] }) : null);
  }

  function renderRunList() {
    const busy = isBusy();
    document.querySelectorAll('#runs-toolbar .btn').forEach((btn) => { btn.disabled = busy; });
    byId('run-list').innerHTML = state.runs.slice().reverse().map((run) => `
      <button class="run-item${run.id === state.activeRun ? ' selected' : ''}" data-run="${escapeHtml(run.id)}" aria-pressed="${run.id === state.activeRun}">
        <span class="run-status ${escapeHtml(run.status)}">${escapeHtml(STATUS_TEXT[run.status] || run.status)}</span>
        <span class="run-label">${escapeHtml(run.label)}</span>
      </button>`).join('') || '<div class="empty-state"><span>NO RUNS YET. PRESS ▶ ON AN ITEM.</span></div>';
  }

  function renderLog() {
    const run = state.runs.find((r) => r.id === state.activeRun);
    byId('run-log').textContent = run ? run.lines.join('\n') : '';
    byId('run-log').scrollTop = byId('run-log').scrollHeight;
  }

  function showRunError(message) {
    const box = byId('run-error');
    box.textContent = String(message || '').toUpperCase();
    box.hidden = !message;
  }

  function addLine(run, line) {
    if (run.lines.length >= MAX_LINES) return;
    run.lines.push(line);
    if (run.id !== state.activeRun) return;
    const log = byId('run-log');
    const atEnd = log.scrollTop + log.clientHeight >= log.scrollHeight - 24;
    log.append(`${run.lines.length > 1 ? '\n' : ''}${line}`);
    if (atEnd) log.scrollTop = log.scrollHeight;
  }

  function appendLog(id, line) {
    const run = state.runs.find((r) => r.id === id);
    if (run) return addLine(run, line);
    const ids = Object.keys(state.orphans);
    if (!(id in state.orphans) && ids.length >= MAX_ORPHAN_RUNS) delete state.orphans[ids[0]];
    const pending = (state.orphans[id] = state.orphans[id] || []);
    if (pending.length < MAX_LINES) pending.push(line);
  }

  function applyRunEvent(data) {
    const run = knownRun(data.run, data.label);
    if (!run) return;
    if (data.label) run.label = data.label;
    if (data.status) run.status = data.status;
    renderRunList();
    window.FirmwareView.onRun(run.status);
  }

  async function postRun(action, target) {
    const response = await fetch('/api/run', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(target ? { action, target } : { action }),
    });
    const body = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(body.error || `HTTP ${response.status}`);
    return body.run;
  }

  async function startRun(action, target) {
    openSidebar('runs');
    let id;
    try {
      id = await postRun(action, target);
    } catch (err) {
      showRunError(err.message || err);
      return;
    }
    showRunError('');
    const run = knownRun(id, target ? `${action} ${target}` : `${action} all`);
    run.label = target ? `${action} ${target}` : `${action} all`;
    state.activeRun = run.id;
    renderRunList();
    renderLog();
    window.FirmwareView.onRun(run.status);
  }

  function onTreeClick(event) {
    const runButton = event.target.closest('.tree-run');
    if (runButton) {
      startRun(runButton.dataset.action, runButton.dataset.target);
      return;
    }
    const row = event.target.closest('.tree-item');
    if (!row) return;
    window.BomView.hide();
    if (row.dataset.kind !== 'bom') return select(row.dataset.kind, row.dataset.name);
    state.item = 'bom:';
    renderTree();
    window.BomView.show();
  }

  function bindEvents() {
    byId('project-tree').addEventListener('click', onTreeClick);
    byId('runs-toolbar').addEventListener('click', (event) => {
      const btn = event.target.closest('.btn');
      if (btn && !btn.disabled) startRun(btn.dataset.action);
    });
    byId('run-list').addEventListener('click', (event) => {
      const row = event.target.closest('.run-item');
      if (!row) return;
      state.activeRun = Number(row.dataset.run);
      showRunError('');
      renderRunList();
      renderLog();
    });
  }

  function initialItem() {
    const wanted = new URLSearchParams(location.search).get('item') || '';
    const [kind, ...rest] = wanted.split(':');
    const name = rest.join(':');
    if (findItem(kind, name)) return [kind, name];
    return ['board', firstBoardName()];
  }

  async function init() {
    try {
      state.project = await getJson('/api/project');
      state.runs = await getJson('/api/runs');
    } catch (err) {
      console.warn('project unavailable', err);
      return;
    }
    state.project.boards.forEach((b) => { b.name = b.name || ''; });
    state.activeRun = state.runs.length ? state.runs[state.runs.length - 1].id : null;
    bindEvents();
    state.board = firstBoardName();
    const [kind, name] = state.project.boards.length ? initialItem() : ['board', ''];
    state.item = `${kind}:${name}`;
    renderTree();
    renderRunList();
    renderLog();
    if (kind !== 'board' || name !== state.board) select(kind, name);
  }

  function handle(data) {
    if (data.event === 'log') return appendLog(data.run, data.line), true;
    if (data.event === 'run') return applyRunEvent(data), true;
    if (data.event === 'sync') return window.Kicad.apply(data), true;
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

  window.Project = { init, handle, board: () => state.board || '', runs: () => state.runs, startRun, isBusy };
})();
