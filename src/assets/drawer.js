(() => {
  const NARROW = window.matchMedia('(max-width: 900px)');
  const KEY = 'fusible.treeCollapsed';
  const BEHIND = ['viewport-container', 'sidebar'];
  const body = document.body;
  let opener = null;

  const byId = (id) => document.getElementById(id);

  function fitViewers() {
    window.dispatchEvent(new Event('resize'));
  }

  function setCollapsed(collapsed) {
    body.classList.toggle('tree-collapsed', collapsed);
    try {
      localStorage.setItem(KEY, collapsed ? '1' : '0');
    } catch (err) {
      console.warn('cannot remember the project tree state', err);
    }
    fitViewers();
  }

  function markDialog(open) {
    const nav = byId('project');
    if (open) {
      nav.setAttribute('role', 'dialog');
      nav.setAttribute('aria-modal', 'true');
    } else {
      nav.removeAttribute('role');
      nav.removeAttribute('aria-modal');
    }
    BEHIND.forEach((id) => { byId(id).inert = open; });
  }

  function setDrawer(open) {
    if (open === body.classList.contains('tree-drawer-open')) return;
    if (open) opener = document.activeElement;
    body.classList.toggle('tree-drawer-open', open);
    byId('tree-open').setAttribute('aria-expanded', String(open));
    markDialog(open);
    if (open) {
      byId('tree-close').focus();
      return;
    }
    const target = opener && opener.isConnected && opener.offsetParent !== null ? opener : byId('tree-open');
    opener = null;
    target.focus();
  }

  function isOpen() {
    return NARROW.matches
      ? body.classList.contains('tree-drawer-open')
      : !body.classList.contains('tree-collapsed');
  }

  function toggle(open = !isOpen()) {
    if (NARROW.matches) setDrawer(open);
    else setCollapsed(!open);
  }

  function onKey(event) {
    if (event.ctrlKey || event.metaKey || event.altKey) return;
    if (event.key === 'Escape' && body.classList.contains('tree-drawer-open')) {
      setDrawer(false);
      return;
    }
    if (event.target.closest('input, textarea, select, [contenteditable]')) return;
    if (event.key === '[' && body.classList.contains('has-project')) toggle();
  }

  function bind() {
    byId('tree-open').addEventListener('click', () => toggle(true));
    byId('tree-close').addEventListener('click', () => toggle(false));
    byId('tree-scrim').addEventListener('click', () => setDrawer(false));
    byId('project-tree').addEventListener('click', (event) => {
      if (NARROW.matches && event.target.closest('.tree-item')) setDrawer(false);
    });
    NARROW.addEventListener('change', () => {
      setDrawer(false);
      fitViewers();
    });
    window.addEventListener('keydown', onKey);
  }

  function init() {
    try {
      if (localStorage.getItem(KEY) === '1') body.classList.add('tree-collapsed');
    } catch (err) {
      console.warn('cannot read the project tree state', err);
    }
    bind();
  }

  document.addEventListener('DOMContentLoaded', init);
})();
