const fs = require('node:fs/promises');
const path = require('node:path');
const { chromium } = require('playwright');
const OUT = path.resolve(__dirname, '..');
const BASE = process.env.AURA_PREVIEW_URL || 'http://127.0.0.1:1421/';
const raw = path.join(OUT, '03-capturas-originais');
const manifest = [];
const pages = [
  ['general','Geral','Do seu jeito.','Tema, aparência, contexto e preferências para o dia a dia.'],
  ['account','Conta e modelos','Sua conta. Seu modelo.','Conta ChatGPT e esforço de raciocínio por modelo e modo.'],
  ['providers','Provedores','Escolha de onde vem a IA.','Provedores com suas próprias credenciais e modelos locais.'],
  ['privacy','Privacidade','Você decide o contexto.','Controle sobre tela, microfone, áudio do sistema e retenção.'],
  ['personal','Lembretes e notas','As ideias ficam.','Lembretes, notas e textos salvos em um só lugar.'],
  ['voice','Voz','Fale. O Aura acompanha.','Reconhecimento de fala local, vocabulário e leitura em voz alta.'],
  ['extensions','Extensões','Um assistente que se expande.','Skills, servidores MCP e comandos rápidos.'],
  ['profiles','Perfis de app','Cada app, um contexto.','Instruções e preferências para o aplicativo em que você está.'],
  ['shortcuts','Atalhos','Sempre a um atalho.','Abra o Overlay, fale ou dite em qualquer aplicativo.'],
  ['diagnostics','Diagnóstico','Tudo à vista.','Estado dos componentes e exportação de diagnóstico para suporte.'],
  ['about','Sobre','Aura, por dentro.','Versão, código-fonte, licenças e atualizações.'],
];
async function seed(page) {
  await page.evaluate(async () => {
    const {getBridge} = await import('/src/ipc/bridge.ts');
    const {useApp} = await import('/src/state/app.ts');
    await import('/src/state/app.ts').then(m=>m.bootstrap());
    const b=await getBridge(); const s=b.state; window.presentationBridge=b;
    const now = Date.parse('2026-10-11T14:00:00-03:00');
    s.account={clientId:'presentation',subject:'demo',email:'criador@exemplo.com',scopes:['openid','chatgpt.tokens.use.direct'],planUsageEnabled:true,expiresAt:null,active:true,welcomed:true,signedIn:true};
    s.settings={...s.settings,theme:'dark',opacity:1,defaultModel:'gpt-6.1-sol',personalInstructions:'Responda em português. Seja claro e direto. Use exemplos quando ajudarem.',memories:true,asrLanguage:'pt',asrVocabulary:['Aura','Tauri','TypeScript'],effortPresets:{'chatgpt-plan':{'gpt-6.1-sol:chat':'medium'}}};
    s.memorySummary='- Prefere respostas em português.\n- Trabalha com desenvolvimento de software.\n- Gosta de exemplos práticos.';
    s.memoryRegistry=s.memorySummary;
    const model=(id,name,images=true)=>({id,displayName:name,contextWindow:128000,maxOutput:16000,supportsImages:images,supportsTools:true,supportsReasoning:false,estimated:true});
    const quirks={noParallelToolCalls:false,reasoningEffort:false,noStreamOptions:false};
    s.providers=[
      {id:'demo-openai',name:'OpenAI',preset:'openai',wire:'responses',baseUrl:'https://api.openai.com/v1',auth:'bearer',extraHeaders:{},models:[model('modelo-exemplo','Modelo de demonstração')],credentialHint:'••••demo',status:'verified',lastError:null,quirks},
      {id:'demo-local',name:'Ollama · local',preset:'ollama',wire:'chat',baseUrl:'http://127.0.0.1:11434/v1',auth:'bearer',extraHeaders:{},models:[model('modelo-local','Modelo local',false)],credentialHint:null,status:'verified',lastError:null,quirks}
    ];
    s.skills=[['aura-estudo','Estude com explicações, exemplos e perguntas de revisão.'],['aura-documentos','Entenda documentos e organize os pontos principais.'],['aura-preparo','Prepare reuniões, entrevistas e apresentações.'],['aura-configurar','Configure o Aura com ajuda do agente.']].map(([name,description])=>({manifest:{name,description,extra:{}},skillMd:description,files:[{path:'SKILL.md',bytes:240,isScript:false}],warning:''}));
    s.mcp=[{name:'documentos-demo',transport:{type:'stdio',command:'servidor-mcp-demo',args:[],env:{},cwd:null},enabled:true,disabledTools:[],approvalMode:'askForWrites',startupTimeoutSec:null,toolTimeoutSec:null}];
    s.profiles=[
      {id:'demo-code',name:'Desenvolvimento',processPattern:'Code.exe',titleGlob:null,instructions:'Explique o código e proponha alterações pequenas, com contexto.',attachScreen:false,defaultMode:'task',defaultModel:null},
      {id:'demo-browser',name:'Pesquisa e leitura',processPattern:'msedge.exe',titleGlob:null,instructions:'Destaque os pontos principais e cite as fontes quando pesquisar.',attachScreen:false,defaultMode:'chat',defaultModel:null},
      {id:'demo-docs',name:'Escrita',processPattern:'WINWORD.EXE',titleGlob:null,instructions:'Revise a clareza do texto e preserve a minha voz.',attachScreen:false,defaultMode:'chat',defaultModel:null}
    ];
    s.reminderList=[{id:'r1',text:'Revisar a apresentação do portfólio',dueAt:now/1000+3600,repeat:'none',prompt:null},{id:'r2',text:'Separar os pontos para a reunião do projeto',dueAt:now/1000+86400,repeat:'none',prompt:null}];
    s.noteList=[{id:'n1',kind:'note',text:'Ideia para o portfólio: mostrar o Aura no fluxo de trabalho, com exemplos de contexto e voz.',createdAt:now/1000-3600},{id:'n2',kind:'note',text:'Próxima reunião: alinhar prioridades, escopo e responsáveis.',createdAt:now/1000-7200},{id:'s1',kind:'saved',text:'Obrigado pelo retorno! Vou revisar os pontos e enviar uma atualização amanhã.',createdAt:now/1000-5400}];
    s.history=[['h1','Resumo da reunião do projeto','Prioridades, decisões e próximos passos.'],['h2','Revisar o texto do portfólio','Uma versão mais clara para apresentar o projeto.'],['h3','Entender este trecho de código','Explicação do fluxo e possíveis melhorias.'],['h4','Preparar a apresentação','Estrutura, exemplos e pontos de discussão.']].map(([id,title,preview],i)=>({id,title,preview,updatedAt:now/1000-i*3600,pinned:i===0,archived:false}));
    s.voiceModels=s.voiceModels.map((m,i)=>({...m,installed:i===0,selected:i===0}));
    s.privacy.screen={mode:{type:'recentBuffer',minutes:10},agent:'ask'};
    s.recordings=[{id:'rec-demo',source:'screen,mic',title:'Demonstração do projeto',startedAt:now/1000-900,endedAt:now/1000-300,bytes:18400000,durationMs:600000}];
    s.accessLog=[{id:1,at:now/1000-480,source:'screen',requester:'agent',tool:'screen_capture',conversation:null,decision:'allow',reason:'user_request',hasThumbnail:false,threadId:null},{id:2,at:now/1000-240,source:'screen',requester:'agent',tool:'screen_text',conversation:null,decision:'deny',reason:'excluded_window',hasThumbnail:false,threadId:null}];
    s.meetingBrief='Alinhar prioridades do projeto, revisar o escopo e definir os próximos passos.';
    s.projectList=[{id:'p1',name:'Portfólio',instructions:'Apresentação do projeto Aura.'}];
    s.meetingList=[{id:'meeting-demo',title:'Alinhamento do projeto',kind:'work',briefing:s.meetingBrief,origin:'live',status:'ended',startedAt:now-900000,endedAt:now-300000,projectId:'p1'}];
    s.meetingLines['meeting-demo']=[
      {id:1,meetingId:'meeting-demo',t0:0,t1:19000,speaker:'you',text:'Quero apresentar o app pelo que ele permite fazer no dia a dia.'},
      {id:2,meetingId:'meeting-demo',t0:22000,t1:46000,speaker:'them',text:'Vamos começar pela conversa e depois mostrar contexto, voz e tarefas.'},
      {id:3,meetingId:'meeting-demo',t0:48000,t1:60000,speaker:'note',text:'Prioridade: exemplos claros e imagens de cada área.'},
      {id:4,meetingId:'meeting-demo',t0:65000,t1:88000,speaker:'you',text:'Também quero mostrar os controles de privacidade e as extensões.'},
      {id:5,meetingId:'meeting-demo',t0:94000,t1:122000,speaker:'them',text:'Ótimo. A revisão do material fica para amanhã.'}
    ];
    s.actionList=[{id:'a1',meetingId:'meeting-demo',text:'Revisar o material de apresentação',owner:'you',due:'Amanhã',status:'open',t0:94000,createdAt:now,overdue:false}];
    const originalInvoke=b.invoke.bind(b);
    b.invoke=async (cmd,args)=>{
      if(cmd==='diagnostics') return {...await originalInvoke(cmd,args),version:'0.3.0',os:'prévia de demonstração',dataDir:'Perfil de demonstração',account:{email:'criador@exemplo.com',signedIn:true,planUsageEnabled:true}};
      if(cmd==='workspace_files') return [{path:'resumo-do-projeto.md',absolute:'C:/demo/resumo-do-projeto.md',bytes:1840,modified:now/1000},{path:'proximos-passos.md',absolute:'C:/demo/proximos-passos.md',bytes:920,modified:now/1000}];
      if(cmd==='workspace_read') return '# Resumo do projeto\n\n## Objetivo\nApresentar o Aura no fluxo de trabalho do Windows.\n\n## Próximos passos\n- Revisar o material do portfólio\n- Selecionar imagens para o carrossel\n- Organizar exemplos de contexto e voz';
      return originalInvoke(cmd,args);
    };
    useApp.getState().setSettings(s.settings); useApp.setState({auth:{accounts:[s.account],active:s.account},privacy:s.privacy,appServer:{state:'ready',version:'demonstração'},ready:true,welcome:false,notices:[]});
  });
  await page.waitForTimeout(220);
}
async function save(page,name,opts={}) {
  const file=path.join(raw,name+'.png');
  await page.screenshot({path:file,...opts});
  return file;
}
async function settled(page) {
  await page.evaluate(()=>document.fonts.ready);
  await page.waitForTimeout(180);
}
(async()=>{
  await fs.mkdir(raw,{recursive:true});
  const browser=await chromium.launch({headless:true});
  const page=await browser.newPage({viewport:{width:1180,height:850},deviceScaleFactor:2,colorScheme:'dark'});
  await page.goto(BASE+'#/settings/general'); await page.waitForSelector('main section'); await seed(page);
  for(const [id,label,headline,description] of pages){
    await page.evaluate(id=>location.hash='#/settings/'+id,id); await settled(page);
    await page.setViewportSize({width:1180,height:850});
    await page.locator('main').evaluate(e=>e.scrollTop=0);
    const sectionInfo=await page.locator('main section').evaluateAll(es=>es.map((e,i)=>({index:i,title:e.querySelector('h2')?.textContent,height:Math.ceil(e.getBoundingClientRect().height)})));
    const firstHeight=Math.min(850,Math.max(520,sectionInfo.slice(0,2).reduce((v,s)=>v+s.height+16,48)));
    await page.setViewportSize({width:1180,height:firstHeight});
    await settled(page); const file=await save(page,id);
    const item={id,label,headline,description,file,width:1180,height:firstHeight,kind:'settings',sections:[]};
    const fullHeight=await page.locator('main').evaluate(e=>e.scrollHeight);
    await page.setViewportSize({width:1180,height:Math.max(520,fullHeight+10)}); await settled(page);
    await save(page,id+'-completa');
    for(const info of sectionInfo){
      const section=page.locator('main section').nth(info.index);
      const filename=id+'-secao-'+(info.index+1);
      await section.screenshot({path:path.join(raw,filename+'.png')});
      item.sections.push({...info,file:path.join(raw,filename+'.png'),width:720});
    }
    manifest.push(item); console.log('Capturada: '+label+' ('+sectionInfo.length+' seções)');
  }
  await page.goto(BASE); await page.reload(); await page.waitForSelector('.overlay-shell'); await seed(page);
  await page.evaluate(async()=>{
    const {useSession}=await import('/src/overlay/session.ts'); const {useConversation,emptyThread}=await import('/src/state/conversation.ts');
    window.session=useSession; window.conversation=useConversation; window.emptyThread=emptyThread;
    await useSession.getState().loadCatalog();
    useSession.setState({overlayMode:'expanded',mode:'chat',model:'gpt-6.1-sol',effort:'medium',threadId:'demo',historyOpen:false,workOpen:false,meetingOpen:false,minibar:false});
    const blocks=[{type:'user',id:'u1',text:'Me ajude a organizar a apresentação do meu projeto.',chips:[]},{type:'assistant',id:'m1',streaming:false,text:'Podemos organizar a apresentação em **três partes**:\n\n1. **O problema:** por que você criou o projeto.\n2. **O produto em ação:** mostre as telas com exemplos do dia a dia.\n3. **As decisões:** explique os principais cuidados com contexto, voz e privacidade.\n\nPara o portfólio, comece com uma visão geral e depois apresente cada recurso com uma imagem.'}];
    window.chatBlocks=blocks;
    useConversation.setState({threads:{demo:{...emptyThread('demo'),blocks,tokens:{used:2400,window:200000},lastStatus:'completed'}},requests:{}});
  });
  async function overlay(id,label,headline,description,width=820,height=610){
    await page.setViewportSize({width,height}); await settled(page);
    const file=await save(page,id); manifest.push({id,label,headline,description,file,width,height,kind:'overlay',sections:[]});
    console.log('Capturado: '+label);
  }
  await overlay('conversa','Conversa','A IA entra no seu fluxo.','Converse, organize ideias e receba respostas no Overlay.');
  await page.evaluate(()=>session.setState({overlayMode:'compact',threadId:null}));
  await page.setViewportSize({width:820,height:440}); await settled(page);
  const box=await page.locator('.overlay-shell').boundingBox();
  const compactFile=await save(page,'overlay-compacto',{clip:{x:0,y:0,width:820,height:Math.ceil(box.height)}});
  manifest.push({id:'overlay-compacto',label:'Overlay compacto',headline:'Sempre à mão.',description:'Um atalho abre o Aura sobre o seu aplicativo.',file:compactFile,width:820,height:Math.ceil(box.height),kind:'overlay',sections:[]});
  await page.evaluate(()=>session.setState({overlayMode:'expanded',threadId:'demo'}));
  await page.setViewportSize({width:820,height:610}); await settled(page);
  await page.locator('textarea').first().fill('@'); await settled(page);
  await overlay('contexto','Contexto','Mostre o que importa.','Tela, região, seleção, áudio ou arquivos no próximo turno.');
  await page.locator('textarea').first().fill('');
  await page.evaluate(()=>{
    const blocks=[{type:'user',id:'u2',text:'Crie um arquivo com os próximos passos do projeto.',chips:[]},{type:'plan',id:'p1',text:'Vou organizar os próximos passos e salvar o resultado no workspace da conversa.'},{type:'approval',approval:{requestId:'approval-demo',kind:'fileChange',command:null,cwd:null,reason:'Salvar os próximos passos no workspace da conversa.',changes:[{path:'proximos-passos.md',kind:'add',diff:'+ # Próximos passos\n+ - Revisar a apresentação\n+ - Selecionar as imagens'}],options:['accept','acceptForSession','decline']}}];
    session.setState({mode:'task'});conversation.setState({threads:{demo:{...emptyThread('demo'),blocks,running:true,turnId:'demo-turn',tokens:{used:4200,window:200000}}},requests:{'approval-demo':'demo'}});
  });
  await overlay('tarefas','Modo Tarefa','Da conversa à ação.','Plano de trabalho e aprovação antes de alterar arquivos.');
  await page.evaluate(()=>{session.setState({mode:'task',workOpen:true});conversation.setState({threads:{demo:{...emptyThread('demo'),blocks:[{type:'assistant',id:'done',text:'Organizei o resumo do projeto e os próximos passos. Os arquivos estão no workspace da conversa.',streaming:false}],diff:'diff --git a/proximos-passos.md b/proximos-passos.md\n--- /dev/null\n+++ b/proximos-passos.md\n@@ -0,0 +1,4 @@\n+# Próximos passos\n+\n+- Revisar a apresentação\n+- Selecionar as imagens',lastStatus:'completed',tokens:{used:4100,window:200000}}},requests:{}});});
  await page.setViewportSize({width:1180,height:630}); await settled(page);
  await page.getByText('resumo-do-projeto.md',{exact:true}).first().click();
  await overlay('arquivos','Arquivos','O resultado fica aqui.','Arquivos gerados, prévias e alterações no workspace da conversa.',1180,630);
  await page.getByRole('button',{name:'Alterações',exact:true}).click();
  await overlay('alteracoes','Alterações','Revise o que mudou.','Acompanhe as alterações produzidas em uma tarefa.',1180,630);
  await page.evaluate(()=>{session.setState({workOpen:false,historyOpen:true,mode:'chat'});conversation.setState({threads:{demo:{...emptyThread('demo'),blocks:window.chatBlocks,lastStatus:'completed',tokens:{used:2400,window:200000}}},requests:{}});});
  await overlay('historico','Histórico','Continue de onde parou.','Encontre, fixe e retome suas conversas.',1180,630);
  await page.evaluate(()=>session.setState({historyOpen:false,meetingOpen:true}));
  await overlay('preparo','Preparo','Antes da reunião.','Prepare o contexto, escolha uma receita e organize os compromissos.',1180,760);
  await page.setViewportSize({width:1180,height:1800}); await settled(page); await save(page,'preparo-completa');
  await page.locator('aside').screenshot({path:path.join(raw,'preparo-painel.png')});
  await page.evaluate(()=>{
    const b=window.presentationBridge; b.state.meetingNow={...b.state.meetingList[0],status:'active',startedAt:Date.now()-125000,endedAt:null};
    const {useApp}=window;
  });
  await page.evaluate(async()=>{const {useApp}=await import('/src/state/app.ts');useApp.setState(s=>({meetingRevision:s.meetingRevision+1}));conversation.setState({threads:{demo:{...emptyThread('demo'),blocks:[{type:'user',id:'q',text:'Quais são os pontos principais até agora?',chips:[]},{type:'assistant',id:'a',streaming:false,text:'Até agora, vocês alinharam:\n\n- Começar pela **conversa no Overlay**.\n- Mostrar **contexto, voz e tarefas** com exemplos.\n- Destacar os controles de **privacidade**.\n- Revisar o material de apresentação amanhã.\n\nVocê também marcou uma nota sobre exemplos claros e imagens de cada área.'}],lastStatus:'completed'}},requests:{}});});
  await overlay('reuniao','Reunião','A conversa vira contexto.','Transcrição, notas e ajuda do agente durante a reunião.',1180,650);
  await page.evaluate(()=>{session.setState({meetingOpen:false,minibar:true});conversation.setState({threads:{demo:{...emptyThread('demo'),blocks:window.chatBlocks,running:true}},requests:{}});});
  await page.setViewportSize({width:680,height:100}); await settled(page);
  const miniBox=await page.locator('.overlay-shell').boundingBox();
  const miniFile=await save(page,'minibar',{clip:{x:0,y:0,width:680,height:Math.ceil(miniBox.height)}});
  manifest.push({id:'minibar',label:'Minibar',headline:'Acompanhe enquanto trabalha.',description:'Um indicador compacto para a resposta em andamento.',file:miniFile,width:680,height:Math.ceil(miniBox.height),kind:'overlay',sections:[]});
  await fs.writeFile(path.join(OUT,'manifesto.json'),JSON.stringify({created:'2026-10-11',source:'Aura 0.3.0 — componentes reais com backend de demonstração local',images:manifest},(key,value)=>key==='file'?path.relative(OUT,value).split(path.sep).join('/'):value,2));
  await browser.close(); console.log('Capturas prontas: '+OUT);
})().catch(e=>{console.error(e);process.exit(1)});
