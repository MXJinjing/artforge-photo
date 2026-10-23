const { chromium } = require('/Users/wjj/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const path = require('node:path');
const { pathToFileURL } = require('node:url');
(async () => {
  const browser = await chromium.launch({ executablePath: '/Applications/Chromium.app/Contents/MacOS/Chromium', headless: true });
  const page = await browser.newPage({ viewport: { width: 1920, height: 1280 }, deviceScaleFactor: 1 });
  const errors=[]; page.on('pageerror',e=>errors.push(String(e)));
  await page.goto(pathToFileURL(path.join(__dirname,'index.html')).href+'?export');
  await page.evaluate(async()=>{await document.fonts.ready;await Promise.all([...document.images].map(im=>im.decode()));});
  const names=['01-home','02-layers','03-color','04-curves','05-brush'];
  for(let i=0;i<names.length;i++){
    await page.evaluate(i=>showPoster(i),i);
    const title=await page.locator('.scene.active h1, .scene.active h2').innerText();
    for(const ext of ['png','jpg']) await page.screenshot({path:path.join(__dirname,`text-right50-${names[i]}-1920x1280.${ext}`),type:ext==='jpg'?'jpeg':'png',...(ext==='jpg'?{quality:96}:{})});
    console.log(names[i],title.replace(/\n/g,' '));
  }
  await browser.close();
  if(errors.length)throw new Error(errors.join('\n'));
})().catch(e=>{console.error(e);process.exit(1)});
