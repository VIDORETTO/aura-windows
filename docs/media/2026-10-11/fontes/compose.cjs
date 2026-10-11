const fs=require('node:fs/promises');
const path=require('node:path');
const {pathToFileURL}=require('node:url');
const {chromium}=require('playwright');
const OUT = path.resolve(__dirname, '..');
const frames=[];
const escape=s=>String(s).replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;').replaceAll('"','&quot;');
const slug=s=>s.normalize('NFD').replace(/[\u0300-\u036f]/g,'').toLowerCase().replace(/[^a-z0-9]+/g,'-').replace(/^-|-$/g,'');
const asset=file=>'../03-capturas-originais/'+path.basename(file);
const logo='../fontes/logo.svg';
function img(file,cls='',style=''){return `<img class="${cls}" src="${escape(asset(file))}" style="${style}">`;}
function shot(file,width,height,cls=''){return `<div class="screen ${cls}" style="aspect-ratio:${width}/${height}">${img(file)}</div>`;}
function crop(file,sw,sh,y=0,dw=940,caption=''){
  // Settings sections are 720 CSS px wide after the container's padding.
  if(sw===768) sw=720;
  return `<div class="focus" style="width:${dw}px">${caption?`<div class="focus-label">${escape(caption)}</div>`:''}<div class="screen crop" style="height:${Math.round(sh*dw/sw)}px">${img(file,'',`width:${dw}px;top:${-y*dw/sw}px`)}</div></div>`;
}
const points={
  conversa:['Respostas em Markdown','Copiar, ouvir e salvar','Inserir no app'],
  contexto:['Tela e região','Seleção e arquivos','Recortes recentes'],
  tarefas:['Chat · Tarefa · Plano','Workspace da conversa','Aprovações visíveis'],
  arquivos:['Arquivos gerados','Prévia no painel','Alterações revisáveis'],
  historico:['Busca nas conversas','Fixar o que importa','Retomar o contexto'],
  preparo:['Contexto antes de começar','Receitas de reunião','Projetos e compromissos'],
  reuniao:['Transcrição','Notas durante a conversa','Ajuda sob demanda'],
  general:['Tema e aparência','Instruções pessoais','Memórias revisáveis'],
  account:['Conta ChatGPT','Catálogo de modelos','Esforço por modo'],
  providers:['Sua credencial','Provedores BYOK','Modelos locais'],
  privacy:['Controle por fonte','Janelas excluídas','Retenção e registros'],
  personal:['Lembretes','Notas','Textos salvos'],
  voice:['Modelos ASR locais','Vocabulário pessoal','Leitura em voz alta'],
  extensions:['Skills','Servidores MCP','Comandos rápidos'],
  profiles:['Por aplicativo','Instruções específicas','Modo e modelo padrão'],
  shortcuts:['Abrir o Aura','Segurar para falar','Ditado global'],
  diagnostics:['Estado dos componentes','Diagnóstico redigido','Dados locais'],
  about:['Aura 0.3.0','Código aberto','Atualizações'],
};
function mast(chapter,theme){return `<header><div class="brand"><img src="${logo}"><b>Aura</b></div><span>ASSISTENTE DE IA PARA WINDOWS</span></header><div class="orbit ${theme}"><i></i><i></i><i></i></div>`;}
function footer(number,total=20){return `<footer><span>AURA <b>·</b> WINDOWS <b>·</b> 0.3.0</span><span class="demo">Interface real · conteúdo de demonstração</span><strong>${String(number).padStart(2,'0')}${total?' / '+String(total).padStart(2,'0'):''}</strong></footer>`;}
function pills(id){return `<div class="pills">${(points[id]||[]).map(x=>`<span>${escape(x)}</span>`).join('')}</div>`;}
function add(name,kind,html){frames.push({name,kind,html});}
function overview(item,n){
  const dark=item.kind==='overlay';
  const special=item.height<150;
  add(`${String(n).padStart(2,'0')}-${item.id}`,'portfolio',`<article class="frame landscape ${dark?'dark':'light'}">${mast(item.label)}<div class="left"><div class="kicker">${escape(item.label)}</div><h1>${escape(item.headline)}</h1><p>${escape(item.description)}</p><div class="vertical-points">${(points[item.id]||[]).map(x=>`<div>${escape(x)}</div>`).join('')}</div></div><div class="visual ${special?'compact-visual':''}">${shot(item.file,item.width,item.height)}${special?'<div class="shortcut">Ctrl <em>+</em> Shift <em>+</em> Space</div>':''}</div>${footer(n,null)}</article>`);
}
function cover(m,kind){
  const chat=m.images.find(x=>x.id==='conversa'); const compact=m.images.find(x=>x.id==='overlay-compacto');
  const car=kind==='instagram';
  add('01-capa',kind,`<article class="frame ${car?'portrait':'landscape'} dark cover">${mast('Aura')}<div class="cover-copy"><div class="kicker">PROJETO · AURA 0.3.0</div><h1>Aura</h1><h2>Seu assistente de IA<br>no Windows.</h2><p>Conversa, contexto e voz.<br>Sempre à mão.</p></div><div class="cover-screen">${shot(chat.file,chat.width,chat.height)}</div>${!car?`<div class="cover-compact">${shot(compact.file,compact.width,compact.height)}</div>`:''}${footer(1,car?20:null)}</article>`);
}
function carousel(item,n){
  const dark=['conversa','tarefas','reuniao','about'].includes(item.id);
  let visual=''; let layout='';
  const s=item.sections;
  switch(item.id){
    case 'general': visual=crop(s[0].file,768,300,0,940,'Aparência e comportamento')+crop(s[1].file,768,s[1].height,0,940,'Seu jeito de conversar');break;
    case 'account':visual=crop(s[0].file,768,s[0].height,0,940)+crop(s[1].file,768,s[1].height,0,940);break;
    case 'providers':visual=crop(s[0].file,768,s[0].height,0,940)+`<div class="editorial-list"><span>ChatGPT.</span><span>Sua chave.</span><span>Modelo local.</span></div>`;break;
    case 'privacy':visual=crop(s[0].file,768,s[0].height,0,910)+crop(s[2].file,768,s[2].height,0,910);break;
    case 'personal':visual=s.slice(0,3).map(x=>crop(x.file,768,x.height,0,870)).join('');break;
    case 'voice':visual=crop(s[0].file,768,s[0].height,0,870)+crop(s[2].file,768,s[2].height,0,870);break;
    case 'extensions':visual=crop(s[0].file,768,s[0].height,0,870)+crop(s[1].file,768,s[1].height,0,870);break;
    case 'profiles':visual=crop(s[0].file,768,s[0].height,0,940)+`<div class="editorial-list"><span>Código.</span><span>Pesquisa.</span><span>Escrita.</span></div>`;break;
    case 'shortcuts':visual=crop(s[0].file,768,s[0].height,0,940)+`<div class="key-scene"><span>Ctrl</span><b>+</b><span>Shift</span><b>+</b><span>Space</span></div>`;break;
    case 'diagnostics':visual=crop(s[0].file,768,s[0].height,0,860)+crop(s[1].file,768,s[1].height,0,860);break;
    case 'about':visual=crop(s[0].file,768,s[0].height,0,940)+`<div class="about-brand"><img src="${logo}"><div>Feito para<br>estar por perto.</div></div>`;break;
    case 'arquivos':visual=crop(item.file,1180,575,0,940)+crop(item.file,320,145,105,850,'Prévia do arquivo').replace(`width:850px;top:`, `width:${850*1180/320}px;left:${-860*850/320}px;top:`);break;
    case 'preparo':visual=shot(item.file,item.width,item.height)+`<div class="caption-line">Contexto, receitas e preparo antes de começar.</div>`;break;
    default:visual=shot(item.file,item.width,item.height);
  }
  if(item.id==='arquivos'){
    visual=shot(item.file,item.width,item.height)+`<div class="editorial-row"><b>Gerar.</b><b>Revisar.</b><b>Reutilizar.</b></div>`;
  }
  add(`${String(n).padStart(2,'0')}-${item.id}`,'instagram',`<article class="frame portrait ${dark?'dark':'light'}">${mast(item.label)}<div class="intro"><div class="kicker">${escape(item.label)}</div><h1>${escape(item.headline)}</h1><p>${escape(item.description)}</p></div><div class="portrait-visual ${layout}">${visual}</div>${pills(item.id)}${footer(n)}</article>`);
}
function detail(item,section,n,part=0,count=1){
  const y=part*492;const h=Math.min(540,section.height-y);
  const title=section.title;
  const fileName=`${String(n).padStart(2,'0')}-${item.id}-${slug(title)}${count>1?'-'+(part+1):''}`;
  add(fileName,'portfolio',`<article class="frame landscape light detail">${mast(item.label)}<div class="detail-title"><div class="kicker">${escape(item.label)} ${count>1?'· DETALHE '+(part+1)+'/'+count:''}</div><h1>${escape(title)}</h1></div><div class="detail-visual">${crop(section.file,768,h,y,Math.min(920,720*490/h))}</div>${footer(n,null)}</article>`);
}
const css=`
*{box-sizing:border-box}html,body{margin:0;padding:0}body{background:#20202b;font-family:'Segoe UI Variable Display','Segoe UI',sans-serif;-webkit-font-smoothing:antialiased}.frame{position:relative;overflow:hidden;isolation:isolate;margin:24px auto}.landscape{width:1280px;height:800px}.portrait{width:1080px;height:1350px}.dark{--fg:#f3f1fc;--muted:#b5b0c8;--rule:#38313e;--accent:#a9a3ff;background:#111019;color:var(--fg)}.light{--fg:#231d3c;--muted:#70697f;--rule:#d9d4e7;--accent:#5b50d8;background:#f0edf7;color:var(--fg)}
header{position:absolute;top:38px;left:52px;right:52px;display:flex;align-items:center;justify-content:space-between;z-index:4}header>span{font-size:10px;letter-spacing:1.8px;color:var(--muted)}.brand{display:flex;align-items:center;gap:10px;font-size:22px;font-weight:650;letter-spacing:-.5px}.brand img{width:30px;height:30px}.portrait header{top:48px;left:60px;right:60px}.portrait .brand{font-size:30px;gap:14px}.portrait .brand img{width:40px;height:40px}.portrait header>span{font-size:11px;letter-spacing:1.6px}.kicker{font-size:11px;font-weight:600;letter-spacing:2px;text-transform:uppercase;color:var(--accent);margin-bottom:18px}h1,h2,p{margin:0}h1{font-weight:650;line-height:1.08;letter-spacing:-2px}p{color:var(--muted);line-height:1.55;font-weight:400}.left{position:absolute;left:52px;top:215px;width:270px;z-index:3}.left h1{font-size:43px;margin-bottom:25px}.left p{font-size:17px}.vertical-points{margin-top:28px;font-size:12px;color:var(--muted);line-height:2.45}.vertical-points>div{border-top:1px solid var(--rule)}.vertical-points>div:first-child{border-top:0}.visual{position:absolute;left:370px;right:50px;top:110px;bottom:84px;display:flex;align-items:center;justify-content:center;flex-direction:column;gap:35px}.screen{position:relative;overflow:hidden;background:#26262c;border-radius:12px;border:1px solid #ffffff16;box-shadow:0 24px 55px #180d4226,0 4px 9px #17102118;line-height:0;flex-shrink:0}.screen img{display:block;width:100%;height:auto}.visual>.screen{width:100%;max-height:605px}.visual .screen img{height:100%;object-fit:contain}.compact-visual .screen{border-radius:8px}.shortcut{font-size:27px;font-weight:600;letter-spacing:-.5px;color:var(--muted)}.shortcut em{font-size:17px;font-style:normal;color:var(--accent);padding:0 8px}
.orbit{position:absolute;right:-150px;bottom:-280px;width:630px;height:630px;z-index:-1;border-radius:50%;border:1px solid var(--rule);opacity:.65}.orbit i{position:absolute;border:1px solid var(--rule);border-radius:50%;inset:75px}.orbit i:nth-child(2){inset:160px}.orbit i:nth-child(3){inset:250px;background:var(--rule);border:none}.portrait .orbit{width:900px;height:900px;right:-310px;bottom:-430px;opacity:.55}.portrait .orbit i{inset:120px}.portrait .orbit i:nth-child(2){inset:250px}.portrait .orbit i:nth-child(3){inset:380px}
footer{position:absolute;left:52px;right:52px;bottom:30px;display:flex;align-items:center;justify-content:space-between;border-top:1px solid var(--rule);padding-top:17px;font-size:10px;letter-spacing:1.3px;color:var(--muted);z-index:4}footer b{padding:0 8px;color:var(--accent)}footer .demo{font-size:9px;letter-spacing:.1px}footer strong{font-weight:500;font-size:12px;color:var(--fg)}.portrait footer{left:60px;right:60px;bottom:36px;padding-top:20px;font-size:10px}.portrait footer strong{font-size:16px}.portrait footer .demo{font-size:10px}
.intro{position:absolute;top:145px;left:60px;right:60px;z-index:2}.intro .kicker{font-size:13px;margin-bottom:22px}.intro h1{font-size:64px;max-width:940px;line-height:1.07;letter-spacing:-2.5px;margin-bottom:24px}.intro p{font-size:25px;max-width:890px;line-height:1.45}.portrait-visual{position:absolute;top:440px;left:60px;right:60px;display:flex;align-items:center;flex-direction:column;gap:24px}.portrait-visual>.screen{width:960px}.portrait .screen{border-radius:14px}.focus{flex:none}.crop>img{position:absolute;height:auto;max-width:none;left:0}.focus-label{font-size:13px;line-height:1.3;color:var(--muted);margin-bottom:12px;letter-spacing:.3px}.pills{position:absolute;bottom:103px;left:60px;right:60px;display:flex;justify-content:center;gap:20px;color:var(--muted);font-size:12px;z-index:3}.pills span+span:before{content:'·';margin-right:20px;color:var(--accent)}.editorial-list{align-self:flex-start;margin-left:40px;display:flex;flex-direction:column;gap:8px;font-size:57px;font-weight:550;letter-spacing:-1.5px;margin-top:60px}.editorial-list span:nth-child(2){color:var(--accent)}.editorial-row{display:flex;gap:30px;align-self:center;font-size:42px;color:var(--fg);margin-top:65px;font-weight:550;letter-spacing:-1.5px}.editorial-row b:nth-child(2){color:var(--accent)}.key-scene{display:flex;gap:18px;align-items:center;margin-top:100px;font-size:33px}.key-scene span{padding:27px 29px;border:1px solid var(--rule);border-bottom-width:5px;border-radius:14px;background:#ffffff77;color:var(--fg);font-weight:550}.key-scene b{font-weight:400;color:var(--accent)}.caption-line{font-size:21px;color:var(--muted);margin-top:30px}.about-brand{display:flex;gap:40px;align-items:center;align-self:flex-start;margin:120px 30px 0}.about-brand img{width:170px;height:170px}.about-brand div{font-size:56px;line-height:1.15;letter-spacing:-1.5px;font-weight:600}
.cover .orbit{right:-100px;top:-260px;bottom:auto;border-color:#8074ff55;opacity:.65;width:750px;height:750px}.cover .orbit i{border-color:#8074ff55}.cover .orbit i:nth-child(3){background:#8074ff44}.cover-copy{position:absolute;top:160px;left:60px;z-index:3}.cover-copy h1{font-size:126px;letter-spacing:-7px;line-height:1.05;margin-bottom:17px}.cover-copy h1 span{display:none}.cover-copy h2{font-size:37px;line-height:1.15;font-weight:550;letter-spacing:-1px}.cover-copy p{font-size:19px;margin-top:25px}.cover-screen{position:absolute;left:520px;top:158px;width:710px;z-index:2;transform:rotate(-2deg)}.cover-compact{position:absolute;left:62px;top:586px;width:585px;transform:rotate(1deg);z-index:4}.cover .screen{box-shadow:0 30px 80px #0008,0 0 0 1px #b9a9ff22}.portrait.cover .cover-copy{top:157px;left:60px}.portrait.cover .cover-copy h1{font-size:170px;letter-spacing:-8px;margin-bottom:10px}.portrait.cover .cover-copy h2{font-size:49px;line-height:1.16}.portrait.cover .cover-copy p{font-size:25px;margin-top:20px}.portrait.cover .cover-screen{left:80px;top:622px;width:920px;transform:rotate(-2deg)}.portrait.cover .orbit{right:-260px;top:-270px;width:940px;height:940px}
.detail-title{position:absolute;left:60px;top:117px;right:60px}.detail-title .kicker{margin-bottom:12px}.detail-title h1{font-size:36px;letter-spacing:-1px;line-height:1.15}.detail-visual{position:absolute;top:215px;bottom:90px;left:60px;right:60px;display:flex;align-items:center;justify-content:center}.detail .screen{border-radius:12px}.closing .intro h1{font-size:74px;line-height:1.02}.closing-visual{position:absolute;top:545px;left:60px;right:60px}.closing-visual .screen{width:960px;margin-top:42px}.closing-key{font-size:55px;letter-spacing:-2px;margin-bottom:18px;font-weight:600}.closing-label{font-size:16px;color:var(--muted);margin:20px 0 56px}.closing .smallbar{width:860px;margin-left:50px}.closing-note{position:absolute;top:1060px;left:60px;right:60px;font-size:30px;line-height:1.4;color:var(--muted)}
`;
(async()=>{
 const m=JSON.parse(await fs.readFile(path.join(OUT,'manifesto.json'),'utf8'));
 for(const folder of ['01-portfolio','02-instagram','fontes'])await fs.mkdir(path.join(OUT,folder),{recursive:true});
 await fs.access(path.join(OUT,'fontes/logo.svg'));
 cover(m,'portfolio'); let n=2;
 const order=['conversa','contexto','tarefas','arquivos','alteracoes','historico','preparo','reuniao','overlay-compacto','minibar','general','account','providers','privacy','personal','voice','extensions','profiles','shortcuts','diagnostics','about'];
 for(const id of order)overview(m.images.find(x=>x.id===id),n++);
 const details={general:[1,2,3],privacy:[2,3,4,5],personal:[2,3],voice:[1,2],extensions:[2]};
 for(const [id,indices] of Object.entries(details))for(const index of indices){const item=m.images.find(x=>x.id===id),section=item.sections[index]; const count=Math.max(1,Math.ceil((section.height-48)/492));for(let p=0;p<count;p++)detail(item,section,n++,p,count);}
 cover(m,'instagram');const carouselIds=['conversa','contexto','tarefas','arquivos','historico','preparo','reuniao','general','account','providers','privacy','personal','voice','extensions','profiles','shortcuts','diagnostics','about'];carouselIds.forEach((id,i)=>carousel(m.images.find(x=>x.id===id),i+2));
 const compact=m.images.find(x=>x.id==='overlay-compacto'),mini=m.images.find(x=>x.id==='minibar');
 add('20-overlay-e-minibar','instagram',`<article class="frame portrait dark closing">${mast('Aura')}<div class="intro"><div class="kicker">OVERLAY + MINIBAR</div><h1>Por perto.<br>Sem sair do fluxo.</h1><p>Abra pelo atalho. Acompanhe a resposta<br>enquanto continua trabalhando.</p></div><div class="closing-visual"><div class="closing-key">Ctrl + Shift + Space</div>${shot(compact.file,compact.width,compact.height)}<div class="closing-label">Overlay compacto · pronto para a próxima ideia.</div><div class="smallbar">${shot(mini.file,mini.width,mini.height)}</div><div class="closing-label">Minibar · uma resposta continua em andamento.</div></div><div class="closing-note">Conversa. Contexto. Voz.<br><span style="color:var(--fg)">Tudo no seu Windows.</span></div>${footer(20)}</article>`);
 const source=`<!doctype html><html lang="pt-BR"><head><meta charset="utf-8"><title>Aura — composições editáveis</title><style>${css}\n.portrait.cover .cover-screen{left:105px;top:602px;width:870px;}</style></head><body>${frames.map((f,i)=>`<div id="f${i}" data-kind="${f.kind}" data-name="${f.name}">${f.html}</div>`).join('')}<script>const id=new URLSearchParams(location.search).get('frame');if(id){document.querySelectorAll('body>div').forEach(e=>{e.style.display=e.id===id?'block':'none'});document.getElementById(id).querySelector('.frame').style.margin='0';}window.addEventListener('load',()=>{document.querySelectorAll('.portrait-visual').forEach(e=>{if(e.offsetHeight>760){e.style.transform='scale('+(760/e.offsetHeight)+')';e.style.transformOrigin='top center';}});});</script></body></html>`;
 const sourcePath=path.join(OUT,'fontes/composicoes.html');await fs.writeFile(sourcePath,source);
 const browser=await chromium.launch({headless:true});const page=await browser.newPage({deviceScaleFactor:2});
 for(let i=0;i<frames.length;i++){
   const f=frames[i];await page.setViewportSize(f.kind==='instagram'?{width:1080,height:1350}:{width:1280,height:800});
   await page.goto(pathToFileURL(sourcePath).href+'?frame=f'+i);await page.evaluate(()=>Promise.all([...document.images].map(im=>im.decode())));await page.evaluate(()=>document.fonts.ready);
   const dir=f.kind==='instagram'?'02-instagram':'01-portfolio';const filename=f.name+'.png';
   await page.screenshot({path:path.join(OUT,dir,filename)});f.file=dir+'/'+filename;
   if(i%10===0)console.log('Compostas '+(i+1)+'/'+frames.length);
 }
 await fs.writeFile(path.join(OUT,'fontes/indice.json'),JSON.stringify(frames.map(({html,...f})=>f),null,2));
 const gallery=`<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Aura — imagens para portfólio e Instagram</title><style>body{margin:0;background:#111019;color:#f3f1fc;font:16px 'Segoe UI',sans-serif;padding:40px}h1{font-size:40px;letter-spacing:-1px;margin:0 0 10px}p{color:#b5b0c8;max-width:900px;line-height:1.6}h2{font-size:24px;margin-top:40px}a{color:inherit;text-decoration:none}.grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(310px,1fr));gap:20px}.grid img{width:100%;display:block;border-radius:8px;background:#292530}.card{border:1px solid #393143;border-radius:10px;overflow:hidden}.card div{padding:14px;font-size:13px;color:#b5b0c8}.portrait img{aspect-ratio:4/5;object-fit:contain}.landscape img{aspect-ratio:16/10;object-fit:contain}</style><h1>Aura.</h1><p>Imagens de apresentação para o portfólio e uma sequência de 20 imagens para o Instagram. As telas usam os componentes reais do Aura 0.3.0 com dados de demonstração. Clique para abrir cada imagem.</p>${['instagram','portfolio'].map(kind=>`<h2>${kind==='instagram'?'Carrossel · 2160 × 2700 px':'Portfólio · 2560 × 1600 px'}</h2><div class="grid ${kind==='instagram'?'portrait':'landscape'}">${frames.filter(f=>f.kind===kind).map(f=>`<a class="card" href="${encodeURI(f.file)}"><img loading="lazy" src="${encodeURI(f.file)}" alt="${escape(f.name)}"><div>${escape(f.name.replaceAll('-',' '))}</div></a>`).join('')}</div>`).join('')}</html>`;
 await fs.writeFile(path.join(OUT,'galeria.html'),gallery);
 // A compact contact sheet for checking every exported composition together.
 const sheet=`<!doctype html><html><meta charset="utf-8"><style>body{margin:0;background:#17131f;color:white;padding:20px;font:12px 'Segoe UI';}.grid{display:grid;grid-template-columns:repeat(4,1fr);gap:18px;}img{width:100%;display:block;}label{display:block;padding:7px 0 10px;font-size:12px;}h1{font-size:26px}h2{margin-top:28px}</style><h1>Aura — índice visual</h1>${['instagram','portfolio'].map(kind=>`<h2>${kind}</h2><div class="grid">${frames.filter(f=>f.kind===kind).map(f=>`<div><img src="${pathToFileURL(path.join(OUT,f.file)).href}"><label>${escape(f.name)}</label></div>`).join('')}</div>`).join('')}</html>`;
 await page.setViewportSize({width:1440,height:900});await page.setContent(sheet);await page.evaluate(()=>Promise.all([...document.images].map(im=>im.decode())));await page.screenshot({path:path.join(OUT,'indice-visual.png'),fullPage:true,scale:'css'});
 for(const kind of ['instagram','portfolio']){
   const contact=sheet.replace(/<h1>Aura — índice visual<\/h1>[\s\S]*$/,'<h1>Aura — '+kind+'</h1><div class="grid">'+frames.filter(f=>f.kind===kind).map(f=>`<div><img src="${pathToFileURL(path.join(OUT,f.file)).href}"><label>${escape(f.name)}</label></div>`).join('')+'</div></html>');
   await page.setContent(contact);await page.evaluate(()=>Promise.all([...document.images].map(im=>im.decode())));await page.screenshot({path:path.join(OUT,'indice-'+kind+'.png'),fullPage:true,scale:'css'});
 }
 await fs.writeFile(path.join(OUT,'LEIA-ME.txt'),`AURA — IMAGENS PARA PORTFÓLIO E INSTAGRAM\nCriado em 11/10/2026.\n\n01-portfolio: ${frames.filter(f=>f.kind==='portfolio').length} imagens PNG horizontais, 2560 x 1600 px.\n02-instagram: 20 imagens PNG verticais, 2160 x 2700 px (4:5), numeradas na ordem do carrossel.\n03-capturas-originais: telas completas, recortes de cada seção e painéis, em resolução 2x.\n\nAbra galeria.html para ver tudo. indice-visual.png é o contato visual do conjunto.\n\nAs telas foram renderizadas a partir dos componentes reais da versão 0.3.0, na prévia local do projeto. Conta, conversas, provedores, notas, reuniões e estados técnicos são dados de demonstração. Não há credenciais reais. A função de ocultação de capturas do Aura impediu a captura direta das janelas de produção, por isso foi usada a prévia local.\n\nAs 11 abas de configurações estão incluídas. As páginas longas têm capturas completas nos originais e imagens adicionais de detalhes no conjunto do portfólio. Também foram incluídos conversa, contexto, Modo Tarefa, arquivos, alterações, histórico, preparo de reunião, reunião, Overlay compacto e Minibar.\n\nfontes/composicoes.html contém os layouts editáveis. O logo é o ativo original do projeto. Nenhum texto ou controle foi redesenhado dentro das capturas.\n`);
 await browser.close();console.log('Final: '+frames.length+' composições em '+OUT);
})().catch(e=>{console.error(e);process.exit(1)});
