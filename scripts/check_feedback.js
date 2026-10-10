// Evaluate in the loaded playground. Uses public controls and the WebAssembly boundary.
(async () => {
  const failures = [];
  let assertions = 0;
  const check = (condition, message) => { assertions += 1; if (!condition) failures.push(message); };
  const $ = id => document.getElementById(id);
  const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
  const change = (name, value) => {
    const input = $(`setting-${name}`);
    if (input.type === 'checkbox') input.checked = value; else input.value = value;
    input.dispatchEvent(new Event('input', { bubbles: true }));
  };
  const waitFor = async predicate => {
    const start = performance.now();
    while (!predicate() && performance.now() - start < 5000) await pause(50);
    return predicate();
  };
  const api = await import(new URL('./pkg/playground.js', location.href));
  const rejected = (action, message) => { try { action(); check(false, message); } catch { check(true, message); } };
  $('reset').click();
  $('tab-shape').click();
  check(!$('fire').disabled, 'The renderer must be ready');
  // New visual controls leave the CS2 text profile intact.
  const original = $('configuration').value;
  // Every console setting remains reachable, including channels grouped in color pickers.
  const names = api.defaults().trim().split('\n').map(line => line.split(' ')[0]);
  check(names.length === 43, 'The complete 43-setting console profile must remain available');
  for (const name of names) {
    const colorPrefix = name.replace(/_[rgb]$/, '');
    check(Boolean($(`setting-${name}`) || $(colorPrefix)), `Missing control for ${name}`);
  }
  $('configuration').value = 'cl_crosshair_gap invalid'; $('apply').click();
  check(!$('error').hidden && $('setting-cl_crosshair_gap').value === '4', 'Invalid console input must leave settings unchanged');
  $('configuration').value = original; $('apply').click();
  check($('error').hidden, 'A valid configuration must recover from input errors');
  change('fillAlpha', 120); change('dotShape', 1);
  check($('configuration').value === original, 'Visual extensions must not add console settings');
  change('cl_crosshairdot', true);
  change('cl_crosshair_gap', 30);
  check($('configuration').value.includes('cl_crosshairdot true'), 'Center-dot control must publish');
  // State-dependent settings select an observable host state automatically.
  change('cl_grenadecrosshairdelay_smoke', 1.23);
  check($('setting-grenade').value === '4' && Number($('setting-held').value) >= 1230, 'Grenade setting must hold the selected grenade through its delay');
  check($('context-status').textContent.includes('guide visible'), 'Grenade context must explain its visible guide');
  change('cl_grenadecrosshair_smoke', false);
  check($('context-status').textContent.includes('guide disabled'), 'Disabled grenade guide must be explained');
  change('cl_grenadecrosshair_smoke', true);
  $('try-grenade').click();
  check(Number($('setting-held').value) === 0, 'Hold grenade must restart its delay');
  await pause(150);
  check(Number($('setting-held').value) > 0, 'Held duration must advance');
  $('try-sniper').click();
  check($('setting-scope').value === '1' && $('setting-grenade').value === '0', 'Sniper preview must leave grenade mode');
  change('cl_sniper_delay_unscope', false);
  check($('setting-scope').value === '2', 'Delay-unscope option must preview cycling');
  change('cl_sniper_auto_rezoom', false);
  check($('setting-scope').value === '3', 'Auto-rezoom option must preview readiness');
  change('cl_ironsight_dot_scale', 1.5);
  check($('setting-scope').value === '4', 'Optical-sight option must show that sight');
  $('try-split').click();
  check($('setting-cl_crosshairstyle').value === '2' && $('setting-spread').value === '40', 'Split preview must choose legacy arms and nonzero spread');
  check($('setting-cl_crosshair_dynamic_maxdist_splitratio').value === '0.5', 'Split preview needs inner and outer pieces');
  $('try-friend').click();
  check($('setting-spectator').value === '1', 'Friend preview must select the friend');
  change('cl_show_observer_crosshair', 2);
  check($('setting-spectator').value === '2', 'Observer policy must preview a stranger');
  change('cl_observed_bot_crosshair', 2);
  check($('setting-spectator').value === '4', 'Bot policy must preview unavailable takeover');
  $('aim-normal').click();
  check($('preview-context').hidden && $('setting-spectator').value === '0', 'Reset simulation must return to local unscoped play');
  // Visual replays must not reach the independent game-event observer.
  const receipt = $('receipt').textContent;
  $('tab-hit').click();
  for (let shape = 0; shape < 5; shape += 1) change('markerShape', shape);
  for (let animation = 0; animation < 4; animation += 1) change('animation', animation);
  change('damage', 72); change('duration', 100); change('kind', 1);
  change('repeat', true);
  await pause(650);
  check($('receipt').textContent === receipt, 'Marker edits and repeat preview must not dispatch hit events');
  $('replay-marker').click(); await pause(100);
  check($('receipt').textContent === receipt, 'Replay button must not dispatch a hit event');
  $('fire').click();
  check(await waitFor(() => $('receipt').textContent !== receipt), 'Fire must dispatch a real hit');
  check($('receipt').textContent.includes('Headshot · 72 damage'), 'Independent observer must receive current metadata');
  $('tab-shape').click();
  check(!$('setting-repeat').checked, 'Leaving the hit tab must stop automatic replay');
  // Preserve the collection, selection, and editable settings across navigation.
  $('tab-hit').click();
  $('browse').click();
  check(document.querySelectorAll('#gallery-grid button').length === 100, 'Gallery must contain 100 choices');
  check(await waitFor(() => window.crosshairMeasurements.at(-1)?.crosshairs === 100), 'Bevy must render 100 real crosshair entities');
  const labels = [...document.querySelectorAll('#gallery-grid button')].map(button => button.ariaLabel);
  $('viewport').scrollTop = 200;
  const scroll = $('viewport').scrollTop;
  document.querySelectorAll('#gallery-grid button')[37].click();
  check(!document.body.classList.contains('gallery-mode'), 'Selecting a crosshair must open its editor');
  check($('tab-shape').getAttribute('aria-selected') === 'true', 'Selecting a preset must show its crosshair settings even when browsing from Hits');
  check(document.querySelector('.editor').scrollTop === 0, 'Selected preset settings must start at Style');
  check($('setting-cl_crosshairstyle').value === '7', 'Selected preset must supply its actual settings');
  check(await waitFor(() => window.crosshairMeasurements.at(-1)?.crosshairs === 1), 'Only the editor entity must remain visible');
  change('cl_crosshair_gap', 42);
  $('browse').click();
  check([...document.querySelectorAll('#gallery-grid button')].map(button => button.ariaLabel).join('|') === labels.join('|'), 'Back must preserve the same collection');
  check(document.querySelectorAll('#gallery-grid button')[37].getAttribute('aria-pressed') === 'true', 'Selected crosshair must remain marked');
  check($('viewport').scrollTop === scroll, 'Back must preserve gallery scroll');
  $('browse').click();
  check($('setting-cl_crosshair_gap').value === '42', 'Returning to editor must preserve unsaved edits');
  $('browse').click(); $('regenerate').click();
  check(document.querySelectorAll('#gallery-grid button').length === 100 && !document.querySelector('#gallery-grid [aria-pressed="true"]'), 'Generate must replace all choices and clear selection');
  $('browse').click();
  // Reject invalid boundary values before changing the pending scene or marker.
  rejected(() => api.appearance(0, 0, 0, 255, 2), 'Invalid dot shape must fail');
  rejected(() => api.marker_style(5, 0), 'Invalid marker shape must fail');
  rejected(() => api.marker_style(0, 4), 'Invalid animation must fail');
  for (const points of [-1, 1001, NaN, Infinity]) rejected(() => api.preview_hit(points, 0, 100, 1, false), 'Invalid preview damage must fail');
  for (const scale of [-1, 9, NaN, Infinity]) rejected(() => api.preview_hit(10, 0, 100, scale, false), 'Invalid preview scale must fail');
  rejected(() => api.preview_hit(10, 3, 100, 1, false), 'Invalid hit kind must fail');
  rejected(() => api.browse('', 6), 'Invalid gallery column count must fail');
  rejected(() => api.browse(Array(101).fill(api.defaults()).join('\0'), 10), 'Gallery count must be bounded');
  rejected(() => api.browse(' '.repeat(1000001), 10), 'Gallery source bytes must be bounded');
  rejected(() => api.browse(`${api.defaults()}\0unknown 1`, 10), 'A malformed gallery member must reject the batch');
  for (const shape of [0, 1, 2, 3, 4]) for (const animation of [0, 1, 2, 3]) api.marker_style(shape, animation);
  for (const points of [0, 1000]) for (const scale of [0, 8]) api.preview_hit(points, 2, 0, scale, true);
  check(await waitFor(() => window.crosshairMeasurements.at(-1)?.crosshairs === 1), 'Rejected gallery changes must preserve the editor');
  check($('error').hidden, 'Valid controls must not produce errors');
  $('reset').click();
  change('duration', 600); change('damage', 40); change('kind', 0); change('markerShape', 0); change('animation', 3);
  return { passed: failures.length === 0, assertions, failures, viewport: [innerWidth, innerHeight] };
})()
