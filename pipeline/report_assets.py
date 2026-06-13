"""Static assets for the HTML report — CSS styles and JavaScript."""

# ── CSS ────────────────────────────────────────────────────────────────────────

STYLES = """
:root {
  --badge-green: #2e7d32;
  --badge-red: #c62828;
  --badge-amber: #e65100;
  --badge-gray: #757575;
}
body {
  margin: 0;
  font-family: "Fira Sans", Arial, sans-serif;
  display: flex;
  min-height: 100vh;
}
.sidebar {
  width: 260px;
  flex-shrink: 0;
  background: #f5f5f5;
  padding: 16px;
  overflow-y: auto;
  border-right: 1px solid #e0e0e0;
  font-size: 14px;
}
.sidebar h2 {
  font-size: 16px;
  margin: 0 0 12px 0;
  color: #333;
}
.sidebar details {
  margin: 4px 0;
}
.sidebar summary {
  cursor: pointer;
  font-weight: 600;
  color: #555;
  padding: 2px 0;
}
.sidebar ul {
  list-style: none;
  padding-left: 16px;
  margin: 2px 0;
}
.sidebar li {
  padding: 1px 0;
}
.sidebar a {
  color: #c25c00;
  text-decoration: none;
}
.sidebar a:hover {
  text-decoration: underline;
}
.sidebar-type {
  font-size: 12px;
  color: #888;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.main-content {
  flex: 1;
  padding: 24px 40px;
  max-width: 960px;
}
h1 {
  font-size: 24px;
  border-bottom: 1px solid #ddd;
  padding-bottom: 8px;
  margin-bottom: 20px;
}
.breadcrumbs {
  font-size: 14px;
  margin-bottom: 16px;
  color: #555;
}
.breadcrumbs a {
  color: #c25c00;
  text-decoration: none;
}
.breadcrumbs a:hover {
  text-decoration: underline;
}
.summary-row {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  margin-bottom: 24px;
}
.summary-card {
  padding: 12px 16px;
  border-radius: 6px;
  text-align: center;
  font-size: 14px;
  color: white;
}
.summary-card.pass { background: var(--badge-green); }
.summary-card.fail { background: var(--badge-red); }
.summary-card.timeout { background: var(--badge-amber); }
.summary-card.notrun { background: var(--badge-gray); }
.summary-card .count {
  display: block;
  font-size: 28px;
  font-weight: 700;
}
.index-module-list {
  margin: 16px 0;
}
.index-module-list details {
  margin: 8px 0;
}
.index-module-list summary {
  cursor: pointer;
  font-size: 16px;
  font-weight: 600;
  color: #333;
  padding: 4px 0;
}
.index-module-list ul {
  list-style: none;
  padding-left: 24px;
  margin: 4px 0;
}
.index-module-list li {
  padding: 4px 0;
  font-size: 14px;
}
.index-module-list a {
  color: #c25c00;
  text-decoration: none;
}
.index-module-list a:hover {
  text-decoration: underline;
}
.index-module-list .fn-count {
  color: #888;
  font-size: 12px;
  margin-left: 6px;
}
.section-header {
  font-size: 18px;
  margin: 24px 0 12px 0;
  padding-bottom: 4px;
  border-bottom: 1px solid #eee;
  color: #333;
}
.spec-card {
  margin: 16px 0;
  padding: 12px 16px;
  border: 1px solid #e8e8e8;
  border-radius: 6px;
}
.spec-heading {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 16px;
  font-weight: 600;
}
a.fn {
  color: #c25c00;
  font-family: "Fira Mono", monospace;
}
.src-link {
  font-size: 11px;
  color: #888;
  text-decoration: none;
  font-family: "Fira Mono", monospace;
  margin-left: 4px;
  cursor: pointer;
}
.src-link:hover {
  color: #c25c00;
  text-decoration: underline;
}
.verification-badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.3px;
  text-transform: uppercase;
  color: white;
}
.badge-pass  { background: var(--badge-green); }
.badge-fail  { background: var(--badge-red); }
.badge-timeout { background: var(--badge-amber); }
.badge-not-run { background: var(--badge-gray); }
.docblock {
  margin: 8px 0 8px 24px;
  color: #555;
  font-style: italic;
}
.sig {
  margin: 4px 0 4px 24px;
  font-size: 13px;
  color: #666;
}
.sig code {
  font-family: "Fira Mono", monospace;
}
.source-details {
  margin-top: 8px;
}
.source-details summary {
  cursor: pointer;
  color: #1a73e8;
  font-size: 13px;
}
.source-details pre {
  background: #f8f8f8;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  padding: 14px;
  overflow-x: auto;
  font-size: 13px;
  line-height: 1.5;
}
.orig-source summary {
  color: #388e3c;
}
.error-details {
  margin-top: 10px;
}
.error-details summary {
  cursor: pointer;
  color: #c62828;
  font-size: 13px;
  font-weight: 600;
}
.error-output {
  background: #fff5f5;
  border: 1px solid #ffcdd2;
  border-radius: 4px;
  padding: 12px;
  margin-top: 8px;
  overflow-x: auto;
  font-size: 12px;
  line-height: 1.5;
  max-height: 400px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-all;
}
.selection-toolbar {
  position: sticky;
  top: 0;
  background: #fff;
  border-bottom: 2px solid #c25c00;
  padding: 10px 16px;
  margin-bottom: 20px;
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  z-index: 100;
}
.selection-toolbar button {
  padding: 6px 14px;
  border: 1px solid #ccc;
  border-radius: 4px;
  background: #fafafa;
  cursor: pointer;
  font-size: 13px;
}
.selection-toolbar button:hover { background: #eee; }
.selection-toolbar button.primary {
  background: #c25c00;
  color: white;
  border-color: #c25c00;
}
.selection-toolbar button.primary:hover { background: #a04e00; }
.selection-count {
  font-size: 13px;
  color: #555;
  font-weight: 600;
  min-width: 90px;
}
.fn-checkbox {
  margin-right: 6px;
  accent-color: #c25c00;
  width: 16px;
  height: 16px;
  cursor: pointer;
  vertical-align: middle;
  flex-shrink: 0;
}
.modal-overlay {
  display: none;
  position: fixed;
  top: 0; left: 0; right: 0; bottom: 0;
  background: rgba(0,0,0,0.5);
  z-index: 1000;
  justify-content: center;
  align-items: flex-start;
  padding-top: 80px;
}
.modal-overlay.active { display: flex; }
.modal-box {
  background: white;
  border-radius: 8px;
  box-shadow: 0 4px 24px rgba(0,0,0,0.2);
  max-width: 700px;
  width: 90%;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
}
.modal-header {
  padding: 16px 20px;
  border-bottom: 1px solid #e0e0e0;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.modal-header h3 { margin: 0; font-size: 16px; }
.modal-close {
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  color: #888;
}
.modal-close:hover { color: #333; }
.modal-body { padding: 16px 20px; overflow-y: auto; }
.modal-body p.path-hint {
  font-size: 12px;
  color: #888;
  margin: 0 0 8px 0;
  font-family: monospace;
  background: #f5f5f5;
  padding: 4px 8px;
  border-radius: 3px;
}
.modal-body textarea {
  width: 100%;
  height: 300px;
  font-family: "Fira Mono", monospace;
  font-size: 13px;
  border: 1px solid #ddd;
  border-radius: 4px;
  padding: 10px;
  resize: vertical;
  box-sizing: border-box;
}
.modal-footer {
  padding: 12px 20px;
  border-top: 1px solid #e0e0e0;
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}
.modal-footer button {
  padding: 8px 18px;
  border: 1px solid #ccc;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
}
.modal-footer button.primary {
  background: #2e7d32;
  color: white;
  border-color: #2e7d32;
}
.modal-footer button.primary:hover { background: #1b5e20; }
.modal-footer button:hover { background: #eee; }
.copy-toast {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  background: #333;
  color: white;
  padding: 10px 24px;
  border-radius: 6px;
  font-size: 14px;
  z-index: 2000;
  opacity: 0;
  transition: opacity 0.3s;
}
.copy-toast.show { opacity: 1; }
"""

