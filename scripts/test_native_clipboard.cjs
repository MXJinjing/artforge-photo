// node scripts/test_native_clipboard.cjs <typescript module>
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
function loadTypeScript() {
  const candidates = [
    process.argv[2],
    process.env.PHOTOCRAFT_TYPESCRIPT,
    'typescript',
    '/Applications/DevEco-Studio.app/Contents/tools/ohpm/node_modules/typescript'
  ].filter(Boolean);
  for (const candidate of candidates) {
    try { return require(candidate); }
    catch (error) {
      if (error.code !== 'MODULE_NOT_FOUND') throw error;
    }
  }
  throw new Error('TypeScript not found; pass its module directory as the first argument');
}
const ts = loadTypeScript();

const alpha = { OPAQUE: 1, PREMUL: 2, UNPREMUL: 3 };
const format = { RGBA_8888: 3, BGRA_8888: 4 };
const calls = [];
let granted = true;
let current;
let active = true;
let reads = 0;
let pendingWrites = 0;
let resumeRead;
let resumeWrite;
let delayRead;
let delayWrite;
let fallbackMap;

class PixelMap {
  constructor(width = 1, height = 1, pixels = [25, 50, 100, 128],
    alphaType = alpha.PREMUL, pixelFormat = format.BGRA_8888,
    failColor = false, failRead = false) {
    this.info = { size: { width, height }, alphaType, pixelFormat };
    this.pixels = pixels;
    this.released = 0;
    this.failColor = failColor;
    this.failRead = failRead;
  }
  async getImageInfo() { return this.info; }
  async applyColorSpace(space) {
    assert.equal(space, 'sRGB');
    if (this.failColor) throw new Error('source format is wrong!');
    calls.push('color');
  }
  async convertPixelFormat() { throw new Error('format conversion must not be needed'); }
  async readPixels(area) {
    if (this.failRead) throw new Error('source format is wrong!');
    assert.equal(area.stride, this.info.size.width * 4);
    new Uint8Array(area.pixels).set(this.pixels);
  }
  async release() { this.released++; }
}

const image = {
  AlphaType: alpha,
  PixelMapFormat: format,
  async createPixelMap(bytes, options) {
    assert.equal(options.srcPixelFormat, format.RGBA_8888);
    assert.equal(options.alphaType, alpha.UNPREMUL);
    const map = new PixelMap(options.size.width, options.size.height,
      [...new Uint8Array(bytes)], options.alphaType, options.pixelFormat);
    calls.push(map);
    return map;
  },
  createImageSource(fd) {
    if (fd instanceof ArrayBuffer) {
      return {
        async getImageInfo() { return fallbackMap.info; },
        async createPixelMap() { return fallbackMap; },
        async release() { calls.push('fallback-source-release'); }
      };
    }
    assert.equal(fd, 42);
    return {
      async getImageInfo() { return { size: { width: 1, height: 1 } }; },
      async createPixelMap() { return current.uriMap; },
      async release() { calls.push('source-release'); }
    };
  },
  createImagePacker() {
    return {
      async packToData(source, options) {
        assert.equal(source.failRead, true);
        assert.equal(options.format, 'image/png');
        calls.push('fallback-pack');
        return new ArrayBuffer(4);
      },
      async release() { calls.push('packer-release'); }
    };
  }
};

function pasteData(map, text = '', uri = '') {
  const types = map !== undefined ? ['pixelMap'] : uri.length > 0 ? ['text/uri'] : ['text/plain'];
  const record = {
    mimeType: types[0],
    pixelMap: map,
    uri,
    getMimeTypes: () => types,
    async getData() { return map; }
  };
  return {
    getRecordCount: () => 1,
    getRecord: () => record,
    getPrimaryPixelMap: () => map,
    getPrimaryText: () => text,
    getPrimaryUri: () => uri,
    hasType: type => type === 'pixelMap' && map !== undefined,
    pasteStart() { calls.push('paste-start'); },
    pasteComplete() { calls.push('paste-complete'); }
  };
}

