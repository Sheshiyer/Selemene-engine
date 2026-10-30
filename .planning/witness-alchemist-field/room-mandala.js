/* ---------- Engines: the Engine Mandala ----------
   A bronze bench plate milled with a 19-cell honeycomb (1 + 6 + 12). Witness sits in the centre tray;
   seventeen engine trays spiral outward in ENGINES.md order, each carrying its Instruments Codex Dark still;
   the nineteenth pocket is empty and reads "the question". One tray lifts at a time. */
const ENGINES17=[
  ['panchanga','Panchanga','Five nested bronze rings on one spindle, a sun and a moon pointer: tithi, nakshatra, yoga, karana, vara.',51.2,33.6],
  ['human-design','Human Design','A nine-socket patch loom: hexagonal sockets joined by gold wire channels, gate pins along each wire.',50,44],
  ['gene-keys','Gene Keys','A cipher drum with sixty-four teeth and three depths of engraving: shadow, gift, siddhi.',50,42],
  ['vimshottari','Vimshottari','A nine-ring armillary, each ring a planet, one time index through all nine.',50.9,39.9],
  ['numerology','Numerology','A reduction abacus: gold beads fall through gates until one bead remains.',50,45],
  ['biorhythm','Biorhythm','A three-pen harmonograph writing three sine traces on a bronze drum.',51,30],
  ['vedic-clock','Vedic Clock','A single-hand twenty-four-hour face with a hora ring and an organ-meridian ring.',48.7,34.4],
  ['biofield','Biofield','A resonance capacitor: a bronze hand plate under an emerald glass dome, coil below.',50,38],
  ['face-reading','Face Reading','A craniometric armature: calipers on a ring around a featureless bronze mask.',49,40],
  ['nadabrahman','Nadabrahman','A monochord over a bronze resonator with a sand plate beneath.',50.8,37.2],
  ['transits','Transits','An orrery: planets on bronze arms around a central lens.',53.9,36.6],
  ['tarot','Tarot','A cabinet of seventy-eight shallow drawers, one drawer out.',50,45],
  ['i-ching','I Ching','A six-bar flip board, each bar whole or broken, with a yarrow counter.',50,45],
  ['enneagram','Enneagram','A nine-point escapement, the inner lines as gold wire, a pointer.',50.7,36.9],
  ['sacred-geometry','Sacred Geometry','A pantograph drawing a flower of life onto a bronze plate.',76.8,53.1],
  ['sigil-forge','Sigil Forge','A screw press that strikes a sigil into a bronze blank.',53.1,52.2],
  ['raaga','Raaga','A raga clock: seventy-two spokes, a sun pointer, seven bells on the rim.',52.7,37.9]];
const ENG_LINK={'human-design':['#design','Open Design'],'gene-keys':['#keys','Open Keys'],'vimshottari':['#jyotish','Open Jyotish'],'panchanga':['#jyotish','Open Jyotish'],'transits':['#sky','Open the Sky']};
function engState(id){if(ENG[id]&&ENG[id].result)return 'c';if(ENG[id])return 'u';return 'n'}
const STATE_WORD={c:'Computed',u:'Locked on this key',n:'Not called'};
const MANDALA={sel:0,lifted:null};
// the 19 cells: centre, then rings walked clockwise from the top, as a spiral
function mandalaCells(s){const out=[{q:0,r:0}];const dirs=[[1,0],[1,-1],[0,-1],[-1,0],[-1,1],[0,1]];
  for(let k=1;k<=2;k++){let q=-k,r=k;const ring=[];for(let d=0;d<6;d++)for(let i=0;i<k;i++){ring.push({q,r});q+=dirs[d][0];r+=dirs[d][1]}
    ring.forEach(c=>{c.x=s*1.5*c.q;c.z=s*Math.sqrt(3)*(c.r+c.q/2);c.ang=Math.atan2(-c.x,c.z)});ring.sort((a,b)=>((a.ang+Math.PI*2+1e-9)%(Math.PI*2))-((b.ang+Math.PI*2+1e-9)%(Math.PI*2)));out.push(...ring)}
  out[0].x=0;out[0].z=0;return out}
