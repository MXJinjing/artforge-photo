import { pasteboard } from '@kit.BasicServicesKit';
import { abilityAccessCtrl, common } from '@kit.AbilityKit';
import { image } from '@kit.ImageKit';
import { colorSpaceManager } from '@kit.ArkGraphics2D';
import { fileIo } from '@kit.CoreFileKit';

const MAX_IMAGE_BYTES: number = 120 * 1024 * 1024;
const MAX_TEXT_LENGTH: number = 1024 * 1024;

export class ClipboardReply {
  text: string = '';
  bytes: ArrayBuffer = new ArrayBuffer(0);

  constructor(text: string = '', bytes: ArrayBuffer = new ArrayBuffer(0)) {
    this.text = text;
    this.bytes = bytes;
  }
}

export function clipboardByteLength(width: number, height: number): number {
  const length = width * height * 4;
  if (!Number.isSafeInteger(width) || !Number.isSafeInteger(height) || width <= 0 || height <= 0 ||
    !Number.isSafeInteger(length) || length > MAX_IMAGE_BYTES) {
    throw new Error('剪贴板图片为空或超过 120 MiB');
  }
  return length;
}

// PhotoCraft expects straight alpha RGBA. HarmonyOS may deliver premultiplied BGRA pixels.
export function straightAlpha(bytes: ArrayBuffer, alpha: image.AlphaType): void {
  const pixels = new Uint8Array(bytes);
  for (let i = 0; i < pixels.length; i += 4) {
    if (alpha === image.AlphaType.OPAQUE) {
      pixels[i + 3] = 255;
    } else if (alpha === image.AlphaType.PREMUL) {
      const a = pixels[i + 3];
      for (let channel = 0; channel < 3; channel++) {
        pixels[i + channel] = a === 0 ? 0 : Math.min(255, Math.round(pixels[i + channel] * 255 / a));
      }
    }
  }
}

export class NativeClipboardAdapter {
  private context: common.UIAbilityContext;
  // HarmonyOS rejects overlapping copy/paste operations. Reads share this queue so a
  // quick Ctrl+C/Ctrl+V sequence keeps user intent order.
  private pending: Promise<void> = Promise.resolve();

  constructor(context: common.UIAbilityContext) {
    this.context = context;
  }

  handle(kind: string, text: string, bytes: ArrayBuffer,
    active: () => boolean): Promise<ClipboardReply> {
    const result = this.pending.then((): Promise<ClipboardReply> =>
      this.perform(kind, text, bytes, active));
    this.pending = result.then((): void => {}, (): void => {});
    return result;
  }

  private async authorize(active: () => boolean): Promise<void> {
    const manager = abilityAccessCtrl.createAtManager();
    const result = await manager.requestPermissionsFromUser(this.context, ['ohos.permission.READ_PASTEBOARD']);
    if (!active()) {
      throw new Error('cancelled');
    }
    if (result.authResults.length !== 1 || result.authResults[0] !== 0) {
      throw new Error('请允许艺工坊·修图读取剪贴板后再粘贴');
    }
  }

  private async readBgra(pixelMap: image.PixelMap): Promise<ClipboardReply> {
    const original = await pixelMap.getImageInfo();
    clipboardByteLength(original.size.width, original.size.height);
    // Color-space conversion is best effort. Some clipboard PixelMaps are backed by
    // YUV or other source formats that reject conversion. readPixels() still emits
    // BGRA_8888 for every supported PixelMap format.
    try {
      await pixelMap.applyColorSpace(colorSpaceManager.create(colorSpaceManager.ColorSpace.SRGB));
    } catch (error) {
      console.info(`Clipboard color conversion unavailable: ${(error as Error).message}`);
    }
    const info = await pixelMap.getImageInfo();
    const length = clipboardByteLength(info.size.width, info.size.height);
    const bytes = new ArrayBuffer(length);
    // Supplying stride explicitly removes any row padding from the native PixelMap.
    await pixelMap.readPixels({
      pixels: bytes,
      offset: 0,
      stride: info.size.width * 4,
      region: { x: 0, y: 0, size: info.size }
    });
    const rgba = new Uint8Array(bytes);
    for (let i = 0; i < rgba.length; i += 4) {
      const blue = rgba[i];
      rgba[i] = rgba[i + 2];
      rgba[i + 2] = blue;
    }
    straightAlpha(bytes, info.alphaType);
    return new ClipboardReply(JSON.stringify([info.size.width, info.size.height]), bytes);
  }

  private async pixels(pixelMap: image.PixelMap): Promise<ClipboardReply> {
    try {
      return await this.readBgra(pixelMap);
    } catch (directError) {
      // Some clipboard providers expose PixelMaps backed by a source format that
      // rejects direct extraction. Re-encode through PNG and decode into a fresh
      // RGBA PixelMap before trying again.
      console.info(`Clipboard direct pixel read unavailable: ${(directError as Error).message}`);
      const info = await pixelMap.getImageInfo();
      const pixelBytes = clipboardByteLength(info.size.width, info.size.height);
      const packer = image.createImagePacker();
      try {
        const png = await packer.packToData(pixelMap, {
          format: 'image/png',
          quality: 100,
          bufferSize: pixelBytes + 1024 * 1024
        });
        const source = image.createImageSource(png);
        try {
          const normalized = await source.createPixelMap({
            desiredPixelFormat: image.PixelMapFormat.RGBA_8888,
            desiredColorSpace: colorSpaceManager.create(colorSpaceManager.ColorSpace.SRGB)
          });
          try {
            return await this.readBgra(normalized);
          } finally {
            await normalized.release();
          }
        } finally {
          await source.release();
        }
      } catch (fallbackError) {
        throw new Error(`无法读取剪贴板图片：${(fallbackError as Error).message}`);
      } finally {
        await packer.release();
      }
    }
  }

