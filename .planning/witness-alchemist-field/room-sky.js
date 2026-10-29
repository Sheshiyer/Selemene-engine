/* ---------- Sky: the home room. Three.js night sky with a Western / Vedic toggle; 2D wheel under reduced motion or low power ---------- */
const WHEEL=[41,19,13,49,30,55,37,63,22,36,25,17,21,51,42,3,27,24,2,23,8,20,16,35,45,12,15,52,39,53,62,56,31,33,7,4,29,59,40,64,47,6,46,18,48,57,32,50,28,44,1,43,14,34,9,5,26,11,10,58,38,54,61,60];
const HEX={Sun:0xE6B84D,Moon:0xF0EDE3,Mercury:0x8A9BA8,Venus:0x7B68EE,Mars:0xC65D3B,Jupiter:0xC5A017,Saturn:0x75838E,Uranus:0x10B5A7,Neptune:0x4A7DFF,Pluto:0xEE977C,NorthNode:0x89A5FF,SouthNode:0x89A5FF,Earth:0x67D4C7};
const HEXS=Object.fromEntries(Object.entries(HEX).map(([k,v])=>[k,'#'+v.toString(16).padStart(6,'0')]));
const RADIUS={Sun:1.25,Moon:1.0,Jupiter:1.0,Saturn:0.9,Earth:0.7,Mercury:0.6,Venus:0.7,Mars:0.65,Uranus:0.7,Neptune:0.7,Pluto:0.55,NorthNode:0.5,SouthNode:0.5};
const ASPECTS=[['Conjunction',0,3,0xC5A017],['Opposition',180,3,0xC65D3B],['Square',90,2.5,0x7B68EE],['Trine',120,2.5,0x10B5A7]];
const NATAL_AYAN=(()=>{try{return ENG['human-design'].result.personality_activations.sun.longitude-ENG.panchanga.result.solar_longitude}catch(e){return 23.72}})();
function ayanOf(r){return ((r.p[0][3]-r.sid[1])%360+360)%360}
const norm=l=>((l%360)+360)%360;
function sidInfo(lon,ayan){const s=norm(lon-ayan),n=Math.floor(s/(360/27)),pada=Math.floor((s%(360/27))/(360/108))+1;return {lon:s,sign:RASHI[Math.floor(s/30)],deg:s%30,nak:NAKS[n],pada}}
const fmtDeg=d=>`${Math.floor(d)}°${String(Math.round((d%1)*60)).padStart(2,'0')}'`;
function angDiff(a,b){return Math.abs(((a-b+540)%360)-180)}
function aspectBetween(a,b){const d=angDiff(a,b);for(const [n,ang,orb,col] of ASPECTS){if(Math.abs(d-ang)<=orb)return {name:n,orb:Math.abs(d-ang),col}}return null}
const SKYPREF=(()=>{try{return Object.assign({mode:'western',gates:false,natal:true,lines:true},JSON.parse(localStorage.getItem('wa-sky')||'{}'))}catch(e){return {mode:'western',gates:false,natal:true,lines:true}}})();
function savePref(){try{localStorage.setItem('wa-sky',JSON.stringify(SKYPREF))}catch(e){}}
function reducedMotion(){try{return window.matchMedia&&window.matchMedia('(prefers-reduced-motion: reduce)').matches}catch(e){return false}}
function lowPower(){const n=navigator;return (n.hardwareConcurrency&&n.hardwareConcurrency<=2)||(n.deviceMemory&&n.deviceMemory<=2)||(n.connection&&n.connection.saveData)}
function webglOK(){try{const c=document.createElement('canvas');return !!(window.WebGLRenderingContext&&(c.getContext('webgl')||c.getContext('experimental-webgl')))}catch(e){return false}}