const board = {
  async getData() {
    reads++;
    if (delayRead) await delayRead;
    return current;
  },
  async setData(data) {
    assert.equal(pendingWrites++, 0, 'copy operations must be serialized');
    calls.push(data);
    if (delayWrite) await delayWrite;
    pendingWrites--;
  }
};
const mock = {
  '@kit.BasicServicesKit': { pasteboard: {
    getSystemPasteboard: () => board,
    MIMETYPE_PIXELMAP: 'pixelMap',
    MIMETYPE_TEXT_PLAIN: 'text/plain',
    createData: (type, value) => ({ type, value })
  } },
  '@kit.AbilityKit': { abilityAccessCtrl: { createAtManager: () => ({
    async requestPermissionsFromUser() {
      calls.push('permission');
      return { authResults: [granted ? 0 : -1] };
    }
  }) } },
  '@kit.ImageKit': { image },
  '@kit.ArkGraphics2D': {
    colorSpaceManager: {
      create: value => value,
      ColorSpace: { SRGB: 'sRGB' }
    }
  },
  '@kit.CoreFileKit': { fileIo: {
    OpenMode: { READ_ONLY: 0 },
    async open(uri) {
      assert(uri.startsWith('file://'));
      return { fd: 42 };
    },
    async close(file) {
      assert.equal(file.fd, 42);
      calls.push('file-close');
    }
  } }
};

const moduleOutput = { exports: {} };
const transpiled = ts.transpileModule(
  fs.readFileSync('entry/src/main/ets/input/NativeClipboardAdapter.ets', 'utf8'),
  {
    fileName: 'NativeClipboardAdapter.ts',
    reportDiagnostics: true,
    compilerOptions: {
      target: ts.ScriptTarget.ES2020,
      module: ts.ModuleKind.CommonJS
    }
  }
);
assert.deepEqual(
  (transpiled.diagnostics || []).filter(diagnostic =>
    diagnostic.category === ts.DiagnosticCategory.Error),
  []
);
vm.runInNewContext(transpiled.outputText, {
  require: name => {
    assert(mock[name], name);
    return mock[name];
  },
  module: moduleOutput,
  exports: moduleOutput.exports,
  ArrayBuffer,
  Uint8Array,
  Number,
  JSON,
  Error,
  Math,
  Promise,
  console
});

const { NativeClipboardAdapter, clipboardByteLength, straightAlpha } = moduleOutput.exports;
const adapter = new NativeClipboardAdapter({});
const run = (kind, text = '', bytes = new ArrayBuffer(0)) =>
  adapter.handle(kind, text, bytes, () => active);

