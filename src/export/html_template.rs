pub const CSS: &str = r#"<style>
*{margin:0;padding:0;box-sizing:border-box}
body{background:#1a1a2e;color:#e0e0e0;font-family:'Segoe UI',system-ui,sans-serif;height:100vh}
#app{display:flex;flex-direction:column;height:100vh;max-width:900px;margin:0 auto}
header{display:flex;justify-content:space-between;align-items:center;padding:12px 16px;
  border-bottom:1px solid #333}
header h1{font-size:1.2em;color:#8ab4f8}
#restart-btn{background:#3b5998;color:#fff;border:none;padding:6px 16px;border-radius:4px;
  cursor:pointer;font-size:0.9em}
#restart-btn:hover{background:#4c6fb0}
#main{flex:1;display:flex;flex-direction:column;overflow:hidden;padding:16px}
#chat-log{flex:1;overflow-y:auto;padding-bottom:12px}
.msg{margin-bottom:10px;padding:8px 12px;border-radius:8px;background:#16213e;max-width:85%}
.msg .speaker{font-weight:bold;margin-bottom:2px}
.msg .text{line-height:1.5}
.msg .emotion{font-style:italic;color:#888;font-size:0.85em}
.msg.system{color:#aaa;font-style:italic;background:transparent;max-width:100%}
#choices{padding:8px 0}
#choices button{display:block;width:100%;text-align:left;padding:10px 16px;margin:4px 0;
  background:#16213e;color:#e0e0e0;border:1px solid #3b5998;border-radius:6px;cursor:pointer;
  font-size:0.95em;transition:background 0.2s}
#choices button:hover{background:#1f3460}
#choices button:disabled{opacity:0.4;cursor:not-allowed}
#continue-btn{background:#2d5a27;border-color:#4a8;margin-top:8px}
#continue-btn:hover{background:#3a7233}
#sidebar{border-top:1px solid #333;padding:10px 16px;max-height:200px;overflow-y:auto}
#sidebar h3{cursor:pointer;color:#8ab4f8;margin-bottom:6px;font-size:0.9em}
#var-list{display:grid;grid-template-columns:1fr 1fr;gap:2px 12px;font-size:0.85em}
.var-name{color:#a5d6a7}.var-val{color:#e0e0e0;font-family:monospace}
</style>"#;

pub const JS_ENGINE: &str = r#"
let vars = JSON.parse(JSON.stringify(INIT_VARS));
let currentNode = null;
let sidebarOpen = true;

function start() {
  document.getElementById('chat-log').innerHTML = '';
  document.getElementById('choices').innerHTML = '';
  vars = JSON.parse(JSON.stringify(INIT_VARS));
  currentNode = findStart();
  if (currentNode) {
    let n = GRAPH[currentNode];
    if (n && n.next) currentNode = n.next;
  }
  processNode();
}

function findStart() {
  for (let id in GRAPH) {
    if (GRAPH[id].type === 'start') return id;
  }
  return null;
}

function processNode() {
  if (!currentNode || !GRAPH[currentNode]) {
    addSystemMsg('End of dialogue.');
    updateVars();
    return;
  }
  let node = GRAPH[currentNode];
  switch (node.type) {
    case 'dialogue': showDialogue(node); break;
    case 'choice': showChoices(node); break;
    case 'condition': evalCondition(node); break;
    case 'event': execEvent(node); break;
    case 'random': execRandom(node); break;
    case 'end': addSystemMsg('Dialogue ended' + (node.tag ? ': [' + node.tag + ']' : '.'));
      updateVars(); break;
    default: currentNode = node.next || null; processNode();
  }
}

function showDialogue(node) {
  let speaker = node.speaker || '???';
  let charInfo = findChar(speaker);
  let color = charInfo ? charInfo.color : '#8ab4f8';
  let text = interpolate(node.text || '');
  let html = '<div class="msg"><div class="speaker" style="color:' + color + '">' +
    esc(speaker) + '</div><div class="text">' + esc(text) + '</div>';
  if (node.emotion && node.emotion !== 'neutral') {
    html += '<div class="emotion">[' + esc(node.emotion) + ']</div>';
  }
  html += '</div>';
  appendLog(html);
  let choices = document.getElementById('choices');
  choices.innerHTML = '<button id="continue-btn" onclick="advanceDialogue()">Continue &gt;&gt;</button>';
  updateVars();
}

function advanceDialogue() {
  let node = GRAPH[currentNode];
  currentNode = node.next || null;
  document.getElementById('choices').innerHTML = '';
  processNode();
}

function showChoices(node) {
  if (node.prompt) {
    addSystemMsg(interpolate(node.prompt));
  }
  let choices = document.getElementById('choices');
  choices.innerHTML = '';
  (node.options || []).forEach(function(opt, i) {
    let btn = document.createElement('button');
    btn.textContent = interpolate(opt.text);
    let available = true;
    if (opt.condition && opt.condition.variable) {
      available = evalCond(opt.condition.variable, opt.condition.operator, opt.condition.value);
    }
    btn.disabled = !available;
    btn.onclick = function() {
      addSystemMsg('> ' + interpolate(opt.text));
      currentNode = opt.next || null;
      choices.innerHTML = '';
      processNode();
    };
    choices.appendChild(btn);
  });
  updateVars();
}

function evalCondition(node) {
  let result = evalCond(node.variable, node.operator, node.value);
  addSystemMsg('[Condition: ' + node.variable + ' ' + node.operator + ' ' +
    JSON.stringify(node.value) + ' = ' + result + ']');
  currentNode = result ? (node.true_next || null) : (node.false_next || null);
  processNode();
}

function execEvent(node) {
  (node.actions || []).forEach(function(a) {
    if (a.action === 'setvariable') vars[a.key] = a.value;
  });
  addSystemMsg('[Event: ' + (node.actions || []).length + ' action(s)]');
  currentNode = node.next || null;
  processNode();
}

function execRandom(node) {
  let branches = node.branches || [];
  let total = branches.reduce(function(s, b) { return s + b.weight; }, 0);
  let roll = Math.random() * total;
  let chosen = 0;
  for (let i = 0; i < branches.length; i++) {
    roll -= branches[i].weight;
    if (roll <= 0) { chosen = i; break; }
  }
  addSystemMsg('[Random: branch ' + (chosen + 1) + ' selected]');
  currentNode = branches[chosen] ? branches[chosen].next : null;
  processNode();
}

function evalCond(variable, operator, value) {
  let v = vars[variable];
  if (v === undefined) return false;
  switch (operator) {
    case '==': return v == value;
    case '!=': return v != value;
    case '>':  return v > value;
    case '<':  return v < value;
    case '>=': return v >= value;
    case '<=': return v <= value;
    case 'contains': return typeof v === 'string' && v.includes(String(value));
    default: return false;
  }
}

function findChar(name) {
  for (let cname in CHARACTERS) {
    if (cname === name || cname.toLowerCase() === name.toLowerCase()) return CHARACTERS[cname];
  }
  for (let cname in CHARACTERS) {
    if (cname === name) return CHARACTERS[cname];
  }
  return null;
}

function interpolate(text) {
  return text.replace(/\{(\w+)\}/g, function(m, name) {
    return vars[name] !== undefined ? String(vars[name]) : m;
  });
}

function addSystemMsg(text) {
  appendLog('<div class="msg system">' + esc(text) + '</div>');
}

function appendLog(html) {
  let log = document.getElementById('chat-log');
  log.innerHTML += html;
  log.scrollTop = log.scrollHeight;
}

function updateVars() {
  let list = document.getElementById('var-list');
  list.innerHTML = '';
  for (let name in vars) {
    list.innerHTML += '<span class="var-name">' + esc(name) +
      '</span><span class="var-val">' + esc(String(vars[name])) + '</span>';
  }
}

function toggleSidebar() {
  let list = document.getElementById('var-list');
  sidebarOpen = !sidebarOpen;
  list.style.display = sidebarOpen ? 'grid' : 'none';
}

function esc(s) {
  let d = document.createElement('div');
  d.textContent = s;
  return d.innerHTML;
}

document.getElementById('restart-btn').onclick = start;
start();
"#;
