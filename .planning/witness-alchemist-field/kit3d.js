/* ---------- kit3d: one world for every 3D surface in the field ----------
   Render Bible: void ground, one warm key from the upper left, a thin gold rim from behind right,
   dark matte bronze with gold as line, one glass per object carrying evidence state, a hex plinth.
   One curve: cubic-bezier(0.19,1,0.22,1). Everything collapses to static under reduced motion. */
const K3=(()=>{
  const HEX={void:0x070B1D,surface:0x0E1428,gold:0xC5A017,goldWarm:0xE6B84D,emerald:0x10B5A7,violet:0x7B68EE,terracotta:0xC65D3B,bronze:0x5A4A2E,iron:0x1A1A24,parchment:0xF0EDE3,silver:0x8A9BA8,indigo:0x0B50FB,dis:0x75838E};
  const hasTHREE=()=>typeof THREE!=='undefined';
  const col=h=>new THREE.Color(h).convertSRGBToLinear();
  function bezier(p1x,p1y,p2x,p2y){const cx=3*p1x,bx=3*(p2x-p1x)-cx,ax=1-cx-bx,cy=3*p1y,by=3*(p2y-p1y)-cy,ay=1-cy-by;
    const sx=t=>((ax*t+bx)*t+cx)*t,sy=t=>((ay*t+by)*t+cy)*t,dx=t=>(3*ax*t+2*bx)*t+cx;
    return x=>{if(x<=0)return 0;if(x>=1)return 1;let t=x;for(let i=0;i<8;i++){const e=sx(t)-x;if(Math.abs(e)<1e-6)break;const d=dx(t);if(Math.abs(d)<1e-6)break;t-=e/d}
      if(t<0||t>1||Math.abs(sx(t)-x)>1e-4){let lo=0,hi=1;t=x;for(let i=0;i<24;i++){if(sx(t)<x)lo=t;else hi=t;t=(lo+hi)/2}}return sy(t)}}
  const EASE=bezier(.19,1,.22,1);
  function reduced(){try{return !!(window.matchMedia&&window.matchMedia('(prefers-reduced-motion: reduce)').matches)}catch(e){return false}}
  function lowPower(){const n=navigator;return !!((n.hardwareConcurrency&&n.hardwareConcurrency<=2)||(n.deviceMemory&&n.deviceMemory<=2)||(n.connection&&n.connection.saveData))}
  let _gl=null;function webgl(){if(_gl!==null)return _gl;try{const c=document.createElement('canvas');_gl=!!(window.WebGLRenderingContext&&(c.getContext('webgl')||c.getContext('experimental-webgl')))}catch(e){_gl=false}return _gl}
  function ok(){return hasTHREE()&&webgl()&&!lowPower()}

  /* ---- procedural textures (cached) ---- */
  const TEX={};
  function canvasTex(key,w,h,draw,opt){if(TEX[key])return TEX[key];const c=document.createElement('canvas');c.width=w;c.height=h;draw(c.getContext('2d'),w,h);const t=new THREE.CanvasTexture(c);t.encoding=THREE.sRGBEncoding;if(opt&&opt.repeat){t.wrapS=t.wrapT=THREE.RepeatWrapping}t.anisotropy=4;TEX[key]=t;return t}
  function rand(seed){let s=seed>>>0;return()=>{s=(s*1664525+1013904223)>>>0;return s/4294967296}}
  // dark patinated bronze: mottled base, faint brushed grain, verdigris only in specks
  const patina=()=>canvasTex('patina',512,512,(g,w,h)=>{const R=rand(7);g.fillStyle='#3a2f1f';g.fillRect(0,0,w,h);
    for(let i=0;i<2600;i++){const x=R()*w,y=R()*h,r=2+R()*16,a=.04+R()*.08;const v=R();g.fillStyle=v<.5?`rgba(20,15,9,${a})`:`rgba(92,74,44,${a})`;g.beginPath();g.arc(x,y,r,0,7);g.fill()}
    g.globalAlpha=.06;for(let i=0;i<500;i++){const y=R()*h;g.strokeStyle=R()<.5?'#1c160e':'#6b5534';g.lineWidth=R()*1.2;g.beginPath();g.moveTo(0,y);g.lineTo(w,y+(R()-.5)*8);g.stroke()}
    g.globalAlpha=1;for(let i=0;i<70;i++){const x=R()*w,y=R()*h,r=1+R()*5;g.fillStyle=`rgba(46,${110+R()*40|0},${96+R()*30|0},${.12+R()*.18})`;g.beginPath();g.arc(x,y,r,0,7);g.fill()}},{repeat:true});
  const radial=(key,stops)=>canvasTex(key,256,256,(g,w,h)=>{const gr=g.createRadialGradient(w/2,h/2,0,w/2,h/2,w/2);stops.forEach(([o,c])=>gr.addColorStop(o,c));g.fillStyle=gr;g.fillRect(0,0,w,h)});
  const glowTex=()=>radial('glow',[[0,'rgba(255,255,255,1)'],[.18,'rgba(255,255,255,.55)'],[.45,'rgba(255,255,255,.12)'],[1,'rgba(255,255,255,0)']]);
  const haloTex=()=>radial('halo',[[0,'rgba(230,184,77,.55)'],[.35,'rgba(230,184,77,.18)'],[.7,'rgba(197,160,23,.05)'],[1,'rgba(0,0,0,0)']]);

  /* ---- environment: the studio the bronze reflects (void room, one warm key, one gold rim) ---- */
  function envFor(renderer){if(renderer.__env)return renderer.__env;const pm=new THREE.PMREMGenerator(renderer);const s=new THREE.Scene();
    s.add(new THREE.Mesh(new THREE.BoxGeometry(40,40,40),new THREE.MeshBasicMaterial({color:col(0x05070f),side:THREE.BackSide})));
    const key=new THREE.Mesh(new THREE.PlaneGeometry(14,9),new THREE.MeshBasicMaterial({color:col(0xffd9a0).multiplyScalar(2.2),side:THREE.DoubleSide}));key.position.set(-12,14,8);key.lookAt(0,0,0);s.add(key);
    const rim=new THREE.Mesh(new THREE.PlaneGeometry(18,2.2),new THREE.MeshBasicMaterial({color:col(0xE6B84D).multiplyScalar(1.4),side:THREE.DoubleSide}));rim.position.set(14,6,-14);rim.lookAt(0,0,0);s.add(rim);
    const fill=new THREE.Mesh(new THREE.PlaneGeometry(30,30),new THREE.MeshBasicMaterial({color:col(0x0E1428),side:THREE.DoubleSide}));fill.position.set(0,-16,0);fill.rotation.x=Math.PI/2;s.add(fill);
    const rt=pm.fromScene(s,.035);pm.dispose();s.traverse(o=>{if(o.geometry)o.geometry.dispose();if(o.material)o.material.dispose()});renderer.__env=rt.texture;return rt.texture}

  /* ---- materials ---- */
  function mats(env){return {
    bronze:(o)=>new THREE.MeshStandardMaterial(Object.assign({color:col(0x8a7350),map:patina(),metalness:.82,roughness:.52,envMap:env,envMapIntensity:.95},o||{})),
    bronzeDark:(o)=>new THREE.MeshStandardMaterial(Object.assign({color:col(0x4d3f29),map:patina(),metalness:.78,roughness:.62,envMap:env,envMapIntensity:.6},o||{})),
    gold:(o)=>new THREE.MeshStandardMaterial(Object.assign({color:col(HEX.gold),metalness:1,roughness:.3,envMap:env,envMapIntensity:1.25,emissive:col(HEX.gold),emissiveIntensity:.08},o||{})),
    glass:(h,o)=>new THREE.MeshStandardMaterial(Object.assign({color:col(h).multiplyScalar(.32),emissive:col(h).multiplyScalar(.85),emissiveIntensity:.3,metalness:.05,roughness:.03,envMap:env,envMapIntensity:3.2,transparent:true,opacity:.93},o||{})),
    line:(h,op)=>new THREE.LineBasicMaterial({color:col(h),transparent:true,opacity:op==null?1:op,toneMapped:false,depthWrite:false}),
    flat:(h,op)=>new THREE.MeshBasicMaterial({color:col(h),transparent:op!=null&&op<1,opacity:op==null?1:op,toneMapped:false}),
  }}

  /* ---- geometry helpers ---- */
  // flat-topped hexagon of circumradius r (vertex on +x)
  function hexPts(r,rot){const a0=rot||0;return [...Array(6)].map((_,i)=>{const a=a0+i*Math.PI/3;return [r*Math.cos(a),r*Math.sin(a)]})}
  function hexShape(r,rot){const s=new THREE.Shape();hexPts(r,rot).forEach(([x,y],i)=>i?s.lineTo(x,y):s.moveTo(x,y));s.closePath();return s}
  // an extruded prism lying in XZ, top face at y=h
  function prism(shape,h,bevel){const g=new THREE.ExtrudeGeometry(shape,{depth:h,bevelEnabled:!!bevel,bevelThickness:bevel||0,bevelSize:bevel||0,bevelSegments:2,curveSegments:24});g.rotateX(-Math.PI/2);g.computeVertexNormals();return g}
  function loopXZ(pts,y){return new THREE.BufferGeometry().setFromPoints([...pts,pts[0]].map(([x,z])=>new THREE.Vector3(x,y,-z)))}
  function circleXZ(r,y,n){const p=[];const N=n||128;for(let i=0;i<=N;i++){const a=i/N*Math.PI*2;p.push(new THREE.Vector3(r*Math.cos(a),y,-r*Math.sin(a)))}return new THREE.BufferGeometry().setFromPoints(p)}
  function arcXZ(r,y,a0,a1,n){const p=[];const N=n||Math.max(8,Math.ceil(Math.abs(a1-a0)/2));for(let i=0;i<=N;i++){const a=(a0+(a1-a0)*i/N)*Math.PI/180;p.push(new THREE.Vector3(r*Math.cos(a),y,-r*Math.sin(a)))}return new THREE.BufferGeometry().setFromPoints(p)}
  const P=(lonDeg,r,y)=>{const a=lonDeg*Math.PI/180;return new THREE.Vector3(r*Math.cos(a),y||0,-r*Math.sin(a))};
  // annulus band (ring of bronze) lying in XZ
  function band(r0,r1,h,seg){const s=new THREE.Shape();s.absarc(0,0,r1,0,Math.PI*2,false);const hole=new THREE.Path();hole.absarc(0,0,r0,0,Math.PI*2,true);s.holes.push(hole);const g=new THREE.ExtrudeGeometry(s,{depth:h,bevelEnabled:true,bevelThickness:.08,bevelSize:.08,bevelSegments:2,curveSegments:seg||160});g.rotateX(-Math.PI/2);return g}

  /* ---- labels: crisp canvas sprites held at a constant pixel size ---- */
  const FONT={mono:'"JetBrains Mono","SF Mono",ui-monospace,monospace',disp:'Unbounded,Panchang,sans-serif',engr:'Cinzel,"Trajan Pro",serif',sans:'Manrope,Satoshi,system-ui,sans-serif'};
  function textCanvas(text,o){const px=(o.px||12)*2,pad=Math.ceil(px*.35);const c=document.createElement('canvas'),g=c.getContext('2d');const font=`${o.weight||500} ${px}px ${FONT[o.font||'mono']}`;g.font=font;
    const ls=(o.track||0)*px;const w=Math.ceil([...text].reduce((a,ch)=>a+g.measureText(ch).width+ls,0)+pad*2),h=Math.ceil(px*1.5+pad);c.width=Math.max(4,w);c.height=h;g.font=font;g.textBaseline='middle';
    if(o.bg){g.fillStyle=o.bg;const r=h/2;g.beginPath();g.moveTo(r,0);g.lineTo(c.width-r,0);g.arc(c.width-r,r,r,-Math.PI/2,Math.PI/2);g.lineTo(r,h);g.arc(r,r,r,Math.PI/2,Math.PI*1.5);g.fill()}
    g.fillStyle=o.color||'#F0EDE3';let x=pad;for(const ch of text){g.fillText(ch,x,h/2+1);x+=g.measureText(ch).width+ls}return c}
  function label(text,o){o=o||{};const c=textCanvas(text,o);const t=new THREE.CanvasTexture(c);t.encoding=THREE.sRGBEncoding;t.minFilter=THREE.LinearFilter;t.generateMipmaps=false;
    const m=new THREE.SpriteMaterial({map:t,transparent:true,depthWrite:false,depthTest:o.depthTest!==false,toneMapped:false,sizeAttenuation:false,opacity:o.opacity==null?1:o.opacity});
    const s=new THREE.Sprite(m);s.userData.label={px:c.height/2,aspect:c.width/c.height};s.center.set(o.cx==null?.5:o.cx,o.cy==null?.5:o.cy);s.renderOrder=o.order||10;return s}
  // engraved flat text lying on a surface (rotates with the object)
  function engrave(text,o){o=o||{};const c=textCanvas(text,Object.assign({px:28},o));const t=new THREE.CanvasTexture(c);t.encoding=THREE.sRGBEncoding;t.anisotropy=8;
    const hgt=o.height||1,w=hgt*c.width/c.height;const m=new THREE.Mesh(new THREE.PlaneGeometry(w,hgt),new THREE.MeshBasicMaterial({map:t,transparent:true,depthWrite:false,toneMapped:false,opacity:o.opacity==null?1:o.opacity,side:THREE.DoubleSide}));m.rotation.x=-Math.PI/2;m.renderOrder=o.order||5;return m}
  function glow(h,size,op){const m=new THREE.SpriteMaterial({map:glowTex(),color:col(h),transparent:true,opacity:op==null?.6:op,blending:THREE.AdditiveBlending,depthWrite:false,toneMapped:false});const s=new THREE.Sprite(m);s.scale.set(size,size,1);s.renderOrder=3;return s}
  function halo(size,op){const m=new THREE.Mesh(new THREE.PlaneGeometry(size,size),new THREE.MeshBasicMaterial({map:haloTex(),transparent:true,opacity:op==null?.9:op,blending:THREE.AdditiveBlending,depthWrite:false,toneMapped:false}));m.rotation.x=-Math.PI/2;m.renderOrder=1;return m}

  /* ---- starfield: soft round points that breathe in once, a few twinkle ---- */
  function starfield(n,r0,r1,seed){const R=rand(seed||11),pos=new Float32Array(n*3),sz=new Float32Array(n),ph=new Float32Array(n),tint=new Float32Array(n*3);
    const tints=[[.94,.93,.89],[.77,.63,.09],[.54,.61,.66],[.9,.72,.3]];
    for(let i=0;i<n;i++){const u=R()*2-1,th=R()*Math.PI*2,s=Math.sqrt(1-u*u),r=r0+R()*(r1-r0);pos[i*3]=r*s*Math.cos(th);pos[i*3+1]=r*u*.85+r*.1;pos[i*3+2]=r*s*Math.sin(th);
      sz[i]=Math.pow(R(),3)*3.2+.6;ph[i]=R()*6.28;const tt=tints[R()<.08?1:R()<.12?3:R()<.45?2:0];tint.set(tt,i*3)}
    const g=new THREE.BufferGeometry();g.setAttribute('position',new THREE.BufferAttribute(pos,3));g.setAttribute('size',new THREE.BufferAttribute(sz,1));g.setAttribute('phase',new THREE.BufferAttribute(ph,1));g.setAttribute('tint',new THREE.BufferAttribute(tint,3));
    const m=new THREE.ShaderMaterial({transparent:true,depthWrite:false,blending:THREE.AdditiveBlending,uniforms:{uTime:{value:0},uFade:{value:0},uPx:{value:Math.min(window.devicePixelRatio||1,2)}},
      vertexShader:`attribute float size;attribute float phase;attribute vec3 tint;uniform float uTime;uniform float uPx;varying float vA;varying vec3 vT;
        void main(){vec4 mv=modelViewMatrix*vec4(position,1.);gl_Position=projectionMatrix*mv;float tw=.75+.25*sin(uTime*.6+phase*3.);vA=tw;vT=tint;gl_PointSize=size*uPx*(260./-mv.z);}`,
      fragmentShader:`uniform float uFade;varying float vA;varying vec3 vT;void main(){vec2 c=gl_PointCoord-.5;float d=length(c);if(d>.5)discard;float a=smoothstep(.5,0.,d);a=a*a;gl_FragColor=vec4(vT,a*vA*uFade);}`});
    const p=new THREE.Points(g,m);p.renderOrder=0;p.frustumCulled=false;return p}

  /* ---- the stage: renderer, camera, orbit, picking, tweens, render-on-demand ---- */
  function stage(host,opt){
    opt=Object.assign({fov:34,yaw:-Math.PI/2,pitch:.6,dist:90,target:[0,0,0],minDist:30,maxDist:180,minPitch:.08,maxPitch:1.45,orbit:true,pitchDrag:true,tilt:0,ambient:null,exposure:1.15,label:'3D view'},opt||{});
    const RM=reduced();
    const renderer=new THREE.WebGLRenderer({antialias:true,alpha:false,powerPreference:'high-performance'});
    renderer.setPixelRatio(Math.min(window.devicePixelRatio||1,2));renderer.outputEncoding=THREE.sRGBEncoding;renderer.toneMapping=THREE.ACESFilmicToneMapping;renderer.toneMappingExposure=opt.exposure;renderer.setClearColor(col(HEX.void),1);
    const el=renderer.domElement;el.className='k3';el.tabIndex=0;el.setAttribute('role','img');el.setAttribute('aria-label',opt.label);
    const env=envFor(renderer),M=mats(env);
    const scene=new THREE.Scene();scene.fog=new THREE.Fog(col(HEX.void),opt.fogNear||opt.maxDist*1.1,opt.fogFar||opt.maxDist*2.4);
    const camera=new THREE.PerspectiveCamera(opt.fov,1,.1,2000);
    const key=new THREE.DirectionalLight(col(0xffe0b0),2.4);key.position.set(-60,80,45);scene.add(key);
    const rim=new THREE.DirectionalLight(col(HEX.goldWarm),1.35);rim.position.set(70,35,-80);scene.add(rim);
    scene.add(new THREE.HemisphereLight(col(0x1a2240),col(0x070B1D),.35));
    const view={yaw:opt.yaw,pitch:opt.pitch,dist:opt.dist,target:new THREE.Vector3(...opt.target)};
    const base={yaw:view.yaw,pitch:view.pitch};const tiltCur={x:0,y:0},tiltGoal={x:0,y:0};
    function place(){const p=view.pitch+tiltCur.y,y=view.yaw+tiltCur.x;camera.position.set(view.target.x+view.dist*Math.cos(p)*Math.cos(y),view.target.y+view.dist*Math.sin(p),view.target.z+view.dist*Math.cos(p)*Math.sin(y));camera.lookAt(view.target)}
    // labels at constant pixel size
    let W=1,H=1;const labels=new Set();
    function sizeLabel(s){const u=s.userData.label;if(!u)return;const k=2*Math.tan(camera.fov*Math.PI/360)/H;s.scale.set(u.px*u.aspect*k,u.px*k,1)}
    function track(s){if(s.userData.label){labels.add(s);sizeLabel(s)}s.traverse&&s.traverse(o=>{if(o!==s&&o.userData&&o.userData.label){labels.add(o);sizeLabel(o)}});return s}
    function resize(){const w=host.clientWidth||600,h=host.clientHeight||400;if(w===W&&h===H)return;W=w;H=h;renderer.setSize(w,h,false);camera.aspect=w/h;camera.updateProjectionMatrix();if(opt.onResize)opt.onResize(w,h);labels.forEach(sizeLabel);invalidate()}
    // tweens
    const tweens=new Set();
    function tween(ms,fn,done,ease){if(RM||!ms){fn(1);done&&done();invalidate();return {stop(){}}}const t={t0:performance.now(),ms,fn,done,ease:ease||EASE,dead:false,stop(){t.dead=true;tweens.delete(t)}};tweens.add(t);kick();return t}
    function setView(v,ms){const from={yaw:view.yaw,pitch:view.pitch,dist:view.dist,tx:view.target.x,ty:view.target.y,tz:view.target.z};const to=Object.assign({},from,v);if(v.target){to.tx=v.target[0];to.ty=v.target[1];to.tz=v.target[2]}
      let dy=to.yaw-from.yaw;dy=Math.atan2(Math.sin(dy),Math.cos(dy));to.yaw=from.yaw+dy;
      return tween(ms==null?900:ms,k=>{view.yaw=from.yaw+(to.yaw-from.yaw)*k;view.pitch=from.pitch+(to.pitch-from.pitch)*k;view.dist=from.dist+(to.dist-from.dist)*k;view.target.set(from.tx+(to.tx-from.tx)*k,from.ty+(to.ty-from.ty)*k,from.tz+(to.tz-from.tz)*k);base.yaw=view.yaw;base.pitch=view.pitch;place()})}
    // loop: render on demand, ambient motion capped near 30fps, paused offscreen or hidden
    let raf=0,alive=true,visible=true,dirty=true,interacting=false,last=0,lastAmb=0;const frames=new Set();
    function invalidate(){dirty=true;kick()}
    function kick(){if(!raf&&alive&&visible&&!document.hidden)raf=requestAnimationFrame(tick)}
    function tick(now){raf=0;if(!alive)return;const dt=Math.min(.1,(now-(last||now))/1000);last=now;let more=false;
      for(const t of [...tweens]){const k=Math.min(1,(now-t.t0)/t.ms);t.fn(t.ease(k));if(k>=1){tweens.delete(t);t.done&&t.done()}else more=true;dirty=true}
      if(opt.tilt){const fx=tiltCur.x+(tiltGoal.x-tiltCur.x)*Math.min(1,dt*6),fy=tiltCur.y+(tiltGoal.y-tiltCur.y)*Math.min(1,dt*6);if(Math.abs(fx-tiltCur.x)+Math.abs(fy-tiltCur.y)>1e-5){tiltCur.x=fx;tiltCur.y=fy;place();dirty=true;more=true}}
      for(const f of frames){if(f(dt,now)){more=true;dirty=true}}
      const amb=!RM&&opt.ambient&&!interacting;
      if(amb&&now-lastAmb>=32){lastAmb=now;if(opt.ambient(dt,now,view)!==false){place();dirty=true}}
      if(dirty){renderer.render(scene,camera);dirty=false}
      if(more||interacting||amb)kick()}
    const io='IntersectionObserver' in window?new IntersectionObserver(es=>{visible=es[0].isIntersecting;if(visible)kick()}):null;
    const onVis=()=>{if(!document.hidden)kick()};document.addEventListener('visibilitychange',onVis);
    let ro=null;function observe(h){if(ro)ro.disconnect();if(io){io.disconnect();io.observe(h)}ro=new ResizeObserver(()=>resize());ro.observe(h)}
    function attach(h){host=h;h.appendChild(el);observe(h);W=H=0;resize();invalidate()}
    // picking
    const ray=new THREE.Raycaster();ray.camera=camera;const ndc=new THREE.Vector2();const pickables=[];
    function pickAt(cx,cy){const r=el.getBoundingClientRect();ndc.set(((cx-r.left)/r.width)*2-1,-((cy-r.top)/r.height)*2+1);ray.setFromCamera(ndc,camera);const hits=ray.intersectObjects(pickables,true);for(const h of hits){let o=h.object;while(o&&!o.userData.pick)o=o.parent;if(o&&o.visible!==false)return o}return null}
    // pointer: drag to orbit, pinch or ctrl-wheel to zoom, tap to pick, hover to highlight
    const pts=new Map();let drag=null,pinch=null,hoverObj=null,hoverQ=null;
    el.addEventListener('pointerdown',e=>{pts.set(e.pointerId,{x:e.clientX,y:e.clientY});if(pts.size===1){drag={x:e.clientX,y:e.clientY,yaw:view.yaw,pitch:view.pitch,moved:false,type:e.pointerType}}
      else if(pts.size===2){const a=[...pts.values()];pinch={d:Math.hypot(a[0].x-a[1].x,a[0].y-a[1].y),dist:view.dist};drag&&(drag.moved=true)}
      interacting=true;opt.onInteract&&opt.onInteract();kick();try{el.setPointerCapture(e.pointerId)}catch(_){}});
    el.addEventListener('pointermove',e=>{if(pts.has(e.pointerId))pts.set(e.pointerId,{x:e.clientX,y:e.clientY});
      if(pinch&&pts.size===2){const a=[...pts.values()],d=Math.hypot(a[0].x-a[1].x,a[0].y-a[1].y);view.dist=Math.max(opt.minDist,Math.min(opt.maxDist,pinch.dist*pinch.d/Math.max(20,d)));place();invalidate();return}
      if(drag&&opt.orbit){const dx=e.clientX-drag.x,dy=e.clientY-drag.y;if(Math.abs(dx)+Math.abs(dy)>5)drag.moved=true;if(drag.moved){view.yaw=drag.yaw+dx*.0065;if(opt.yawClamp)view.yaw=Math.max(opt.yawClamp[0],Math.min(opt.yawClamp[1],view.yaw));if(opt.pitchDrag&&drag.type!=='touch')view.pitch=Math.max(opt.minPitch,Math.min(opt.maxPitch,drag.pitch+dy*.005));base.yaw=view.yaw;base.pitch=view.pitch;place();invalidate()}return}
      if(drag&&!opt.orbit){if(Math.abs(e.clientX-drag.x)+Math.abs(e.clientY-drag.y)>5)drag.moved=true}
      if(opt.tilt&&e.pointerType==='mouse'){const r=el.getBoundingClientRect();tiltGoal.x=((e.clientX-r.left)/r.width-.5)*opt.tilt;tiltGoal.y=-((e.clientY-r.top)/r.height-.5)*opt.tilt*.6;kick()}
      if(e.pointerType==='mouse'&&!drag&&opt.onHover){hoverQ=[e.clientX,e.clientY];if(!hoverQ.raf){hoverQ.raf=requestAnimationFrame(()=>{const q=hoverQ;hoverQ=null;if(!q)return;const o=pickAt(q[0],q[1]);el.style.cursor=o?'pointer':(opt.orbit?'grab':'default');if(o!==hoverObj){hoverObj=o;opt.onHover(o);invalidate()}})}}});
    const end=e=>{const d=drag,wasPinch=!!pinch;pts.delete(e.pointerId);if(pts.size<2)pinch=null;if(pts.size===0){drag=null;interacting=false;
      if(d&&!d.moved&&!wasPinch&&e.type==='pointerup'&&opt.onPick){const o=pickAt(e.clientX,e.clientY);opt.onPick(o,e)}}kick()};
    el.addEventListener('pointerup',end);el.addEventListener('pointercancel',end);
    el.addEventListener('pointerleave',e=>{if(e.pointerType==='mouse'){if(opt.tilt){tiltGoal.x=tiltGoal.y=0;kick()}if(hoverObj&&opt.onHover){hoverObj=null;opt.onHover(null);invalidate()}}});
    el.addEventListener('wheel',e=>{if(!e.ctrlKey&&!e.metaKey)return;e.preventDefault();view.dist=Math.max(opt.minDist,Math.min(opt.maxDist,view.dist*(1+Math.sign(e.deltaY)*.07)));place();invalidate()},{passive:false});
    el.addEventListener('keydown',e=>{if(opt.onKey&&opt.onKey(e)){e.preventDefault();return}
      const step=.12;if(e.key==='ArrowLeft'&&opt.orbit){view.yaw-=step}else if(e.key==='ArrowRight'&&opt.orbit){view.yaw+=step}else if(e.key==='ArrowUp'&&opt.orbit){view.pitch=Math.min(opt.maxPitch,view.pitch+step)}else if(e.key==='ArrowDown'&&opt.orbit){view.pitch=Math.max(opt.minPitch,view.pitch-step)}
      else if(e.key==='+'||e.key==='='){view.dist=Math.max(opt.minDist,view.dist*.92)}else if(e.key==='-'){view.dist=Math.min(opt.maxDist,view.dist*1.08)}else return;e.preventDefault();base.yaw=view.yaw;base.pitch=view.pitch;place();invalidate()});
    function fitDist(radius,pad){const v=camera.fov*Math.PI/360,hh=Math.atan(Math.tan(v)*camera.aspect);const a=Math.min(v,hh);return radius*(pad||1.08)/Math.sin(a)}
    function toScreen(v){const p=v.clone().project(camera);return {x:(p.x+1)/2*W,y:(1-p.y)/2*H,behind:p.z>1}}
    function dispose(){alive=false;if(raf)cancelAnimationFrame(raf);tweens.clear();frames.clear();document.removeEventListener('visibilitychange',onVis);ro&&ro.disconnect();io&&io.disconnect();
      scene.traverse(o=>{if(o.geometry)o.geometry.dispose();const ms=o.material?(Array.isArray(o.material)?o.material:[o.material]):[];ms.forEach(m=>{for(const k in m){const v=m[k];if(v&&v.isTexture&&!Object.values(TEX).includes(v))v.dispose()}m.dispose()})});
      renderer.dispose();try{renderer.forceContextLoss()}catch(_){}if(el.parentNode)el.parentNode.removeChild(el)}
    place();attach(host);
    return {THREE,scene,camera,renderer,el,M,view,base,place,setView,tween,invalidate,kick,onFrame:f=>{frames.add(f);kick();return ()=>frames.delete(f)},pick:o=>{o.userData.pick=o.userData.pick||true;pickables.push(o);return o},unpick:o=>{const i=pickables.indexOf(o);if(i>=0)pickables.splice(i,1)},track,attach,toScreen,fitDist,size:()=>({w:W,h:H}),dispose,reduced:RM};
  }

  /* ---- one live stage per room: survives same-room re-renders, disposed on room change ---- */
  const ST={room:null,inst:null};
  function mount(room,host,make){if(ST.inst&&ST.room===room&&ST.inst.attach){ST.inst.attach(host);return {inst:ST.inst,fresh:false}}release();ST.room=room;ST.inst=make(host);return {inst:ST.inst,fresh:true}}
  function release(){if(ST.inst){try{ST.inst.dispose()}catch(e){}ST.inst=null;ST.room=null}}
  function current(room){return ST.room===room?ST.inst:null}

  return {HEX,col,EASE,bezier,reduced,lowPower,webgl,ok,stage,mount,release,current,hexPts,hexShape,prism,loopXZ,circleXZ,arcXZ,band,P,label,engrave,glow,halo,starfield,patina,rand};
})();
