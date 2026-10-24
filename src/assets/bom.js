(() => {
  const state = { data: null, error: '', timer: null, seq: 0 };
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

  function costCell(line) {
    if (line.kind === 'print') {
      const guess = line.estimated ? ` placeholder="~${line.grams}" title="Estimated from the model. Type a weight to override."` : '';
      return `<div class="bom-unit">${input('grams', line.estimated ? null : line.grams, `type="number" min="0" step="1" class="bom-num"${guess}`)}<span>g</span></div>`;
    }
    const guess = line.estimated ? ` placeholder="~${price(line.unit_price)}" title="Estimated price. Type one to override."` : '';
    return input('unit_price', line.estimated ? null : line.unit_price, `type="number" min="0" step="0.01" class="bom-num"${guess}`);
  }

  function nameCell(line) {
    if (line.kind === 'manual') return input('description', line.description, 'class="bom-wide"');
    const refs = [line.mpn, line.designators.join(' ')].filter(Boolean).join(' · ');
    return `<div class="bom-name">${escapeHtml(line.description)}</div><div class="bom-refs">${escapeHtml(refs)}</div>`;
  }

  function linkCell(line) {
    if (line.kind === 'print') return '';
    const open = safeUrl(line.url) ? `<a class="btn icon-only" href="${escapeHtml(line.url)}" target="_blank" rel="noopener noreferrer" title="Open link" aria-label="Open link"><iconify-icon icon="pixelarticons:external-link"></iconify-icon></a>` : '';
    return `<div class="bom-link">${input('url', line.url, 'placeholder="https://"')}${open}</div>`;
  }

  function rowHtml(line) {
    const remove = line.kind === 'manual' ? '<button class="btn icon-only" data-remove title="Remove part" aria-label="Remove part">&times;</button>' : '';
    return `<tr data-id="${escapeHtml(line.id)}" data-kind="${line.kind}"${line.picked ? '' : ' class="off"'}>
      <td><input type="checkbox" data-f="picked" ${line.picked ? 'checked' : ''} aria-label="Pick"></td>
      <td class="bom-r">${input('quantity', line.quantity, 'type="number" min="0" step="1" class="bom-num"')}</td>
      <td>${nameCell(line)}</td>
      <td>${line.kind === 'print' ? '' : input('supplier', line.supplier)}</td>
      <td>${linkCell(line)}</td>
      <td class="bom-r">${costCell(line)}</td>
      <td class="bom-total bom-r">${line.estimated && line.total != null ? '~' : ''}${price(line.total)}</td>
      <td>${remove}</td>
    </tr>`;
  }

  function summaryHtml(report) {
    const t = report.totals;
    const unit = `<i>${escapeHtml(report.currency)}</i>`;
    const cell = (label, value, cls = '') => `<div class="${cls}"><span>${label}</span><b>${price(value)} ${unit}</b></div>`;
    const missing = t.unpriced ? `<div class="bom-warn">${t.unpriced} PICKED LINE(S) HAVE NO PRICE YET</div>` : '';
    const guessed = t.estimated ? `<div class="bom-warn bom-note">${t.estimated} PICKED LINE(S) USE AN ESTIMATE (~). TYPE A PRICE TO REPLACE IT.</div>` : '';
    return `<div class="bom-sum">${cell('ELECTRONICS', t.electronics)}${cell('3D PRINTS', t.printed)}${cell('OTHER PARTS', t.manual)}${cell('TOTAL', t.total, 'bom-grand')}</div>${missing}${guessed}`;
  }

  const GROUPS = [['component', 'ELECTRONICS'], ['print', '3D PRINTS'], ['manual', 'OTHER PARTS']];

  function groupSum(report, kind) {
    const sum = report.lines.filter((l) => l.kind === kind && l.picked && l.total != null).reduce((n, l) => n + l.total, 0);
    return `${price(sum)} ${escapeHtml(report.currency)}`;
  }

  function groupsHtml(report) {
    return GROUPS.map(([kind, label]) => {
      const lines = report.lines.filter((l) => l.kind === kind);
      if (!lines.length) return '';
      return `<tr class="bom-group"><td colspan="6">${label} &middot; ${lines.length}</td><td class="bom-r" colspan="2"><b data-sum="${kind}">${groupSum(report, kind)}</b></td></tr>${lines.map(rowHtml).join('')}`;
    }).join('');
  }

  const CURRENCIES = ['EUR', 'USD', 'GBP', 'CHF', 'CAD', 'AUD', 'JPY', 'CNY', 'SEK', 'NOK', 'DKK', 'PLN'];

  function currencyOptions(current) {
    const all = CURRENCIES.includes(current) ? CURRENCIES : [current, ...CURRENCIES];
    return all.map((c) => `<option${c === current ? ' selected' : ''}>${escapeHtml(c)}</option>`).join('');
  }

  function settingsHtml(data) {
    return `<div class="bom-settings">
      <label>CURRENCY <select id="bom-currency" title="Only the label changes. Prices are not converted.">${currencyOptions(data.report.currency)}</select></label>
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
    const rows = groupsHtml(data.report) || '<tr><td colspan="8" class="bom-empty">NO PARTS YET. ADD ONE ABOVE.</td></tr>';
    view().innerHTML = `<div class="bom-head"><div class="fw-title">PARTS &amp; COSTS</div>
      <div class="fw-meta">SAVED TO bom.yml</div></div>
      ${notes.map((n) => `<div class="bom-warn">${escapeHtml(n)}</div>`).join('')}
      <div class="fw-error" id="bom-error">${escapeHtml(state.error)}</div>
      <div id="bom-summary">${summaryHtml(data.report)}</div>${settingsHtml(data)}
      <div class="bom-scroll"><table class="bom-table"><thead><tr><th aria-label="Pick"></th><th class="bom-r">QTY</th><th>PART</th><th>SUPPLIER</th><th>LINK</th><th class="bom-r">UNIT</th><th class="bom-r">TOTAL</th><th></th></tr></thead><tbody>${rows}</tbody></table></div>`;
  }

  function patch(report) {
    document.getElementById('bom-summary').innerHTML = summaryHtml(report);
    view().querySelectorAll('[data-sum]').forEach((cell) => {
      cell.innerHTML = groupSum(report, cell.dataset.sum);
    });
    report.lines.forEach((line) => {
      const row = view().querySelector(`tr[data-id="${CSS.escape(line.id)}"]`);
      if (!row) return;
      row.classList.toggle('off', !line.picked);
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
    if (!document.body.classList.contains('bom-open')) return;
    document.body.classList.remove('bom-open');
    view().hidden = true;
    window.dispatchEvent(new Event('resize'));
  }

  document.addEventListener('DOMContentLoaded', () => {
    view().addEventListener('change', onChange);
    view().addEventListener('click', onClick);
    view().addEventListener('keydown', onKey);
  });

  window.BomView = { show, hide };
})();
