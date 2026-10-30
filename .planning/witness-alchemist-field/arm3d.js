/* ---------- The dasha armillary: Vimshottari as three nested bronze rings on one index ----------
   Outer ring: the ten Mahadashas of the engine's 120-year timeline. Middle: the Antardashas of the chosen
   Mahadasha. Inner: the Pratyantardashas of the chosen Antardasha. Each ring turns so the period running on
   the witnessed date sits under the gold index; moving the date makes the rings precess at nested rates. */
const DASHA_SYM={Sun:'☉',Moon:'☽',Mars:'♂',Mercury:'☿',Jupiter:'♃',Venus:'♀',Saturn:'♄',Rahu:'☊',Ketu:'☋'};
function dashaRings(){const V=ENG.vimshottari.result,T=V.timeline.mahadashas;const md=T.find(m=>m.planet+m.start_date.slice(0,10)===jyState.md)||T[0];
  const ms=md.start_date.slice(0,10),me=md.end_date.slice(0,10);
  const ads=SKY.ad.filter(a=>a[0]===md.planet&&a[2]>=addDays(ms,-2)&&a[2]<addDays(me,-2)).map(a=>({p:a[1],s:a[2],e:a[3],raw:a}));
  const ad=ads.find(a=>a.p+a.s===jyState.ad)||ads[0];const pds=ad?ad.raw[4].map((p,i)=>({p:p[0],s:p[1],e:i+1<ad.raw[4].length?ad.raw[4][i+1][1]:ad.e})):[];
  const mds=T.map(m=>({p:m.planet,s:m.start_date.slice(0,10),e:m.end_date.slice(0,10),key:m.planet+m.start_date.slice(0,10)}));
  return {mds,md:{p:md.planet,s:ms,e:me,key:md.planet+ms},ads,ad,pds}}