(async () => {
  assert.equal(reads, 0, 'creating the adapter must not read the clipboard');
  for (const dimensions of [
    [0, 1], [1, -1], [NaN, 1], [1.5, 1], [8192, 8192], [Infinity, 1]
  ]) {
    assert.throws(() => clipboardByteLength(...dimensions));
  }

  const transparent = new Uint8Array([12, 24, 36, 0, 1, 2, 3, 255]);
  straightAlpha(transparent.buffer, alpha.PREMUL);
  assert.deepEqual([...transparent], [0, 0, 0, 0, 1, 2, 3, 255]);
  const opaque = new Uint8Array([1, 2, 3, 0]);
  straightAlpha(opaque.buffer, alpha.OPAQUE);
  assert.equal(opaque[3], 255);

  current = pasteData(new PixelMap());
  let result = await run('clipboard_get_image');
  assert.equal(result.text, '[1,1]');
  assert.deepEqual([...new Uint8Array(result.bytes)], [199, 100, 50, 128]);
  assert.equal(current.getPrimaryPixelMap().released, 1);
  assert(calls.includes('paste-start') && calls.includes('paste-complete'));

  current = pasteData(new PixelMap(1, 1, [10, 20, 30, 255], alpha.UNPREMUL, 99));
  result = await run('clipboard_get_image');
  assert.deepEqual([...new Uint8Array(result.bytes)], [30, 20, 10, 255],
    'readPixels handles non-BGRA source formats without convertPixelFormat');
  current = pasteData(new PixelMap(1, 1, [0, 255, 0, 255], alpha.OPAQUE, 99, true));
  result = await run('clipboard_get_image');
  assert.deepEqual([...new Uint8Array(result.bytes)], [0, 255, 0, 255],
    'best-effort color conversion failure must not abort the paste');
  fallbackMap = new PixelMap(1, 1, [1, 2, 3, 4], alpha.UNPREMUL, format.RGBA_8888);
  current = pasteData(new PixelMap(1, 1, [0, 0, 0, 0], alpha.OPAQUE, 99, false, true));
  result = await run('clipboard_get_image');
  assert.deepEqual([...new Uint8Array(result.bytes)], [3, 2, 1, 4],
    'failed direct extraction must retry through PNG and a normalized PixelMap');
  assert(calls.includes('fallback-pack') && calls.includes('packer-release') &&
    calls.includes('fallback-source-release'));

  fallbackMap = new PixelMap(1, 1, [9, 8, 7, 6], alpha.UNPREMUL, format.RGBA_8888);
  current = {
    getRecordCount: () => 1,
    getRecord: () => ({
      mimeType: 'image/png',
      pixelMap: undefined,
      uri: '',
      getMimeTypes: () => ['image/png'],
      async getData() { return new ArrayBuffer(4); }
    }),
    getPrimaryText: () => '',
    pasteStart() { calls.push('paste-start'); },
    pasteComplete() { calls.push('paste-complete'); }
  };
  result = await run('clipboard_get_image');
  assert.deepEqual([...new Uint8Array(result.bytes)], [7, 8, 9, 6],
    'generic image records must be decoded without relying on the primary record');

  current = pasteData(undefined, '中文 clipboard');
  result = await run('clipboard_get_text');
  assert.equal(result.text, '中文 clipboard');
  result = await run('clipboard_get_image');
  assert.equal(result.text, '');

  current = pasteData(undefined, '', 'https://example.invalid/image.png');
  result = await run('clipboard_get_image');
  assert.equal(result.bytes.byteLength, 0, 'network image URIs must remain unread');
  current = pasteData(undefined, '', 'file:///image.png');
  current.uriMap = new PixelMap();
  await run('clipboard_get_image');
  assert.equal(current.uriMap.released, 1);
  assert(calls.includes('file-close') && calls.includes('source-release'));

  const rgba = new Uint8Array([255, 0, 0, 127]);
  await run('clipboard_set_image', '[1,1]', rgba.buffer);
  const copied = calls.find(value => value && value.type === 'pixelMap');
  assert.deepEqual(copied.value.pixels, [...rgba]);
  assert.equal(copied.value.released, 1);
  await assert.rejects(run('clipboard_set_image', '[2,1]', rgba.buffer));

  granted = false;
  const beforeDeniedRead = reads;
  await assert.rejects(run('clipboard_get_image'), /允许/);
  assert.equal(reads, beforeDeniedRead, 'denied permission must not read the pasteboard');
  granted = true;

  delayWrite = new Promise(resolve => { resumeWrite = resolve; });
  const first = run('clipboard_set_text', 'first');
  const second = run('clipboard_set_text', 'second');
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(pendingWrites, 1);
  resumeWrite();
  await Promise.all([first, second]);
  delayWrite = undefined;
  assert.deepEqual(
    calls.filter(value => value && value.type === 'text/plain').map(value => value.value),
    ['first', 'second']
  );

  active = false;
  await assert.rejects(run('clipboard_set_text', 'stale'), /cancelled/);
  active = true;
  delayRead = new Promise(resolve => { resumeRead = resolve; });
  const stale = run('clipboard_get_image');
  await new Promise(resolve => setImmediate(resolve));
  active = false;
  resumeRead();
  await assert.rejects(stale, /cancelled/);

  console.log('Native clipboard: image/text, alpha, URI, permission, ordering and lifecycle tests passed');
})().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
