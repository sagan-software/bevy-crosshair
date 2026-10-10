import init, { configure, defaults, fire, simulate, appearance, marker_style, preview_hit, browse } from './pkg/playground.js';
import { makePresets } from './presets.js';

const $ = (id) => document.getElementById(id);
const styles = ['Dynamic cross', 'Dynamic circle', 'Legacy dynamic cross', 'Static circle', 'Static cross', 'Shot-feedback cross', 'Dot', 'Dynamic quad', 'Static square', 'Static quad'];
let settings = {};
let ready = false;
let hits = 0;
let audio;
const state = { damage: 40, kind: 0, duration: 600, scale: 1, markerShape: 0, animation: 3, repeat: false, fill: '#00ff80', fillAlpha: 0, dotShape: 0, damageScale: true, spread: 0, shot: 0, recoilX: 0, recoilY: 0, scope: 0, friendly: false, grenade: 0, held: 2000, spectator: 0 };
const hitFields = new Set(['damage', 'kind', 'duration', 'scale', 'damageScale', 'markerShape', 'animation', 'repeat']);
let previewTimer, repeatTimer, grenadeTimer;
let collection = [];
let selectedPreset = null;
let galleryOpen = false;
let galleryScroll = 0;
const narrow = matchMedia('(max-width: 760px)');
const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)');
if (reducedMotion.matches) state.animation = 0;
const measurements = [];
window.crosshairMeasurements = measurements;
window.crosshairFrame = (fps, longestMs, crosshairs) => {
  measurements.push({ time: performance.now(), fps, longestMs, crosshairs });
  if (measurements.length > 180) measurements.shift();
  $('fps').textContent = `${fps.toFixed(0)} FPS`;
  $('frame-time').textContent = `${longestMs.toFixed(1)} ms max`;
  $('loading').hidden = true;
  $('fire').disabled = false;
  $('browse').disabled = false;
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
function text(source = settings) { return Object.entries(source).map(([name, value]) => `${name} ${value}`).join('\n') + '\n'; }
function publishAppearance() {
  const channels = [1, 3, 5].map(start => parseInt(state.fill.slice(start, start + 2), 16));
  appearance(...channels, state.fillAlpha, state.dotShape);
}
function publish() {
  attempt(() => { configure(text()); publishAppearance(); });
  $('configuration').value = text();
  if (!galleryOpen) $('style-name').textContent = styles[Number(settings.cl_crosshairstyle)];
  const help = $('style-help');
  if (help) help.textContent = ['Opens with weapon spread. Fire to preview its recovery.', 'Radius follows weapon spread. Fire to preview its recovery.', 'Arms split when spread exceeds the split threshold.', 'A fixed circle. Spread does not change its radius.', 'A fixed cross. Spread does not change its gap.', 'Gap opens after each shot, then returns to rest.', 'Shows only the center dot, using its selected shape.', 'Curved arms open with weapon spread. Fire to preview.', 'A fixed square. Fill colors its interior.', 'Four fixed arcs. Fill colors the disk inside them.'][Number(settings.cl_crosshairstyle)];
  updateContext();
}
function simulation() { attempt(() => simulate(state.spread, state.shot, state.recoilX, state.recoilY, state.scope, state.friendly, state.grenade, state.held, state.spectator)); updateContext(); }
function field(parent, label, name, kind, options, simulationField = false, help = '') {
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
  row.append(input);
  if (help) {
    const hint = document.createElement('p'); hint.className = 'help'; hint.id = `${id}-help`; hint.textContent = help;
    input.setAttribute('aria-describedby', hint.id); row.append(hint);
  }
  parent.append(row);
  input.addEventListener('input', () => {
    const next = kind === 'checkbox' ? input.checked : Number(input.value);
    if (output) output.textContent = input.value;
    if (simulationField) {
      state[name] = next;
      if (name === 'scope' && next !== 0) { state.grenade = 0; $('setting-grenade').value = 0; }
      if (name === 'grenade' && next !== 0) { state.scope = 0; $('setting-scope').value = 0; }
      hostChanged(name);
    }
    else { settings[name] = String(next); publish(); contextualSetting(name); }
  });
}
function section(parent, name, description = '') {
  const details = document.createElement('details'); const summary = document.createElement('summary'); summary.textContent = name;
  details.append(summary); if (description) note(details, description, 'section-help'); parent.append(details); return details;
}
function note(parent, text, className = '') {
  const paragraph = document.createElement('p'); paragraph.className = `help ${className}`; paragraph.textContent = text; parent.append(paragraph); return paragraph;
}
function actions(parent, entries) {
  const row = document.createElement('div'); row.className = 'try-row';
  entries.forEach(([label, action, id]) => { const button = document.createElement('button'); button.textContent = label; if (id) button.id = id; button.addEventListener('click', action); row.append(button); });
  parent.append(row);
}
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
  const styleHelp = note(shape, '', 'section-help'); styleHelp.id = 'style-help';
  $('setting-cl_crosshairstyle').setAttribute('aria-describedby', 'style-help');
  field(shape, 'Length', 'cl_crosshair_length', 'range', [0, 255, 1]);
  field(shape, 'Thickness', 'cl_crosshair_thickness', 'range', [0, 32, 1]);
  field(shape, 'Gap', 'cl_crosshair_gap', 'range', [-10, 128, 1]);
  color(shape, 'Color', 'cl_crosshaircolor');
  const fill = section(shape, 'Fill color', 'Colors the space inside the crosshair. Its opacity is independent of the arms and center dot.');
  const fillRow = document.createElement('div'); fillRow.className = 'field';
  const fillLabel = document.createElement('label'); fillLabel.htmlFor = 'fill-color'; fillLabel.textContent = 'Fill color';
  const fillInput = document.createElement('input'); fillInput.id = 'fill-color'; fillInput.type = 'color'; fillInput.value = state.fill;
  fillInput.addEventListener('input', () => { state.fill = fillInput.value; attempt(publishAppearance); });
  fillRow.append(fillLabel, fillInput); fill.append(fillRow);
  field(fill, 'Fill opacity', 'fillAlpha', 'range', [0, 255, 1], true, 'Zero is transparent. Increase Gap to make the filled area larger.');
  field(shape, 'Center dot', 'cl_crosshairdot', 'checkbox');
  field(shape, 'Dot shape', 'dotShape', 'select', ['Square', 'Circle'], true, 'Applies when Center dot is on or the Dot style is selected.');
  field(shape, 'T shape', 'cl_crosshair_t', 'checkbox');
  field(shape, 'Follow recoil', 'cl_crosshair_recoil', 'checkbox');
  const outline = section(shape, 'Outline');
  field(outline, 'Outline', 'cl_crosshair_drawoutline', 'select', ['None', 'Full', 'Half']); color(outline, 'Outline color', 'cl_crosshairoutline');
  const dynamic = section(shape, 'Movement and split arms', 'Dynamic styles open with weapon spread. Legacy dynamic cross can split each arm into an inner and outer piece.');
  actions(dynamic, [['Try split arms', () => scenario('split'), 'try-split']]);
  const dynamicHelp = {
    spread_limit: 'Caps weapon spread at 128 plus this value, scaled from the reference screen height.',
    splitdist: 'In Legacy dynamic cross, spread above this distance separates the arm pieces.',
    maxdist_splitratio: 'Fraction of each legacy arm in its outer piece. Static quad uses this value for arc coverage.',
    splitalpha_innermod: 'Opacity of the inner pieces after legacy arms split.',
    splitalpha_outermod: 'Opacity of the outer pieces after legacy arms split.',
  };
  [['Extra spread limit', 'spread_limit', 0, 255, 1], ['Split threshold', 'splitdist', 0, 127, 1], ['Outer length ratio', 'maxdist_splitratio', 0, 1, .01], ['Inner piece opacity', 'splitalpha_innermod', 0, 1, .01], ['Outer piece opacity', 'splitalpha_outermod', .3, 1, .01]].forEach(([label, name, ...range]) => field(dynamic, label, `cl_crosshair_dynamic_${name}`, 'range', range, false, dynamicHelp[name]));
  const display = section(shape, 'Visibility and scale', 'Dimensions scale with the preview height. Lower the reference height to make the crosshair larger.');
  field(display, 'Crosshair visible', 'crosshair', 'checkbox');
  field(display, 'Reference screen height, pixels', 'cl_crosshair_screen_height', 'range', [240, 4320, 1]);
  field(display, 'Friendly warning', 'cl_crosshair_friendly_warning', 'checkbox', null, false, 'Adds a warning when your aim is on a teammate.');
  actions(display, [['Aim at a teammate', () => scenario('friendly'), 'try-friendly']]);
  const other = section(shape, 'Sniper and optical sights', 'These replace the ordinary crosshair while scoped. Preview each weapon phase to see the corresponding option.');
  actions(other, [['Preview sniper', () => scenario('sniper'), 'try-sniper'], ['Preview optical sight', () => scenario('ironsight'), 'try-ironsight']]);
  field(other, 'Sniper line width', 'cl_crosshair_sniper_width', 'range', [1, 6, 1]);
  field(other, 'Show sniper inaccuracy', 'cl_sniper_show_inaccuracy', 'checkbox', null, false, 'Shows a ring around the scope center when weapon spread is above zero.');
  field(other, 'Keep scope while cycling', 'cl_sniper_delay_unscope', 'checkbox', null, false, 'After a shot, keep the scope visible while the weapon cycles. Changing this previews that phase.');
  field(other, 'Scope again after cycling', 'cl_sniper_auto_rezoom', 'checkbox', null, false, 'When the weapon is ready again, return to its scope. Changing this previews that phase.');
  field(other, 'Sight uses crosshair color', 'cl_ironsight_usecrosshaircolor', 'checkbox');
  field(other, 'Sight dot scale', 'cl_ironsight_dot_scale', 'range', [.1, 2, .01]);
  const grenades = section(shape, 'Grenade lineup guides', 'Holding a grenade draws screen-wide alignment lines after its delay. Use them to line up landmarks; they do not predict a throw.');
  actions(grenades, [['Hold a smoke grenade', () => holdGrenade(4, true), 'try-grenade']]);
  field(grenades, 'Keep crosshair with guide', 'cl_grenadecrosshair_keepusercrosshair', 'checkbox', null, false, 'Keep your ordinary crosshair visible beneath the alignment lines.');
  ['flash', 'explosive', 'fire', 'smoke', 'decoy'].forEach((name) => { const label = name[0].toUpperCase() + name.slice(1); field(grenades, `${label} guide`, `cl_grenadecrosshair_${name}`, 'checkbox'); field(grenades, `${label} delay, seconds`, `cl_grenadecrosshairdelay_${name}`, 'range', [0, 5, .01]); });
  const spectators = section(shape, 'Spectator crosshairs', 'Choose whose crosshair you see while watching another player. The sample player uses a magenta dot so the switch is visible.');
  actions(spectators, [['Watch a friend', () => scenario('friend'), 'try-friend'], ['Watch a stranger', () => scenario('stranger'), 'try-stranger'], ['Watch a bot', () => scenario('bot'), 'try-bot']]);
  field(spectators, 'Observed crosshair', 'cl_show_observer_crosshair', 'select', ['Use own', 'Friends and party', 'Everyone']);
  field(spectators, 'Observed bot crosshair', 'cl_observed_bot_crosshair', 'select', ['Use own', 'Own when takeover is available', 'Use default'], false, 'The sample bot cannot be taken over. Only Use own keeps your current crosshair.');
  const hit = $('hit');
  note(hit, 'Changes replay the marker. Fire also sends a hit event and can play a sound.', 'section-help');
  actions(hit, [['Replay marker', replayMarker, 'replay-marker']]);
  field(hit, 'Repeat preview', 'repeat', 'checkbox', null, true);
  field(hit, 'Marker shape', 'markerShape', 'select', ['Diagonal cross', 'Plus', 'Chevron', 'Diamond', 'Ring'], true);
  field(hit, 'Animation', 'animation', 'select', ['Fade only', 'Expand from center', 'Contract to center', 'Out and back'], true,
    'Moving markers ease out, fade in during the first 20%, and fade out during the final 40% of their duration.');
  field(hit, 'Hit kind', 'kind', 'select', ['Body', 'Headshot', 'Elimination'], true);
  field(hit, 'Damage, health points', 'damage', 'range', [0, 200, 1], true);
  field(hit, 'Duration, milliseconds', 'duration', 'range', [0, 1500, 10], true);
  field(hit, 'Marker scale', 'scale', 'range', [0, 4, .05], true);
  field(hit, 'Scale with damage', 'damageScale', 'checkbox', null, true);
  const game = $('game');
  note(game, 'Set the weapon state supplied to the crosshair. Fire adds a brief kick, then returns to these values.', 'section-help');
  [['Weapon spread, pixels', 'spread', 0, 200, 1], ['Shot feedback, pixels', 'shot', 0, 80, 1], ['Recoil X, pixels', 'recoilX', -80, 80, 1], ['Recoil Y, pixels', 'recoilY', -80, 80, 1]].forEach(([label, name, ...range]) => field(game, label, name, 'range', range, true));
  field(game, 'Scope phase', 'scope', 'select', ['Unscoped', 'Sniper', 'Cycling after shot', 'Ready after cycling', 'Optical sight'], true);
  field(game, 'Aiming at a friendly', 'friendly', 'checkbox', null, true);
  field(game, 'Grenade', 'grenade', 'select', ['None', 'Flash', 'Explosive', 'Fire', 'Smoke', 'Decoy'], true);
  field(game, 'Grenade held, milliseconds', 'held', 'range', [0, 5000, 10], true);
  field(game, 'Spectator', 'spectator', 'select', ['Local player', 'Friend with dot crosshair', 'Player with dot crosshair', 'Bot available for takeover', 'Bot unavailable for takeover'], true);
}
// Host simulations make state-dependent options visible without changing their policy.
const grenadeNames = ['None', 'Flash', 'Explosive', 'Fire', 'Smoke', 'Decoy'];
function syncStateControls() {
  for (const [name, value] of Object.entries(state)) {
    const input = $(`setting-${name}`); if (!input) continue;
    if (input.type === 'checkbox') input.checked = value; else input.value = value;
    const output = input.closest('.field').querySelector('output'); if (output) output.textContent = value;
  }
}
function stopRepeating() {
  clearTimeout(repeatTimer); clearTimeout(previewTimer); state.repeat = false;
  if ($('setting-repeat')) $('setting-repeat').checked = false;
}
function replayMarker() {
  clearTimeout(repeatTimer);
  if (!ready || galleryOpen) return;
  attempt(() => { marker_style(state.markerShape, state.animation); preview_hit(state.damage, state.kind, state.duration, state.scale, state.damageScale); });
  if (state.repeat) repeatTimer = setTimeout(replayMarker, Math.max(state.duration, 100) + 350);
}
function hostChanged(name) {
  if (hitFields.has(name)) { clearTimeout(previewTimer); previewTimer = setTimeout(replayMarker, 60); }
  else if (name === 'dotShape' || name === 'fillAlpha') attempt(publishAppearance);
  else { clearInterval(grenadeTimer); simulation(); }
}
function clearWeaponState() {
  clearInterval(grenadeTimer);
  Object.assign(state, { spread: 0, shot: 0, recoilX: 0, recoilY: 0, scope: 0, friendly: false, grenade: 0, held: 0, spectator: 0 });
}
function resetSimulation() { clearWeaponState(); syncStateControls(); simulation(); }
function holdGrenade(kind, animate = false) {
  clearWeaponState(); state.grenade = kind;
  const delay = Number(settings[`cl_grenadecrosshairdelay_${grenadeNames[kind].toLowerCase()}`]) * 1000;
  state.held = animate ? 0 : Math.min(5000, Math.ceil(delay) + 1);
  syncStateControls(); simulation();
  if (animate) {
    const start = performance.now();
    grenadeTimer = setInterval(() => {
      state.held = Math.min(5000, Math.round(performance.now() - start));
      syncStateControls(); simulation();
      if (state.held >= 5000) clearInterval(grenadeTimer);
    }, 100);
  }
}
function scenario(name) {
  clearWeaponState();
  if (name === 'split') {
    settings.cl_crosshairstyle = '2'; settings.cl_crosshair_dynamic_maxdist_splitratio = '0.5';
    settings.cl_crosshair_dynamic_splitdist = '3'; state.spread = 40;
    for (const key of ['cl_crosshairstyle', 'cl_crosshair_dynamic_maxdist_splitratio', 'cl_crosshair_dynamic_splitdist']) {
      const input = $(`setting-${key}`); input.value = settings[key];
      const output = input.closest('.field').querySelector('output'); if (output) output.textContent = settings[key];
    }
    publish();
  }
  if (name === 'sniper') { state.scope = 1; state.spread = 32; }
  if (name === 'cycling') state.scope = 2;
  if (name === 'ready') state.scope = 3;
  if (name === 'ironsight') state.scope = 4;
  if (name === 'friendly') state.friendly = true;
  if (name === 'friend') state.spectator = 1;
  if (name === 'stranger') state.spectator = 2;
  if (name === 'bot') state.spectator = 4;
  syncStateControls(); simulation();
}
function contextualSetting(name) {
  if (name.startsWith('cl_grenadecrosshair')) {
    const kind = grenadeNames.findIndex(grenade => name.endsWith(`_${grenade.toLowerCase()}`));
    holdGrenade(kind > 0 ? kind : state.grenade || 4);
  } else if (name === 'cl_sniper_delay_unscope') scenario('cycling');
  else if (name === 'cl_sniper_auto_rezoom') scenario('ready');
  else if (name.startsWith('cl_sniper_') || name === 'cl_crosshair_sniper_width') scenario('sniper');
  else if (name.startsWith('cl_ironsight_')) scenario('ironsight');
  else if (name === 'cl_show_observer_crosshair') scenario('stranger');
  else if (name === 'cl_observed_bot_crosshair') scenario('bot');
  else if (name === 'cl_crosshair_friendly_warning') scenario('friendly');
}
function updateContext() {
  let context = '';
  if (state.grenade) {
    const name = grenadeNames[state.grenade]; const key = name.toLowerCase();
    const enabled = ['true', '1'].includes(settings[`cl_grenadecrosshair_${key}`]);
    const delay = Number(settings[`cl_grenadecrosshairdelay_${key}`]);
    context = `${name} held ${(state.held / 1000).toFixed(1)}s · ${!enabled ? 'guide disabled' : state.held >= delay * 1000 ? 'guide visible' : `guide after ${delay}s`}`;
  } else if (state.scope) context = ['','Sniper scope', 'Weapon cycling after shot', 'Weapon ready after cycling', 'Optical sight'][state.scope];
  else if (state.spectator) context = ['', 'Watching a friend', 'Watching a stranger', 'Watching a bot · takeover available', 'Watching a bot · no takeover'][state.spectator];
  else if (state.friendly) context = 'Aiming at a teammate';
  else if (state.spread || state.shot || state.recoilX || state.recoilY) context = `Manual simulation · ${state.spread}px spread`;
  $('preview-context').hidden = !context; $('context-status').textContent = context;
}
function galleryColumns() { return narrow.matches ? 5 : 10; }
function renderCollection() {
  browse(collection.map(preset => text(preset)).join('\0'), galleryColumns());
  $('gallery-grid').replaceChildren(...collection.map((preset, index) => {
    const button = document.createElement('button');
    button.setAttribute('aria-label', `Edit crosshair ${index + 1}: ${styles[Number(preset.cl_crosshairstyle)]}`);
    button.setAttribute('aria-pressed', String(index === selectedPreset));
    const number = document.createElement('span'); number.textContent = index + 1; button.append(number);
    button.addEventListener('click', () => {
      galleryScroll = $('viewport').scrollTop; selectedPreset = index; settings = { ...preset };
      state.fillAlpha = 0; state.dotShape = 0; resetSimulation(); closeGallery(); controls(); publish();
      selectTab($('tab-shape')); document.querySelector('.editor').scrollTop = 0;
      $('setting-cl_crosshairstyle').focus({ preventScroll: true });
    });
    return button;
  }));
  $('viewport').scrollTop = galleryScroll;
}
function openGallery() {
  stopRepeating(); clearInterval(grenadeTimer);
  if (!collection.length) collection = makePresets(parse(defaults()));
  galleryOpen = true; document.body.classList.add('gallery-mode');
  $('gallery-grid').hidden = false; $('regenerate').hidden = false;
  $('browse').textContent = 'Back to editor'; $('style-name').textContent = '100 random crosshairs';
  attempt(renderCollection);
  const buttons = $('gallery-grid').querySelectorAll('button'); buttons[selectedPreset ?? 0]?.focus({ preventScroll: true });
}
function closeGallery() {
  galleryScroll = $('viewport').scrollTop;
  galleryOpen = false; document.body.classList.remove('gallery-mode');
  $('gallery-grid').hidden = true; $('regenerate').hidden = true; $('viewport').scrollTop = 0;
  $('browse').textContent = selectedPreset === null ? 'Browse 100' : 'Back to gallery';
  attempt(() => browse('', galleryColumns())); publish();
}
$('browse').addEventListener('click', () => { if (galleryOpen) { closeGallery(); $('browse').focus(); } else openGallery(); });
$('regenerate').addEventListener('click', () => { collection = makePresets(parse(defaults())); selectedPreset = null; galleryScroll = 0; attempt(renderCollection); });
$('aim-normal').addEventListener('click', resetSimulation);
narrow.addEventListener('change', () => { if (galleryOpen) attempt(renderCollection); });
reducedMotion.addEventListener('change', () => { if (reducedMotion.matches) { stopRepeating(); state.animation = 0; syncStateControls(); } });
const tabs = [...document.querySelectorAll('[role=tab]')];
function selectTab(tab) {
  if (tab.id === 'tab-hit') replayMarker(); else stopRepeating();
  tabs.forEach((button) => { const selected = button === tab; button.setAttribute('aria-selected', selected); button.tabIndex = selected ? 0 : -1; $(button.getAttribute('aria-controls')).hidden = !selected; });
}
tabs.forEach((tab, index) => { tab.addEventListener('click', () => selectTab(tab)); tab.addEventListener('keydown', (event) => { if (['ArrowLeft', 'ArrowRight'].includes(event.key)) { event.preventDefault(); const next = tabs[(index + (event.key === 'ArrowRight' ? 1 : -1) + tabs.length) % tabs.length]; selectTab(next); next.focus(); } }); });
$('sound').addEventListener('change', async () => { if ($('sound').checked) { audio ??= new AudioContext(); await audio.resume(); } });
$('fire').addEventListener('click', () => { if (ready) attempt(() => { marker_style(state.markerShape, state.animation); fire(state.damage, state.kind, state.duration, state.scale, state.damageScale); }); });
$('apply').addEventListener('click', () => attempt(() => { configure($('configuration').value); settings = parse(defaults()); $('configuration').value.split(/[;\n]/).forEach((assignment) => { const match = assignment.trim().match(/^(\S+)\s+"?([^"\s]+)"?$/); if (match) settings[match[1]] = match[2]; }); controls(); publish(); }));
$('copy').addEventListener('click', async () => { try { await navigator.clipboard.writeText($('configuration').value); $('copy').textContent = 'Copied'; setTimeout(() => $('copy').textContent = 'Copy', 1500); } catch { error('Copy failed. Select the configuration text and copy it manually.'); } });
$('reset').addEventListener('click', () => { settings = parse(defaults()); state.fillAlpha = 0; state.dotShape = 0; resetSimulation(); controls(); publish(); });
window.addEventListener('unhandledrejection', (event) => error(`Renderer failed: ${event.reason}. Reload the page to retry.`));
try { await init(); settings = parse(defaults()); controls(); publish(); } catch (cause) { $('loading').textContent = 'Renderer unavailable'; error(`Could not start Bevy: ${cause}. Reload the page to retry.`); }
