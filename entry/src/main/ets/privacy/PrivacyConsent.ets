import { common } from '@kit.AbilityKit';
import { BusinessError } from '@kit.BasicServicesKit';
import { preferences } from '@kit.ArkData';

/** 自有网站隐私政策；仅在用户明确同意并成功保存后启动编辑器。 */
export const PRIVACY_ALLOWED_KEY: string = 'artforgePrivacyAllowed';
export const PRIVACY_STATEMENT_URL: string = 'https://www.a999.cloud/statics/artforge-photo-privacy-policy.html';
// 涉及信息处理方式的政策更新时递增，使旧同意记录失效。
export const PRIVACY_CONSENT_VERSION: string = '2026-10-22.1';
const STORE_NAME: string = 'artforge_privacy_consent';
const VERSION_KEY: string = 'acceptedVersion';
const TAG: string = 'PrivacyConsent';

export function queryPrivacyAgreed(context: common.UIAbilityContext): boolean {
  try {
    let store = preferences.getPreferencesSync(context, { name: STORE_NAME });
    return store.getSync(VERSION_KEY, '') === PRIVACY_CONSENT_VERSION;
  } catch (error) {
    console.error(`${TAG}: read consent failed: ${(error as Error).message}`);
    return false;
  }
}

export async function savePrivacyConsent(context: common.UIAbilityContext): Promise<void> {
  let store = preferences.getPreferencesSync(context, { name: STORE_NAME });
  store.putSync(VERSION_KEY, PRIVACY_CONSENT_VERSION);
  try {
    await store.flush();
  } catch (error) {
    // putSync 已修改内存缓存，写盘失败时必须撤回，避免下次检查误放行。
    store.deleteSync(VERSION_KEY);
    throw error as Error;
  }
}

/** AppPrivacyLink.type：1 隐私声明，2 用户协议（兼容旧桥接调用）。 */
export const PRIVACY_LINK_STATEMENT: number = 1;
export const PRIVACY_LINK_AGREEMENT: number = 2;

/** Help / 桥接用的伪协议，由 openHostedPrivacyScheme 解析为网站 URL。 */
export const PRIVACY_SCHEME_STATEMENT: string = 'artforge-privacy://statement';
export const PRIVACY_SCHEME_AGREEMENT: string = 'artforge-privacy://agreement';

/**
 * 返回隐私声明的网站 URL；不再向华为托管查询链接。
 * 当前没有独立用户协议，其他类型返回空。
 */
export function readHostedPrivacyUrl(linkType: number): string {
  if (linkType === PRIVACY_LINK_STATEMENT) {
    return PRIVACY_STATEMENT_URL;
  }
  return '';
}

/** 返回应用网站上的隐私声明 URL。 */
export function readHostedPrivacyStatementUrl(): string {
  return readHostedPrivacyUrl(PRIVACY_LINK_STATEMENT);
}

function openHttpsUrl(context: common.UIAbilityContext, url: string): boolean {
  if (url.length === 0) {
    return false;
  }
  context.startAbility({
    action: 'ohos.want.action.viewData',
    entities: ['entity.system.browsable'],
    uri: url
  }).catch((error: BusinessError) => {
    console.error(`${TAG}: open privacy url ${error.code} ${error.message}`);
  });
  return true;
}

/** 用系统浏览器打开指定类型的政策链接；无链接时返回 false。 */
export function openHostedPrivacyLink(context: common.UIAbilityContext, linkType: number): boolean {
  let url: string = readHostedPrivacyUrl(linkType);
  if (url.length === 0) {
    console.info(`${TAG}: no privacy URL for type ${linkType}`);
    return false;
  }
  return openHttpsUrl(context, url);
}

/** 用系统浏览器打开应用的远程隐私声明。 */
export function openHostedPrivacyStatement(context: common.UIAbilityContext): boolean {
  return openHttpsUrl(context, PRIVACY_STATEMENT_URL);
}

/**
 * 解析 Help 菜单下发的 artforge-privacy:// 伪协议并打开对应网站页面。
 * 未知 scheme 返回 false。
 */
export function openHostedPrivacyScheme(context: common.UIAbilityContext, schemeOrUrl: string): boolean {
  if (schemeOrUrl === PRIVACY_SCHEME_STATEMENT || schemeOrUrl.indexOf('artforge-privacy://statement') === 0) {
    return openHostedPrivacyLink(context, PRIVACY_LINK_STATEMENT);
  }
  if (schemeOrUrl === PRIVACY_SCHEME_AGREEMENT || schemeOrUrl.indexOf('artforge-privacy://agreement') === 0) {
    return openHostedPrivacyLink(context, PRIVACY_LINK_AGREEMENT);
  }
  return false;
}

/** 是否为 Help / 原生桥使用的隐私伪协议。 */
export function isHostedPrivacyScheme(url: string): boolean {
  return url.indexOf('artforge-privacy://') === 0;
}
