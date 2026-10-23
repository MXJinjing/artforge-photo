"""Build editable still-poster HTML from the preserved screenshot layout."""
from pathlib import Path
import re
ROOT=Path(__file__).resolve().parent
SOURCE=ROOT.parent/'app-intro'

def transform(html):
 css=re.search(r'<style>(.*?)</style>',html,re.S).group(1)
 sections=re.findall(r'<section class="scene.*?</section>',html,re.S)
 # Home, layers, color balance, curves, brushes.
 selected=[sections[i] for i in [5,1,0,2,3]]
 selected[0]=selected[0].replace('<div class="cta">即刻开启创作 <span>↗</span></div>','<div class="chips"><span class="chip">HarmonyOS 原生适配</span><span class="chip">平板创作</span></div>')
 selected[0]=selected[0].replace('让创作，<br><span class="accent">在此展开。</span>','让创作，<br><span class="accent">在此展开。</span>')
 selected[1]=selected[1].replace('<div class="feature-num">01</div>','').replace('每一层，<br>都有<span class="accent">想象空间。</span>','每一层，<br>皆有<span class="accent">想象。</span>')
 selected[2]=selected[2].replace('在平板上，认真创作','色彩 · 调整').replace('把灵感<br>修成<span class="accent">作品。</span>','你的风景，<br>你的<span class="accent">色彩。</span>').replace('熟悉的修图工作流<br>现在，随手展开。','用色彩平衡与调整图层<br>细细打磨照片的氛围。').replace('<span class="chip">图层编辑</span><span class="chip">色彩调整</span><span class="chip">自由绘画</span>','<span class="chip">色彩平衡</span><span class="chip">调整图层</span>')
 selected[3]=selected[3].replace('<div class="feature-num">02</div>','').replace('色彩 · 光影','曲线 · 光影').replace('好风景，<br>值得<span class="accent">细细调整。</span>','调好光影，<br>留住<span class="accent">风景。</span>')
 selected[4]=selected[4].replace('<div class="feature-num">03</div>','')
 # Give every image a small product label within the content, without corner overlays.
 for i,s in enumerate(selected):
  s=re.sub(r'<div class="note".*?</div>','',s,flags=re.S)
  s=s.replace('</section>','<div class="note">艺工坊·修图 · 真实应用画面</div></section>')
  s=re.sub(r' data-start="[^"]+" data-end="[^"]+"','',s)
  selected[i]=s
 override='''html,body{background:#080d18}.scene{inset:0;height:1280px;opacity:1;visibility:visible;display:none}.scene.active{display:block}.copy,.end .copy{left:1310px;top:270px;width:630px}.device,.device.large,.end .device{left:24px;top:220px;width:1200px;transform:rotate(-.55deg)}.note{left:1310px;bottom:130px;font-size:22px;color:#9aaac0}.halo{left:-280px;right:auto;top:30px;width:1250px;height:1250px}#stage{background:radial-gradient(ellipse at 24% 44%,#183049 0%,#101a2b 36%,#080d18 75%)}.eyebrow{font-size:30px;margin-bottom:32px}.sub{font-size:35px;line-height:1.6;margin-top:34px}.chips{gap:12px;flex-wrap:wrap;margin-top:36px}.chip{font-size:25px;padding:14px 19px}.brand,.edition,.bottom,.progress,#controls{display:none!important}h1{font-size:120px;line-height:1.22}h2{font-size:114px;line-height:1.24}.gallery-controls{position:fixed;bottom:18px;left:50%;transform:translateX(-50%);display:flex;gap:9px;z-index:100;background:#080d18cc;padding:10px;border-radius:15px}.gallery-controls button{padding:8px 17px}body.export .gallery-controls{display:none}'''
 script='''const stage=document.querySelector('#stage');const posters=[...document.querySelectorAll('.scene')];function resize(){const s=Math.min(innerWidth/1920,innerHeight/1280);stage.style.transform=`scale(${s})`;stage.style.left=(innerWidth-1920*s)/2+'px';stage.style.top=(innerHeight-1280*s)/2+'px'}window.showPoster=i=>{posters.forEach((p,n)=>p.classList.toggle('active',i===n));return i};addEventListener('resize',resize);resize();const q=new URLSearchParams(location.search);if(q.has('export'))document.body.classList.add('export');showPoster(Math.max(0,Math.min(4,Number(q.get('slide')||0))));'''
 buttons=''.join(f'<button onclick="showPoster({i})">{i+1} · {title}</button>' for i,title in enumerate(['首页','图层','色彩','曲线','画笔']))
 return '<!doctype html><html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>艺工坊·修图｜宣传截图</title><style>'+css+override+'</style></head><body><main id="stage"><div class="grid"></div><div class="halo"></div>'+''.join(selected)+'</main><nav class="gallery-controls">'+buttons+'</nav><script>'+script+'</script></body></html>'

(ROOT/'posters.template.html').write_text(transform((SOURCE/'intro-v1.template.html').read_text()))
(ROOT/'index.html').write_text(transform((SOURCE/'index-v1.html').read_text()))
print('Built five static posters and self-contained HTML.')
