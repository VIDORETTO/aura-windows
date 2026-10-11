const fs=require('node:fs/promises');
const path=require('node:path');
const {pathToFileURL}=require('node:url');
const sharp=require('sharp');
const {chromium}=require('playwright');
const OUT = path.resolve(__dirname, '..');
(async()=>{
 const index=JSON.parse(await fs.readFile(path.join(OUT,'fontes/indice.json'),'utf8'));
 const originals=(await fs.readdir(path.join(OUT,'03-capturas-originais'))).filter(f=>f.endsWith('.png'));
 const failures=[];let pngBytes=0,webpBytes=0;
 await fs.mkdir(path.join(OUT,'01-portfolio/webp'),{recursive:true});
 for(let i=0;i<index.length;i+=4){
  await Promise.all(index.slice(i,i+4).map(async f=>{
   const file=path.join(OUT,f.file);const meta=await sharp(file).metadata();
   const expected=f.kind==='instagram'?[2160,2700]:[2560,1600];
   if(meta.width!==expected[0]||meta.height!==expected[1])failures.push({file:f.file,dimensions:[meta.width,meta.height],expected});
   const stats=await sharp(file).stats();if(stats.channels.slice(0,3).every(c=>c.stdev<1))failures.push({file:f.file,error:'Imagem sem conteúdo'});
   if(f.kind==='portfolio'){
    const webp=path.join(OUT,'01-portfolio/webp',f.name+'.webp');await sharp(file).webp({quality:92,effort:5}).toFile(webp);
    pngBytes+=(await fs.stat(file)).size;webpBytes+=(await fs.stat(webp)).size;
   }
  }));
 }
 const browser=await chromium.launch({headless:true});const page=await browser.newPage();const layout=[];
 for(let i=0;i<index.length;i++){
  const f=index[i];await page.setViewportSize(f.kind==='instagram'?{width:1080,height:1350}:{width:1280,height:800});
  await page.goto(pathToFileURL(path.join(OUT,'fontes/composicoes.html')).href+'?frame=f'+i);
  const result=await page.locator('.frame:visible').evaluate(e=>{
   const r=e.getBoundingClientRect();const p=e.querySelector('.portrait-visual');const ft=e.querySelector('footer');const intro=e.querySelector('.intro');
   return {origin:[r.x,r.y],size:[r.width,r.height],visualBottom:p?.getBoundingClientRect().bottom??null,footerTop:ft.getBoundingClientRect().top,introBottom:intro?.getBoundingClientRect().bottom??null,missingImages:[...e.querySelectorAll('img')].filter(im=>!im.complete||!im.naturalWidth).length};
  });
  if(result.origin.some(v=>v!==0)||result.missingImages||(result.visualBottom!==null&&result.visualBottom>result.footerTop)||(result.introBottom!==null&&result.introBottom>435))failures.push({file:f.file,layout:result});
  layout.push({file:f.file,...result});
 }
 await browser.close();
 for(const file of originals){const b=await fs.readFile(path.join(OUT,'03-capturas-originais',file));if(b.readUInt32BE(16)<100||b.readUInt32BE(20)<30)failures.push({file,error:'Captura inválida'});}
 await fs.writeFile(path.join(OUT,'fontes/verificacao.json'),JSON.stringify({compositionCount:index.length,originalCount:originals.length,portfolioCount:36,instagramCount:20,failures,layout,compression:{pngBytes,webpBytes}},null,2));
 let readme=await fs.readFile(path.join(OUT,'LEIA-ME.txt'),'utf8');
 readme=readme.replace('imagens PNG horizontais, 2560 x 1600 px.','imagens PNG horizontais, 2560 x 1600 px. A subpasta webp tem versões leves para publicar no site.');
 await fs.writeFile(path.join(OUT,'LEIA-ME.txt'),readme);
 console.log(JSON.stringify({composicoes:index.length,portfolio:36,instagram:20,originais:originals.length,verificacao:failures.length===0?'OK':failures,webpReducao:Math.round((1-webpBytes/pngBytes)*100)+'%'}));
 if(failures.length)process.exitCode=1;
})().catch(e=>{console.error(e);process.exit(1)});
