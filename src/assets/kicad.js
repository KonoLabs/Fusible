(() => {
  const state = { boards: [], open: false };
  const refs = { chip: null, panel: null };

  const STYLE = `
    .kicad-chip.kicad-in_sync { background: color-mix(in srgb, var(--green) 14%, transparent); color: var(--green); }
    .kicad-chip.kicad-differs { background: color-mix(in srgb, var(--olive) 16%, transparent); color: var(--olive); }
    .kicad-chip.kicad-error { background: color-mix(in srgb, var(--alert) 18%, transparent); color: var(--alert); }
    .kicad-chip[hidden] { display: none; }
    .kicad-notice {
      position: fixed;
      z-index: 40;
      width: 340px;
      max-width: calc(100vw - 24px);
      max-height: 50vh;
      overflow-y: auto;
      padding: 12px;
      background: var(--glass);
      box-shadow: var(--shadow);
      backdrop-filter: blur(12px);
      font-size: 11px;
      letter-spacing: 0.05em;
      user-select: text;
    }
    .kicad-notice[hidden] { display: none; }
    .kicad-notice h3 { font-size: 11px; color: var(--text); margin-bottom: 8px; text-transform: uppercase; }
    .kicad-notice ul { list-style: none; display: flex; flex-direction: column; gap: 4px; }
    .kicad-notice li { color: var(--text-body); overflow-wrap: anywhere; }
    .kicad-notice p { color: var(--text-subtle); margin-top: 8px; line-height: 1.5; }
  `;

  function summary() {
    const errors = state.boards.filter((board) => board.state === 'error');
    if (errors.length) return { kind: 'error', text: 'KICAD: ERROR' };
    const count = state.boards.reduce((sum, board) => sum + board.differences.length, 0);
    if (count === 0) return { kind: 'in_sync', text: 'KICAD: IN SYNC' };
    return { kind: 'differs', text: `KICAD: ${count} ${count === 1 ? 'DIFFERENCE' : 'DIFFERENCES'}` };
  }

  function lines() {
    const several = state.boards.length > 1;
    return state.boards.flatMap((board) => {
      const prefix = several && board.board ? `${board.board}: ` : '';
      if (board.state === 'error') return [`${prefix}${board.message}`];
      return board.differences.map((line) => prefix + line);
    });
  }

  function heading(kind) {
    return kind === 'error' ? 'KiCad could not be compared' : 'KiCad differs from the design';
  }

  function note(kind) {
    if (kind === 'error') return 'Fusible changed nothing. Fix the file and save it again.';
    return 'Fusible only copies part positions from KiCad. It changed nothing for these items: update the TSX to match KiCad, or undo the change in KiCad.';
  }

  function fillPanel(kind) {
    const title = document.createElement('h3');
    title.textContent = heading(kind);
    const list = document.createElement('ul');
    lines().forEach((text) => {
      const item = document.createElement('li');
      item.textContent = text;
      list.appendChild(item);
    });
    const hint = document.createElement('p');
    hint.textContent = note(kind);
    refs.panel.replaceChildren(title, list, hint);
  }

  function place() {
    const rect = refs.chip.getBoundingClientRect();
    const below = rect.top < window.innerHeight / 2;
    refs.panel.style.left = `${Math.max(12, Math.min(rect.left, window.innerWidth - 352))}px`;
    refs.panel.style.top = below ? `${rect.bottom + 6}px` : '';
    refs.panel.style.bottom = below ? '' : `${window.innerHeight - rect.top + 6}px`;
  }

  function render() {
    if (!refs.chip) return;
    const { kind, text } = summary();
    refs.chip.hidden = state.boards.length === 0;
    refs.chip.className = `fuse-badge kicad-chip kicad-${kind}`;
    refs.chip.textContent = text;
    const showable = kind !== 'in_sync' && state.open;
    refs.panel.hidden = !showable;
    if (showable) {
      fillPanel(kind);
      place();
    }
  }

  function toggle() {
    state.open = !state.open;
    render();
  }

  function mount() {
    const anchor = document.getElementById('fuse-status');
    if (!anchor) return;
    const style = document.createElement('style');
    style.textContent = STYLE;
    document.head.appendChild(style);
    refs.chip = document.createElement('button');
    refs.chip.id = 'kicad-status';
    refs.chip.hidden = true;
    refs.chip.title = 'KiCad board compared with the design';
    refs.chip.addEventListener('click', toggle);
    refs.panel = document.createElement('div');
    refs.panel.className = 'kicad-notice';
    refs.panel.hidden = true;
    anchor.insertAdjacentElement('afterend', refs.chip);
    document.body.appendChild(refs.panel);
  }

  function apply(data) {
    state.boards = data.boards || [];
    render();
  }

  async function load() {
    try {
      const response = await fetch('/api/kicad');
      if (response.ok) apply({ boards: await response.json() });
    } catch (err) {
      console.warn('cannot read the KiCad status', err);
    }
  }

  mount();
  load();
  window.Kicad = { apply };
})();