# ── Toolbar HTML ───────────────────────────────────────────────────────────────

SELECTION_TOOLBAR = """\
<div class="selection-toolbar" id="selection-toolbar">
  <button onclick="selectAllVisible()" title="Select all functions on this page">Select all</button>
  <button onclick="deselectAllVisible()" title="Deselect all functions on this page">Deselect all</button>
  <span class="selection-count" id="selection-count">0 selected</span>
  <button class="primary" onclick="saveSpecsToml()">Save specs.toml</button>
  <button onclick="clearAllSelection()" title="Clear selection across all pages">Clear all</button>
</div>"""

# ── JavaScript ─────────────────────────────────────────────────────────────────

SELECTION_SCRIPT = """\
<script>
(function() {
var STORAGE_KEY = 'autospec_selected_functions';
var SERVER_MODE = (window.location.protocol === 'http:' ||
                   window.location.protocol === 'https:');

function getSelected() {
  try {
    var raw = localStorage.getItem(STORAGE_KEY);
    return raw ? new Set(JSON.parse(raw)) : new Set();
  } catch(e) { return new Set(); }
}

function saveSelected(s) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(Array.from(s)));
}

function buildToml() {
  var selected = Array.from(getSelected()).sort();
  var toml = '# Functions selected for verification\\n';
  toml += '# Generated from the autospec HTML report.\\n\\n';
  toml += '[verify]\\n';
  toml += 'functions = [\\n';
  for (var i = 0; i < selected.length; i++) {
    toml += '    \"' + selected[i] + '\",\\n';
  }
  toml += ']\\n';
  return toml;
}

function showToast(msg, duration) {
  var toast = document.getElementById('copy-toast');
  toast.textContent = msg;
  toast.classList.add('show');
  setTimeout(function() { toast.classList.remove('show'); }, duration || 2000);
}

window.toggleFn = function(key, checked) {
  var s = getSelected();
  if (checked) s.add(key); else s.delete(key);
  saveSelected(s);
  updateAll();
};

window.selectAllVisible = function() {
  var s = getSelected();
  document.querySelectorAll('.fn-checkbox').forEach(function(cb) {
    s.add(cb.getAttribute('data-key'));
  });
  saveSelected(s);
  updateAll();
};

window.deselectAllVisible = function() {
  var s = getSelected();
  document.querySelectorAll('.fn-checkbox').forEach(function(cb) {
    s.delete(cb.getAttribute('data-key'));
  });
  saveSelected(s);
  updateAll();
};

window.clearAllSelection = function() {
  localStorage.removeItem(STORAGE_KEY);
  updateAll();
};

window.saveSpecsToml = function() {
  var selected = Array.from(getSelected()).sort();
  if (selected.length === 0) {
    alert('No functions selected. Check some checkboxes first.');
    return;
  }
  var toml = buildToml();

  if (SERVER_MODE) {
    // Direct save via API — writes straight to specs/<crate>/specs.toml
    fetch('/api/save-specs', {method: 'POST', body: toml})
      .then(function(r) {
        if (!r.ok) throw new Error('Server error: ' + r.status);
        return r.json();
      })
      .then(function(data) {
        showToast('Saved: ' + data.path + ' (' + data.size + ' bytes)', 3000);
      })
      .catch(function(err) {
        showToast('Save failed: ' + err.message, 3000);
      });
  } else {
    // File:// mode — show modal with copy/download
    document.getElementById('modal-textarea').value = toml;
    var crate = document.title.split(' — ')[1] || 'crate';
    document.getElementById('modal-path-hint').textContent =
      'Save as: specs/' + crate + '/specs.toml  (or use --serve for direct save)';
    document.getElementById('modal-overlay').classList.add('active');
  }
};

window.closeModal = function() {
  document.getElementById('modal-overlay').classList.remove('active');
};

window.copyToml = function() {
  var textarea = document.getElementById('modal-textarea');
  textarea.select();
  textarea.setSelectionRange(0, 99999);
  navigator.clipboard.writeText(textarea.value).then(function() {
    showToast('Copied to clipboard');
  });
};

window.downloadToml = function() {
  var toml = document.getElementById('modal-textarea').value;
  var blob = new Blob([toml], {type: 'text/plain'});
  var url = URL.createObjectURL(blob);
  var a = document.createElement('a');
  a.href = url;
  a.download = 'specs.toml';
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
};

// Restore selections from server — server is source of truth in serve mode
function loadFromServer() {
  fetch('/api/specs-toml')
    .then(function(r) { if (r.ok) return r.text(); else return null; })
    .then(function(toml) {
      var s = new Set();
      if (toml) {
        var re = /"([^"]+)"/g;
        var m;
        while ((m = re.exec(toml)) !== null) {
          s.add(m[1]);
        }
      }
      saveSelected(s);
      updateAll();
    })
    .catch(function() { updateAll(); });
}

function updateAll() {
  var count = getSelected().size;
  var el = document.getElementById('selection-count');
  if (el) el.textContent = count + ' selected';
  var selected = getSelected();
  document.querySelectorAll('.fn-checkbox').forEach(function(cb) {
    cb.checked = selected.has(cb.getAttribute('data-key'));
  });
}

document.addEventListener('DOMContentLoaded', function() {
  if (SERVER_MODE) {
    // Load server-side specs.toml first, then overlay localStorage on top
    loadFromServer();
  } else {
    updateAll();
  }
});
})();
</script>"""