function cellInfo(i){if(i===0)return {id:'witness',name:'Witness',n:0,st:'w',mech:'The dyad behind /witness/interpret. Pichet holds structure, Aletheios names. Not an engine: the centre every engine reports to.'};
  if(i===18)return {id:'question',name:'The question',n:18,st:'q',mech:'The empty pocket at the end of the spiral. Enter when the question is active.'};
  const e=ENGINES17[i-1];return {id:e[0],name:e[1],n:i,st:engState(e[0]),mech:e[2],lx:e[3],ly:e[4]}}
const stillSrc=i=>`img/engines/${String(i).padStart(2,'0')}-${ENGINES17[i-1][0]}.jpg`;

function mandala3d(host){
  const K=K3,T=THREE;const RM=K.reduced();const s=5.2,cells=mandalaCells(s);
  let S=null;S=K.stage(host,{fov:30,yaw:-Math.PI/2,pitch:.98,dist:120,target:[0,-1.5,2],minDist:70,maxDist:200,minPitch:.55,maxPitch:1.4,label:'Engine mandala. Arrow keys walk the spiral; Enter opens an engine.',
    onResize:()=>{if(S&&!intro){S.view.dist=fit();S.place()}},
    ambient:(dt,now)=>{S.view.yaw=S.base.yaw+Math.sin(now/5200)*.05;return true},
    onPick:o=>{if(o&&o.userData.cell!=null){setSel(o.userData.cell,true)}},
    onHover:o=>{hoverCell=o?o.userData.cell:null;quiet=performance.now()},
    onInteract:()=>{quiet=performance.now()},
    onKey:e=>{const n=19;let i=MANDALA.sel;if(e.key==='ArrowRight'||e.key==='ArrowDown')i=(i+1)%n;else if(e.key==='ArrowLeft'||e.key==='ArrowUp')i=(i+n-1)%n;else if(e.key==='Home')i=0;else if(e.key==='End')i=18;
      else if(e.key==='Enter'||e.key===' '){openCell(MANDALA.sel);return true}else return false;setSel(i,true);return true}});
  const {scene,M}=S;let intro=false,hoverCell=null,quiet=performance.now(),tour=null,tourAt=0;
  function fit(){const c=S.camera,v=c.fov*Math.PI/360,hh=Math.atan(Math.tan(v)*c.aspect);const R=27;return Math.max(R*1.05/Math.sin(hh),(R*Math.sin(.98)+5)/Math.sin(v)*1.04,64)}
  const root=new T.Group();scene.add(root);
  const stars=K.starfield(700,240,420,31);stars.material.uniforms.uFade.value=.6;scene.add(stars);
  // plate: pointy-topped hexagon holding the flat-topped honeycomb
  const PR=27.5,PH=2.2,top=0;
  const plate=new T.Mesh(K.prism(K.hexShape(PR,Math.PI/2),PH,.45),M.bronze());plate.position.y=top-PH-.45;root.add(plate);
  const edge=new T.LineLoop(K.loopXZ(K.hexPts(PR+.42,Math.PI/2),0),M.line(K.HEX.gold,.9));edge.position.y=top+.02;root.add(edge);
  const groove=new T.LineLoop(K.loopXZ(K.hexPts(PR-1.4,Math.PI/2),0),M.line(K.HEX.gold,.35));groove.position.y=top+.02;root.add(groove);
  K.hexPts(PR-.7,Math.PI/2).forEach(([x,y])=>{const sc=new T.Mesh(new T.CylinderGeometry(.42,.42,.16,14),M.gold());sc.position.set(x,top+.08,-y);root.add(sc);
    const f=new T.Mesh(new T.CylinderGeometry(1.1,1.5,2.6,6),M.bronzeDark());f.position.set(x*.86,top-PH-.45-1.3,-y*.86);root.add(f)});
  const hal=K.halo(110,.55);hal.position.y=top-PH-3;root.add(hal);
  // rule strip and label plate on the near edge
  const lp=new T.Mesh(K.prism((()=>{const sh=new T.Shape();sh.moveTo(-4.6,-1);sh.lineTo(4.6,-1);sh.lineTo(3.4,1);sh.lineTo(-3.4,1);sh.closePath();return sh})(),.22,.06),M.bronzeDark());lp.position.set(0,top,-24.6);root.add(lp);
  const lpe=new T.LineLoop(K.loopXZ([[-4.7,-1.08],[4.7,-1.08],[3.45,1.08],[-3.45,1.08]].map(([x,y])=>[x,y]),0),M.line(K.HEX.gold,.8));lpe.position.set(0,top+.3,-24.6);root.add(lpe);
  const lpt=K.engrave('SELEMENE · 17',{font:'mono',px:30,color:'#C5A017',height:.62,track:.24});lpt.position.set(0,top+.31,-24.6);lpt.rotation.z=Math.PI;root.add(lpt);
  // engraved spiral through the cells in order
  const spiralPts=cells.map(c=>new T.Vector3(c.x,top+.03,-c.z));const spiral=new T.CatmullRomCurve3(spiralPts,false,'centripetal',.5);
  const spG=new T.BufferGeometry().setFromPoints(spiral.getPoints(400));const sp=new T.Line(spG,M.line(K.HEX.gold,.45));root.add(sp);
  // pockets and trays
  const loader=new T.TextureLoader();const trays=[];
  const hexTop=(r)=>{const g=new T.ShapeGeometry(K.hexShape(r,0),1);g.rotateX(-Math.PI/2);return g};
  cells.forEach((c,i)=>{const info=cellInfo(i);const g=new T.Group();g.position.set(c.x,0,-c.z);root.add(g);
    const pocket=new T.Mesh(hexTop(s*.94),M.flat(0x04060f,1));pocket.position.y=top+.015;g.add(pocket);
    const pr=new T.LineLoop(K.loopXZ(K.hexPts(s*.95,0),0),M.line(K.HEX.gold,.5));pr.position.y=top+.03;g.add(pr);
    const tray=new T.Group();g.add(tray);
    if(info.id==='question'){const dl=new T.LineLoop(K.loopXZ(K.hexPts(s*.62,0),0),new T.LineDashedMaterial({color:K.col(K.HEX.gold),dashSize:.5,gapSize:.45,transparent:true,opacity:.7,toneMapped:false}));dl.computeLineDistances();dl.position.y=top+.05;tray.add(dl);
      const q=S.track(K.label('THE QUESTION',{font:'mono',px:9,track:.2,color:'#C5A017',opacity:.85}));q.position.y=top+.6;tray.add(q)}
    else{const h=info.id==='witness'?2.4:1.1;
      const body=new T.Mesh(K.prism(K.hexShape(s*.86,0),h,.12),M.bronze());body.position.y=top-h*.55;tray.add(body);
      const rim=new T.Mesh(K.prism((()=>{const sh=K.hexShape(s*.9,0);const hole=new T.Path();K.hexPts(s*.8,0).forEach(([x,y],k)=>k?hole.lineTo(x,y):hole.moveTo(x,y));hole.closePath();sh.holes.push(hole);return sh})(),.16,0),M.gold());rim.position.y=top-h*.55+h+.1;tray.add(rim);
      const faceY=top-h*.55+h+.13;
      if(info.id==='witness'){[[0,.95],[-.82,-.48],[.82,-.48]].forEach(([x,y])=>{const t=new T.Mesh(new T.TorusGeometry(1.55,.13,12,64),M.gold());t.rotation.x=Math.PI/2;t.position.set(x*1.05,faceY+.3,-y*1.05);tray.add(t)});
        const fc=new T.Mesh(hexTop(s*.8),M.bronzeDark());fc.position.y=faceY;tray.add(fc);
        const ln=new T.Mesh(new T.SphereGeometry(.62,32,24),M.glass(K.HEX.emerald,{emissiveIntensity:.5}));ln.position.y=faceY+.45;tray.add(ln);const gw=K.glow(K.HEX.emerald,4.5,.5);gw.position.y=faceY+.5;tray.add(gw)}
      else{const g2=hexTop(s*.8);const uv=g2.attributes.uv,pos=g2.attributes.position;const W=s*.8*2,Hh=s*.8*Math.sqrt(3);
        for(let k=0;k<uv.count;k++){const x=pos.getX(k),z=pos.getZ(k);const u=.5+x/W;const v=.58+(-z/Hh)*(Hh/W)/1.5;uv.setXY(k,u,v)}uv.needsUpdate=true;
        const mat=new T.MeshBasicMaterial({color:new T.Color(1,1,1).multiplyScalar(info.st==='c'?.78:info.st==='u'?.4:.24),toneMapped:false});
        loader.load(stillSrc(i),tx=>{tx.encoding=T.sRGBEncoding;tx.anisotropy=8;mat.map=tx;mat.needsUpdate=true;S.invalidate()});
        const face=new T.Mesh(g2,mat);face.position.y=faceY;tray.add(face);c.face=mat;
        if(info.st!=='n'){const cab=new T.Mesh(new T.SphereGeometry(.42,24,16),M.glass(info.st==='c'?K.HEX.emerald:K.HEX.terracotta,{emissiveIntensity:.55}));cab.scale.y=.6;cab.position.set(0,faceY+.12,s*.72);tray.add(cab)}}
      const nm=S.track(K.label(info.name.toUpperCase(),{font:'mono',px:9,track:.16,color:info.st==='c'?'#C2CBD1':info.st==='w'?'#E6B84D':'#75838E'}));nm.center.set(.5,1.2);nm.position.set(0,top+.5,s*.92);g.add(nm);c.name=nm}
    const hit=new T.Mesh(new T.CylinderGeometry(s*.9,s*.9,3,6),new T.MeshBasicMaterial({visible:false}));hit.rotation.y=Math.PI/6;hit.position.y=top+1;hit.userData={cell:i,pick:true};S.pick(hit);g.add(hit);
    const gl=K.glow(K.HEX.goldWarm,s*3.2,0);gl.position.y=top+.6;g.add(gl);
    trays.push({g,tray,lift:0,goal:0,glow:gl,info,c})});
  // eased follow: one tray lifts at a time
  S.onFrame((dt)=>{let moving=false;const lift=hoverCell!=null?hoverCell:(tour!=null?tour:MANDALA.sel);
    trays.forEach((t,i)=>{t.goal=i===lift?3.2:(i===MANDALA.sel?1.2:0);const k=RM?1:1-Math.exp(-dt/.22);const nl=t.lift+(t.goal-t.lift)*k;if(Math.abs(nl-t.lift)>.002){moving=true}t.lift=nl;t.tray.position.y=nl;
      t.glow.material.opacity=Math.min(.55,nl/3.2*.55);t.glow.position.y=.6+nl*.5;if(t.c.face){const b=t.info.st==='c'?.78:t.info.st==='u'?.4:.24;t.c.face.color.setScalar(b+(1-b)*Math.min(1,nl/3.2)*.9)}});
    const now=performance.now();
    if(!RM&&hoverCell==null&&now-quiet>3600&&now-tourAt>1500){tourAt=now;tour=tour==null?(MANDALA.sel+1)%19:(tour+1)%19;moving=true}
    if(hoverCell!=null||now-quiet<3600){if(tour!=null){tour=null;moving=true}}
    return moving});
  function setSel(i,fromScene){MANDALA.sel=i;quiet=performance.now();tour=null;engReadout(i);document.querySelectorAll('.eng-index button').forEach(b=>b.setAttribute('aria-pressed',+b.dataset.cell===i));S.invalidate();S.kick()}
  // intro: trays condense out of the hub, the spiral draws outward
  if(!RM){intro=true;const d=fit();S.view.dist=d*1.5;S.view.pitch=1.32;S.place();
    trays.forEach((t,i)=>{const p=t.g.position.clone();t.g.position.set(0,0,0);t.g.scale.setScalar(.55);S.tween(900+i*55,k=>{t.g.position.copy(p).multiplyScalar(k);t.g.scale.setScalar(.55+.45*k)})});
    spG.setDrawRange(0,0);S.tween(1800,k=>{spG.setDrawRange(0,Math.floor(400*k))});
    S.tween(1900,k=>{S.view.dist=d*(1.5-.5*k);S.view.pitch=1.32+(.98-1.32)*k;S.base.pitch=S.view.pitch;S.place();if(k>=1)intro=false})}
  else{S.view.dist=fit();S.place()}
  return Object.assign(S,{select:i=>setSel(i),update(){}});
}

