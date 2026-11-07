/* ---------- The loom: Human Design as the Render Bible's nine-socket patch loom ----------
   Bronze sockets in the nine center shapes on a standing plate; gold wire for your channels, emerald glass
   where the sky completes a channel with your gate, violet for transit-only; gate pins seat along the wires.
   When the day changes, newly lit channels draw along the wire. */
const LOOM_SHAPES={Head:[[0,-36],[34,24],[-34,24]],Ajna:[[-34,-26],[34,-26],[0,34]],G:[[0,-44],[44,0],[0,44],[-44,0]],Heart:[[0,-26],[26,20],[-26,20]],Spleen:[[-30,-44],[30,0],[-30,44]],SolarPlexus:[[30,-44],[-30,0],[30,44]]};
const LOOM_RECT={Throat:[80,72],Sacral:[84,80],Root:[84,80]};
function loomState(r){const tg=new Set(),tb={};if(r)tPos(r).forEach(t=>{if(t.body!=='Moon'){tg.add(t.gate);(tb[t.gate]=tb[t.gate]||[]).push(t.body)}});
  const ch=Object.fromEntries(r?r.ch:NATAL_CH.map(k=>[k,'natal']));const cen=new Set(r?r.cen:[...NATAL_CEN]);return {tg,tb,ch,cen}}
