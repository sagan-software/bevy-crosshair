import assert from 'node:assert/strict';
import test from 'node:test';
import { makePresets } from '../web/presets.js';

test('all 100 samples retain defaults and span ten styles at both random boundaries', () => {
  for (const random of [() => 0, () => 1 - Number.EPSILON]) {
    const defaults = { crosshair: 'true', cl_crosshair_screen_height: '1080' };
    const presets = makePresets(defaults, random);
    assert.equal(presets.length, 100);
    assert.equal(new Set(presets.map(preset => preset.cl_crosshairstyle)).size, 10);
    for (const preset of presets) {
      assert.equal(preset.crosshair, 'true');
      assert.equal(preset.cl_crosshair_screen_height, '1080');
      assert.ok(+preset.cl_crosshair_length >= 3 && +preset.cl_crosshair_length <= 18);
      assert.ok(+preset.cl_crosshair_thickness >= 1 && +preset.cl_crosshair_thickness <= 5);
      assert.ok(+preset.cl_crosshair_gap >= 2 && +preset.cl_crosshair_gap <= 18);
      assert.ok(+preset.cl_crosshair_dynamic_maxdist_splitratio >= .15 && +preset.cl_crosshair_dynamic_maxdist_splitratio <= .8);
      for (const channel of ['r', 'g', 'b']) assert.ok(+preset[`cl_crosshaircolor_${channel}`] >= 0 && +preset[`cl_crosshaircolor_${channel}`] <= 255);
    }
    presets[0].crosshair = 'false';
    assert.equal(presets[1].crosshair, 'true');
    assert.equal(defaults.crosshair, 'true');
  }
});
