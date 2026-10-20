import { common } from '@kit.AbilityKit';
import { BusinessError, commonEventManager } from '@kit.BasicServicesKit';
import { privacyManager } from '@kit.AppGalleryKit';

/**
 * 华为 AppGallery「隐私声明托管」对接。
 *
 * 使用托管后，系统会在首次启动自动弹出标准化同意窗；应用内不得再自绘首启隐私弹窗，
 * 否则可能因重复弹窗被拒审。本模块只做：查询签署状态、监听签署结果、延后业务初始化、
 * 以及查询托管隐私声明链接（设置/关于入口用）。
 *
 * 本地未配置托管 / API 不可用时（侧载调试、模拟器报 1006700003），默认放行以便开发。
 * 正式上架包以 AGC 托管配置为准。
 *
 * result: 0 不同意；1 同意完整模式；2 协议变更需重签。
 */
export const PRIVACY_ALLOWED_KEY: string = 'artforgePrivacyAllowed';

const PRIVACY_EVENT: string = 'usual.event.PRIVACY_STATE_CHANGED';
const TAG: string = 'PrivacyConsent';

/** 0 不同意；1 已同意；2 需重签。 */
export type PrivacyResultCode = 0 | 1 | 2;

export type PrivacyGateResult = 'agreed' | 'pending' | 'unavailable';

export function queryPrivacyAgreed(): PrivacyGateResult {
  try {
    let results: privacyManager.AppPrivacyResult[] = privacyManager.getAppPrivacyResult();
    if (!results || results.length === 0) {
      console.info(`${TAG}: no privacy results; treat as unavailable (dev fallback)`);
      return 'unavailable';
    }
    for (let i = 0; i < results.length; i++) {
      let code: number = results[i].result as number;
      if (code === 1) {
        return 'agreed';
      }
      if (code === 2) {
        return 'pending';
      }
    }
    // 全部为 0 或其他：视为待同意
    return 'pending';
  } catch (error) {
    let failure = error as BusinessError;
    console.info(`${TAG}: getAppPrivacyResult unavailable: ${failure.code} ${failure.message}`);
    return 'unavailable';
  }
}

/** AppPrivacyLink.type：1 隐私声明，2 用户协议（以官方枚举为准）。 */
export const PRIVACY_LINK_STATEMENT: number = 1;
export const PRIVACY_LINK_AGREEMENT: number = 2;

/** Help / 桥接用的伪协议，由 openHostedPrivacyScheme 解析为托管真实 URL。 */
export const PRIVACY_SCHEME_STATEMENT: string = 'artforge-privacy://statement';
export const PRIVACY_SCHEME_AGREEMENT: string = 'artforge-privacy://agreement';

/**
 * 查询托管下发的指定类型 URL（不要写死自建链接作为正式声明）。
 * 找不到对应 type 时：隐私声明可退回第一条非空；用户协议则返回空。
 */
export function readHostedPrivacyUrl(linkType: number): string {
  try {
    let info: privacyManager.AppPrivacyMgmtInfo = privacyManager.getAppPrivacyMgmtInfo();
    let links: privacyManager.AppPrivacyLink[] = info.privacyInfo;
    if (!links) {
      return '';
    }
    let fallback: string = '';
    for (let i = 0; i < links.length; i++) {
      let url: string = links[i].url ?? '';
      if (url.length === 0) {
        continue;
      }
      let currentType: number = (links[i] as privacyManager.AppPrivacyLink).type as number;
      if (currentType === linkType) {
        return url;
      }
      if (fallback.length === 0) {
        fallback = url;
      }
    }
    // 仅隐私声明允许退回第一条非空；用户协议必须精确匹配，避免误开隐私页。
    if (linkType === PRIVACY_LINK_STATEMENT) {
      return fallback;
    }
    return '';
  } catch (error) {
    let failure = error as BusinessError;
    console.info(`${TAG}: getAppPrivacyMgmtInfo unavailable: ${failure.code} ${failure.message}`);
  }
  return '';
}

/** 查询托管隐私声明 URL（type=1，找不到则退回第一条非空）。 */
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

/** 用系统浏览器打开指定类型的托管链接；无链接时返回 false。 */
export function openHostedPrivacyLink(context: common.UIAbilityContext, linkType: number): boolean {
  let url: string = readHostedPrivacyUrl(linkType);
  if (url.length === 0) {
    console.info(`${TAG}: no hosted privacy URL for type ${linkType}`);
    return false;
  }
  return openHttpsUrl(context, url);
}

/** 用系统浏览器打开托管隐私声明；无链接时返回 false。 */
export function openHostedPrivacyStatement(context: common.UIAbilityContext): boolean {
  return openHostedPrivacyLink(context, PRIVACY_LINK_STATEMENT);
}

/**
 * 解析 Help 菜单下发的 artforge-privacy:// 伪协议并打开对应托管页。
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

/**
 * 等待用户通过系统托管弹窗同意隐私政策。
 * - agreed / unavailable → 立即回调 true（unavailable 供本地开发 / 模拟器）
 * - pending → 订阅公共事件，同意后回调 true；明确拒绝则 terminateSelf 并回调 false
 */
export function awaitPrivacyConsent(
  context: common.UIAbilityContext,
  onReady: (allowed: boolean) => void
): void {
  let state: PrivacyGateResult = queryPrivacyAgreed();
  if (state === 'agreed' || state === 'unavailable') {
    onReady(true);
    return;
  }

  let finished: boolean = false;
  let subscriber: commonEventManager.CommonEventSubscriber | undefined = undefined;

  const finish = (allowed: boolean): void => {
    if (finished) {
      return;
    }
    finished = true;
    try {
      if (subscriber) {
        commonEventManager.unsubscribe(subscriber);
      }
    } catch (error) {
      console.info(`${TAG}: unsubscribe ignored`);
    }
    onReady(allowed);
  };

  let subscribeInfo: commonEventManager.CommonEventSubscribeInfo = {
    events: [PRIVACY_EVENT]
  };

  try {
    commonEventManager.createSubscriber(subscribeInfo)
      .then((sub: commonEventManager.CommonEventSubscriber) => {
        subscriber = sub;
        return commonEventManager.subscribe(sub, (err: BusinessError, data: commonEventManager.CommonEventData) => {
          if (err) {
            console.error(`${TAG}: subscribe callback err ${err.code} ${err.message}`);
            return;
          }
          let resultType: number = 0;
          try {
            let payload = JSON.parse(data?.data ?? '{}') as Record<string, Object>;
            resultType = Number(payload['resultType'] ?? 0);
          } catch (parseError) {
            console.info(`${TAG}: privacy event parse failed`);
          }
          if (resultType === 1) {
            finish(true);
          } else {
            // 不同意：退出应用（与应用市场托管弹窗行为一致）
            finish(false);
            context.terminateSelf().catch((termErr: BusinessError) => {
              console.error(`${TAG}: terminateSelf ${termErr.code} ${termErr.message}`);
            });
          }
        });
      })
      .catch((error: BusinessError) => {
        console.error(`${TAG}: createSubscriber failed ${error.code} ${error.message}`);
        let again: PrivacyGateResult = queryPrivacyAgreed();
        finish(again !== 'pending');
      });
  } catch (error) {
    let failure = error as BusinessError;
    console.error(`${TAG}: awaitPrivacyConsent exception ${failure.code} ${failure.message}`);
    finish(true);
  }

  // 订阅建立后再次确认，避免事件已在订阅前发出
  let again: PrivacyGateResult = queryPrivacyAgreed();
  if (again === 'agreed' || again === 'unavailable') {
    finish(true);
  }
}
