// Render the actual HTML timeline to H.264, with deterministic 30 fps frames.
const { chromium } = require('/Users/wjj/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const { spawn } = require('node:child_process');
const { once } = require('node:events');
const path = require('node:path');
const { pathToFileURL } = require('node:url');
const root = __dirname;
(async () => {
  const browser = await chromium.launch({ executablePath: '/Applications/Chromium.app/Contents/MacOS/Chromium', headless: true });
  const page = await browser.newPage({ viewport: { width: 1920, height: 1280 }, deviceScaleFactor: 1 });
  const errors = [];
  page.on('pageerror', err => errors.push(String(err)));
  await page.goto(pathToFileURL(path.join(root, 'index.html')).href + '?export');
  await page.evaluate(async () => { await document.fonts.ready; await window.assetsReady; await Promise.all([...document.images].map(im => im.decode())); });
  for (const t of [2.5, 6.5, 7.5, 11, 12.5, 17, 21, 24.5, 27.5, 31.5]) {
    await page.evaluate(t => renderAt(t), t);
    await page.screenshot({ path: path.join(root, `preview-${t}.jpg`), type: 'jpeg', quality: 92 });
  }
  if (process.argv.includes('--preview')) { await browser.close(); console.log('Previews ready', errors); return; }
  const ff = spawn('ffmpeg', ['-y', '-hide_banner', '-loglevel', 'error', '-f', 'image2pipe', '-vcodec', 'mjpeg', '-framerate', '30', '-i', 'pipe:0', '-i', path.join(root, 'music.wav'), '-c:v', 'libx264', '-preset', 'fast', '-crf', '18', '-pix_fmt', 'yuv420p', '-c:a', 'aac', '-b:a', '192k', '-t', '34', '-movflags', '+faststart', path.join(root, 'artforge-photo-intro-1920x1280.mp4')], { stdio: ['pipe', 'inherit', 'inherit'] });
  const done = once(ff, 'close');
  for (let i = 0; i < 34 * 30; i++) {
    // Keep the first and last video frames visible; crossfades apply internally.
    await page.evaluate(t => renderAt(t), Math.max(.4, Math.min(33.6, i / 30)));
    const jpg = await page.screenshot({ type: 'jpeg', quality: 94 });
    if (!ff.stdin.write(jpg)) await once(ff.stdin, 'drain');
    if (i % 150 === 0) console.log(`Rendered ${i} / 1020 frames`);
  }
  ff.stdin.end();
  const [code] = await done;
  await browser.close();
  if (errors.length || code !== 0) throw new Error(JSON.stringify({ errors, code }));
  console.log('Export complete: 1920 × 1280, 30 fps, 34 seconds.');
})().catch(err => { console.error(err); process.exit(1); });
