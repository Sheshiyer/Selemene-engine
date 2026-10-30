/* ---------- Sky: the home room. A bronze orrery in the world of the Transits instrument ----------
   Geocentric: the emerald lens at the hub is you. Today's bodies ride emerald glass beads on bronze arms;
   the zodiac band turns by the ayanamsa between Western and Vedic; birth studs sit on the inner band. */
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
const SKYPREF=(()=>{const d={mode:'western',gates:false,natal:true,lines:true,view:'orbit'};try{return Object.assign(d,JSON.parse(localStorage.getItem('wa-sky')||'{}'))}catch(e){return d}})();
function savePref(){try{localStorage.setItem('wa-sky',JSON.stringify(SKYPREF))}catch(e){}}

function skyBodies(r){
  const ayan=ayanOf(r);
  const t=tPos(r).filter(x=>x.body!=='SouthNode').map(x=>({kind:'transit',body:x.body,lon:x.lon,gate:x.gate,line:x.line,retro:x.retro,sid:sidInfo(x.lon,ayan)}));
  const n=NAT.filter(x=>x.body!=='SouthNode').map(x=>({kind:'natal',side:x.side,body:x.body,lon:x.lon,gate:x.gate,line:x.line,sid:sidInfo(x.lon,NATAL_AYAN)}));
  return {ayan,transit:t,natal:n};
}
function skyLines(B){
  const out=[];const T=B.transit.filter(b=>b.body!=='Earth');
  for(let i=0;i<T.length;i++)for(let j=i+1;j<T.length;j++){const a=aspectBetween(T[i].lon,T[j].lon);if(a)out.push({a:T[i],b:T[j],asp:a})}
  if(SKYPREF.natal){const N=B.natal.filter(b=>b.body!=='Earth'&&b.body!=='NorthNode');for(const t of T)for(const n of N){const a=aspectBetween(t.lon,n.lon);if(a&&a.orb<=(SKYPREF.tight?1:2))out.push({a:t,b:n,asp:a})}}
  return out;
}
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
function showLens(r){const R=dayReading(r);
  openDrawer(`<div class="eyebrow">The lens · ${fmtDate(r.d)} · 06:00 IST</div><h2>${esc(R.decl)}</h2><p class="muted">${esc(R.trav)}</p>
  <dl class="kv"><dt>Sun</dt><dd><b>${R.sun.gate}.${R.sun.line}</b> ${GATES[R.sun.gate][0]}</dd><dt>Earth</dt><dd><b>${R.earth.gate}.${R.earth.line}</b> ${GATES[R.earth.gate][0]}</dd><dt>Definition</dt><dd>${defLabel(r.def)}${r.m2t?' · motor to Throat':''}</dd><dt>Centers</dt><dd>${r.cen.length} of 9 defined</dd></dl>
  <h3>Channels</h3><div class="list">${r.ch.map(([k,t])=>`<button class="item" data-ch="${k}"><span class="tag ${t==='natal'?'natal':t==='bridge'?'c':'w'}">${t==='natal'?'yours':t==='bridge'?'bridged':'transit'}</span><span><b>${k} ${CHANNELS[k][0]}</b><small>${CHANNELS[k][1]} circuit</small></span></button>`).join('')}</div>
  <p style="margin-top:16px"><a class="pill sm" href="#today">Open the day</a></p>`);wire($('#drawerBody'))}

