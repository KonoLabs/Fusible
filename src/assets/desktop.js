(() => {
  if (!window.FUSIBLE_DESKTOP) return;
  const dock = document.querySelector('.floating-dock');
  if (!dock) return;
  const divider = document.createElement('span');
  divider.className = 'dock-divider';
  const button = document.createElement('button');
  button.className = 'btn';
  button.id = 'btn-projects';
  button.title = 'Back to projects';
  button.setAttribute('aria-label', 'Back to projects');
  button.innerHTML = '<iconify-icon icon="pixelarticons:folder"></iconify-icon> <span class="btn-label">PROJECTS</span>';
  button.addEventListener('click', () => {
    const tauri = window.__TAURI_INTERNALS__;
    if (tauri) tauri.invoke('open_projects');
  });
  dock.append(divider, button);
})();