const dms=s=>Date.parse(s+'T00:00:00Z');
function arm3d(host){
  const K=K3,T=THREE;const RM=K.reduced();
  let S=null;S=K.stage(host,{fov:30,yaw:-Math.PI/2,pitch:.72,dist:110,target:[0,-3.5,0],minDist:60,maxDist:200,minPitch:.2,maxPitch:1.4,label:'Dasha armillary. Tap a ring segment to drill in.',
    onResize:()=>{if(S){S.view.dist=fit();S.place()}},
    ambient:(dt,now)=>{S.view.yaw=S.base.yaw+Math.sin(now/6000)*.06;return true},
    onPick:o=>{if(!o||!o.userData.seg)return;const u=o.userData.seg;if(u.level==='md'){jyState.md=u.key;jyState.ad=null}else if(u.level==='ad'){jyState.ad=u.p+u.s}else return;rebuild(true);if(window.onArmPick)window.onArmPick()},
    onHover:o=>{hov=o&&o.userData.seg?o:null;tipShow()}});
  const {scene,M}=S;let hov=null;
  function fit(){const c=S.camera,v=c.fov*Math.PI/360,hh=Math.atan(Math.tan(v)*c.aspect);const R=31;return Math.max(R*1.06/Math.sin(hh),(R*Math.sin(.72)+17)/Math.sin(v)*1.02,62)}
  const root=new T.Group();scene.add(root);
  scene.add(Object.assign(K.starfield(600,240,420,53),{}));
  // pedestal: hex plinth, turned column, halo
  const plinth=new T.Mesh(K.prism(K.hexShape(15,Math.PI/6),2.2,.3),M.bronze());plinth.position.y=-16;root.add(plinth);
  const pe=new T.LineLoop(K.loopXZ(K.hexPts(15.2,Math.PI/6),0),M.line(K.HEX.gold,.8));pe.position.y=-16+2.5+.02;root.add(pe);
  const hal=K.halo(80,.7);hal.position.y=-16.2;root.add(hal);
  const prof=[[0,0],[3.2,0],[3.2,.5],[2,1],[1.3,2.2],[1.0,8],[1.5,8.8],[1.0,9.6],[1.0,12.4],[2.2,13.2],[0,13.2]].map(([x,y])=>new T.Vector2(x,y));
  const col=new T.Mesh(new T.LatheGeometry(prof,40),M.bronze());col.position.y=-13.5;root.add(col);
  // decorative armillary cage: two tilted gold meridians around the rings
  [[.0,.55],[Math.PI/2,.55]].forEach(([ry,rx])=>{const m=new T.Mesh(new T.TorusGeometry(30.5,.16,8,160),M.gold({emissiveIntensity:.05,roughness:.4}));m.rotation.set(Math.PI/2+rx,ry,0);m.position.y=1.5;root.add(m)});
  const eq=new T.Mesh(new T.TorusGeometry(31.2,.22,8,180),M.gold({roughness:.4}));eq.rotation.x=Math.PI/2;eq.position.y=1.5;root.add(eq);
  // the gold index blade at the front, through all three rings
  const blade=new T.Mesh(new T.BoxGeometry(.35,7.4,.12),M.gold({emissiveIntensity:.5}));blade.position.copy(K.P(90,31.4,2.6));root.add(blade);
  const bl=new T.Mesh(new T.ConeGeometry(.8,2,4),M.gold({emissiveIntensity:.6}));bl.rotation.x=-Math.PI/2;bl.position.copy(K.P(90,32.6,6.4));root.add(bl);
  const idx=new T.Line(new T.BufferGeometry().setFromPoints([K.P(90,8,5.6),K.P(90,31.6,5.6)]),M.line(K.HEX.goldWarm,.95));root.add(idx);
  // hub lens with the running period
  const lens=new T.Mesh(new T.SphereGeometry(2.6,48,32),M.glass(K.HEX.emerald,{emissiveIntensity:.4}));lens.position.y=3;root.add(lens);
  const lg=K.glow(K.HEX.emerald,10,.45);lg.position.y=3;root.add(lg);
  const hubLab=S.track(K.label(' ',{font:'engr',px:13,track:.24,color:'#E6B84D',weight:600}));hubLab.position.y=7.6;root.add(hubLab);
  const RINGS={md:{r0:24,r1:29,y:0,h:1.1},ad:{r0:17.2,r1:22.4,y:2.1,h:1},pd:{r0:10.6,r1:15.6,y:4,h:.9}};
  const groups={md:new T.Group(),ad:new T.Group(),pd:new T.Group()};Object.values(groups).forEach(g=>root.add(g));
  const rot={md:0,ad:0,pd:0};
  function sector(r0,r1,a0,a1,h){const s=new T.Shape();s.absarc(0,0,r1,a0*Math.PI/180,a1*Math.PI/180,false);s.absarc(0,0,r0,a1*Math.PI/180,a0*Math.PI/180,true);s.closePath();return K.prism(s,h,.06)}
  const clear=g=>{while(g.children.length){const c=g.children.pop();S.unpick(c);c.traverse(o=>{if(o.geometry)o.geometry.dispose();if(o.material){if(o.material.map)o.material.map.dispose();o.material.dispose()}})}};
  function buildRing(level,list,span,selKey){const g=groups[level],R=RINGS[level];clear(g);if(!list.length)return [];const t0=dms(span[0]),t1=dms(span[1]),tot=t1-t0;const now=state.date;const out=[];
    list.forEach(seg=>{const a0=(dms(seg.s)-t0)/tot*360,a1=(dms(seg.e)-t0)/tot*360;if(a1-a0<.05)return;const gap=Math.min(.5,(a1-a0)*.08);const isNow=seg.s<=now&&now<seg.e;const isSel=selKey&&(seg.key||seg.p+seg.s)===selKey;
      const m=new T.Mesh(sector(R.r0,R.r1,a0+gap,a1-gap,R.h),isNow?M.glass(K.HEX.emerald,{emissiveIntensity:.5,opacity:.95}):M.bronze({color:K.col(isSel?0xa8874a:0x8a7350)}));m.position.y=R.y;m.userData.seg={level,p:seg.p,s:seg.s,e:seg.e,key:seg.key};S.pick(m);g.add(m);
      const edge=new T.Line(new T.BufferGeometry().setFromPoints([K.P(a0+gap,R.r0,R.y+R.h+.38),K.P(a0+gap,R.r1,R.y+R.h+.38)]),M.line(K.HEX.gold,.8));g.add(edge);
      if(isSel){const arc=new T.Line(K.arcXZ(R.r1+.15,R.y+R.h+.4,a0+gap,a1-gap),M.line(K.HEX.goldWarm,1));g.add(arc)}
      if(a1-a0>7){const txt=(DASHA_SYM[seg.p]||'')+(a1-a0>22||level==='md'&&a1-a0>14?' '+seg.p.toUpperCase():'');const l=S.track(K.label(txt,{font:'mono',px:level==='md'?11:10,track:.14,color:isNow?'#F0EDE3':'#C5A017',bg:isNow?'rgba(16,181,167,.25)':null}));l.position.copy(K.P((a0+a1)/2,(R.r0+R.r1)/2,R.y+R.h+1.2));g.add(l)}
      out.push({seg,a0,a1})});
    [R.r0-.05,R.r1+.05].forEach(rr=>g.add(new T.LineLoop(K.circleXZ(rr,R.y+R.h+.37,200),M.line(K.HEX.gold,.4))));
    return out}
  function nowAngle(span){const t0=dms(span[0]),t1=dms(span[1]);const t=Math.min(t1,Math.max(t0,dms(state.date)));return (t-t0)/(t1-t0)*360}
  function rebuild(animate){const D=dashaRings();const V=ENG.vimshottari.result;const T0=V.timeline.mahadashas;
    buildRing('md',D.mds,[D.mds[0].s,D.mds[D.mds.length-1].e],D.md.key);
    buildRing('ad',D.ads,[D.md.s,D.md.e],D.ad?D.ad.p+D.ad.s:null);
    if(D.ad)buildRing('pd',D.pds,[D.ad.s,D.ad.e],null);else clear(groups.pd);
    const want={md:90-nowAngle([D.mds[0].s,D.mds[D.mds.length-1].e]),ad:90-nowAngle([D.md.s,D.md.e]),pd:D.ad?90-nowAngle([D.ad.s,D.ad.e]):0};
    const from={...rot};Object.keys(want).forEach(k=>{let d=want[k]-from[k];d=((d+540)%360)-180;want[k]=from[k]+d});
    S.tween(animate&&!RM?1100:0,k=>{Object.keys(want).forEach(n=>{rot[n]=from[n]+(want[n]-from[n])*k;groups[n].rotation.y=rot[n]*Math.PI/180})});
    const cur=D.mds.find(m=>m.s<=state.date&&state.date<m.e)||D.md;const old=hubLab.material.map;const nl=K.label(`${(DASHA_SYM[cur.p]||'')} ${cur.p.toUpperCase()} · TILL ${cur.e.slice(0,4)}`,{font:'engr',px:13,track:.24,color:'#E6B84D',weight:600,bg:'rgba(7,11,29,.55)'});hubLab.material.map=nl.material.map;hubLab.userData.label=nl.userData.label;S.track(hubLab);nl.material.dispose();old&&old.dispose();
    S.invalidate()}
  const tip=document.createElement('div');tip.className='tip';
  function tipShow(){if(!tip.isConnected)host.appendChild(tip);if(!hov){tip.classList.remove('on');return}const u=hov.userData.seg;const p=new T.Vector3();hov.getWorldPosition(p);const box=new T.Box3().setFromObject(hov);box.getCenter(p);const s=S.toScreen(p);
    tip.style.left=s.x+'px';tip.style.top=s.y+'px';tip.textContent=`${u.level==='md'?'Mahadasha':u.level==='ad'?'Antardasha':'Pratyantardasha'} · ${u.p} · ${fmtDate(u.s)} to ${fmtDate(u.e)}`;tip.classList.add('on')}
  rebuild(false);
  if(!RM){const d=fit();S.view.dist=d*1.5;S.view.pitch=1.25;S.place();Object.values(groups).forEach((g,i)=>{g.scale.setScalar(.55);S.tween(1200+i*200,k=>g.scale.setScalar(.55+.45*k))});
    S.tween(1700,k=>{S.view.dist=d*(1.5-.5*k);S.view.pitch=1.25+(.72-1.25)*k;S.base.pitch=S.view.pitch;S.place()})}
  else{S.view.dist=fit();S.place()}
  return Object.assign(S,{update(){rebuild(true)},sync(){rebuild(true)}});
}