/* Bodies of the day and of birth, as one list the scene and the wheel share */
function skyBodies(r){
  const ayan=ayanOf(r);
  const t=tPos(r).filter(x=>x.body!=='SouthNode').map(x=>({kind:'transit',body:x.body,lon:x.lon,gate:x.gate,line:x.line,retro:x.retro,sid:sidInfo(x.lon,ayan)}));
  const n=NAT.filter(x=>x.body!=='SouthNode').map(x=>({kind:'natal',side:x.side,body:x.body,lon:x.lon,gate:x.gate,line:x.line,sid:sidInfo(x.lon,NATAL_AYAN)}));
  return {ayan,transit:t,natal:n};
}
function skyLines(B){
  const out=[];const T=B.transit.filter(b=>b.body!=='Earth');
  for(let i=0;i<T.length;i++)for(let j=i+1;j<T.length;j++){const a=aspectBetween(T[i].lon,T[j].lon);if(a)out.push({a:T[i],b:T[j],asp:a})}
  if(SKYPREF.natal){const N=B.natal.filter(b=>b.body!=='Earth'&&b.body!=='NorthNode');for(const t of T)for(const n of N){const a=aspectBetween(t.lon,n.lon);if(a&&a.orb<=2)out.push({a:t,b:n,asp:a})}}
  return out;
}
/* Drawer for a tapped body */
function showBody(b,r){
  const trop=fmtLon(b.lon),s=b.sid,G=GATES[b.gate];
  const title=b.kind==='transit'?`${SYM[b.body]} ${PNAME(b.body)} today`:`${SYM[b.body]} ${b.side==='P'?'Personality':'Design'} ${PNAME(b.body)}`;
  const B=skyBodies(r);
  const asp=(b.kind==='transit'?B.natal.filter(n=>n.body!=='Earth'):B.transit.filter(t=>t.body!=='Earth')).map(o=>({o,a:aspectBetween(b.lon,o.lon)})).filter(x=>x.a).sort((x,y)=>x.a.orb-y.a.orb);
  const con=b.kind==='transit'?r.con.filter(c=>c[0]===b.body):r.con.filter(c=>c[1]===b.side&&c[2]===b.body);
  openDrawer(`<div class="eyebrow">${b.kind==='transit'?fmtDate(r.d)+' · 06:00 IST':'Birth · 13 Aug 1991 13:31 IST'}</div><h2>${title}</h2>
  <dl class="kv"><dt>Gate</dt><dd><b>${b.gate}.${b.line}</b> ${G[0]}${b.retro?' · retrograde':''}${b.kind==='transit'&&NATG.has(b.gate)?' · <span class="ev-c">your gate</span>':''}</dd>
  <dt>Western</dt><dd>${trop}</dd><dt>Vedic</dt><dd>${s.sign} ${fmtDeg(s.deg)} · ${s.nak} pada ${s.pada}</dd><dt>Center</dt><dd>${G[1].replace('SolarPlexus','Solar Plexus')}</dd></dl>
  ${spectrum(b.gate)}
  ${con.length?`<h3>Exact contacts (within 1°)</h3><div class="list">${con.map(c=>{const n=NAT.find(x=>x.side===c[1]&&x.body===c[2]);return `<button class="item" data-gate="${n.gate}"><span class="k">${c[3]}°</span><span><b>${b.kind==='transit'?'On your '+(c[1]==='P'?'Personality':'Design')+' '+PNAME(c[2]):'Transit '+PNAME(c[0])+' on it'}</b><small>Gate ${n.gate}.${n.line} ${GATES[n.gate][0]}</small></span></button>`}).join('')}</div>`:''}
  ${asp.length?`<h3>Alignments</h3><div class="list">${asp.slice(0,8).map(x=>`<button class="item" data-gate="${x.o.gate}"><span class="k" style="color:#${x.a.col.toString(16).padStart(6,'0')}">${x.a.name}</span><span><b>${x.o.kind==='natal'?(x.o.side==='P'?'Personality ':'Design ')+PNAME(x.o.body):'Transit '+PNAME(x.o.body)} ${x.o.gate}.${x.o.line}</b><small>Orb ${x.a.orb.toFixed(1)}° · ${fmtLon(x.o.lon)}</small></span></button>`).join('')}</div>`:'<p class="muted">No alignment within orb today.</p>'}
  <p class="meta" style="margin-top:12px">Positions: Swiss Ephemeris, tropical. Vedic positions use the Lahiri ayanamsa (${(b.kind==='transit'?B.ayan:NATAL_AYAN).toFixed(2)}°).</p>`);
  wire($('#drawerBody'));
}