/* ---------- the orrery ---------- */
const ORB_R={Moon:9,Mercury:11.6,Venus:14.2,Sun:16.8,Mars:19.2,Jupiter:21.6,Saturn:23.8,Uranus:25.4,Neptune:26.6,Pluto:27.8,NorthNode:29.4};
const BEAD_R={Sun:1.45,Moon:1.2,Jupiter:1.2,Saturn:1.1,Mars:.9,Venus:.95,Mercury:.8,Uranus:.85,Neptune:.85,Pluto:.7,NorthNode:.7};
const ORB_ORDER=['Moon','Mercury','Venus','Sun','Mars','Jupiter','Saturn','Uranus','Neptune','Pluto','NorthNode'];
const RZ0=34,RZ1=39.5,RN0=30.6,RN1=33.4,RG0=40.4,RG1=43.6;
function fitOrbit(S){const c=S.camera,v=c.fov*Math.PI/360,hh=Math.atan(Math.tan(v)*c.aspect),R=SKYPREF.gates?44.5:40.5;return Math.max(R*1.04/Math.sin(hh),(R*Math.sin(.62)+9)/Math.sin(v)*1.06,70)}
function fitAbove(S){const c=S.camera,v=c.fov*Math.PI/360,hh=Math.atan(Math.tan(v)*c.aspect),R=SKYPREF.gates?44.5:40.5;return Math.max(R*1.03/Math.sin(Math.min(v,hh)),80)}
function orrery(host,r0){
  const K=K3,T=THREE;let r=r0;const RM=K.reduced();
  let fitO=104,fitA=118,S=null,introOn=false;
  S=K.stage(host,{fov:32,yaw:-Math.PI/2,target:[0,SKYPREF.view==='above'?0:-7,0],pitch:SKYPREF.view==='above'?1.42:.62,dist:104,minDist:56,maxDist:260,
    onResize:()=>{if(!S)return;fitO=fitOrbit(S);fitA=fitAbove(S);if(!introOn){S.view.dist=SKYPREF.view==='above'?fitA:fitO;S.place()}},minPitch:.12,maxPitch:1.48,label:'Orrery of the sky over your design. Drag to turn, tap a bead.',
    ambient:(dt)=>{if(SKYPREF.view==='above')return false;S.view.yaw+=dt*.018;return true},
    onPick:o=>{if(!o)return;const u=o.userData;if(u.lens)showLens(r);else if(u.arm)showBody(u.body,r);else if(u.body)showBody(u.body,r);select(o)},
    onHover:o=>hover(o)});
  const {scene,M}=S;
  // starfield breathes in once, then twinkles
  const stars=K.starfield(1600,220,420,23);scene.add(stars);
  const root=new T.Group();scene.add(root);
  // plinth: hex prism, gold edge, engraved inset, halo beneath
  const plinth=new T.Mesh(K.prism(K.hexShape(22,Math.PI/6),2.6,.35),M.bronze());plinth.position.y=-11.6;root.add(plinth);
  const pTop=-11.6+2.6+.35;
  const e1=new T.LineLoop(K.loopXZ(K.hexPts(22.2,Math.PI/6),0),M.line(K.HEX.gold,.85));e1.position.y=pTop+.02;root.add(e1);
  const e2=new T.LineLoop(K.loopXZ(K.hexPts(20.2,Math.PI/6),0),M.line(K.HEX.gold,.35));e2.position.y=pTop+.02;root.add(e2);
  for(let i=0;i<6;i++){const a=(i*60+30)*Math.PI/180;const st=new T.Mesh(new T.CylinderGeometry(.42,.42,.18,16),M.gold());st.position.set(19.6*Math.cos(a),pTop+.08,-19.6*Math.sin(a));root.add(st)}
  const hal=K.halo(96,.75);hal.position.y=-11.9;root.add(hal);
  // turned column
  const prof=[[0,0],[3.6,0],[3.6,.5],[2.6,.9],[2.2,1.6],[1.5,2.4],[1.25,6],[1.6,6.7],[1.25,7.3],[1.25,9.6],[2.4,10.4],[3.6,10.8],[3.6,11.3],[0,11.3]].map(([x,y])=>new T.Vector2(x,y));
  const colm=new T.Mesh(new T.LatheGeometry(prof,48),M.bronze());colm.position.y=pTop;root.add(colm);
  const hubY=pTop+11.3;
  // hub: gold bezel + emerald lens (you)
  const spH=ORB_ORDER.length*.34+1.6;
  const spindle=new T.Mesh(new T.CylinderGeometry(.34,.42,spH,20),M.gold({roughness:.35}));spindle.position.y=hubY+spH/2;root.add(spindle);
  const capY=hubY+spH;
  const bez=new T.Mesh(new T.TorusGeometry(2.0,.26,16,64),M.gold());bez.rotation.x=Math.PI/2;bez.position.y=capY+.25;root.add(bez);
  const cupL=new T.Mesh(new T.SphereGeometry(2.05,32,16,0,Math.PI*2,Math.PI/2,Math.PI/2),M.bronze());cupL.position.y=capY+.45;root.add(cupL);
  const lens=new T.Mesh(new T.SphereGeometry(1.85,48,32),M.glass(K.HEX.emerald,{emissiveIntensity:.32}));lens.position.y=capY+1.25;lens.userData={lens:true};S.pick(lens);root.add(lens);
  const fin=new T.Mesh(new T.ConeGeometry(.22,1.6,12),M.gold());fin.position.y=capY+3.6;root.add(fin);
  const lensGlow=K.glow(K.HEX.emerald,8,.38);lensGlow.position.copy(lens.position);root.add(lensGlow);
  const Y0=hubY+.35;
  // zodiac band and inner band
  const zband=new T.Mesh(K.band(RZ0,RZ1,.9),M.bronze());zband.position.y=Y0-.9;root.add(zband);
  const nband=new T.Mesh(K.band(RN0,RN1,.7),M.bronzeDark());nband.position.y=Y0-.7;root.add(nband);
  [RZ0-.05,RZ1+.05,RN0-.05,RN1+.05].forEach((rr,i)=>root.add(new T.LineLoop(K.circleXZ(rr,Y0+.12,256),M.line(K.HEX.gold,i<2?.75:.4))));
  {const pts=[];for(let d=0;d<360;d+=1){const L=d%5?.55:1.1;pts.push(K.P(d,RZ1-L,Y0+.13),K.P(d,RZ1-.05,Y0+.13))}root.add(new T.LineSegments(new T.BufferGeometry().setFromPoints(pts),M.line(K.HEX.gold,.28)))}
  // the turning zodiac
  const zod=new T.Group();root.add(zod);
  let zodRot=SKYPREF.mode==='vedic'?ayanOf(r):0;
  const clear=g=>{while(g.children.length){const c=g.children.pop();if(c.geometry)c.geometry.dispose();if(c.material){if(c.material.map)c.material.map.dispose();c.material.dispose()}}};
  function buildZod(){clear(zod);const ved=SKYPREF.mode==='vedic',names=ved?RASHI:SIGNS;const segs=[];
    for(let i=0;i<12;i++){segs.push(K.P(i*30,RZ0,Y0+.14),K.P(i*30,RZ1,Y0+.14))}
    zod.add(new T.LineSegments(new T.BufferGeometry().setFromPoints(segs),M.line(K.HEX.gold,.9)));
    for(let i=0;i<12;i++){const l=S.track(K.label(names[i].toUpperCase(),{font:'engr',px:11,track:.22,color:'#C5A017',weight:600}));l.position.copy(K.P(i*30+15,(RZ0+RZ1)/2,Y0+.9));zod.add(l)}
    if(ved){const ns=[];for(let i=0;i<27;i++){ns.push(K.P(i*360/27,RN0,Y0+.02),K.P(i*360/27,RN1,Y0+.02))}zod.add(new T.LineSegments(new T.BufferGeometry().setFromPoints(ns),M.line(K.HEX.silver,.55)));
      for(let i=0;i<27;i++){const l=S.track(K.label(NAKS[i],{font:'mono',px:9,color:'#AEBBC4',opacity:.85}));l.position.copy(K.P(i*360/27+360/54,(RN0+RN1)/2,Y0+.5));zod.add(l)}}
    else{const dc=[];for(let i=0;i<36;i++){dc.push(K.P(i*10,RN1-.7,Y0+.02),K.P(i*10,RN1,Y0+.02))}zod.add(new T.LineSegments(new T.BufferGeometry().setFromPoints(dc),M.line(K.HEX.silver,.35)))}
    zod.rotation.y=zodRot*Math.PI/180;S.invalidate()}
  buildZod();
  // 64-gate outer ring
  const gring=new T.Group();root.add(gring);
  function buildGates(){clear(gring);if(!SKYPREF.gates)return;const b=new T.Mesh(K.band(RG0,RG1,.5),M.bronzeDark());b.position.y=Y0-.6;gring.add(b);
    const segs=[];for(let i=0;i<64;i++){const l=302+i*5.625;segs.push(K.P(l,RG0,Y0-.04),K.P(l,RG1,Y0-.04))}gring.add(new T.LineSegments(new T.BufferGeometry().setFromPoints(segs),M.line(K.HEX.gold,.45)));
    for(let i=0;i<64;i++){const g=WHEEL[i];const l=S.track(K.label(String(g),{font:'mono',px:9,color:NATG.has(g)?'#E6B84D':'#75838E'}));l.position.copy(K.P(302+i*5.625+2.8,(RG0+RG1)/2,Y0+.3));gring.add(l)}}
  buildGates();
  // birth studs: Personality gold, Design violet glass
  const studs=new T.Group();root.add(studs);
  skyBodies(r).natal.filter(n=>n.body!=='Earth').forEach(n=>{const m=new T.Mesh(new T.OctahedronGeometry(.62,0),n.side==='P'?M.gold({emissiveIntensity:.25}):M.glass(K.HEX.violet,{emissiveIntensity:.5}));m.scale.set(1,.7,1);
    m.position.copy(K.P(n.lon,n.side==='P'?RN1-.75:RN0+.75,Y0+.3));m.userData={body:n};S.pick(m);studs.add(m)});
  studs.visible=SKYPREF.natal;
  // arms and beads
  const arms={};
  ORB_ORDER.forEach((name,i)=>{const R=ORB_R[name],y=hubY+.8+i*.34;const g=new T.Group();g.position.y=y;root.add(g);
    const len=name==='NorthNode'?R*2:R-.6;const rod=new T.Mesh(new T.CylinderGeometry(.12,.12,len,10),M.gold({roughness:.48,color:K.col(0x8a6a26),emissiveIntensity:.02}));rod.rotation.z=Math.PI/2;rod.position.x=name==='NorthNode'?0:.6+len/2;g.add(rod);
    const collar=new T.Mesh(new T.TorusGeometry(.62,.1,8,24),M.gold());collar.rotation.x=Math.PI/2;g.add(collar);
    const cup=new T.Mesh(new T.TorusGeometry(BEAD_R[name]*.82,.08,8,32),M.gold());cup.rotation.x=Math.PI/2;cup.position.set(R,-BEAD_R[name]*.55,0);g.add(cup);
    const bead=new T.Mesh(new T.SphereGeometry(BEAD_R[name],32,24),M.glass(K.HEX.emerald));bead.position.set(R,0,0);g.add(bead);
    const gl=K.glow(K.HEX.emerald,BEAD_R[name]*5,.32);gl.position.set(R,0,0);g.add(gl);
    if(name==='Sun'){const cor=new T.Mesh(new T.TorusGeometry(BEAD_R.Sun*1.5,.05,6,64),M.gold({emissiveIntensity:.6}));cor.position.set(R,0,0);g.add(cor)}
    let tail=null;if(name==='NorthNode'){tail=new T.Mesh(new T.SphereGeometry(BEAD_R.NorthNode*.85,24,16),M.glass(K.HEX.emerald,{emissiveIntensity:.25}));tail.position.set(-R,0,0);g.add(tail)}
    const lab=S.track(K.label(' ',{px:11}));lab.center.set(.5,-.35);lab.position.set(R,BEAD_R[name]+.4,0);g.add(lab);
    arms[name]={g,bead,lab,glow:gl,R,lon:0,tail,text:'',b:null};bead.userData={arm:name};S.pick(bead);if(tail){tail.userData={arm:name,tail:true};S.pick(tail)}});
  function armLabel(a,b,sel){const t=`${SYM[b.body]} ${b.gate}.${b.line}${b.retro?' ℞':''}`;const key=t+(sel?'*':'');if(a.text===key)return;a.text=key;const old=a.lab.material.map;
    const nl=K.label(t,{px:11,color:sel?'#E6B84D':'#F0EDE3',bg:'rgba(7,11,29,.62)'});a.lab.material.map=nl.material.map;a.lab.userData.label=nl.userData.label;S.track(a.lab);if(old)old.dispose();nl.material.dispose()}
  // alignment arcs
  const lines=new T.Group();root.add(lines);
  function posOf(b){if(b.kind==='natal')return K.P(b.lon,b.side==='P'?RN1-.75:RN0+.75,Y0+.3);const a=arms[b.body];if(!a)return null;const p=K.P(b.lon,a.R,0);p.y=a.g.position.y;return p}
  let cur=skyBodies(r),sel=null,armTween=null;
  function buildLines(){clear(lines);if(!SKYPREF.lines)return;
    SKYPREF.tight=true;const LL=skyLines(cur);SKYPREF.tight=false;for(const L of LL){const a=posOf(L.a),b=posOf(L.b);if(!a||!b)continue;const mid=a.clone().add(b).multiplyScalar(.5);mid.y+=2.5+a.distanceTo(b)*.06;
      const crv=new T.QuadraticBezierCurve3(a,mid,b);const ln=new T.Line(new T.BufferGeometry().setFromPoints(crv.getPoints(40)),M.line(L.asp.col,L.b.kind==='natal'?.4:.7));ln.renderOrder=4;lines.add(ln)}S.invalidate()}
  function applyDay(rr,ms){r=rr;cur=skyBodies(rr);const from={},to={};
    ORB_ORDER.forEach(n=>{const b=cur.transit.find(x=>x.body===n);const a=arms[n];from[n]=a.lon;let d=b.lon-a.lon;d=((d+540)%360)-180;to[n]=a.lon+d;a.b=b;a.bead.userData.body=b;armLabel(a,b,sel===a.bead);
      if(a.tail){a.tail.userData.body=Object.assign({},b,{body:'SouthNode',lon:norm(b.lon+180),gate:rr.p[12][0],line:rr.p[12][1],sid:sidInfo(norm(b.lon+180),cur.ayan)})}});
    studs.children.forEach(m=>{const u=m.userData.body;m.userData.body=cur.natal.find(x=>x.side===u.side&&x.body===u.body)||u});
    lines.visible=false;if(armTween)armTween.stop();
    armTween=S.tween(ms,k=>{ORB_ORDER.forEach(n=>{const a=arms[n];a.lon=from[n]+(to[n]-from[n])*k;a.g.rotation.y=a.lon*Math.PI/180})},()=>{ORB_ORDER.forEach(n=>{arms[n].lon=norm(to[n]);arms[n].g.rotation.y=arms[n].lon*Math.PI/180});buildLines();lines.visible=true;S.invalidate()});
    const want=SKYPREF.mode==='vedic'?ayanOf(rr):0;if(Math.abs(want-zodRot)>.01){const z0=zodRot;S.tween(ms||0,k=>{zodRot=z0+(want-z0)*k;zod.rotation.y=zodRot*Math.PI/180})}
    S.invalidate()}
  function select(o){const name=o&&o.userData.arm;
    if(sel){const a=arms[sel.userData.arm];a.glow.material.opacity=.32;a.glow.material.color=K.col(K.HEX.emerald);sel=null;armLabel(a,a.b,false)}
    if(name){const a=arms[name];sel=a.bead;a.glow.material.color=K.col(K.HEX.goldWarm);a.glow.material.opacity=.75;armLabel(a,a.b,true)}S.invalidate()}
  const tip=document.createElement('div');tip.className='tip';
  function hover(o){if(!tip.isConnected)host.appendChild(tip);const u=o&&o.userData;const b=u&&(u.body||(u.arm&&arms[u.arm].b));
    if(!u||(!b&&!u.lens)){tip.classList.remove('on');return}
    const p=new T.Vector3();o.getWorldPosition(p);const s=S.toScreen(p);tip.style.left=s.x+'px';tip.style.top=s.y+'px';
    tip.textContent=u.lens?'The lens · your design under this sky':`${b.kind==='natal'?(b.side==='P'?'Personality ':'Design ')+PNAME(b.body):PNAME(b.body)} · ${b.gate}.${b.line} ${GATES[b.gate][0]} · ${SKYPREF.mode==='vedic'?b.sid.sign+' '+fmtDeg(b.sid.deg):fmtLon(b.lon)}`;tip.classList.add('on')}
  // intro: the field condenses out of the hub, spokes draw outward, the starfield breathes in once
  applyDay(r,0);
  const pitch0=SKYPREF.view==='above'?1.42:.62;fitO=fitOrbit(S);fitA=fitAbove(S);const d0=SKYPREF.view==='above'?fitA:fitO;
  if(!RM){introOn=true;root.scale.setScalar(.55);S.view.dist=d0*1.65;S.view.pitch=.18;S.place();
    S.tween(1700,k=>{root.scale.setScalar(.55+.45*k);S.view.dist=d0*(1.65-.65*k);if(k>=1)introOn=false;S.view.pitch=.18+(pitch0-.18)*k;S.place();stars.material.uniforms.uFade.value=Math.min(1,k*1.4)});
    ORB_ORDER.forEach((n,i)=>{const a=arms[n],tl=a.lon;a.g.scale.setScalar(.01);S.tween(1300+i*80,k=>{a.g.scale.setScalar(Math.max(.01,k));a.g.rotation.y=(tl-90*(1-k))*Math.PI/180})})}
  else{stars.material.uniforms.uFade.value=1;S.view.dist=d0;S.place()}
  S.onFrame((dt,now)=>{stars.material.uniforms.uTime.value=now/1000;return false});
  return Object.assign(S,{
    update(rr){applyDay(rr,RM?0:900)},
    goto(rr,ms){applyDay(rr,ms)},
    setMode(){buildZod();const want=SKYPREF.mode==='vedic'?ayanOf(r):0;const z0=zodRot;S.tween(RM?0:1200,k=>{zodRot=z0+(want-z0)*k;zod.rotation.y=zodRot*Math.PI/180})},
    setLayers(){studs.visible=SKYPREF.natal;buildGates();buildLines();fitO=fitOrbit(S);fitA=fitAbove(S);S.setView({dist:SKYPREF.view==='above'?fitA:fitO},700)},
    applyView(){S.setView({pitch:SKYPREF.view==='above'?1.42:.62,dist:SKYPREF.view==='above'?fitA:fitO,target:[0,SKYPREF.view==='above'?0:-7,0]},1100)},
  });
}

