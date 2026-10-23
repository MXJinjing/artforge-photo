// Exercise the real consent store and browser link with mocked HarmonyOS services.
// Usage: node scripts/test_privacy_consent.cjs <typescript module>
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const ts = require(process.argv[2] || 'typescript');
const source = fs.readFileSync('entry/src/main/ets/privacy/PrivacyConsent.ets', 'utf8');
const compiled = ts.transpileModule(source, {
  compilerOptions: { target: ts.ScriptTarget.ES2020, module: ts.ModuleKind.CommonJS }
});
const memory = new Map();
let disk = new Map();
let readFails = false, flushFails = false;
const store = {
  getSync: (k, fallback) => memory.get(k) ?? fallback,
  putSync: (k, v) => memory.set(k, v),
  deleteSync: k => memory.delete(k),
  async flush() { if (flushFails) throw Error('disk full'); disk = new Map(memory); }
};
function load() {
  const module = { exports: {} };
  vm.runInNewContext(compiled.outputText, {
    module, exports: module.exports, console: { info() {}, error() {} },
    require(name) {
      assert.equal(name, '@kit.ArkData', 'no AppGallery dependency');
      return { preferences: { getPreferencesSync() { if (readFails) throw Error('store unavailable'); return store; } } };
    }
  });
  return module.exports;
}
(async () => {
  let api = load();
  const wants = [];
  const context = { async startAbility(want) { wants.push(want); } };
  assert.equal(api.queryPrivacyAgreed(context), false, 'clean install requires consent');
  api.openHostedPrivacyStatement(context);
  assert.equal(wants[0].uri, 'https://www.a999.cloud/statics/artforge-photo-privacy-policy.html');
  assert.equal(wants[0].action, 'ohos.want.action.viewData');
  assert.equal(api.queryPrivacyAgreed(context), false, 'reading policy does not consent');
  readFails = true;
  assert.equal(api.queryPrivacyAgreed(context), false, 'read error must not bypass gate');
  readFails = false; flushFails = true;
  await assert.rejects(api.savePrivacyConsent(context));
  assert.equal(api.queryPrivacyAgreed(context), false, 'failed flush must roll back cached acceptance');
  flushFails = false;
  await api.savePrivacyConsent(context);
  memory.clear(); for (const [k, v] of disk) memory.set(k, v);
  api = load();
  assert.equal(api.queryPrivacyAgreed(context), true, 'explicit consent survives process restart');
  memory.set('acceptedVersion', 'old-policy');
  assert.equal(api.queryPrivacyAgreed(context), false, 'changed policy requires fresh consent');
  memory.clear(); disk.clear();
  assert.equal(api.queryPrivacyAgreed(context), false, 'clearing app data restores gate');
  // Execute the actual page action methods without needing an ArkUI renderer.
  const page = fs.readFileSync('entry/src/main/ets/pages/Index.ets', 'utf8');
  const actions = page.slice(page.indexOf('  private startAfterPrivacyConsent()'),
    page.indexOf('  private async initializeHost('));
  const back = page.slice(page.indexOf('  onBackPress()'), page.indexOf('  onChromeChanged()'));
  const calls = [];
  const module = { exports: {} };
  const harness = ts.transpileModule(`export class Harness { ${actions} ${back} }`, {
    compilerOptions: { target: ts.ScriptTarget.ES2020, module: ts.ModuleKind.CommonJS }
  });
  let saveFails = false;
  vm.runInNewContext(harness.outputText, {
    module, exports: module.exports,
    PRIVACY_ALLOWED_KEY: 'allowed',
    AppStorage: { setOrCreate: (...args) => calls.push(args) },
    savePrivacyConsent: async () => { if (saveFails) throw Error('disk full'); calls.push(['save']); },
    native: { input() { throw Error('native input before consent'); } }
  });
  const view = new module.exports.Harness();
  Object.assign(view, {
    alive: true, privacyWaiting: true, privacySaving: false, privacyReading: true,
    generation: 0, printing: { start: () => calls.push(['print']) },
    initializeHost: () => calls.push(['init']),
    context: { terminateSelf: async () => calls.push(['exit']) }
  });
  view.onBackPress();
  assert.equal(view.privacyReading, false, 'back from policy returns to choice');
  assert.equal(view.privacyWaiting, true);
  assert.equal(calls.length, 0, 'reading or returning does not start editor');
  view.onBackPress();
  assert.equal(calls.pop()[0], 'exit', 'back at prompt exits without accepting');
  saveFails = true;
  await view.agreePrivacy();
  assert.equal(view.privacyWaiting, true);
  assert.equal(calls.length, 0, 'failed save cannot initialize editor');
  saveFails = false;
  await view.agreePrivacy();
  assert.equal(view.privacyWaiting, false);
  assert.deepEqual(calls.map(call => call[0]), ['save', 'allowed', 'print', 'init']);
  await view.agreePrivacy();
  assert.equal(calls.filter(call => call[0] === 'init').length, 1, 'double click does not initialize twice');
  console.log('Passed: consent persistence, website link, storage failures, policy version, prompt/back/agree actions and initialization gate.');
})().catch(error => { console.error(error); process.exitCode = 1; });