/* Labels as canvas sprites */
function makeLabel(text,color,size){const c=document.createElement('canvas');const S=size||48;c.width=512;c.height=128;const g=c.getContext('2d');g.font=`500 ${S}px "JetBrains Mono","SF Mono",monospace`;g.textAlign='center';g.textBaseline='middle';g.fillStyle=color;g.fillText(text,256,64);const tx=new THREE.CanvasTexture(c);tx.minFilter=THREE.LinearFilter;const m=new THREE.SpriteMaterial({map:tx,transparent:true,depthWrite:false});const sp=new THREE.Sprite(m);sp.scale.set(8,2,1);return sp}
function ringLine(R,y,color,op,seg){const pts=[];const n=seg||256;for(let i=0;i<=n;i++){const a=i/n*Math.PI*2;pts.push(new THREE.Vector3(R*Math.cos(a),y,-R*Math.sin(a)))}return new THREE.Line(new THREE.BufferGeometry().setFromPoints(pts),new THREE.LineBasicMaterial({color,transparent:true,opacity:op}))}
const P3=(lon,R,y)=>{const a=lon*Math.PI/180;return new THREE.Vector3(R*Math.cos(a),y||0,-R*Math.sin(a))};

let SCENE=null;
function sky3d(host,r){
  const B=skyBodies(r);
  const W=host.clientWidth||800,H=host.clientHeight||500;
  const renderer=new THREE.WebGLRenderer({antialias:true,alpha:false,powerPreference:'low-power'});
  renderer.setPixelRatio(Math.min(window.devicePixelRatio||1,2));renderer.setSize(W,H);renderer.setClearColor(0x070B1D,1);host.appendChild(renderer.domElement);
  const scene=new THREE.Scene();scene.fog=new THREE.Fog(0x070B1D,120,260);
  const cam=new THREE.PerspectiveCamera(42,W/H,0.1,600);
  const view={yaw:Math.PI/2,pitch:0.62,dist:92,auto:true};
  function place(){cam.position.set(view.dist*Math.cos(view.pitch)*Math.cos(view.yaw),view.dist*Math.sin(view.pitch),view.dist*Math.cos(view.pitch)*Math.sin(view.yaw));cam.lookAt(0,0,0)}
  scene.add(new THREE.AmbientLight(0xffffff,0.55));const key=new THREE.PointLight(0xE6B84D,0.9,400);key.position.set(60,80,40);scene.add(key);
  // starfield
  const N=2600,pos=new Float32Array(N*3),col=new Float32Array(N*3);const c1=new THREE.Color(0xF0EDE3),c2=new THREE.Color(0xC5A017),c3=new THREE.Color(0x8A9BA8);
  for(let i=0;i<N;i++){const u=Math.random()*2-1,th=Math.random()*Math.PI*2,s=Math.sqrt(1-u*u),R=190+Math.random()*40;pos[i*3]=R*s*Math.cos(th);pos[i*3+1]=R*u;pos[i*3+2]=R*s*Math.sin(th);const c=Math.random()<.08?c2:Math.random()<.5?c1:c3;const k=.35+Math.random()*.65;col[i*3]=c.r*k;col[i*3+1]=c.g*k;col[i*3+2]=c.b*k}
  const sg=new THREE.BufferGeometry();sg.setAttribute('position',new THREE.BufferAttribute(pos,3));sg.setAttribute('color',new THREE.BufferAttribute(col,3));
  const sm=new THREE.PointsMaterial({size:1.6,vertexColors:true,transparent:true,opacity:0,sizeAttenuation:true,depthWrite:false});scene.add(new THREE.Points(sg,sm));
  // rings
  const RT=30,RN=22,RZ=36,RG=26;
  scene.add(ringLine(RT,0,0xC5A017,.35),ringLine(RN,0,0x7B68EE,.3),ringLine(RZ,0,0xC5A017,.5),ringLine(RZ+3.5,0,0xC5A017,.18));
  const zod=new THREE.Group(),gates=new THREE.Group();scene.add(zod,gates);
  function buildZodiac(){while(zod.children.length)zod.remove(zod.children[0]);
    const ayan=SKYPREF.mode==='vedic'?B.ayan:0;const names=SKYPREF.mode==='vedic'?RASHI:SIGNS;const mat=new THREE.LineBasicMaterial({color:0xC5A017,transparent:true,opacity:.45});
    for(let i=0;i<12;i++){const l=i*30+ayan;zod.add(new THREE.Line(new THREE.BufferGeometry().setFromPoints([P3(l,RZ),P3(l,RZ+3.5)]),mat));const sp=makeLabel(names[i].toUpperCase(),'#C5A017',40);sp.position.copy(P3(l+15,RZ+7.5,0.4));sp.scale.set(12,3,1);zod.add(sp)}
    if(SKYPREF.mode==='vedic'){const nm=new THREE.LineBasicMaterial({color:0x8A9BA8,transparent:true,opacity:.4});for(let i=0;i<27;i++){const l=i*(360/27)+ayan;zod.add(new THREE.Line(new THREE.BufferGeometry().setFromPoints([P3(l,RZ-1.6),P3(l,RZ)]),nm));const sp=makeLabel(NAKS[i],'#AEBBC4',30);sp.position.copy(P3(l+360/54,RZ-4.2,-0.2));sp.scale.set(9,2.2,1);zod.add(sp)}}
  }
  function buildGates(){while(gates.children.length)gates.remove(gates.children[0]);if(!SKYPREF.gates)return;const m=new THREE.LineBasicMaterial({color:0x8A9BA8,transparent:true,opacity:.35});
    for(let i=0;i<64;i++){const l=302+i*5.625;gates.add(new THREE.Line(new THREE.BufferGeometry().setFromPoints([P3(l,RG-.8),P3(l,RG+.8)]),m));const g=WHEEL[i];const sp=makeLabel(String(g),NATG.has(g)?'#E6B84D':'#75838E',38);sp.position.copy(P3(l+2.8,RG,-0.6));sp.scale.set(5,1.25,1);gates.add(sp)}}
  // bodies
  const picks=[];const bodies=new THREE.Group();scene.add(bodies);
  for(const b of B.transit){const m=new THREE.Mesh(new THREE.SphereGeometry(RADIUS[b.body]||.6,20,16),new THREE.MeshStandardMaterial({color:HEX[b.body],emissive:HEX[b.body],emissiveIntensity:.35,roughness:.6}));m.position.copy(P3(b.lon,RT,0));m.userData=b;bodies.add(m);picks.push(m);
    const lab=makeLabel(`${SYM[b.body]} ${PNAME(b.body)}${b.retro?' R':''} ${b.gate}.${b.line}`,HEXS[b.body],40);lab.position.copy(P3(b.lon,RT,2.6));lab.scale.set(11,2.75,1);bodies.add(lab);
    const stem=new THREE.Line(new THREE.BufferGeometry().setFromPoints([P3(b.lon,RT,0),P3(b.lon,RZ,0)]),new THREE.LineBasicMaterial({color:HEX[b.body],transparent:true,opacity:.2}));bodies.add(stem)}
  const natalG=new THREE.Group();scene.add(natalG);
  for(const b of B.natal){const col=b.side==='P'?0xC5A017:0x7B68EE;const m=new THREE.Mesh(new THREE.OctahedronGeometry(.55),new THREE.MeshBasicMaterial({color:col}));m.position.copy(P3(b.lon,RN,0));m.userData=b;natalG.add(m);picks.push(m)}
  const linesG=new THREE.Group();scene.add(linesG);
  function buildLines(){while(linesG.children.length)linesG.remove(linesG.children[0]);natalG.visible=SKYPREF.natal;if(!SKYPREF.lines)return;
    for(const L of skyLines(B)){const pa=P3(L.a.lon,L.a.kind==='transit'?RT:RN),pb=P3(L.b.lon,L.b.kind==='transit'?RT:RN);linesG.add(new THREE.Line(new THREE.BufferGeometry().setFromPoints([pa,pb]),new THREE.LineBasicMaterial({color:L.asp.col,transparent:true,opacity:L.b.kind==='natal'?.45:.7})))}}
  buildZodiac();buildGates();buildLines();
  // interaction
  const ray=new THREE.Raycaster();ray.params.Points={threshold:0};const ptr=new THREE.Vector2();let down=null,moved=false;
  const el=renderer.domElement;
  el.addEventListener('pointerdown',e=>{down={x:e.clientX,y:e.clientY,yaw:view.yaw,pitch:view.pitch};moved=false;view.auto=false;el.setPointerCapture&&el.setPointerCapture(e.pointerId)});
  el.addEventListener('pointermove',e=>{if(!down)return;const dx=e.clientX-down.x,dy=e.clientY-down.y;if(Math.abs(dx)+Math.abs(dy)>4)moved=true;view.yaw=down.yaw-dx*.006;view.pitch=Math.max(.12,Math.min(1.35,down.pitch+dy*.005));place()});
  el.addEventListener('pointerup',e=>{if(down&&!moved){const rc=el.getBoundingClientRect();ptr.x=((e.clientX-rc.left)/rc.width)*2-1;ptr.y=-((e.clientY-rc.top)/rc.height)*2+1;ray.setFromCamera(ptr,cam);const hit=ray.intersectObjects(picks,false)[0];if(hit)showBody(hit.object.userData,r)}down=null});
  el.addEventListener('wheel',e=>{e.preventDefault();view.dist=Math.max(40,Math.min(160,view.dist*(1+Math.sign(e.deltaY)*.08)));place()},{passive:false});
  // loop, capped near 30fps, paused when hidden
  let raf=0,last=0,t0=performance.now(),alive=true;
  function frame(now){if(!alive)return;raf=requestAnimationFrame(frame);if(now-last<33)return;last=now;const k=Math.min(1,(now-t0)/1000);sm.opacity=1-Math.pow(1-k,4);if(view.auto)view.yaw+=.0012;place();renderer.render(scene,cam)}
  place();raf=requestAnimationFrame(frame);
  const onVis=()=>{if(document.hidden){cancelAnimationFrame(raf);alive=false}else if(!alive){alive=true;raf=requestAnimationFrame(frame)}};document.addEventListener('visibilitychange',onVis);
  const ro=new ResizeObserver(()=>{const w=host.clientWidth,h=host.clientHeight;if(!w||!h)return;renderer.setSize(w,h);cam.aspect=w/h;cam.updateProjectionMatrix()});ro.observe(host);
  SCENE={dispose(){alive=false;cancelAnimationFrame(raf);document.removeEventListener('visibilitychange',onVis);ro.disconnect();scene.traverse(o=>{if(o.geometry)o.geometry.dispose();if(o.material){if(o.material.map)o.material.map.dispose();o.material.dispose()}});renderer.dispose();if(el.parentNode)el.parentNode.removeChild(el)},refresh(){buildZodiac();buildGates();buildLines()}};
}
window.skyDispose=()=>{if(SCENE){SCENE.dispose();SCENE=null}};

