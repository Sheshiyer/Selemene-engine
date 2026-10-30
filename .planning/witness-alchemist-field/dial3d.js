/* ---------- The Day Dial: the Vedic Clock engine's object ----------
   A single-hand twenty-four-hour bronze face on a hex plinth. From the rim inward: the hour bezel with the
   day and night arcs, the hora ring (24 unequal hours from sunrise), the choghadiya ring (8 day, 8 night),
   the organ-meridian ring and the dosha ring. Midnight sits nearest you; noon at the far side. */
const DIAL={h:null,date:null};
const hourAng=h=>90-((h%24)+24)%24*15;
function dialSeg(list,r0,r1){return list.map(x=>({x,a0:hourAng(x.e),a1:hourAng(x.s)}))}
function dialPick(u,C){const f=DL.fmtH;let html='';
  if(u.kind==='hora'){const x=u.x;html=`<div class="eyebrow">Hora ${x.n} of 24 · ${x.period}</div><h2>${SYM[x.lord]||''} ${x.lord} hora</h2><dl class="kv"><dt>Window</dt><dd>${f(x.s)} to ${f(x.e)} IST</dd><dt>Length</dt><dd>${Math.round((x.e-x.s)*60)} minutes</dd><dt>Classical key</dt><dd>${x.key}</dd><dt>Day lord</dt><dd>${C.lord}, so the first hora after sunrise is ${C.lord}'s</dd></dl>`}
  else if(u.kind==='chog'){const x=u.x;html=`<div class="eyebrow">Choghadiya ${x.n} of 8 · ${x.period}</div><h2>${x.name}</h2><dl class="kv"><dt>Window</dt><dd>${f(x.s)} to ${f(x.e)} IST</dd><dt>Ruler</dt><dd>${x.ruler}</dd><dt>Classical quality</dt><dd>${x.meaning} · ${x.nature}</dd></dl>`}
  else if(u.kind==='organ'){const x=u.x;html=`<div class="eyebrow">Organ clock</div><h2>${x.organ}</h2><dl class="kv"><dt>Window</dt><dd>${f(x.s)} to ${f(x.e)}</dd><dt>Element</dt><dd>${x.element}</dd></dl><p class="muted">The two-hour meridian window the Vedic Clock engine reports, from the traditional Chinese organ clock.</p>`}
  else if(u.kind==='dosha'){const x=u.x;html=`<div class="eyebrow">Dosha clock</div><h2>${x.dosha}</h2><dl class="kv"><dt>Window</dt><dd>${f(x.s)} to ${f(x.e%24)}</dd></dl><p class="muted">The Ayurvedic four-hour cycle: Kapha, Pitta, Vata, twice a day.</p>`}
  openDrawer(html+`<p class="meta" style="margin-top:12px">Field figure by the Vedic Clock engine's method, sunrise ${f(C.rise)} and sunset ${f(C.set)} at Bangalore. The method reproduces the engine's response for ${ENG['vedic-clock']?istDate(ENG['vedic-clock'].result.calculated_for):'its call'}.</p>`)}