function loom3d(host,r0){
  const K=K3,T=THREE;const RM=K.reduced();const k=.1;
  const X=x=>(x-200)*k,Y=y=>(312-y)*k;
  let S=null;S=K.stage(host,{fov:28,yaw:Math.PI/2,pitch:.1,dist:150,target:[0,-.5,0],minDist:70,maxDist:220,minPitch:-.15,maxPitch:.6,yawClamp:[Math.PI/2-.55,Math.PI/2+.55],tilt:.12,label:'Bodygraph loom. Tap a center or a gate.',
    onResize:()=>{if(S){S.view.dist=fit();S.place()}},
    onPick:o=>{if(!o)return;const u=o.userData;if(u.center){const st=loomState(cur);showCenter(u.center,st.cen.has(u.center),st.cen.has(u.center)&&!NATAL_CEN.has(u.center))}else if(u.gate)showGate(u.gate)},
    onHover:o=>{hov=o;S.invalidate()}});
  const {scene,M}=S;let cur=r0,hov=null;
  function fit(){const c=S.camera,v=c.fov*Math.PI/360,hh=Math.atan(Math.tan(v)*c.aspect);return Math.max(36/Math.sin(v),26/Math.sin(hh),60)}
  const root=new T.Group();scene.add(root);
  // the standing plate: bronze, chamfered corners, gold edge and inset rule, on a hex plinth with a halo
  const pw=24,ph=34,ch=4;const ps=new T.Shape();ps.moveTo(-pw+ch,-ph);ps.lineTo(pw-ch,-ph);ps.lineTo(pw,-ph+ch);ps.lineTo(pw,ph-ch);ps.lineTo(pw-ch,ph);ps.lineTo(-pw+ch,ph);ps.lineTo(-pw,ph-ch);ps.lineTo(-pw,-ph+ch);ps.closePath();
  const plate=new T.Mesh(new T.ExtrudeGeometry(ps,{depth:1.6,bevelEnabled:true,bevelThickness:.3,bevelSize:.3,bevelSegments:2}),M.bronzeDark());plate.position.z=-1.9;root.add(plate);
  const pe=ps.getPoints().map(p=>new T.Vector3(p.x,p.y,0));pe.push(pe[0].clone());
  root.add(new T.Line(new T.BufferGeometry().setFromPoints(pe.map(p=>p.clone().multiplyScalar(1.006).setZ(.03))),M.line(K.HEX.gold,.85)));
  root.add(new T.Line(new T.BufferGeometry().setFromPoints(pe.map(p=>new T.Vector3(p.x*.95,p.y*.965,.03))),M.line(K.HEX.gold,.25)));
  [[-pw+2,ph-2],[pw-2,ph-2],[-pw+2,-ph+2],[pw-2,-ph+2]].forEach(([x,y])=>{const d=new T.Mesh(new T.OctahedronGeometry(.6,0),M.gold());d.position.set(x,y,.2);d.scale.z=.4;root.add(d)});
  const base=new T.Mesh(K.prism(K.hexShape(16,0),2.4,.3),M.bronze());base.position.set(0,-ph-4.6,-1);root.add(base);
  const neck=new T.Mesh(new T.CylinderGeometry(3.2,4.2,2.2,6),M.bronze());neck.position.set(0,-ph-1.4,-1);root.add(neck);
  const be=new T.LineLoop(K.loopXZ(K.hexPts(16.2,0),0),M.line(K.HEX.gold,.8));be.position.set(0,-ph-1.88,-1);root.add(be);
  const hal=K.halo(70,.7);hal.position.set(0,-ph-4.8,-1);root.add(hal);
  const stars=K.starfield(500,240,420,41);stars.material.uniforms.uFade.value=.5;scene.add(stars);
  // grooves for all 36 channels, engraved in the plate
  const groove=[];for(const key of Object.keys(CHANNELS)){const [a,b]=key.split('-').map(Number),[x1,y1]=gxy(a),[x2,y2]=gxy(b);groove.push(new T.Vector3(X(x1),Y(y1),.05),new T.Vector3(X(x2),Y(y2),.05))}
  root.add(new T.LineSegments(new T.BufferGeometry().setFromPoints(groove),M.line(K.HEX.gold,.16)));
  // wire tubes, one per channel, shown when lit
  const wm={natal:M.gold({emissiveIntensity:.35,roughness:.25}),bridge:M.glass(K.HEX.emerald,{emissiveIntensity:.7}),transit:M.glass(K.HEX.violet,{emissiveIntensity:.6,opacity:.85})};
  const wires={};for(const key of Object.keys(CHANNELS)){const [a,b]=key.split('-').map(Number),[x1,y1]=gxy(a),[x2,y2]=gxy(b);
    const p1=new T.Vector3(X(x1),Y(y1),.45),p2=new T.Vector3(X(x2),Y(y2),.45),mid=p1.clone().add(p2).multiplyScalar(.5).setZ(.75);
    const g=new T.TubeGeometry(new T.QuadraticBezierCurve3(p1,mid,p2),48,.26,10,false);const m=new T.Mesh(g,wm.natal);m.visible=false;root.add(m);wires[key]={m,g,type:null,full:g.index.count}}
  // sockets
  const sockets={};const depth=1.4;
  function centerShape(c){const [cx,cy]=CPOS[c];const s=new T.Shape();if(LOOM_RECT[c]){const [w,h]=LOOM_RECT[c],r=10;const x0=X(cx-w/2),x1=X(cx+w/2),y0=Y(cy+h/2),y1=Y(cy-h/2),rr=r*k;s.moveTo(x0+rr,y0);s.lineTo(x1-rr,y0);s.quadraticCurveTo(x1,y0,x1,y0+rr);s.lineTo(x1,y1-rr);s.quadraticCurveTo(x1,y1,x1-rr,y1);s.lineTo(x0+rr,y1);s.quadraticCurveTo(x0,y1,x0,y1-rr);s.lineTo(x0,y0+rr);s.quadraticCurveTo(x0,y0,x0+rr,y0)}
    else LOOM_SHAPES[c].forEach(([dx,dy],i)=>{const x=X(cx+dx),y=Y(cy+dy);i?s.lineTo(x,y):s.moveTo(x,y)});s.closePath();return s}
  const sm={open:M.bronzeDark({color:K.col(0x2a2418),roughness:.75}),natal:M.bronze({emissive:K.col(0x3a2a08),emissiveIntensity:.6}),transit:M.glass(K.HEX.emerald,{emissiveIntensity:.45,opacity:.9})};
  for(const c of Object.keys(CPOS)){const sh=centerShape(c);
    const body=new T.Mesh(new T.ExtrudeGeometry(sh,{depth,bevelEnabled:true,bevelThickness:.25,bevelSize:.22,bevelSegments:3,curveSegments:8}),sm.open);body.position.z=.1;body.userData={center:c};S.pick(body);root.add(body);
    const pts=sh.getPoints(8).map(p=>new T.Vector3(p.x,p.y,depth+.42));pts.push(pts[0].clone());
    const rim=new T.Line(new T.BufferGeometry().setFromPoints(pts),M.line(K.HEX.silver,.5));root.add(rim);
    const lab=S.track(K.label(c==='SolarPlexus'?'SOLAR PLEXUS':c.toUpperCase(),{font:'mono',px:8,track:.18,color:'#8A9BA8',opacity:.8}));const [cx,cy]=CPOS[c];lab.position.set(X(cx)+(c==='Spleen'?-5.6:c==='SolarPlexus'?5.6:c==='Heart'?4.2:0),Y(cy)+(['Spleen','SolarPlexus','Heart'].includes(c)?-5.4:c==='Head'?-4.6:c==='Root'||c==='Sacral'?0:0),depth+.6);
    if(['Throat','Sacral','Root','G','Ajna'].includes(c)){lab.position.x=X(cx)+(c==='G'?7.4:c==='Ajna'?6:7.4)*(c==='Throat'||c==='Sacral'||c==='Root'?-1:1)}
    root.add(lab);sockets[c]={body,rim,lab}}
  // gate pins
  const pinG=new T.CylinderGeometry(.62,.62,.5,20);pinG.rotateX(Math.PI/2);
  const pm={off:M.bronzeDark({color:K.col(0x1d1a14)}),natal:M.gold({emissiveIntensity:.3}),transit:M.glass(K.HEX.emerald,{emissiveIntensity:.6}),both:M.gold({emissive:K.col(K.HEX.emerald),emissiveIntensity:.5})};
  const pins={};for(const g of Object.keys(GOFF).map(Number)){const [x,y]=gxy(g);const p=new T.Mesh(pinG,pm.off);p.position.set(X(x),Y(y),depth+.55);p.userData={gate:g};S.pick(p);root.add(p);
    const l=S.track(K.label(String(g),{font:'mono',px:8,color:'#AEBBC4'}));l.position.set(X(x),Y(y),depth+1.05);l.renderOrder=12;root.add(l);pins[g]={p,l,state:null}}
  function apply(r,animate){cur=r;const st=loomState(r);
    for(const key in wires){const w=wires[key],t=st.ch[key]||null;if(t!==w.type){const was=w.type;w.type=t;if(!t){w.m.visible=false;continue}w.m.material=wm[t];w.m.visible=true;
        if(animate&&!RM&&!was){w.g.setDrawRange(0,0);S.tween(900,kk=>{w.g.setDrawRange(0,Math.floor(w.full*kk/6)*6)})}else w.g.setDrawRange(0,Infinity)}}
    for(const c in sockets){const nat=NATAL_CEN.has(c),d=st.cen.has(c);const s=sockets[c];s.body.material=nat?sm.natal:d?sm.transit:sm.open;s.rim.material.color=K.col(nat?K.HEX.gold:d?K.HEX.emerald:K.HEX.silver);s.rim.material.opacity=nat||d?.95:.45;
      s.body.scale.z=d?1:.55}
    for(const g in pins){const n=NATG.has(+g),t=st.tg.has(+g);const p=pins[g];const key=n&&t?'both':n?'natal':t?'transit':'off';if(p.state!==key){p.state=key;p.p.material=pm[key];p.p.scale.z=key==='off'?.6:1.5;
        const old=p.l.material.map;const nl=K.label(String(g),{font:'mono',px:8,color:key==='off'?'#75838E':key==='natal'?'#070B1D':'#F0EDE3'});p.l.material.map=nl.material.map;nl.material.dispose();old&&old.dispose()}}
    S.invalidate()}
  // overlays: today's planets at their gates; the eleven Gene Keys spheres at their natal gates
  const ov=new T.Group();root.add(ov);let ovOpt={planets:false,keys:false};
  function overlays(){while(ov.children.length){const c=ov.children.pop();if(c.material){if(c.material.map)c.material.map.dispose();c.material.dispose()}}
    if(ovOpt.planets){const by={};tPos(cur).forEach(t=>{(by[t.gate]=by[t.gate]||[]).push(SYM[t.body])});
      for(const g in by){if(!GOFF[g])continue;const [x,y]=gxy(+g);const l=S.track(K.label(by[g].join(''),{font:'mono',px:12,color:'#E6B84D',bg:'rgba(7,11,29,.78)'}));l.center.set(.5,-.55);l.position.set(X(x),Y(y),depth+1.4);l.renderOrder=14;ov.add(l)}}
    if(ovOpt.keys){const tg=new Set(tPos(cur).filter(t=>t.body!=='Moon').map(t=>t.gate));const by={};
      SPHERES.forEach(sp=>{const n=NAT.find(x=>x.side===sp.src[0]&&x.body===sp.src[1]);if(!n||!GOFF[n.gate])return;(by[n.gate]=by[n.gate]||{n,names:[]}).names.push(sp.name)});
      let alt=0;const placed=[];Object.values(by).sort((a,b)=>gxy(a.n.gate)[1]-gxy(b.n.gate)[1]).forEach(({n,names})=>{const [x,y]=gxy(n.gate);const hit=tg.has(n.gate);const side=x<196?'L':x>204?'R':(alt++%2?'L':'R');
        let yy=Y(y);while(placed.some(p=>p.side===side&&Math.abs(p.y-yy)<1.5))yy-=1.6;placed.push({side,y:yy});
        const l=S.track(K.label(names.map(s=>s.toUpperCase()).join(' · ')+' '+n.gate,{font:'mono',px:8,track:.1,color:hit?'#67D4C7':'#C2CBD1',bg:'rgba(7,11,29,.82)'}));
        l.center.set(side==='L'?1.12:-.12,.5);l.position.set(X(x),yy,depth+1.4);l.renderOrder=13;ov.add(l)})}
    S.invalidate()}
  apply(r0,false);
  // intro: the plate rises, sockets seat, then the wires light
  if(!RM){S.view.dist=fit()*1.4;S.view.pitch=.35;S.place();root.position.y=-6;
    S.tween(1500,kk=>{S.view.dist=fit()*(1.4-.4*kk);S.view.pitch=.35+(.1-.35)*kk;S.place();root.position.y=-6*(1-kk)});
    Object.values(wires).forEach((w,i)=>{if(w.type){w.g.setDrawRange(0,0);S.tween(900+i*30,kk=>{w.g.setDrawRange(0,Math.floor(w.full*kk/6)*6)})}})}
  else{S.view.dist=fit();S.place()}
  return Object.assign(S,{update(r){apply(r,true);overlays()},overlay(o){ovOpt=Object.assign(ovOpt,o);overlays()}});
}
