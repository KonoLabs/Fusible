(() => {
  const state = { data: null, error: '', timer: null, seq: 0 };
  const FIND = [
    ['LCSC', 'https://www.lcsc.com/search?q='],
    ['DIGIKEY', 'https://www.digikey.com/en/products/result?keywords='],
    ['MOUSER', 'https://www.mouser.com/c/?q='],
  ];
  const view = () => document.getElementById('bom-view');
  const safeUrl = (url) => (/^https?:\/\//i.test(url || '') ? url : '');
  const price = (n) => (n == null ? '-' : n.toFixed(2));
  const entries = () => state.data.file;

  async function request(method, body) {
    const options = { method, cache: 'no-store' };
    if (body) Object.assign(options, { headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
    const response = await fetch('/api/bom', options);
    const reply = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(reply.error || `HTTP ${response.status}`);
    return reply;
  }

  function input(field, value, attrs = '') {
    const shown = value == null ? '' : value;
    return `<input data-f="${field}" value="${escapeHtml(String(shown))}" ${attrs}>`;
  }

  function finder(line) {
    const query = encodeURIComponent(line.mpn || line.description);
    return FIND.map(([label, base]) => `<a href="${base}${query}" target="_blank" rel="noopener noreferrer">${label}</a>`).join(' · ');
  }

  function costCell(line) {
    if (line.kind === 'print') {
      const guess = line.estimated ? ` placeholder="~${line.grams}" title="Estimated from the model. Type a weight to override."` : ' placeholder="g"';
      return `${input('grams', line.estimated ? null : line.grams, `type="number" min="0" step="1" class="bom-num"${guess}`)} g`;
    }
    const guess = line.estimated ? ` placeholder="~${price(line.unit_price)}" title="Estimated price. Type one to override."` : '';
    return input('unit_price', line.estimated ? null : line.unit_price, `type="number" min="0" step="0.01" class="bom-num"${guess}`);
  }

  function nameCell(line) {
    if (line.kind === 'manual') return input('description', line.description, 'class="bom-wide"');
    const refs = [line.mpn, line.designators.join(' ')].filter(Boolean).join(' · ');
    return `<div>${escapeHtml(line.description)}</div><div class="bom-refs">${escapeHtml(refs)}</div>`;
  }

  function linkCell(line) {
    if (line.kind === 'print') return '';
    const open = safeUrl(line.url) ? `<a href="${escapeHtml(line.url)}" target="_blank" rel="noopener noreferrer">OPEN</a>` : '';
    return `${input('url', line.url, 'class="bom-wide" placeholder="https://"')} ${open}<div class="bom-refs">${finder(line)}</div>`;
  }

  function rowHtml(line) {
    const remove = line.kind === 'manual' ? '<button class="btn icon-only" data-remove title="Remove part" aria-label="Remove part">&times;</button>' : '';
    return `<tr data-id="${escapeHtml(line.id)}" data-kind="${line.kind}">
      <td><input type="checkbox" data-f="picked" ${line.picked ? 'checked' : ''} aria-label="Pick"></td>
      <td>${input('quantity', line.quantity, 'type="number" min="0" step="1" class="bom-num"')}</td>
      <td>${nameCell(line)}</td>
      <td>${line.kind === 'print' ? '' : input('supplier', line.supplier)}</td>
      <td>${linkCell(line)}</td>
      <td>${costCell(line)}</td>
      <td class="bom-total">${line.estimated && line.total != null ? '~' : ''}${price(line.total)}</td>
      <td>${remove}</td>
    </tr>`;
  }

  function summaryHtml(report) {
    const t = report.totals;
    const cell = (label, value) => `<div><span>${label}</span><b>${price(value)} ${escapeHtml(report.currency)}</b></div>`;
    const missing = t.unpriced ? `<div class="bom-warn">${t.unpriced} PICKED LINE(S) HAVE NO PRICE YET</div>` : '';
    const guessed = t.estimated ? `<div class="bom-warn">${t.estimated} PICKED LINE(S) USE AN ESTIMATE (~). TYPE A PRICE TO REPLACE IT.</div>` : '';
    return `<div class="bom-sum">${cell('ELECTRONICS', t.electronics)}${cell('PRINTED', t.printed)}${cell('OTHER', t.manual)}${cell('TOTAL', t.total)}</div>${missing}${guessed}`;
  }

  function settingsHtml(data) {
    return `<div class="bom-settings">
      <label>CURRENCY <input id="bom-currency" value="${escapeHtml(data.report.currency)}" maxlength="8" class="bom-num"></label>
      <label>FILAMENT PER KG <input id="bom-kg" type="number" min="0" step="0.5" value="${data.report.price_per_kg}" class="bom-num"></label>
      <label class="bom-add">ADD A PART <input id="bom-new" placeholder="M3 screw" class="bom-wide"></label>
      <button class="btn" id="bom-add">ADD</button>
    </div>`;
  }

  function render() {
    const data = state.data;
    if (!view()) return;
    if (!data) {
      view().innerHTML = `<div class="empty-state"><span>${state.error ? escapeHtml(state.error.toUpperCase()) : 'LOADING PARTS...'}</span></div>`;
      return;
    }
    const notes = [!data.editable && 'NO fusible.yml: EDITS CANNOT BE SAVED', data.pending && `${data.pending} BOARD(S) STILL BUILDING`].filter(Boolean);
    const rows = data.report.lines.map(rowHtml).join('') || '<tr><td colspan="8" class="bom-refs">NO PARTS YET.</td></tr>';
    view().innerHTML = `<div class="bom-head"><div class="fw-title">PARTS &amp; COSTS</div>
      <div class="fw-meta">STORED IN bom.yml NEXT TO fusible.yml</div></div>
      ${notes.map((n) => `<div class="bom-warn">${escapeHtml(n)}</div>`).join('')}
      <div class="fw-error" id="bom-error">${escapeHtml(state.error)}</div>
      <div id="bom-summary">${summaryHtml(data.report)}</div>${settingsHtml(data)}
      <div class="bom-scroll"><table class="bom-table"><thead><tr><th>PICK</th><th>QTY</th><th>PART</th><th>SUPPLIER</th><th>LINK</th><th>UNIT</th><th>TOTAL</th><th></th></tr></thead><tbody>${rows}</tbody></table></div>`;
  }

  function patch(report) {
    document.getElementById('bom-summary').innerHTML = summaryHtml(report);
    report.lines.forEach((line) => {
      const row = view().querySelector(`tr[data-id="${CSS.escape(line.id)}"]`);
      if (!row) return;
      row.querySelector('.bom-total').textContent = (line.estimated && line.total != null ? '~' : '') + price(line.total);
    });
  }

  function prune() {
    const kinds = Object.fromEntries(state.data.report.lines.map((l) => [l.id, l.kind]));
    for (const [id, entry] of Object.entries(entries().parts)) {
      if (kinds[id] === 'component' && !Object.keys(entry).length) delete entries().parts[id];
    }
    for (const [id, entry] of Object.entries(entries().prints)) {
      if (!Object.keys(entry).length) delete entries().prints[id];
    }
  }

  async function save(rerender) {
    const seq = ++state.seq;
    prune();
    try {
      const reply = await request('POST', entries());
      if (seq !== state.seq) return;
      state.error = '';
      state.data = reply;
      if (rerender) render();
      else patch(reply.report);
    } catch (err) {
      state.error = String(err.message || err);
      const box = document.getElementById('bom-error');
      if (box) box.textContent = state.error.toUpperCase();
    }
  }

  function later(rerender = false) {
    clearTimeout(state.timer);
    state.timer = setTimeout(() => save(rerender), 300);
  }

  function setField(line, field, control) {
    const isPrint = line.kind === 'print';
    const bag = isPrint ? entries().prints : entries().parts;
    const key = isPrint ? line.id.slice('print-'.length) : line.id;
    const entry = (bag[key] = bag[key] || {});
    let value = control.type === 'number' ? (control.value === '' ? undefined : Number(control.value)) : control.value.trim() || undefined;
    if (field === 'picked') value = control.checked ? undefined : false;
    if (field === 'quantity' && line.kind === 'component' && value === line.designators.length) value = undefined;
    if (value === undefined) delete entry[field];
    else entry[field] = value;
  }

  function onChange(event) {
    const control = event.target;
    if (control.id === 'bom-currency') entries().currency = control.value.trim() || undefined;
    else if (control.id === 'bom-kg') entries().print.price_per_kg = Number(control.value) || 0;
    else {
      const row = control.closest('tr[data-id]');
      const line = row && state.data.report.lines.find((l) => l.id === row.dataset.id);
      if (!line || !control.dataset.f) return;
      setField(line, control.dataset.f, control);
    }
    later(control.id === 'bom-currency' || control.dataset.f === 'grams');
  }

  function addPart() {
    const field = document.getElementById('bom-new');
    const name = field.value.trim();
    const id = name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
    if (!id) return;
    if (state.data.report.lines.some((l) => l.id === id)) {
      state.error = `"${id}" is already in the list`;
      return render();
    }
    entries().parts[id] = { description: name, quantity: 1 };
    save(true);
  }

  function onClick(event) {
    if (event.target.closest('#bom-add')) return addPart();
    const remove = event.target.closest('[data-remove]');
    if (!remove) return;
    delete entries().parts[remove.closest('tr').dataset.id];
    save(true);
  }

  function onKey(event) {
    if (event.key === 'Enter' && event.target.id === 'bom-new') addPart();
  }

  async function show() {
    document.body.classList.add('bom-open');
    view().hidden = false;
    state.data = null;
    render();
    try {
      state.data = await request('GET');
      state.error = '';
    } catch (err) {
      state.error = String(err.message || err);
    }
    render();
  }

  function hide() {
    document.body.classList.remove('bom-open');
    view().hidden = true;
  }

  document.addEventListener('DOMContentLoaded', () => {
    view().addEventListener('change', onChange);
    view().addEventListener('click', onClick);
    view().addEventListener('keydown', onKey);
  });

  window.BomView = { show, hide };
})();