function dial3d(host,date,h0){
  const K=K3,T=THREE;const RM=K.reduced();let C=DL.clock(date),H=h0;
  let S=null;S=K.stage(host,{fov:30,yaw:-Math.PI/2,pitch:1.02,dist:120,target:[0,-2.5,0],minDist:60,maxDist:200,minPitch:.4,maxPitch:1.45,label:'Day dial. Tap a ring for its window.',
    onResize:()=>{if(S){S.view.dist=fit();S.place()}},
    ambient:(dt,now)=>{S.view.yaw=S.base.yaw+Math.sin(now/6400)*.05;return true},
    onPick:o=>{if(o&&o.userData.dial)dialPick(o.userData.dial,C)},
    onHover:o=>{tipFor(o)}});
  const {scene,M}=S;
  function fit(){const c=S.camera,v=c.fov*Math.PI/360,hh=Math.atan(Math.tan(v)*c.aspect);const R=31;return Math.max(R*1.05/Math.sin(hh),(R*Math.sin(1.02)+7)/Math.sin(v),60)}
  scene.add(K.starfield(500,240,420,61));
  const root=new T.Group();scene.add(root);
  const plinth=new T.Mesh(K.prism(K.hexShape(33,0),2.2,.4),M.bronze());plinth.position.y=-3.4;root.add(plinth);
  const pe=new T.LineLoop(K.loopXZ(K.hexPts(33.4,0),0),M.line(K.HEX.gold,.85));pe.position.y=-.78;root.add(pe);
  const hal=K.halo(120,.55);hal.position.y=-3.6;root.add(hal);
  const face=new T.Mesh(new T.CylinderGeometry(29,29.6,1,128),M.bronzeDark());face.position.y=-.5;root.add(face);
  const R={bez:[25.4,28.6],hora:[20.6,25],chog:[15.6,20.2],organ:[10.8,15.2],dosha:[6.6,10.4]};
  const rings=new T.Group();root.add(rings);let segs=[];
  const mat={day:M.bronze(),night:M.bronzeDark(),good:M.bronze({color:K.col(0xb08a3a)}),neutral:M.bronze(),bad:M.bronzeDark({color:K.col(0x231d14)}),cur:M.bronze({color:K.col(0xa08048),emissive:K.col(K.HEX.goldWarm),emissiveIntensity:.14,roughness:.42})};
  const clear=g=>{while(g.children.length){const c=g.children.pop();S.unpick(c);c.traverse(o=>{if(o.geometry)o.geometry.dispose();if(o.material&&!Object.values(mat).includes(o.material)){if(o.material.map)o.material.map.dispose();o.material.dispose()}})}};
  const sprite=(t,o,pos)=>{const l=S.track(K.label(t,o));l.position.copy(pos);rings.add(l);return l};
  function build(){clear(rings);segs=[];
    const add=(kind,list,r,lab,matFn)=>{dialSeg(list).forEach(({x,a0,a1})=>{let A0=a0,A1=a1;while(A1<A0)A1+=360;const gap=Math.min(.35,(A1-A0)*.06);
      const m=new T.Mesh(K.sector(r[0],r[1],A0+gap,A1-gap,.55),matFn(x));m.userData.dial={kind,x};S.pick(m);rings.add(m);segs.push({m,kind,x,base:m.material});
      const t=lab(x,A1-A0);if(t){sprite(t,{font:'mono',px:kind==='hora'?11:9,track:.1,color:kind==='hora'?'#E6B84D':'#C2CBD1'},K.P((A0+A1)/2,(r[0]+r[1])/2,1.45))}})};
    add('hora',C.horas,R.hora,x=>SYM[x.lord]||x.lord[0],x=>x.period==='day'?mat.day:mat.night);
    add('chog',C.chog,R.chog,(x,w)=>w>14?x.name.toUpperCase():null,x=>mat[x.nature==='good'?'good':x.nature==='neutral'?'neutral':'bad']);
    add('organ',C.organs,R.organ,(x)=>({Gallbladder:'GB',Liver:'LIVER',Lung:'LUNG','Large intestine':'L.INT',Stomach:'STOM',Spleen:'SPLEEN',Heart:'HEART','Small intestine':'S.INT',Bladder:'BLADDER',Kidney:'KIDNEY',Pericardium:'PERIC','Triple burner':'T.BURN'})[x.organ]||x.organ,()=>mat.neutral);
    add('dosha',C.doshas,R.dosha,x=>x.dosha.toUpperCase(),x=>x.dosha==='Kapha'?mat.night:x.dosha==='Pitta'?mat.good:mat.neutral);
    // bezel: day arc gold, night arc silver, hour ticks, numerals every three hours
    const bz=new T.Mesh(K.band(R.bez[0],R.bez[1],.7),M.bronze());bz.position.y=-.1;rings.add(bz);
    let a0=hourAng(C.set),a1=hourAng(C.rise);while(a1<a0)a1+=360;rings.add(new T.Line(K.arcXZ(R.bez[1]+.25,.75,a0,a1,120),M.line(K.HEX.goldWarm,1)));
    rings.add(new T.Line(K.arcXZ(R.bez[1]+.25,.75,a1,a0+360,120),M.line(K.HEX.silver,.55)));
    const tk=[];for(let q=0;q<96;q++){const a=hourAng(q/4),L=q%4?.5:q%12?1.1:1.8;tk.push(K.P(a,R.bez[1]-L,.66),K.P(a,R.bez[1]-.05,.66))}rings.add(new T.LineSegments(new T.BufferGeometry().setFromPoints(tk),M.line(K.HEX.gold,.6)));
    for(let q=0;q<24;q+=3)sprite(String(q).padStart(2,'0'),{font:'mono',px:10,color:'#C5A017'},K.P(hourAng(q),(R.bez[0]+R.bez[1])/2-.3,1.1));
    sprite(`☉ ${DL.fmtH(C.rise)}`,{font:'mono',px:10,color:'#E6B84D',bg:'rgba(7,11,29,.6)'},K.P(hourAng(C.rise),R.bez[1]+2.6,1.2));
    sprite(`☉ ${DL.fmtH(C.set)}`,{font:'mono',px:10,color:'#8A9BA8',bg:'rgba(7,11,29,.6)'},K.P(hourAng(C.set),R.bez[1]+2.6,1.2));
    // Rahu Kaal (silver dashed) and Abhijit (gold) marked outside the bezel
    const mark=(w,col,dash,txt,off)=>{let b0=hourAng(w.e),b1=hourAng(w.s);while(b1<b0)b1+=360;const ln=new T.Line(K.arcXZ(R.bez[1]+1.1,.8,b0,b1,40),dash?new T.LineDashedMaterial({color:K.col(col),dashSize:.6,gapSize:.4,toneMapped:false}):M.line(col,1));ln.computeLineDistances();rings.add(ln);sprite(txt,{font:'mono',px:9,track:.16,color:'#'+col.toString(16).padStart(6,'0')},K.P((b0+b1)/2,R.bez[1]+4.4+(off||0),1))};
    mark(C.rahu,K.HEX.silver,true,'RAHU KAAL');mark(C.abhijit,K.HEX.gold,false,'ABHIJIT',(C.abhijit.s<C.rahu.e&&C.abhijit.e>C.rahu.s)?3.2:0);
    [R.hora[0]-.05,R.hora[1]+.05,R.chog[0]-.05,R.organ[0]-.05,R.dosha[0]-.05].forEach(rr=>rings.add(new T.LineLoop(K.circleXZ(rr,.62,200),M.line(K.HEX.gold,.35))));
    paint()}
  // hub lens and the single hand
  const lens=new T.Mesh(new T.SphereGeometry(3.4,48,32),M.glass(K.HEX.emerald,{emissiveIntensity:.35}));lens.position.y=1.6;root.add(lens);const lg=K.glow(K.HEX.emerald,12,.4);lg.position.y=1.6;root.add(lg);
  const bezL=new T.Mesh(new T.TorusGeometry(3.7,.3,12,64),M.gold());bezL.rotation.x=Math.PI/2;bezL.position.y=.9;root.add(bezL);
  const hand=new T.Group();hand.position.y=1.5;root.add(hand);
  const blade=new T.Mesh(new T.BoxGeometry(23.5,.16,.55),M.gold({emissiveIntensity:.4,roughness:.28}));blade.position.x=4+23.5/2;hand.add(blade);
  const tip=new T.Mesh(new T.OctahedronGeometry(1.1,0),M.gold({emissiveIntensity:.6}));tip.scale.set(1.4,.5,.7);tip.position.x=28;hand.add(tip);
  const tail=new T.Mesh(new T.CylinderGeometry(.9,.9,.3,20),M.gold());tail.position.x=-3.2;hand.add(tail);
  let handAng=hourAng(H),handTw=null,curRims=[];const rimMat=M.line(K.HEX.goldWarm,1);
  function paint(){const A=DL.at(C,H);curRims.forEach(o=>{rings.remove(o);o.geometry.dispose()});curRims=[];segs.forEach(s=>{const cur=(s.kind==='hora'&&s.x===A.hora)||(s.kind==='chog'&&s.x===A.chog)||(s.kind==='organ'&&s.x===A.organ)||(s.kind==='dosha'&&s.x===A.dosha);s.m.material=cur?mat.cur:s.base;s.m.position.y=cur?.35:0;if(cur){const r=R[s.kind];let a0=hourAng(s.x.e),a1=hourAng(s.x.s);while(a1<a0)a1+=360;const ln=new T.Line(K.arcXZ(r[1]-.1,1.05,a0+.3,a1-.3,48),rimMat);rings.add(ln);curRims.push(ln)}});S.invalidate()}
  function setHour(h,ms){H=h;let to=hourAng(h),d=to-handAng;d=((d+540)%360)-180;const from=handAng;if(handTw)handTw.stop();handTw=S.tween(ms==null?600:ms,k=>{handAng=from+d*k;hand.rotation.y=handAng*Math.PI/180},()=>{handAng=from+d});paint()}
  const tipEl=document.createElement('div');tipEl.className='tip';
  function tipFor(o){if(!tipEl.isConnected)host.appendChild(tipEl);const u=o&&o.userData.dial;if(!u){tipEl.classList.remove('on');return}const box=new T.Box3().setFromObject(o),p=new T.Vector3();box.getCenter(p);const s=S.toScreen(p);tipEl.style.left=s.x+'px';tipEl.style.top=s.y+'px';
    const x=u.x,f=DL.fmtH;tipEl.textContent=u.kind==='hora'?`${x.lord} hora · ${f(x.s)}–${f(x.e)}`:u.kind==='chog'?`${x.name} · ${x.meaning} · ${f(x.s)}–${f(x.e)}`:u.kind==='organ'?`${x.organ} · ${x.element} · ${f(x.s)}–${f(x.e)}`:`${x.dosha} · ${f(x.s)}–${f(x.e%24)}`;tipEl.classList.add('on')}
  build();hand.rotation.y=handAng*Math.PI/180;
  if(!RM){const d=fit();S.view.dist=d*1.45;S.view.pitch=1.4;S.place();rings.scale.setScalar(.6);S.tween(1500,k=>{rings.scale.setScalar(.6+.4*k);S.view.dist=d*(1.45-.45*k);S.view.pitch=1.4+(1.02-1.4)*k;S.base.pitch=S.view.pitch;S.place()});
    const a1=handAng;hand.rotation.y=(a1+180)*Math.PI/180;S.tween(1600,k=>{hand.rotation.y=(a1+180*(1-k))*Math.PI/180})}
  else{S.view.dist=fit();S.place()}
  return Object.assign(S,{update(d,h){if(d!==C.date){C=DL.clock(d);build()}setHour(h,RM?0:700)},setHour(h){setHour(h,RM?0:250)}});
}
/* 2D dial for devices without WebGL: the same rings as SVG */
function dial2d(host,date,h){const C=DL.clock(date),A=DL.at(C,h);const c=200,P=(hh,r)=>{const a=hourAng(hh)*Math.PI/180;return [c-r*Math.cos(a),c+r*Math.sin(a)]};
  const arc=(s,e,r0,r1)=>{let a=P(s,r1),b=P(e,r1),d=P(e,r0),f=P(s,r0);const big=(e-s)>12?1:0;return `M${a[0]} ${a[1]}A${r1} ${r1} 0 ${big} 1 ${b[0]} ${b[1]}L${d[0]} ${d[1]}A${r0} ${r0} 0 ${big} 0 ${f[0]} ${f[1]}Z`};
  const ring=(list,r0,r1,fill)=>list.map(x=>`<path d="${arc(x.s,x.e,r0,r1)}" fill="${fill(x)}" stroke="var(--void)" stroke-width="1.2"/>`).join('');
  const cur=x=>x===A.hora||x===A.chog||x===A.organ||x===A.dosha;
  let s=`<svg viewBox="0 0 400 400" role="img" aria-label="Day dial for ${fmtDate(date)}">`+ring(C.horas,150,180,x=>cur(x)?'var(--gold)':x.period==='day'?'#3b3121':'#1f1a12')+ring(C.chog,115,147,x=>cur(x)?'var(--gold)':x.nature==='good'?'#4a3d22':x.nature==='neutral'?'#3b3121':'#1a160f')+ring(C.organs.map(o=>({...o})),80,112,x=>'#2e271a')+ring(C.doshas,50,77,x=>'#25201a');
  C.horas.forEach(x=>{const [X,Y]=P((x.s+x.e)/2,165);s+=`<text x="${X}" y="${Y}" font-size="10" text-anchor="middle" dominant-baseline="middle" fill="var(--gold-warm)">${SYM[x.lord]}</text>`});
  const [hx,hy]=P(h,184);s+=`<line x1="${c}" y1="${c}" x2="${hx}" y2="${hy}" stroke="var(--gold)" stroke-width="2"/><circle cx="${c}" cy="${c}" r="18" fill="var(--emerald)" opacity=".7"/></svg>`;
  host.innerHTML=`<div class="fallback" style="padding:24px">${s}</div>`}