/* Readout: the engine under the pointer, as a still with InstrumentMotion and its figures */
function engReadout(i){const el=document.getElementById('engRead');if(!el)return;const info=cellInfo(i);const st=info.st;
  const stateBadge=st==='w'?'<span class="state w">Witness</span>':st==='q'?'<span class="state n">Open</span>':`<span class="state ${st}">${STATE_WORD[st]}</span>`;
  const still=i===0?`<div class="eng-still idle" style="aspect-ratio:1;max-width:260px"><img src="img/trinity-hex.jpg" alt="The trinity-hex medallion"><span class="halo"></span><span class="lens" style="--lx:50.2%;--ly:45.2%;display:block"></span><span class="sweep"></span></div>`
    :i===18?`<div class="eng-still idle" style="aspect-ratio:1;max-width:220px;display:grid;place-items:center;color:var(--gold)">${medallion('question',180)}</div>`
    :`<div class="eng-still ${st==='u'?'locked':st==='n'?'idle':''}" style="--lx:${info.lx}%;--ly:${info.ly}%"><img src="${stillSrc(i)}" alt="${esc(info.name)} instrument"><span class="halo"></span><span class="lens"></span><span class="sweep"></span></div>`;
  let figs='';
  if(i===0){const c=ENGINES17.filter(e=>engState(e[0])==='c').length,u=ENGINES17.filter(e=>engState(e[0])==='u').length;figs=`<dl class="kv"><dt>Computed</dt><dd><span class="ev-c">${c} engines</span> returned for your birth data</dd><dt>Locked</dt><dd><span class="ev-u">${u} engines</span> refused on this key</dd><dt>Not called</dt><dd>${17-c-u} engines</dd><dt>Source</dt><dd>selemene.tryambakam.space · v${(ENG['human-design']||{}).version||''}</dd></dl>`}
  else if(i===18){figs='<p class="muted" style="margin:0">Enter when the question is active. The system succeeds when you no longer need it.</p>'}
  else if(st==='c'){figs=`<div class="eng">${engBody(info.id)}</div>`}
  else if(st==='u'){figs=`<p class="muted" style="margin:0">The engine exists and this key may not run it: ${esc((ENG[info.id].error||'ENGINE_SCOPE_FORBIDDEN').replace(/_/g,' ').toLowerCase())}. It opens when the key tier allows.</p>`}
  else figs='<p class="muted" style="margin:0">Not called for this chart. The registry lists it; no response is held here.</p>';
  const prompt=st==='c'&&ENG[info.id].prompt?`<p class="q">${esc(ENG[info.id].prompt)}</p>`:'';
  const link=ENG_LINK[info.id];
  el.innerHTML=`<div style="display:flex;justify-content:space-between;align-items:center;gap:8px"><span class="num">${i===0?'CENTRE':i===18?'19 / 19':String(i).padStart(2,'0')+' / 17'} · ${esc(info.name.toUpperCase())}</span>${stateBadge}</div>
   ${still}<h2>${esc(info.name)}</h2><p class="mech">${esc(info.mech)}</p>${figs}${prompt}
   <div class="chips">${st==='c'?`<button class="pill sm primary" id="engOpen">Open the full return</button>`:''}${link?`<a class="pill sm" href="${link[0]}">${link[1]}</a>`:''}</div>
   ${st==='c'?`<p class="meta" style="margin:0">${engFoot(info.id)}</p>`:''}`;
  const b=document.getElementById('engOpen');if(b)b.onclick=()=>showEngine(info.id);
  const sl=el.querySelector('.eng-still');if(sl&&!K3.reduced()){sl.onpointermove=e=>{const r=sl.getBoundingClientRect();const x=(e.clientX-r.left)/r.width-.5,y=(e.clientY-r.top)/r.height-.5;sl.querySelector('img').style.transform=`perspective(700px) rotateY(${x*6}deg) rotateX(${-y*6}deg) scale(1.03)`};sl.onpointerleave=()=>{sl.querySelector('img').style.transform=''}}
}
function openCell(i){const info=cellInfo(i);if(info.st==='c')showEngine(info.id);else engReadout(i)}

