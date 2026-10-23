# HarmonyOS native clipboard

This branch connects PhotoCraft's existing clipboard commands to the HarmonyOS
system pasteboard. It is a release-audience integration because direct
Ctrl+V reads require `ohos.permission.READ_PASTEBOARD`.

## Supported behavior

| User action | Upstream command | HarmonyOS behavior |
| --- | --- | --- |
| Ctrl+C / Edit > Copy | `edit.copy` | Writes the selected layer pixels, selection alpha, or whole active layer as RGBA |
| Ctrl+X / Edit > Cut | `edit.cut` | Cuts through the same clipboard path |
| Ctrl+Shift+C / Edit > Copy Merged | `edit.copyMerged` | Writes the visible composite inside the selection |
| Ctrl+V / Edit > Paste | `edit.paste` | Reads a PixelMap or local image URI and adds it as a layer |
| Ctrl+Shift+V / Paste in Place | `edit.pasteSpecial.pasteInPlace` | Preserves position for the app's structured clipboard; external images are centered |
| File > New from Clipboard | `file.newFromClipboard` | Creates a document from clipboard dimensions |
| Paste Into / Paste Outside | `edit.pasteSpecial.*` | Continues to use the engine's structured session clipboard |
| Text copy/paste | egui text fields and type tool | Uses system plain text through egui `CopyText` and `RequestPaste` |
| File/image drag import | existing drop adapter | Already imports files on the canvas and over document tabs |

The engine keeps layers, depth, masks, channels and original position in its
session clipboard. The system clipboard carries an RGBA8 image. Upstream's image
signature comparison preserves structured in-app paste behavior when the same
image is still on the system clipboard.

## Native boundary

`native/rust/src/clipboard.rs` validates exact RGBA lengths and limits image
memory to 120 MiB. It sends requests through the existing worker-to-ArkTS bridge.

`entry/src/main/ets/input/NativeClipboardAdapter.ets`:

- serializes copy and paste operations because HarmonyOS rejects overlapping
  pasteboard operations;
- requests `READ_PASTEBOARD` only when the user invokes paste or New from
  Clipboard;
- converts premultiplied BGRA PixelMaps to straight-alpha RGBA;
- treats color-space conversion as best effort and falls back through an
  in-memory PNG when a clipboard provider rejects direct pixel extraction;
- probes clipboard records instead of assuming the first/primary record is an
  image, and accepts PixelMap, generic `image/*` bytes, or local file URIs;
- accepts local `file://` image URIs without fetching network content;
- releases PixelMaps, ImageSources and file descriptors on every path;
- cancels stale replies after the editor page closes or restarts.

No startup or idle code reads the clipboard, and clipboard content is not
logged.

## Permission and signing

The manifest declares:

```json5
{
  "name": "ohos.permission.READ_PASTEBOARD",
  "reason": "$string:permission_read_pasteboard_reason",
  "usedScene": {
    "abilities": ["EntryAbility"],
    "when": "inuse"
  }
}
```

`READ_PASTEBOARD` is a restricted user-grant permission. A normal-APL app needs
both an ACL in its signed Profile and runtime consent. Installing a HAP whose
Profile lacks the ACL fails with error `9568289`.

AppGallery also compares the package's `user_grant` permissions with the
declaration in the hosted privacy statement, so the statement must list
`READ_PASTEBOARD` (see `docs/huawei-privacy-hosting.md`). Otherwise the release
is rejected with `只在软件包中的权限：[READ_PASTEBOARD]`.

Run:

```bash
python3 scripts/check_clipboard_signing.py
```

before installing a signed HAP. `scripts/dev.sh run` performs the same check.
The check validates Profile bundle identity and the clipboard ACL without
printing signing material. It does not validate certificate trust, expiry or
device eligibility.

Unsigned GitHub Release HAPs can still be built and downloaded. They cannot be
installed as-is on a normal HarmonyOS device; users who sign them must use a
Profile that includes the approved clipboard ACL.

## Verification

```bash
cargo test --offline --manifest-path native/rust/Cargo.toml --lib clipboard
node scripts/test_native_clipboard.cjs
python3 scripts/test_clipboard_signing.py
scripts/dev.sh build
```

The native tests cover an actual Ctrl+C selection export and Ctrl+V external
image layer paste through the upstream editor command path. The ArkTS test
covers pixel channel/alpha conversion, file URI cleanup, denied permission,
operation ordering and page lifecycle cancellation.

The current automated suite cannot validate the system permission dialog or a
cross-application clipboard round trip. Those require an installed signed HAP
on a simulator or device with the approved ACL.
