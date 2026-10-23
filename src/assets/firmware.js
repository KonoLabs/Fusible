(() => {
  const state = { name: null, data: null, error: null, timer: null, seq: 0 };
  const STATUS_TEXT = { running: 'RUNNING', ok: 'OK', fuse_blown: 'FUSE BLOWN', error: 'FAILED' };

  const view = () => document.getElementById('firmware-view');

  function lastBuild(name) {
    const runs = (window.Project && window.Project.runs()) || [];
    const own = runs.filter((run) => run.label === `build ${name}` || run.label === 'build all');
    return own[own.length - 1];
  }

  function headerHtml(name, data) {
    const build = lastBuild(name);
    const busy = ((window.Project && window.Project.runs()) || []).some((r) => r.status === 'running');
    const meta = data
      ? [data.board && `BOARD ${data.board}`, `MCU ${data.mcu}`, data.framework && `FRAMEWORK ${data.framework}`]
      : [];
    const status = build
      ? `<span class="run-status ${build.status}">${STATUS_TEXT[build.status] || build.status}</span>`
      : '<span class="run-status">NOT BUILT</span>';
    const button = !data || data.buildable
      ? `<button class="btn" id="fw-build" ${busy ? 'disabled' : ''}>BUILD</button>`
      : '';
    return `<div class="fw-head">
      <div class="fw-title">${escapeHtml(name)}</div>
      <div class="fw-meta">${meta.filter(Boolean).map((m) => escapeHtml(m)).join(' · ')}</div>
      <div class="fw-actions">${button}${status}</div>
    </div>`;
  }

  function pinHtml(pin) {
    const signal = pin.const.replace(/^gpio::/, '');
    const remote = pin.remote ? `<span class="fw-remote">${escapeHtml(pin.remote)}</span>` : '';
    const violation = pin.violation ? `<div class="fw-violation">${escapeHtml(pin.violation)}</div>` : '';
    const part = pin.remote ? pin.remote.split('.')[0] : '';
    return `<div class="fw-pin${pin.violation ? ' blown' : ''}" data-net="${escapeHtml(pin.net)}" data-part="${escapeHtml(part)}" title="${escapeHtml(pin.net)}" tabindex="0">
      <span class="fw-pin-name">${escapeHtml(pin.pin)}</span>
      <span class="fw-net">${escapeHtml(signal)}</span>${remote}
      <span class="fw-rust">${escapeHtml(pin.field)}</span>${violation}
    </div>`;
  }

  function groupsHtml(data) {
    const groups = {};
    data.pins.forEach((pin) => { (groups[pin.group] = groups[pin.group] || []).push(pin); });
    const generated = data.pins_rs
      ? `<div class="fw-file">${escapeHtml(data.pins_rs)} · ${data.generated ? 'GENERATED' : 'NOT GENERATED YET'}</div>`
      : '';
    const conflicts = data.violations.length
      ? `<div class="fw-summary blown">${data.violations.length} PIN-MUX CONFLICT${data.violations.length === 1 ? '' : 'S'}</div>`
      : '';
    const body = Object.keys(groups).map((group) => `<div class="tree-group">${escapeHtml(group.toUpperCase())} · ${groups[group].length}</div>${groups[group].map(pinHtml).join('')}`);
    return `${generated}${conflicts}${body.join('') || '<div class="empty-state"><span>NO GPIO PINS MAPPED.</span></div>'}`;
  }

  function render() {
    if (!view()) return;
    if (!state.name) {
      view().innerHTML = '<div class="empty-state"><span>SELECT A FIRMWARE IN THE PROJECT TREE.</span></div>';
      return;
    }
    const detail = state.error
      ? `<div class="fw-error">${escapeHtml(state.error)}</div>`
      : state.data ? groupsHtml(state.data) : '<div class="empty-state"><span>LOADING PIN MAP...</span></div>';
    view().innerHTML = headerHtml(state.name, state.data) + detail;
  }

  async function load(name) {
    const seq = ++state.seq;
    let data = null;
    let error = null;
    try {
      const response = await fetch(`/api/firmware?name=${encodeURIComponent(name)}`, { cache: 'no-store' });
      const body = await response.json();
      if (response.ok) data = body;
      else error = body.error || `HTTP ${response.status}`;
    } catch (err) {
      error = String(err);
    }
    if (seq !== state.seq) return;
    Object.assign(state, { data, error });
    render();
  }

  function reload(delay) {
    clearTimeout(state.timer);
    if (state.name) state.timer = setTimeout(() => load(state.name), delay);
  }

  function show(name) {
    if (name !== state.name) Object.assign(state, { name, data: null, error: null });
    render();
    switchSidebarTab('pins');
    toggleSidebar(true);
    load(name);
  }

  function onClick(event) {
    if (event.target.closest('#fw-build')) {
      if (state.name) window.Project.startRun('build', state.name);
      return;
    }
    const row = event.target.closest('.fw-pin');
    if (!row) return;
    document.querySelectorAll('.fw-pin').forEach((el) => el.classList.toggle('selected', el === row));
    selectPin(row.dataset.net, row.dataset.part);
  }

  function selectPin(net, part) {
    if (net) selectNet(net);
    const inViewer = typeof netsMap !== 'undefined' && netsMap[net];
    if (part && !inViewer) inspectComponent(part, true);
  }

  function onBoard(board) {
    if (state.data && state.data.board === board) reload(300);
  }

  function onRun() {
    render();
    reload(0);
  }

  document.addEventListener('DOMContentLoaded', () => {
    view().addEventListener('click', onClick);
    view().addEventListener('keydown', (event) => {
      const onRow = event.target.classList && event.target.classList.contains('fw-pin');
      if (onRow && (event.key === 'Enter' || event.key === ' ')) {
        event.preventDefault();
        onClick(event);
      }
    });
  });

  window.FirmwareView = { show, onBoard, onRun };
})();
