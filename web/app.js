import init, { configure, defaults, fire, simulate } from './pkg/playground.js';

const $ = (id) => document.getElementById(id);
const styles = ['Dynamic cross', 'Dynamic circle', 'Legacy dynamic cross', 'Static circle', 'Static cross', 'Shot-feedback cross', 'Dot', 'Dynamic quad', 'Static square', 'Static quad'];
let settings = {};
let ready = false;
let hits = 0;
let audio;
const state = { damage: 40, kind: 0, duration: 250, scale: 1, damageScale: true, spread: 0, shot: 0, recoilX: 0, recoilY: 0, scope: 0, friendly: false, grenade: 0, held: 2000, spectator: 0 };
const measurements = [];
window.crosshairMeasurements = measurements;
window.crosshairFrame = (fps, longestMs) => {
  measurements.push({ time: performance.now(), fps, longestMs });
  if (measurements.length > 180) measurements.shift();
  $('fps').textContent = `${fps.toFixed(0)} FPS`;
  $('frame-time').textContent = `${longestMs.toFixed(1)} ms max`;
  $('loading').hidden = true;
  $('fire').disabled = false;
  ready = true;
};
window.crosshairHit = (damage, kind) => {
  hits += 1;
  const label = ['Hit', 'Headshot', 'Elimination'][kind];
  $('receipt').textContent = `${label} · ${damage} damage · ${hits} events`;
  if ($('sound').checked && audio) {
    const oscillator = audio.createOscillator();
    const gain = audio.createGain();
    const now = audio.currentTime;
    oscillator.frequency.setValueAtTime([650, 950, 450][kind], now);
    oscillator.frequency.exponentialRampToValueAtTime(200, now + 0.075);
    gain.gain.setValueAtTime(0.08, now);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.08);
    oscillator.connect(gain).connect(audio.destination);
    oscillator.start(now);
    oscillator.stop(now + 0.09);
    oscillator.onended = () => { oscillator.disconnect(); gain.disconnect(); };
  }
};
function error(message) { $('error').hidden = !message; $('error').textContent = message || ''; }
let backgroundRequest = 0;
async function background() {
  const request = ++backgroundRequest;
  const option = $('background').selectedOptions[0];
  const { image: source, credit, source: link } = option.dataset;
  $('viewport').style.backgroundImage = 'none';
  $('background-credit').hidden = !source;
  $('background-status').hidden = !source;
  if (!source) return;
  $('background-credit').textContent = credit;
  $('background-credit').href = link;
  $('background-status').textContent = 'Loading background…';
  const image = new Image();
  image.src = source;
  try {
    await image.decode();
    // A slower previous selection must not replace the latest scene.
    if (request !== backgroundRequest) return;
    $('viewport').style.backgroundImage = `url("${source}")`;
    $('background-status').hidden = true;
  } catch {
    if (request !== backgroundRequest) return;
    $('background-status').textContent = 'Background unavailable. Choose another background.';
  }
}
$('background').addEventListener('change', background);
function attempt(action) { try { action(); error(''); } catch (cause) { error(String(cause)); } }
function parse(source) { return Object.fromEntries(source.trim().split('\n').map((line) => line.split(' '))); }
function text() { return Object.entries(settings).map(([name, value]) => `${name} ${value}`).join('\n') + '\n'; }
function publish() { attempt(() => configure(text())); $('configuration').value = text(); $('style-name').textContent = styles[Number(settings.cl_crosshairstyle)]; }
function simulation() { attempt(() => simulate(state.spread, state.shot, state.recoilX, state.recoilY, state.scope, state.friendly, state.grenade, state.held, state.spectator)); }
function field(parent, label, name, kind, options, simulationField = false) {
  const row = document.createElement('div'); row.className = 'field';
  const id = `setting-${name}`;
  const caption = document.createElement('label'); caption.htmlFor = id; caption.textContent = label;
  if (!simulationField) caption.title = name;
  const input = document.createElement(kind === 'select' ? 'select' : 'input'); input.id = id;
  const value = simulationField ? state[name] : settings[name];
  if (kind === 'select') {
    options.forEach((label, index) => { const option = document.createElement('option'); option.value = index; option.textContent = label; input.append(option); });
    input.value = value;
  } else {
    input.type = kind;
    if (kind === 'checkbox') input.checked = value === true || value === 'true' || value === '1';
    else { [input.min, input.max, input.step] = options; input.value = value; }
  }
  row.append(caption);
  let output;
  if (kind === 'range') { output = document.createElement('output'); output.htmlFor = id; output.textContent = value; row.append(output); }
  row.append(input); parent.append(row);
  input.addEventListener('input', () => {
    const next = kind === 'checkbox' ? input.checked : Number(input.value);
    if (output) output.textContent = input.value;
    if (simulationField) {
      state[name] = next;
      if (name === 'scope' && next !== 0) { state.grenade = 0; $('setting-grenade').value = 0; }
      if (name === 'grenade' && next !== 0) { state.scope = 0; $('setting-scope').value = 0; }
      simulation();
    }
    else { settings[name] = String(next); publish(); }
  });
}
function section(parent, name) { const details = document.createElement('details'); const summary = document.createElement('summary'); summary.textContent = name; details.append(summary); parent.append(details); return details; }
function color(parent, name, prefix) {
  const row = document.createElement('div'); row.className = 'field';
  const label = document.createElement('label'); label.htmlFor = prefix; label.textContent = name;
  const input = document.createElement('input'); input.type = 'color'; input.id = prefix;
  input.value = '#' + ['r', 'g', 'b'].map((channel) => Number(settings[`${prefix}_${channel}`]).toString(16).padStart(2, '0')).join('');
  input.addEventListener('input', () => { ['r', 'g', 'b'].forEach((channel, index) => { settings[`${prefix}_${channel}`] = String(parseInt(input.value.slice(1 + index * 2, 3 + index * 2), 16)); }); publish(); });
  row.append(label, input); parent.append(row);
  field(parent, 'Opacity', `${prefix}_a`, 'range', [0, 255, 1]);
}
function controls() {
  $('shape').replaceChildren(); $('hit').replaceChildren(); $('game').replaceChildren();
  const shape = $('shape');
  field(shape, 'Style', 'cl_crosshairstyle', 'select', styles);
  field(shape, 'Length', 'cl_crosshair_length', 'range', [0, 255, 1]);
  field(shape, 'Thickness', 'cl_crosshair_thickness', 'range', [0, 32, 1]);
  field(shape, 'Gap', 'cl_crosshair_gap', 'range', [-10, 128, 1]);
  color(shape, 'Color', 'cl_crosshaircolor');
  field(shape, 'Center dot', 'cl_crosshairdot', 'checkbox');
  field(shape, 'T shape', 'cl_crosshair_t', 'checkbox');
  field(shape, 'Follow recoil', 'cl_crosshair_recoil', 'checkbox');
  const outline = section(shape, 'Outline');
  field(outline, 'Outline', 'cl_crosshair_drawoutline', 'select', ['None', 'Full', 'Half']); color(outline, 'Outline color', 'cl_crosshairoutline');
  const dynamic = section(shape, 'Dynamic and split settings');
  [['Spread limit', 'spread_limit', 0, 255, 1], ['Split distance', 'splitdist', 0, 127, 1], ['Outer length ratio', 'maxdist_splitratio', 0, 1, .01], ['Inner opacity', 'splitalpha_innermod', 0, 1, .01], ['Outer opacity', 'splitalpha_outermod', .3, 1, .01]].forEach(([label, name, ...range]) => field(dynamic, label, `cl_crosshair_dynamic_${name}`, 'range', range));
  const other = section(shape, 'Display and scope');
  field(other, 'Crosshair visible', 'crosshair', 'checkbox');
  field(other, 'Authored screen height', 'cl_crosshair_screen_height', 'range', [240, 4320, 1]);
  field(other, 'Friendly warning', 'cl_crosshair_friendly_warning', 'checkbox');
  field(other, 'Sniper line width', 'cl_crosshair_sniper_width', 'range', [1, 6, 1]);
  field(other, 'Show sniper inaccuracy', 'cl_sniper_show_inaccuracy', 'checkbox');
  field(other, 'Delay unscoping', 'cl_sniper_delay_unscope', 'checkbox');
  field(other, 'Automatic rezoom', 'cl_sniper_auto_rezoom', 'checkbox');
  field(other, 'Sight uses crosshair color', 'cl_ironsight_usecrosshaircolor', 'checkbox');
  field(other, 'Sight dot scale', 'cl_ironsight_dot_scale', 'range', [.1, 2, .01]);
  const grenades = section(shape, 'Grenades');
  field(grenades, 'Keep crosshair with guide', 'cl_grenadecrosshair_keepusercrosshair', 'checkbox');
  ['flash', 'explosive', 'fire', 'smoke', 'decoy'].forEach((name) => { const label = name[0].toUpperCase() + name.slice(1); field(grenades, label, `cl_grenadecrosshair_${name}`, 'checkbox'); field(grenades, `${label} delay, seconds`, `cl_grenadecrosshairdelay_${name}`, 'range', [0, 5, .01]); });
  const spectators = section(shape, 'Spectators');
  field(spectators, 'Observed crosshair', 'cl_show_observer_crosshair', 'select', ['Use own', 'Friends and party', 'Everyone']);
  field(spectators, 'Observed bot crosshair', 'cl_observed_bot_crosshair', 'select', ['Use own', 'Own when takeover is available', 'Use default']);
  const hit = $('hit');
  field(hit, 'Hit kind', 'kind', 'select', ['Body', 'Headshot', 'Elimination'], true);
  field(hit, 'Damage, health points', 'damage', 'range', [0, 200, 1], true);
  field(hit, 'Duration, milliseconds', 'duration', 'range', [0, 1500, 10], true);
  field(hit, 'Marker scale', 'scale', 'range', [0, 4, .05], true);
  field(hit, 'Scale with damage', 'damageScale', 'checkbox', null, true);
  const game = $('game');
  [['Weapon spread, pixels', 'spread', 0, 200, 1], ['Shot feedback, pixels', 'shot', 0, 80, 1], ['Recoil X, pixels', 'recoilX', -80, 80, 1], ['Recoil Y, pixels', 'recoilY', -80, 80, 1]].forEach(([label, name, ...range]) => field(game, label, name, 'range', range, true));
  field(game, 'Scope phase', 'scope', 'select', ['Unscoped', 'Sniper', 'Cycling after shot', 'Ready after cycling', 'Optical sight'], true);
  field(game, 'Aiming at a friendly', 'friendly', 'checkbox', null, true);
  field(game, 'Grenade', 'grenade', 'select', ['None', 'Flash', 'Explosive', 'Fire', 'Smoke', 'Decoy'], true);
  field(game, 'Pin held, milliseconds', 'held', 'range', [0, 5000, 10], true);
  field(game, 'Spectator', 'spectator', 'select', ['Local player', 'Friend with dot crosshair', 'Player with dot crosshair', 'Bot available for takeover', 'Bot unavailable for takeover'], true);
}
const tabs = [...document.querySelectorAll('[role=tab]')];
function selectTab(tab) {
  tabs.forEach((button) => { const selected = button === tab; button.setAttribute('aria-selected', selected); button.tabIndex = selected ? 0 : -1; $(button.getAttribute('aria-controls')).hidden = !selected; });
}
tabs.forEach((tab, index) => { tab.addEventListener('click', () => selectTab(tab)); tab.addEventListener('keydown', (event) => { if (['ArrowLeft', 'ArrowRight'].includes(event.key)) { event.preventDefault(); const next = tabs[(index + (event.key === 'ArrowRight' ? 1 : -1) + tabs.length) % tabs.length]; selectTab(next); next.focus(); } }); });
$('sound').addEventListener('change', async () => { if ($('sound').checked) { audio ??= new AudioContext(); await audio.resume(); } });
$('fire').addEventListener('click', () => { if (ready) attempt(() => fire(state.damage, state.kind, state.duration, state.scale, state.damageScale)); });
$('apply').addEventListener('click', () => attempt(() => { configure($('configuration').value); settings = parse(defaults()); $('configuration').value.split(/[;\n]/).forEach((assignment) => { const match = assignment.trim().match(/^(\S+)\s+"?([^"\s]+)"?$/); if (match) settings[match[1]] = match[2]; }); controls(); publish(); }));
$('copy').addEventListener('click', async () => { try { await navigator.clipboard.writeText($('configuration').value); $('copy').textContent = 'Copied'; setTimeout(() => $('copy').textContent = 'Copy', 1500); } catch { error('Copy failed. Select the configuration text and copy it manually.'); } });
$('reset').addEventListener('click', () => { settings = parse(defaults()); controls(); publish(); });
window.addEventListener('unhandledrejection', (event) => error(`Renderer failed: ${event.reason}. Reload the page to retry.`));
try { await init(); settings = parse(defaults()); controls(); publish(); } catch (cause) { $('loading').textContent = 'Renderer unavailable'; error(`Could not start Bevy: ${cause}. Reload the page to retry.`); }
