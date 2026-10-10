// Evaluate this expression in the loaded playground at each test viewport.
(async () => {
  const failures = [];
  const check = (condition, message) => { if (!condition) failures.push(message); };
  const byId = id => document.getElementById(id);
  const viewport = byId('viewport');
  const canvas = byId('crosshair');
  const bounds = viewport.getBoundingClientRect();
  const rendered = canvas.getBoundingClientRect();
  check(document.title === 'bevy-crosshair', 'Project title must be kebab-cased');
  check(document.documentElement.scrollHeight <= innerHeight + 1, 'Page exceeds window height');
  check(document.documentElement.scrollWidth <= innerWidth + 1, 'Page exceeds window width');
  check(Math.abs(rendered.width - bounds.width) < 1 && Math.abs(rendered.height - bounds.height) < 1, 'Canvas must fit its container');
  check(bounds.height > 0 && bounds.bottom <= innerHeight, 'Preview must remain visible');
  check(byId('fire').getBoundingClientRect().bottom <= innerHeight, 'Fire must remain visible');
  const editor = document.querySelector('.editor');
  const previousScroll = editor.scrollTop;
  editor.scrollTop = 700;
  check(editor.scrollTop > 0, 'Settings must scroll independently');
  check(viewport.getBoundingClientRect().top === bounds.top, 'Settings scrolling moved the preview');
  editor.scrollTop = previousScroll;

  const selector = byId('background');
  check(selector?.options.length === 11, 'Ten game scenes and a solid background must be available');
  for (const game of ['et-', 'db-']) check([...selector.options].filter(option => option.value.startsWith(game)).length >= 3, `${game} requires at least three distinct maps`);
  if (selector) {
    const originalImage = window.Image;
    const previousSelection = selector.value;
    const requests = [];
    const flush = () => new Promise(resolve => queueMicrotask(resolve));
    const select = value => { selector.value = value; selector.dispatchEvent(new Event('change')); };
    // Resolve image requests in a controlled order to exercise stale completions.
    window.Image = class {
      decode() { return new Promise((resolve, reject) => requests.push({ resolve, reject })); }
    };
    try {
      select('cs2-dust2');
      select('valorant-ascent');
      requests[1].resolve(); await flush();
      const newest = viewport.style.backgroundImage;
      check(newest.includes('93f5b5f274df64d938051c4ca8577995190e711a'), 'Newest background did not render');
      requests[0].resolve(); await flush();
      check(viewport.style.backgroundImage === newest, 'A stale image replaced the current background');
      check(byId('background-credit').textContent === 'Riot Games', 'Background credit did not follow the selection');

      select('cs2-overpass');
      requests[2].reject(new Error('Unavailable image')); await flush();
      check(viewport.style.backgroundImage === 'none' && !byId('background-status').hidden, 'Failed background must show recovery text over the solid fallback');
      select('valorant-icebox');
      select('solid');
      requests[3].reject(new Error('Stale failure')); await flush();
      check(viewport.style.backgroundImage === 'none' && byId('background-status').hidden && byId('background-credit').hidden, 'Solid background must ignore stale failures and clear status');
    } finally {
      window.Image = originalImage;
      select(previousSelection);
    }
  }
  return { passed: failures.length === 0, failures, viewport: [innerWidth, innerHeight, devicePixelRatio], canvas: [canvas.width, canvas.height] };
})()