/* 2D wheel: the same rings and bodies as SVG */
function sky2d(host,r){
  const B=skyBodies(r),C=400,RZ=372,RT=300,RN=220,RG=262;const ayan=SKYPREF.mode==='vedic'?B.ayan:0;const names=SKYPREF.mode==='vedic'?RASHI:SIGNS;
  const X=(l,R)=>C+R*Math.cos(l*Math.PI/180),Y=(l,R)=>C-R*Math.sin(l*Math.PI/180);
  let s=`<svg class="wheel" viewBox="0 0 800 800" role="img" aria-label="Sky wheel for ${fmtDate(r.d)}">`;
  for(let i=0;i<180;i++){const a=Math.random()*360,R=385+Math.random()*15;s+=`<circle cx="${X(a,R).toFixed(1)}" cy="${Y(a,R).toFixed(1)}" r="${(Math.random()*1.2+.3).toFixed(1)}" fill="var(--silver)" opacity="${(Math.random()*.6+.2).toFixed(2)}"/>`}
  s+=`<circle cx="${C}" cy="${C}" r="${RZ}" fill="none" stroke="var(--gold)" stroke-opacity=".5"/><circle cx="${C}" cy="${C}" r="${RZ-32}" fill="none" stroke="var(--gold)" stroke-opacity=".2"/><circle cx="${C}" cy="${C}" r="${RT}" fill="none" stroke="var(--gold)" stroke-opacity=".35"/><circle cx="${C}" cy="${C}" r="${RN}" fill="none" stroke="var(--violet-glow)" stroke-opacity=".35"/>`;
  for(let i=0;i<12;i++){const l=i*30+ayan;s+=`<line x1="${X(l,RZ-32)}" y1="${Y(l,RZ-32)}" x2="${X(l,RZ)}" y2="${Y(l,RZ)}" stroke="var(--gold)" stroke-opacity=".5"/><text x="${X(l+15,RZ-16)}" y="${Y(l+15,RZ-16)}" font-size="9" text-anchor="middle" dominant-baseline="middle" fill="var(--gold)">${names[i].slice(0,3).toUpperCase()}</text>`}
  if(SKYPREF.mode==='vedic')for(let i=0;i<27;i++){const l=i*(360/27)+ayan;s+=`<line x1="${X(l,RZ-32)}" y1="${Y(l,RZ-32)}" x2="${X(l,RZ-40)}" y2="${Y(l,RZ-40)}" stroke="var(--silver)" stroke-opacity=".5"/><text x="${X(l+360/54,RZ-52)}" y="${Y(l+360/54,RZ-52)}" font-size="7" text-anchor="middle" dominant-baseline="middle">${NAKS[i].slice(0,5)}</text>`}
  if(SKYPREF.gates)for(let i=0;i<64;i++){const l=302+i*5.625,g=WHEEL[i];s+=`<line x1="${X(l,RG-5)}" y1="${Y(l,RG-5)}" x2="${X(l,RG+5)}" y2="${Y(l,RG+5)}" stroke="var(--silver)" stroke-opacity=".4"/><text x="${X(l+2.8,RG+14)}" y="${Y(l+2.8,RG+14)}" font-size="7" text-anchor="middle" dominant-baseline="middle" fill="${NATG.has(g)?'var(--gold-warm)':'var(--text-dis)'}">${g}</text>`}
  if(SKYPREF.lines)for(const L of skyLines(B)){const ra=L.a.kind==='transit'?RT:RN,rb=L.b.kind==='transit'?RT:RN;s+=`<line x1="${X(L.a.lon,ra)}" y1="${Y(L.a.lon,ra)}" x2="${X(L.b.lon,rb)}" y2="${Y(L.b.lon,rb)}" stroke="#${L.asp.col.toString(16).padStart(6,'0')}" stroke-opacity="${L.b.kind==='natal'?.45:.75}" stroke-width="1.2"/>`}
  if(SKYPREF.natal)B.natal.forEach((b,i)=>{s+=`<rect class="pl" data-n="${i}" x="${X(b.lon,RN)-4}" y="${Y(b.lon,RN)-4}" width="8" height="8" transform="rotate(45 ${X(b.lon,RN)} ${Y(b.lon,RN)})" fill="${b.side==='P'?'var(--gold)':'var(--violet-glow)'}"><title>${b.side==='P'?'Personality':'Design'} ${PNAME(b.body)} ${b.gate}.${b.line}</title></rect>`});
  B.transit.forEach((b,i)=>{const rr=(RADIUS[b.body]||.6)*6;s+=`<line x1="${X(b.lon,RT)}" y1="${Y(b.lon,RT)}" x2="${X(b.lon,RZ-32)}" y2="${Y(b.lon,RZ-32)}" stroke="${HEXS[b.body]}" stroke-opacity=".25"/><circle class="pl" data-t="${i}" cx="${X(b.lon,RT)}" cy="${Y(b.lon,RT)}" r="${rr}" fill="${HEXS[b.body]}"><title>${PNAME(b.body)} ${b.gate}.${b.line}</title></circle><text x="${X(b.lon,RT+22)}" y="${Y(b.lon,RT+22)}" font-size="9" text-anchor="middle" dominant-baseline="middle" fill="${HEXS[b.body]}">${SYM[b.body]} ${b.gate}.${b.line}</text>`});
  s+=`<text x="${C}" y="${C-6}" font-size="11" text-anchor="middle" fill="var(--gold)" letter-spacing="3">${SKYPREF.mode==='vedic'?'LAHIRI':'TROPICAL'}</text><text x="${C}" y="${C+10}" font-size="9" text-anchor="middle">${fmtDate(r.d)}</text></svg>`;
  host.innerHTML=s;
  host.querySelectorAll('.pl').forEach(el=>el.addEventListener('click',()=>showBody(el.dataset.t!==undefined?B.transit[+el.dataset.t]:B.natal[+el.dataset.n],r)));
}