/* 2D honeycomb: Still cells in the same spiral when WebGL is unavailable */
function mandala2d(host){const cells=mandalaCells(1);const k=38,W=k*6+84,H=k*7+76;
  host.innerHTML=`<div class="fallback"><div style="position:relative;width:${W}px;height:${H}px">${cells.map((c,i)=>{const info=cellInfo(i);const x=W/2-c.x*k-36,y=H/2-c.z*k-31;
    const inner=i===0?`<span style="display:grid;place-items:center;width:100%;height:100%;color:var(--gold);background:var(--surface)">${icon('witness',34)}</span>`:i===18?`<span style="display:grid;place-items:center;width:100%;height:100%;color:var(--gold)">${icon('question',30)}</span>`:`<img src="${stillSrc(i)}" alt="" style="filter:brightness(${info.st==='c'?.85:info.st==='u'?.45:.3})">`;
    return `<button class="hexcell" data-cell="${i}" style="position:absolute;left:${x}px;top:${y}px" aria-label="${esc(info.name)}, ${info.st==='c'?'computed':info.st==='u'?'locked':info.st==='w'?'witness':info.st==='q'?'the question':'not called'}">${inner}</button>`}).join('')}</div></div>`;
  host.querySelectorAll('[data-cell]').forEach(b=>b.onclick=()=>{MANDALA.sel=+b.dataset.cell;engReadout(MANDALA.sel)})}