/* ---------- 2D wheel: the same rings as SVG when WebGL is unavailable ---------- */
function sky2d(host,r){
  const B=skyBodies(r),C=400,RZ=372,RT=300,RN=220,RG=262;const ayan=SKYPREF.mode==='vedic'?B.ayan:0;const names=SKYPREF.mode==='vedic'?RASHI:SIGNS;
  const X=(l,R)=>C-R*Math.cos(l*Math.PI/180),Y=(l,R)=>C+R*Math.sin(l*Math.PI/180);
  let s=`<svg class="wheel" viewBox="0 0 800 800" preserveAspectRatio="xMidYMid meet" role="img" aria-label="Sky wheel for ${fmtDate(r.d)}">`;
  const R=K3.rand(5);for(let i=0;i<180;i++){const a=R()*360,rr=385+R()*15;s+=`<circle cx="${X(a,rr).toFixed(1)}" cy="${Y(a,rr).toFixed(1)}" r="${(R()*1.2+.3).toFixed(1)}" fill="var(--silver)" opacity="${(R()*.6+.2).toFixed(2)}"/>`}
  s+=`<circle cx="${C}" cy="${C}" r="${RZ}" fill="none" stroke="var(--gold)" stroke-opacity=".5"/><circle cx="${C}" cy="${C}" r="${RZ-32}" fill="none" stroke="var(--gold)" stroke-opacity=".2"/><circle cx="${C}" cy="${C}" r="${RT}" fill="none" stroke="var(--gold)" stroke-opacity=".35"/><circle cx="${C}" cy="${C}" r="${RN}" fill="none" stroke="var(--violet-glow)" stroke-opacity=".35"/>`;
  for(let i=0;i<12;i++){const l=i*30+ayan;s+=`<line x1="${X(l,RZ-32)}" y1="${Y(l,RZ-32)}" x2="${X(l,RZ)}" y2="${Y(l,RZ)}" stroke="var(--gold)" stroke-opacity=".5"/><text x="${X(l+15,RZ-16)}" y="${Y(l+15,RZ-16)}" font-size="9" text-anchor="middle" dominant-baseline="middle" fill="var(--gold)">${names[i].slice(0,3).toUpperCase()}</text>`}
  if(SKYPREF.mode==='vedic')for(let i=0;i<27;i++){const l=i*(360/27)+ayan;s+=`<line x1="${X(l,RZ-32)}" y1="${Y(l,RZ-32)}" x2="${X(l,RZ-40)}" y2="${Y(l,RZ-40)}" stroke="var(--silver)" stroke-opacity=".5"/><text x="${X(l+360/54,RZ-52)}" y="${Y(l+360/54,RZ-52)}" font-size="7" text-anchor="middle" dominant-baseline="middle">${NAKS[i].slice(0,5)}</text>`}
  if(SKYPREF.gates)for(let i=0;i<64;i++){const l=302+i*5.625,g=WHEEL[i];s+=`<line x1="${X(l,RG-5)}" y1="${Y(l,RG-5)}" x2="${X(l,RG+5)}" y2="${Y(l,RG+5)}" stroke="var(--silver)" stroke-opacity=".4"/><text x="${X(l+2.8,RG+14)}" y="${Y(l+2.8,RG+14)}" font-size="7" text-anchor="middle" dominant-baseline="middle" fill="${NATG.has(g)?'var(--gold-warm)':'var(--text-dis)'}">${g}</text>`}
  if(SKYPREF.lines)for(const L of skyLines(B)){const ra=L.a.kind==='transit'?RT:RN,rb=L.b.kind==='transit'?RT:RN;s+=`<line x1="${X(L.a.lon,ra)}" y1="${Y(L.a.lon,ra)}" x2="${X(L.b.lon,rb)}" y2="${Y(L.b.lon,rb)}" stroke="#${L.asp.col.toString(16).padStart(6,'0')}" stroke-opacity="${L.b.kind==='natal'?.45:.75}" stroke-width="1.2"/>`}
  if(SKYPREF.natal)B.natal.forEach((b,i)=>{s+=`<rect class="pl" data-n="${i}" x="${X(b.lon,RN)-4}" y="${Y(b.lon,RN)-4}" width="8" height="8" transform="rotate(45 ${X(b.lon,RN)} ${Y(b.lon,RN)})" fill="${b.side==='P'?'var(--gold)':'var(--violet-glow)'}"><title>${b.side==='P'?'Personality':'Design'} ${PNAME(b.body)} ${b.gate}.${b.line}</title></rect>`});
  B.transit.forEach((b,i)=>{const rr=(RADIUS[b.body]||.6)*6;s+=`<line x1="${X(b.lon,RT)}" y1="${Y(b.lon,RT)}" x2="${X(b.lon,RZ-32)}" y2="${Y(b.lon,RZ-32)}" stroke="var(--emerald)" stroke-opacity=".25"/><circle class="pl" data-t="${i}" cx="${X(b.lon,RT)}" cy="${Y(b.lon,RT)}" r="${rr}" fill="var(--emerald)"><title>${PNAME(b.body)} ${b.gate}.${b.line}</title></circle><text x="${X(b.lon,RT+22)}" y="${Y(b.lon,RT+22)}" font-size="9" text-anchor="middle" dominant-baseline="middle" fill="var(--text-1)">${SYM[b.body]} ${b.gate}.${b.line}</text>`});
  s+=`<text x="${C}" y="${C-6}" font-size="11" text-anchor="middle" fill="var(--gold)" letter-spacing="3">${SKYPREF.mode==='vedic'?'LAHIRI':'TROPICAL'}</text><text x="${C}" y="${C+10}" font-size="9" text-anchor="middle">${fmtDate(r.d)}</text></svg>`;
  let fb=host.querySelector('.fallback');if(!fb){fb=document.createElement('div');fb.className='fallback';host.prepend(fb)}fb.innerHTML=s;
  fb.querySelectorAll('.pl').forEach(el=>el.addEventListener('click',()=>showBody(el.dataset.t!==undefined?B.transit[+el.dataset.t]:B.natal[+el.dataset.n],r)));
}