  private async readImage(data: pasteboard.PasteData): Promise<ClipboardReply> {
    const count = Math.min(data.getRecordCount(), 16);
    for (let index = 0; index < count; index++) {
      let record: pasteboard.PasteDataRecord | undefined;
      try {
        record = data.getRecord(index);
      } catch (error) {
        console.info(`Clipboard record unavailable: ${(error as Error).message}`);
        continue;
      }
      const type = record.mimeType;
      if (type === pasteboard.MIMETYPE_PIXELMAP) {
        const pixelMap = record.pixelMap;
        if (pixelMap !== undefined && pixelMap !== null) {
          try {
            return await this.pixels(pixelMap);
          } catch (error) {
            console.info(`Clipboard PixelMap unavailable: ${(error as Error).message}`);
          } finally {
            await pixelMap.release();
          }
        }
      }
      if (record.uri.length > 0 && record.uri.startsWith('file://')) {
        try {
          return await this.readUri(record.uri);
        } catch (error) {
          console.info(`Clipboard image URI unavailable: ${(error as Error).message}`);
        }
      }
      if (type !== pasteboard.MIMETYPE_PIXELMAP && type !== pasteboard.MIMETYPE_TEXT_URI &&
        type.startsWith('image/')) {
        try {
          const value = (await record.getData(type)) as pasteboard.ValueType;
          if (value instanceof ArrayBuffer) {
            return await this.readBytes(value);
          }
          if (typeof value === 'string') {
            if (value.startsWith('file://')) {
              return await this.readUri(value);
            }
            continue;
          }
          const pixelMap = value as image.PixelMap;
          try {
            return await this.pixels(pixelMap);
          } finally {
            await pixelMap.release();
          }
        } catch (error) {
          console.info(`Clipboard image record unavailable: ${(error as Error).message}`);
        }
      }
    }
    // A text-only or unrelated clipboard is a normal no-image result for New and Paste.
    return new ClipboardReply();
  }

  private async readUri(uri: string): Promise<ClipboardReply> {
    const file = await fileIo.open(uri, fileIo.OpenMode.READ_ONLY);
    try {
      const source = image.createImageSource(file.fd);
      try {
        const info = await source.getImageInfo();
        clipboardByteLength(info.size.width, info.size.height);
        const pixelMap = await source.createPixelMap({
          desiredPixelFormat: image.PixelMapFormat.RGBA_8888,
          desiredColorSpace: colorSpaceManager.create(colorSpaceManager.ColorSpace.SRGB)
        });
        try {
          return await this.pixels(pixelMap);
        } finally {
          await pixelMap.release();
        }
      } finally {
        await source.release();
      }
    } finally {
      await fileIo.close(file);
    }
  }

  private async readBytes(bytes: ArrayBuffer): Promise<ClipboardReply> {
    const source = image.createImageSource(bytes);
    try {
      const info = await source.getImageInfo();
      clipboardByteLength(info.size.width, info.size.height);
      const pixelMap = await source.createPixelMap({
        desiredPixelFormat: image.PixelMapFormat.RGBA_8888,
        desiredColorSpace: colorSpaceManager.create(colorSpaceManager.ColorSpace.SRGB)
      });
      try {
        return await this.pixels(pixelMap);
      } finally {
        await pixelMap.release();
      }
    } finally {
      await source.release();
    }
  }

  private async perform(kind: string, text: string, bytes: ArrayBuffer,
    active: () => boolean): Promise<ClipboardReply> {
    if (!active()) {
      throw new Error('cancelled');
    }
    const board = pasteboard.getSystemPasteboard();
    if (kind === 'clipboard_set_image') {
      const size = JSON.parse(text) as number[];
      if (size.length !== 2 || bytes.byteLength !== clipboardByteLength(size[0], size[1])) {
        throw new Error('剪贴板 RGBA 数据长度不正确');
      }
      const pixelMap = await image.createPixelMap(bytes, {
        size: { width: size[0], height: size[1] },
        srcPixelFormat: image.PixelMapFormat.RGBA_8888,
        pixelFormat: image.PixelMapFormat.RGBA_8888,
        alphaType: image.AlphaType.UNPREMUL
      });
      try {
        if (!active()) {
          throw new Error('cancelled');
        }
        await board.setData(pasteboard.createData(pasteboard.MIMETYPE_PIXELMAP, pixelMap));
      } finally {
        await pixelMap.release();
      }
      return new ClipboardReply();
    }
    if (kind === 'clipboard_set_text') {
      if (text.length > MAX_TEXT_LENGTH) {
        throw new Error('剪贴板文本过长');
      }
      await board.setData(pasteboard.createData(pasteboard.MIMETYPE_TEXT_PLAIN, text));
      return new ClipboardReply();
    }
    if (kind !== 'clipboard_get_image' && kind !== 'clipboard_get_text') {
      throw new Error('Unknown clipboard operation');
    }
    await this.authorize(active);
    const data = await board.getData();
    data.pasteStart();
    try {
      if (!active()) {
        throw new Error('cancelled');
      }
      const result = kind === 'clipboard_get_image'
        ? await this.readImage(data)
        : new ClipboardReply(data.getPrimaryText() ?? '');
      if (result.text.length > MAX_TEXT_LENGTH) {
        throw new Error('剪贴板文本过长');
      }
      if (!active()) {
        throw new Error('cancelled');
      }
      return result;
    } finally {
      data.pasteComplete();
    }
  }
}
