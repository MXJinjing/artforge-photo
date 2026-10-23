# 艺工坊·修图 · 图片宣传位素材

五张 1920 × 1280 横版宣传图，每张提供 PNG 和高品质 JPG。最新版文件以 `text-right50-` 开头：平板完整显示在画面左侧，宽度从 1100 放大为 1200 像素；文字置于右侧，主标题为 114–120 像素，说明文字为 35 像素，全部宣传文案在放大版基础上向右移动 50 像素（在上一版基础上再移 20 像素），平板位置保持不变。

1. 01-home：应用首页，建议首图。
2. 02-layers：PSD 与图层。
3. 03-color：色彩平衡与调整图层。
4. 04-curves：曲线与光影。
5. 05-brush：画笔与创作。

`artforge-store-screenshots-text-right50-1920x1280.zip` 包含最新十张导出图片。`artforge-store-screenshots-tablet-right.zip` 保留之前的平板在右版本。`index.html` 可离线预览五张图，底部按钮切换；导出图片不含预览按钮。图片来自用户提供的实机截图，未改动画布内容。

沿用原视频截图展示段的深色背景、薄荷绿强调色及悬浮平板风格；未添加四角固定文案或图标。尺寸沿用本次视频的 1920 × 1280，尚未针对具体应用市场的图片上传规格额外适配。

`render.cjs` 通过本机 Chromium 与 Playwright 导出静态 HTML。`build.py` 从保留的原版 HTML 生成版式与嵌图源文件，运行后会覆盖 `index.html` 与 `posters.template.html`；后续可直接编辑独立的 `index.html` 并运行 `node render.cjs` 重新导出。
