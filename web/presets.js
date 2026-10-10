// Randomize valid console settings once per collection, never during rendering.
export function makePresets(defaults, random = Math.random) {
  const colors = ['00ff80', '00dcff', 'ffd040', 'ff608c', 'ffffff', 'bfa0ff', '8dff36', 'ff9944'];
  const integer = (min, max) => min + Math.floor(random() * (max - min + 1));
  return Array.from({ length: 100 }, (_, index) => {
    const settings = { ...defaults };
    const color = colors[integer(0, colors.length - 1)];
    Object.assign(settings, {
      cl_crosshairstyle: String(index % 10),
      cl_crosshair_length: String(integer(3, 18)),
      cl_crosshair_thickness: String(integer(1, 5)),
      cl_crosshair_gap: String(integer(2, 18)),
      cl_crosshairdot: String(integer(0, 1)),
      cl_crosshair_t: String(integer(0, 4) === 0),
      cl_crosshair_drawoutline: String(integer(0, 2)),
      cl_crosshair_dynamic_maxdist_splitratio: String(integer(15, 80) / 100),
      cl_crosshaircolor_r: String(parseInt(color.slice(0, 2), 16)),
      cl_crosshaircolor_g: String(parseInt(color.slice(2, 4), 16)),
      cl_crosshaircolor_b: String(parseInt(color.slice(4, 6), 16)),
    });
    return settings;
  });
}
