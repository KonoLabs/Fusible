(() => {
  const POLL_MS = 30000;
  const CHECK_EVERY = 4;
  const GIT_DOWNLOAD = 'https://git-scm.com/downloads';
  const state = { status: null, busy: false, draft: '', notice: null, ticks: 0 };

  const view = () => document.getElementById('sync-view');
  const plural = (n, word) => `${n} ${word}${n === 1 ? '' : 's'}`;

  const STYLE = `
    #sync-view { flex: none; max-height: 58%; overflow-y: auto; border-top: 1px solid var(--border); padding-bottom: 8px; }
    #sync-view[hidden] { display: none; }
    .sync-head { display: flex; align-items: center; gap: 8px; padding: 12px 16px 4px; font-size: 10px; letter-spacing: 0.05em; color: var(--text-subtle); }
    .sync-head .sync-title { flex: 1; }
    .sync-line { padding: 2px 16px 8px; font-size: 11px; line-height: 1.5; color: var(--text); }
    .sync-line.subtle { color: var(--text-subtle); }
    .sync-box { display: block; width: calc(100% - 32px); margin: 0 16px 8px; min-height: 52px; resize: vertical; padding: 6px 8px; background: var(--bg); color: var(--text); border: 1px solid var(--border); font: inherit; font-size: 11px; user-select: text; }
    .sync-box:focus-visible { border-color: var(--green); outline: none; }
    .sync-actions { display: flex; gap: 2px; padding: 0 12px 8px; }
    .sync-actions .btn { flex: 1; justify-content: center; padding: 0 6px; }
    .sync-actions .btn.primary { background: var(--green); color: var(--on-green); }
    .sync-actions .btn:disabled { opacity: 0.4; cursor: default; }
    .sync-note { margin: 0 16px 8px; padding: 6px 8px; font-size: 11px; line-height: 1.5; background: color-mix(in srgb, var(--green) 14%, transparent); }
    .sync-note.error { background: color-mix(in srgb, var(--alert) 10%, transparent); color: var(--alert); }
    .sync-note pre { margin-top: 6px; white-space: pre-wrap; word-break: break-word; color: var(--text-subtle); user-select: text; }
    .sync-card { margin: 0 12px 8px; padding: 8px; border: 1px solid var(--alert); font-size: 11px; line-height: 1.5; }
    .sync-card b { color: var(--alert); word-break: break-all; }
    .sync-card .sync-actions { padding: 8px 0 0; }
    .sync-more { padding: 0 16px; font-size: 11px; }
    .sync-more summary { cursor: pointer; padding: 4px 0; color: var(--text-subtle); font-size: 10px; letter-spacing: 0.05em; }
    .sync-row { display: flex; gap: 8px; padding: 3px 0; line-height: 1.4; }
    .sync-row .sync-kind { flex: none; width: 56px; color: var(--text-subtle); font-size: 9px; }
    .sync-row .sync-what { min-width: 0; word-break: break-all; }
    .sync-row .sync-who { display: block; color: var(--text-subtle); font-size: 10px; }
    .sync-link { color: var(--green); }
  `;

  function ago(seconds) {
    const diff = Math.max(0, Date.now() / 1000 - seconds);
    if (diff < 90) return 'just now';
    const units = [[86400, 'day'], [3600, 'hour'], [60, 'minute']];
    const [size, name] = units.find(([s]) => diff >= s);
    return `${plural(Math.floor(diff / size), name)} ago`;
  }

  async function call(method, url, body) {
    const options = { method, cache: 'no-store' };
    if (method === 'POST') Object.assign(options, { headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body || {}) });
    const response = await fetch(url, options);
    const reply = await response.json().catch(() => ({}));
    return { ok: response.ok, reply };
  }

  function summary(s) {
    const stuck = s.conflicts.length;
    if (stuck) return `${plural(stuck, 'file')} ${stuck === 1 ? 'needs' : 'need'} your decision.`;
    const parts = [];
    if (s.changes.length) parts.push(`${plural(s.changes.length, 'change')} not saved`);
    if (s.has_remote && s.upstream && s.ahead) parts.push(`${plural(s.ahead, 'save')} not shared yet`);
    if (s.behind) parts.push(`${plural(s.behind, 'update')} waiting`);
    if (parts.length) return parts.join(', ') + '.';
    return s.has_remote ? 'Everything is saved and shared.' : 'Everything is saved.';
  }

  function localOnly(s) {
    return s.has_remote ? '' : '<div class="sync-line subtle">Saved on this computer only. Add a remote to share.</div>';
  }

  function conflictCard(s, path) {
    const author = s.conflict_authors[path];
    const who = author ? `${escapeHtml(author)} and you both changed ` : 'Both you and someone else changed ';
    return `<div class="sync-card" data-path="${escapeHtml(path)}">${who}<b>${escapeHtml(path)}</b>. Pick the version to keep.
      <div class="sync-actions">
        <button class="btn" data-keep="mine" title="Keep the version you made">KEEP MINE</button>
        <button class="btn" data-keep="theirs" title="Keep the version from ${escapeHtml(author || 'the other person')}">KEEP THEIRS</button>
      </div></div>`;
  }

  function changesHtml(s) {
    if (!s.changes.length) return '';
    const rows = s.changes.map((c) => `<div class="sync-row"><span class="sync-kind">${escapeHtml(c.kind.toUpperCase())}</span><span class="sync-what">${escapeHtml(c.path)}</span></div>`);
    return `<details class="sync-more"><summary>CHANGES (${s.changes.length})</summary>${rows.join('')}</details>`;
  }

  function savesHtml(s) {
    if (!s.last_saves.length) return '';
    const rows = s.last_saves.map((v) => `<div class="sync-row" title="${escapeHtml(v.files.join('\n'))}"><span class="sync-what">${escapeHtml(v.message)}<span class="sync-who">${escapeHtml(v.author)} · ${escapeHtml(ago(v.when))}</span></span></div>`);
    return `<details class="sync-more"><summary>SAVES (${s.last_saves.length})</summary>${rows.join('')}</details>`;
  }

  function noticeHtml() {
    const n = state.notice;
    if (!n) return '';
    const details = n.details ? `<details><summary>DETAILS</summary><pre>${escapeHtml(n.details)}</pre></details>` : '';
    return `<div class="sync-note${n.error ? ' error' : ''}" role="${n.error ? 'alert' : 'status'}">${escapeHtml(n.text)}${details}</div>`;
  }

  function missingHtml() {
    return `<div class="sync-head"><span class="sync-title">SYNC</span></div>
      <div class="sync-line">Install Git to sync your work.</div>
      <div class="sync-line subtle">Git is the free program that shares your project with your team. Install it, then come back.</div>
      <div class="sync-line"><a class="sync-link" href="${GIT_DOWNLOAD}" target="_blank" rel="noopener noreferrer">Download Git</a></div>
      <div class="sync-actions"><button class="btn" id="sync-recheck">${state.busy ? 'CHECKING...' : 'RE-CHECK'}</button></div>`;
  }

  function panelHtml(s) {
    const stuck = s.conflicts.length > 0;
    const off = state.busy || stuck ? 'disabled' : '';
    const noUpdates = state.busy || stuck || !s.upstream ? 'disabled' : '';
    const warn = s.fetch_error ? `<div class="sync-line subtle">Could not check for updates: ${escapeHtml(s.fetch_error)}</div>` : '';
    return `<div class="sync-head"><span class="sync-title">SYNC</span>
        <button class="btn" id="sync-refresh" title="Check for updates" ${state.busy ? 'disabled' : ''}>CHECK</button></div>
      <div class="sync-line" id="sync-status">${escapeHtml(summary(s))}</div>${localOnly(s)}${warn}
      ${s.conflicts.map((p) => conflictCard(s, p)).join('')}
      <textarea class="sync-box" id="sync-message" rows="2" placeholder="What did you change?" aria-label="What did you change?"></textarea>
      <div class="sync-actions">
        <button class="btn primary" id="sync-save" ${off}>${state.busy ? 'WORKING...' : 'SAVE'}</button>
        <button class="btn" id="sync-update" ${noUpdates}>GET UPDATES</button>
      </div>${noticeHtml()}${changesHtml(s)}${savesHtml(s)}`;
  }

  function render() {
    const root = view();
    const s = state.status;
    if (!root) return;
    const shown = !!(s && s.repo);
    root.hidden = !shown;
    if (!shown) return;
    const focused = document.activeElement && document.activeElement.id;
    const opened = [...root.querySelectorAll('details.sync-more[open]')].map((d) => d.querySelector('summary').textContent.split(' ')[0]);
    root.innerHTML = s.available === false ? missingHtml() : panelHtml(s);
    root.querySelectorAll('details.sync-more').forEach((d) => {
      d.open = opened.includes(d.querySelector('summary').textContent.split(' ')[0]);
    });
    const box = byIdIn(root, 'sync-message');
    if (box) {
      box.value = state.draft;
      if (focused === 'sync-message') box.focus();
    }
  }

  const byIdIn = (root, id) => root.querySelector(`#${id}`);

  async function load(check) {
    if (state.busy) return;
    try {
      const { ok, reply } = await call('GET', `/api/git${check ? '?check=1' : ''}`);
      if (ok) state.status = reply;
    } catch (err) {
      console.warn('sync status unavailable', err);
    }
    render();
  }

  async function act(url, body) {
    state.busy = true;
    state.notice = null;
    render();
    try {
      const { ok, reply } = await call('POST', url, body);
      if (reply.status) state.status = reply.status;
      state.notice = ok ? { text: reply.message } : { text: reply.error || 'That did not work.', details: reply.details, error: true };
      if (ok && url.endsWith('/save')) state.draft = '';
    } catch (err) {
      state.notice = { text: 'Cannot reach Fusible Studio.', details: String(err), error: true };
    }
    state.busy = false;
    render();
  }

  function onClick(event) {
    const button = event.target.closest('button');
    if (!button || button.disabled) return;
    if (button.id === 'sync-save') act('/api/git/save', { message: state.draft });
    else if (button.id === 'sync-update') act('/api/git/update');
    else if (button.id === 'sync-refresh' || button.id === 'sync-recheck') recheck();
    else if (button.dataset.keep) {
      const path = button.closest('.sync-card').dataset.path;
      act('/api/git/resolve', { path, keep: button.dataset.keep });
    }
  }

  async function recheck() {
    state.notice = null;
    await load(true);
  }

  function start() {
    if (!view()) return;
    const style = document.createElement('style');
    style.textContent = STYLE;
    document.head.appendChild(style);
    view().addEventListener('click', onClick);
    view().addEventListener('input', (event) => {
      if (event.target.id === 'sync-message') state.draft = event.target.value;
    });
    load(true);
    setInterval(() => load(++state.ticks % CHECK_EVERY === 0), POLL_MS);
  }

  window.SyncPanel = { reload: () => load(true) };
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', start);
  else start();
})();