function rEngines(){
  const c=ENGINES17.filter(e=>engState(e[0])==='c').length,u=ENGINES17.filter(e=>engState(e[0])==='u').length;
  $('#main').innerHTML=`<section class="room">
  <div class="hero"><div class="epithet">Engine pool · natal · ${istDate(ENG['human-design']&&ENG['human-design'].at)}</div><h1 class="title">Seventeen instruments. One chart.</h1>
   <p class="lede">${c} engines returned a reading for your birth data, ${u} are locked on this key, and ${17-c-u} have not been called. Tap a tray to lift it. Every figure on the right is read straight from its engine's response.</p></div>
  <div class="eng-layout"><div class="stage eng-stage" id="engHost">
    <div class="hud eng-hud"><div class="readout"><span class="k">Instrument</span><span class="v">Engine mandala · 19 cells</span></div><div class="readout" style="text-align:right"><span class="k">Spiral</span><span class="v">ENGINES.md order</span></div></div>
    <div class="hud eng-hud-b"><div style="display:flex;gap:6px;flex-wrap:wrap"><span class="state c">Computed ${c}</span><span class="state u">Locked ${u}</span><span class="state n">Not called ${17-c-u}</span></div><span class="k">Arrow keys walk · Enter opens</span></div>
   </div><div class="eng-read" id="engRead"></div></div>
  <div><h3 style="margin-bottom:12px">Index</h3><div class="eng-index">${[0,...ENGINES17.map((_,i)=>i+1),18].map(i=>{const info=cellInfo(i);return `<button data-cell="${i}" class="${info.st}" aria-pressed="${i===MANDALA.sel}">${icon(i===0?'witness':i===18?'question':info.id,20)}${esc(info.name)}</button>`}).join('')}</div></div>
  </section>`;
  const host=$('#engHost');
  if(K3.ok()){try{K3.mount('engines',host,h=>mandala3d(h))}catch(e){console.warn('mandala',e);K3.release();mandala2d(host)}}else mandala2d(host);
  engReadout(MANDALA.sel);
  $('#main').querySelectorAll('.eng-index button').forEach(b=>b.onclick=()=>{const i=+b.dataset.cell;const o=K3.current('engines');if(o)o.select(i);else{MANDALA.sel=i;engReadout(i)}$('#main').querySelectorAll('.eng-index button').forEach(x=>x.setAttribute('aria-pressed',x===b))});
}