function rSky(){
  const r=rec(state.date),R=dayReading(r),isToday=state.date===todayIST(),B=skyBodies(r);
  const use3d=!reducedMotion()&&!lowPower()&&webglOK()&&typeof THREE!=='undefined';
  const moon=B.transit.find(b=>b.body==='Moon'),sun=B.transit.find(b=>b.body==='Sun');
  const nL=skyLines(B).length;
  $('#main').innerHTML=`<section class="room">
  <div class="sky-wrap"><div class="sky-layer" id="skyHost"></div>
   <div class="sky-hud"><div class="seg" role="group" aria-label="Zodiac"><button id="skyWest" aria-pressed="${SKYPREF.mode!=='vedic'}">Western</button><button id="skyVedic" aria-pressed="${SKYPREF.mode==='vedic'}">Vedic</button></div>
    <button class="chip" id="skyGates" aria-pressed="${SKYPREF.gates}">64 gates</button><button class="chip" id="skyNatal" aria-pressed="${SKYPREF.natal}">Birth ring</button><button class="chip" id="skyLines" aria-pressed="${SKYPREF.lines}">Alignments</button></div>
   <div class="sky-foot"><span>${isToday?'THE SKY NOW':'WITNESSING'} · ${fmtDate(r.d).toUpperCase()} · 06:00 IST${use3d?' · DRAG TO TURN · TAP A BODY':' · TAP A BODY'}</span><div class="legend"><span><i style="background:var(--gold)"></i>Conjunction</span><span><i style="background:var(--terracotta)"></i>Opposition</span><span><i style="background:var(--violet-glow)"></i>Square</span><span><i style="background:var(--emerald)"></i>Trine</span></div></div>
  </div>
  <div class="sky-decl"><div class="epithet">One field. Eight rooms.</div><p class="decl">${esc(R.decl)}</p>
   <p class="lede">Sun ${sun.gate}.${sun.line}, ${SKYPREF.mode==='vedic'?sun.sid.sign+' '+fmtDeg(sun.sid.deg):fmtLon(sun.lon)}. Moon in ${moon.sid.nak} pada ${moon.sid.pada}. ${nL?nL+' alignment'+(nL>1?'s':'')+' within orb.':'No alignment within orb.'} ${r.def===1?'Single definition today.':'Split definition today.'}</p>
   <div class="chips"><a class="pill sm" href="#today">Open the day</a><a class="pill sm" href="#engines">Engines</a><a class="pill sm" href="#folio">Folio</a></div></div>
  <div><div class="divider"><span></span></div><div class="rooms-grid">${ROOMS.filter(x=>x[0]!=='sky').map(([id,l])=>`<a class="room-card" href="#${id}"><b>${l}</b><small>${ROOM_TXT[id]}</small></a>`).join('')}</div></div>
  <p class="foot">See the pattern. Keep the authority.</p></section>`;
  const host=$('#skyHost');
  const draw=()=>{if(use3d){try{sky3d(host,r)}catch(e){window.skyDispose();host.innerHTML='';sky2d(host,r)}}else sky2d(host,r)};
  draw();
  const set=(k,v)=>{SKYPREF[k]=v;savePref();window.skyDispose();rSky()};
  $('#skyWest').onclick=()=>set('mode','western');$('#skyVedic').onclick=()=>set('mode','vedic');
  $('#skyGates').onclick=()=>set('gates',!SKYPREF.gates);$('#skyNatal').onclick=()=>set('natal',!SKYPREF.natal);$('#skyLines').onclick=()=>set('lines',!SKYPREF.lines);
}
