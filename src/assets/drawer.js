(() => {
  const NARROW = window.matchMedia('(max-width: 900px)');
  const KEY = 'fusible.treeCollapsed';
  const body = document.body;

  function fitViewers() {
    window.dispatchEvent(new Event('resize'));
  }

  function setCollapsed(collapsed) {
    body.classList.toggle('tree-collapsed', collapsed);
    try {
      localStorage.setItem(KEY, collapsed ? '1' : '0');
    } catch (err) {}
    fitViewers();
  }

  function setDrawer(open) {
    body.classList.toggle('tree-drawer-open', open);
    const toggle = document.getElementById('tree-open');
    if (toggle) toggle.setAttribute('aria-expanded', String(open));
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
    if (event.target.closest('input, textarea, select, [contenteditable]')) return;
    if (event.key === '[' && body.classList.contains('has-project')) {
      toggle();
    } else if (event.key === 'Escape' && body.classList.contains('tree-drawer-open')) {
      setDrawer(false);
    }
  }

  function bind() {
    document.getElementById('tree-open').addEventListener('click', () => toggle(true));
    document.getElementById('tree-close').addEventListener('click', () => toggle(false));
    document.getElementById('tree-scrim').addEventListener('click', () => setDrawer(false));
    document.getElementById('project-tree').addEventListener('click', (event) => {
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
    } catch (err) {}
    bind();
  }

  document.addEventListener('DOMContentLoaded', init);
  window.TreeDrawer = { toggle };
})();