/* ---------- the room ---------- */
const SCRUB={anchor:null,playing:null};
function skyHud(r){
  const R=dayReading(r),B=skyBodies(r),moon=B.transit.find(b=>b.body==='Moon'),sun=B.transit.find(b=>b.body==='Sun'),earth=tPos(r)[1];
  SKYPREF.tight=true;const nL=skyLines(B).length;SKYPREF.tight=false;const ad=SKY.ad.find(a=>a[2]<=r.d&&r.d<a[3]);let pd=null;if(ad)for(const p of ad[4])if(p[1]<=r.d)pd=p[0];
  const retro=B.transit.filter(b=>b.retro).map(b=>SYM[b.body]).join(' ');
  const rd=(k,v)=>`<div class="readout"><span class="k">${k}</span><span class="v">${v}</span></div>`;
  return {left:rd('Sun · gate',`<span class="g">${sun.gate}.${sun.line}</span> ${GATES[sun.gate][0]}`)+rd('Earth · gate',`${earth.gate}.${earth.line} ${GATES[earth.gate][0]}`)+rd('Moon · nakshatra',`${moon.sid.nak} · pada ${moon.sid.pada}`)+rd('Definition',`${defLabel(r.def)}${r.m2t?' · motor to Throat':''}`),
    right:rd('Dasha',ad?`${ad[0]} / ${ad[1]}${pd?' / '+pd:''}`:'—')+rd('Ayanamsa · Lahiri',`${B.ayan.toFixed(2)}°`)+rd('Aspects',`${nL} within orb`)+rd('Retrograde',retro||'none'),
    decl:R.decl};
}
function rSky(){
  const r=rec(state.date),isToday=state.date===todayIST();
  if(!SCRUB.anchor)SCRUB.anchor=clampDate(todayIST());
  const off=Math.round((Date.parse(state.date)-Date.parse(SCRUB.anchor))/864e5);
  const H=skyHud(r);
  $('#main').classList.add('full');
  $('#main').innerHTML=`<section class="room">
  <div class="stage sky-stage" id="skyHost">
   <div class="hud sky-top"><div class="grp"><div class="seg" role="group" aria-label="Zodiac"><button id="skyWest" aria-pressed="${SKYPREF.mode!=='vedic'}">Western</button><button id="skyVedic" aria-pressed="${SKYPREF.mode==='vedic'}">Vedic</button></div></div>
    <div class="grp"><button class="tog" id="skyGates" aria-pressed="${SKYPREF.gates}">64 gates</button><button class="tog" id="skyNatal" aria-pressed="${SKYPREF.natal}">Birth</button><button class="tog" id="skyLines" aria-pressed="${SKYPREF.lines}">Aspects</button><div class="seg" role="group" aria-label="View"><button id="skyOrbit" aria-pressed="${SKYPREF.view!=='above'}">Orbit</button><button id="skyAbove" aria-pressed="${SKYPREF.view==='above'}">Above</button></div></div></div>
   <div class="hud sky-left" id="skyL">${H.left}</div>
   <div class="hud sky-right" id="skyR">${H.right}</div>
   <div class="hud sky-bottom"><div class="epithet" id="skyEp">${isToday?'The sky now':'Witnessing'} · ${fmtDate(r.d)} · 06:00 IST</div><p class="decl" id="skyDecl">${esc(H.decl)}</p>
    <div class="scrub"><button class="icon-btn" id="skyPlay" aria-label="Play the next thirty days"><svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor"><path d="M4 2.5v11l9-5.5z"/></svg></button><span class="lab" id="skyOff">${off===0?'TODAY':(off>0?'+':'')+off+' DAYS'}</span><input type="range" id="skyScrub" min="-30" max="90" step="1" value="${off}" aria-label="Scrub the sky by day"><span class="lab" style="text-align:right">+90 D</span></div></div>
  </div>
  <div class="pad"><nav class="honey" aria-label="Rooms">${ROOMS.filter(x=>x[0]!=='sky').map(([id,l])=>`<a href="#${id}" data-say="${esc(ROOM_TXT[id])}">${medallion(id,88)}<b>${l}</b></a>`).join('')}</nav><p class="honey-read" id="honeyRead">Eight rooms around one sky. Hover a medallion to read it.</p></div>
  <p class="foot pad">See the pattern. Keep the authority.</p></section>`;
  const host=$('#skyHost');
  if(K3.ok()){try{const {inst,fresh}=K3.mount('sky',host,h=>orrery(h,r));if(!fresh)inst.update(r)}catch(e){console.warn('orrery',e);K3.release();sky2d(host,r)}}
  else sky2d(host,r);
  const O=()=>K3.current('sky');
  const set=(k,v,how)=>{SKYPREF[k]=v;savePref();$('#skyWest').setAttribute('aria-pressed',SKYPREF.mode!=='vedic');$('#skyVedic').setAttribute('aria-pressed',SKYPREF.mode==='vedic');
    [['Gates','gates'],['Natal','natal'],['Lines','lines']].forEach(([n,k2])=>$('#sky'+n).setAttribute('aria-pressed',SKYPREF[k2]));$('#skyOrbit').setAttribute('aria-pressed',SKYPREF.view!=='above');$('#skyAbove').setAttribute('aria-pressed',SKYPREF.view==='above');
    const o=O();if(o)o[how]();else sky2d(host,rec(state.date))};
  $('#skyWest').onclick=()=>set('mode','western','setMode');$('#skyVedic').onclick=()=>set('mode','vedic','setMode');
  $('#skyGates').onclick=()=>set('gates',!SKYPREF.gates,'setLayers');$('#skyNatal').onclick=()=>set('natal',!SKYPREF.natal,'setLayers');$('#skyLines').onclick=()=>set('lines',!SKYPREF.lines,'setLayers');
  $('#skyOrbit').onclick=()=>set('view','orbit','applyView');$('#skyAbove').onclick=()=>set('view','above','applyView');
  const goto=(d,ms)=>{state.date=clampDate(d);const rr=rec(state.date);$('#dateIn').value=state.date;const o=O();if(o)o.goto(rr,ms);else sky2d(host,rr);
    const h=skyHud(rr);$('#skyL').innerHTML=h.left;$('#skyR').innerHTML=h.right;$('#skyDecl').textContent=h.decl;$('#skyEp').textContent=`${state.date===todayIST()?'The sky now':'Witnessing'} · ${fmtDate(rr.d)} · 06:00 IST`;
    const of=Math.round((Date.parse(state.date)-Date.parse(SCRUB.anchor))/864e5);$('#skyOff').textContent=of===0?'TODAY':(of>0?'+':'')+of+' DAYS';$('#skyScrub').value=of};
  function stopPlay(){if(SCRUB.playing){clearInterval(SCRUB.playing);SCRUB.playing=null;const b=$('#skyPlay');if(b)b.setAttribute('aria-label','Play the next thirty days')}}
  $('#skyScrub').oninput=e=>{stopPlay();goto(addDays(SCRUB.anchor,+e.target.value),K3.reduced()?0:260)};
  $('#skyPlay').onclick=()=>{if(SCRUB.playing)return stopPlay();let n=0;$('#skyPlay').setAttribute('aria-label','Pause');SCRUB.playing=setInterval(()=>{if(state.room!=='sky'||n++>=30)return stopPlay();goto(addDays(state.date,1),K3.reduced()?0:230)},260)};
  $('#main').querySelectorAll('.honey a').forEach(a=>{const say=()=>{$('#honeyRead').textContent=a.dataset.say};a.addEventListener('mouseenter',say);a.addEventListener('focus',say)});
}
