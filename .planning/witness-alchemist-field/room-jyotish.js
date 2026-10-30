/* ---------- Jyotish: sidereal chart, dasha drill-down, the day's Panchanga, Sade Sati, transit aspects ---------- */
const LAGNA_SID=222.117; // Scorpio 12°07' Lahiri, Swiss Ephemeris at birth (compute.py); not an engine figure
const LAGNA_SIGN=Math.floor(LAGNA_SID/30);
const VAB={Sun:'Su',Moon:'Mo',Mercury:'Me',Venus:'Ve',Mars:'Ma',Jupiter:'Ju',Saturn:'Sa',Uranus:'Ur',Neptune:'Ne',Pluto:'Pl',Rahu:'Ra',Ketu:'Ke',NorthNode:'Ra',SouthNode:'Ke',Asc:'As'};
const VNAME=b=>b==='NorthNode'?'Rahu':b==='SouthNode'?'Ketu':b;
const TITHI=['Pratipada','Dwitiya','Tritiya','Chaturthi','Panchami','Shashthi','Saptami','Ashtami','Navami','Dashami','Ekadashi','Dwadashi','Trayodashi','Chaturdashi'];
const YOGA=['Vishkambha','Priti','Ayushman','Saubhagya','Shobhana','Atiganda','Sukarma','Dhriti','Shula','Ganda','Vriddhi','Dhruva','Vyaghata','Harshana','Vajra','Siddhi','Vyatipata','Variyan','Parigha','Shiva','Siddha','Sadhya','Shubha','Shukla','Brahma','Indra','Vaidhriti'];
const KARANA=['Bava','Balava','Kaulava','Taitila','Garaja','Vanija','Vishti'];
const VARA=['Ravivara (Sunday)','Somavara (Monday)','Mangalavara (Tuesday)','Budhavara (Wednesday)','Guruvara (Thursday)','Shukravara (Friday)','Shanivara (Saturday)'];
const RASHI_LORD=['Mars','Venus','Mercury','Moon','Sun','Mercury','Venus','Mars','Jupiter','Saturn','Saturn','Jupiter'];
const JYPREF=(()=>{try{return Object.assign({layout:'south',show:'both'},JSON.parse(localStorage.getItem('wa-jy')||'{}'))}catch(e){return {layout:'south',show:'both'}}})();
const jyState={md:null,ad:null};
const houseOf=lon=>((Math.floor(norm(lon)/30)-LAGNA_SIGN+12)%12)+1;
const NATAL_SID=(()=>{try{return ENG.transits.result.natal_positions.map(p=>({body:p.planet,lon:p.longitude,sign:p.sign,deg:p.degree_in_sign,retro:p.is_retrograde,house:houseOf(p.longitude),nak:NAKS[Math.floor(p.longitude/(360/27))],pada:Math.floor((p.longitude%(360/27))/(360/108))+1}))}catch(e){return []}})();
function todaySid(r){const ayan=ayanOf(r);return tPos(r).filter(t=>t.body!=='Earth').map(t=>{const lon=norm(t.lon-ayan);return {body:VNAME(t.body),lon,sign:RASHI[Math.floor(lon/30)],deg:lon%30,retro:t.retro,house:houseOf(lon),nak:NAKS[Math.floor(lon/(360/27))],pada:Math.floor((lon%(360/27))/(360/108))+1,gate:t.gate,line:t.line}})}
function panchangaOf(r){
  const sun=r.p[0][3],moon=r.p[2][3],el=norm(moon-sun),ti=Math.floor(el/12),paksha=ti<15?'Shukla':'Krishna',tn=ti%15;
  const tithi=tn===14?(paksha==='Shukla'?'Purnima':'Amavasya'):TITHI[tn];
  const sum=norm(r.sid[0]+r.sid[1]),yoga=YOGA[Math.floor(sum/(360/27))];
  const kIdx=Math.floor(el/6);let karana;if(kIdx===0)karana='Kimstughna';else if(kIdx>=57)karana=['Shakuni','Chatushpada','Naga'][kIdx-57];else karana=KARANA[(kIdx-1)%7];
  const nak=NAKS[Math.floor(r.sid[0]/(360/27))],pada=Math.floor((r.sid[0]%(360/27))/(360/108))+1;
  const vara=VARA[new Date(r.d+'T00:00:00Z').getUTCDay()];
  return {tithi:`${tithi} (${paksha})`,tithiFrac:(el%12)/12,nak,pada,yoga,karana,vara,moonSign:RASHI[Math.floor(r.sid[0]/30)],sunSign:RASHI[Math.floor(r.sid[1]/30)]};
}
function sadeSati(satLon){const ms=5,ss=Math.floor(norm(satLon)/30),d=(ss-ms+12)%12;const phase=d===11?'First phase (Saturn in the 12th from the Moon)':d===0?'Second phase (Saturn on the Moon)':d===1?'Third phase (Saturn in the 2nd from the Moon)':null;return {active:!!phase,phase,saturnSign:RASHI[ss],moonSign:RASHI[ms],dhaiya:d===3?'Kantaka Shani (Saturn in the 4th from the Moon)':d===7?'Ashtama Shani (Saturn in the 8th from the Moon)':null}}

/* Chart drawings */
function chartPlanets(mode,r){const out=[];if(mode!=='today')NATAL_SID.forEach(p=>out.push({...p,kind:'natal'}));if(mode!=='natal')todaySid(r).forEach(p=>out.push({...p,kind:'transit'}));return out}
function glyphs(list,x,y,w){const n=list.filter(p=>p.body!=='Uranus'&&p.body!=='Neptune'&&p.body!=='Pluto');const per=Math.max(1,Math.floor(w/24));let s='';n.forEach((p,i)=>{const row=Math.floor(i/per),col=i%per,cx=x-((Math.min(n.length,per)-1)*24)/2+col*24,cy=y+row*15;s+=`<text class="pl" data-k="${p.kind}" data-b="${p.body}" x="${cx}" y="${cy}" font-size="11" text-anchor="middle" dominant-baseline="middle" fill="${p.kind==='natal'?'var(--gold-warm)':'var(--ev-computed)'}" style="cursor:pointer">${VAB[p.body]}${p.retro?'ʳ':''}</text>`});return s}
function southChart(list){
  const cell=100,pos={0:[1,0],1:[2,0],2:[3,0],3:[3,1],4:[3,2],5:[3,3],6:[2,3],7:[1,3],8:[0,3],9:[0,2],10:[0,1],11:[0,0]};
  let s=`<svg viewBox="0 0 400 400" role="img" aria-label="South Indian chart"><rect x="1" y="1" width="398" height="398" fill="none" stroke="var(--gold)" stroke-opacity=".6"/>`;
  for(let i=0;i<12;i++){const [cx,cy]=pos[i];const x=cx*cell,y=cy*cell;s+=`<rect x="${x}" y="${y}" width="${cell}" height="${cell}" fill="none" stroke="var(--gold)" stroke-opacity=".35"/><text x="${x+6}" y="${y+12}" font-size="8" fill="var(--text-3)">${RASHI[i].slice(0,3).toUpperCase()}</text>`;
    if(i===LAGNA_SIGN)s+=`<line x1="${x}" y1="${y+22}" x2="${x+22}" y2="${y}" stroke="var(--gold)" stroke-width="1.5"/>`;
    s+=glyphs(list.filter(p=>Math.floor(p.lon/30)===i),x+cell/2,y+cell/2-6,cell-10)}
  s+=`<text x="200" y="190" font-size="10" text-anchor="middle" fill="var(--gold)" letter-spacing="3">LAHIRI</text><text x="200" y="208" font-size="9" text-anchor="middle" fill="var(--text-3)">Lagna ${RASHI[LAGNA_SIGN]} ${fmtDeg(LAGNA_SID%30)}</text></svg>`;return s}
function northChart(list){
  const C=[[200,95],[100,45],[50,100],[100,200],[50,300],[100,355],[200,305],[300,355],[350,300],[300,200],[350,100],[300,45]];
  let s=`<svg viewBox="0 0 400 400" role="img" aria-label="North Indian chart"><rect x="1" y="1" width="398" height="398" fill="none" stroke="var(--gold)" stroke-opacity=".6"/><path d="M0 0L400 400M400 0L0 400M200 0L400 200L200 400L0 200Z" fill="none" stroke="var(--gold)" stroke-opacity=".4"/>`;
  for(let h=1;h<=12;h++){const sign=(LAGNA_SIGN+h-1)%12;const [x,y]=C[h-1];s+=`<text x="${x}" y="${y-22}" font-size="8" fill="var(--text-3)" text-anchor="middle">${sign+1} ${RASHI[sign].slice(0,3).toUpperCase()}</text>`;s+=glyphs(list.filter(p=>houseOf(p.lon)===h),x,y,70)}
  s+=`</svg>`;return s}

function showGraha(kind,body,r){
  const p=(kind==='natal'?NATAL_SID:todaySid(r)).find(x=>x.body===body);if(!p)return;
  const lord=RASHI_LORD[Math.floor(p.lon/30)],asp=(kind==='natal'?todaySid(r):NATAL_SID).map(o=>({o,a:aspectBetween(p.lon,o.lon)})).filter(x=>x.a).sort((a,b)=>a.a.orb-b.a.orb);
  openDrawer(`<div class="eyebrow">${kind==='natal'?'Birth · engine figure':fmtDate(r.d)+' · 06:00 IST · field figure'}</div><h2>${SYM[p.body]||''} ${p.body}${p.retro?' retrograde':''}</h2>
  <dl class="kv"><dt>Rashi</dt><dd><b>${p.sign} ${fmtDeg(p.deg)}</b> · lord ${lord}</dd><dt>Nakshatra</dt><dd>${p.nak} pada ${p.pada}</dd><dt>House</dt><dd>${p.house} from Lagna · ${HOUSE_KEY[p.house-1]}</dd><dt>From Moon</dt><dd>${((Math.floor(p.lon/30)-5+12)%12)+1}</dd>${p.gate?`<dt>Gate</dt><dd>${p.gate}.${p.line} ${GATES[p.gate][0]}</dd>`:''}</dl>
  ${asp.length?`<h3>${kind==='natal'?'Transits on it today':'On the birth chart'}</h3><div class="list">${asp.slice(0,6).map(x=>`<div class="item" style="cursor:default"><span class="k">${x.a.name}</span><span><b>${x.o.body} ${x.o.sign} ${fmtDeg(x.o.deg)}</b><small>Orb ${x.a.orb.toFixed(1)}°</small></span></div>`).join('')}</div>`:''}
  <p class="meta" style="margin-top:12px">${kind==='natal'?'Position from the Selemene transits engine (natal_positions), Lahiri.':'Tropical position from the Swiss Ephemeris less the day\'s Lahiri ayanamsa ('+ayanOf(r).toFixed(2)+'°).'}</p>`);
}

function dashaPanel(){
  const V=ENG.vimshottari.result,T=V.timeline.mahadashas,cur=V.current_period,today=state.date;
  const mdKey=m=>m.planet+m.start_date.slice(0,10);
  if(!jyState.md){const c=T.find(m=>m.planet===cur.mahadasha.planet&&m.start_date===cur.mahadasha.start);jyState.md=c?mdKey(c):mdKey(T[0])}
  const md=T.find(m=>mdKey(m)===jyState.md)||T[0];
  const ads=SKY.ad.filter(a=>a[0]===md.planet&&a[2]>=addDays(md.start_date.slice(0,10),-2)&&a[2]<addDays(md.end_date.slice(0,10),-2));
  if(!jyState.ad||!ads.some(a=>a[1]+a[2]===jyState.ad)){const c=ads.find(a=>a[2]<=today&&today<a[3])||ads[0];jyState.ad=c?c[1]+c[2]:null}
  const ad=ads.find(a=>a[1]+a[2]===jyState.ad);
  const pds=ad?ad[4].map((p,i)=>[p[0],p[1],i+1<ad[4].length?ad[4][i+1][1]:ad[3]]):[];
  const isCur=(s,e)=>s<=today&&today<e;
  return `<div class="grid3">
   <div><h3>Mahadasha</h3><div class="list">${T.map(m=>{const s=m.start_date.slice(0,10),e=m.end_date.slice(0,10);return `<button class="item${mdKey(m)===jyState.md?' sel':''}" data-md="${mdKey(m)}" style="${mdKey(m)===jyState.md?'border-color:var(--gold)':''}"><span class="k">${m.duration_years.toFixed(1)}y</span><span><b>${m.planet}${isCur(s,e)?' <span class="tag natal">now</span>':''}</b><small>${fmtDate(s)} to ${fmtDate(e)}</small></span></button>`}).join('')}</div><p class="meta" style="margin-top:8px">Engine: Vimshottari timeline.</p></div>
   <div><h3>Antardasha · ${md.planet}</h3><div class="list">${ads.map(a=>`<button class="item" data-ad="${a[1]+a[2]}" style="${a[1]+a[2]===jyState.ad?'border-color:var(--gold)':''}"><span class="k">${a[1].slice(0,2)}</span><span><b>${md.planet} / ${a[1]}${isCur(a[2],a[3])?' <span class="tag natal">now</span>':''}</b><small>${fmtDate(a[2])} to ${fmtDate(a[3])}</small></span></button>`).join('')||'<p class="muted">No antardasha table for this period.</p>'}</div></div>
   <div><h3>Pratyantardasha · ${ad?md.planet+' / '+ad[1]:''}</h3><div class="list">${pds.map(p=>`<div class="item" style="cursor:default;${isCur(p[1],p[2])?'border-color:var(--gold)':''}"><span class="k">${p[0].slice(0,2)}</span><span><b>${md.planet} / ${ad[1]} / ${p[0]}${isCur(p[1],p[2])?' <span class="tag natal">now</span>':''}</b><small>${fmtDate(p[1])} to ${fmtDate(p[2])}</small></span></div>`).join('')}</div><p class="meta" style="margin-top:8px">Antardasha and pratyantardasha: field table, boundaries aligned to the engine's current period (${fmtDate(cur.antardasha.start.slice(0,10))}, ${fmtDate(cur.pratyantardasha.end.slice(0,10))}).</p></div>
  </div>`;
}

function rJyotish(){
  const r=rec(state.date),isToday=state.date===todayIST(),P=panchangaOf(r),ss=sadeSati(r.sid[3]),T=todaySid(r),V=ENG.vimshottari.result,TR=ENG.transits.result;
  const list=chartPlanets(JYPREF.show,r);
  const aspToday=[];for(const t of T)for(const n of NATAL_SID){const a=aspectBetween(t.lon,n.lon);if(a&&a.orb<=2)aspToday.push({t,n,a})}aspToday.sort((x,y)=>x.a.orb-y.a.orb);
  const moonN=NATAL_SID.find(p=>p.body==='Moon'),jup=T.find(p=>p.body==='Jupiter'),sat=T.find(p=>p.body==='Saturn');
  $('#main').innerHTML=`<section class="room">
  <div class="hero"><div class="epithet">Jyotish · Lahiri · ${isToday?'the sky now':'witnessing'} · ${fmtDate(r.d)}</div><p class="decl">${V.current_period.mahadasha.planet} dasha. Moon in ${P.nak}.</p>
   <p class="lede">Lagna ${RASHI[LAGNA_SIGN]}, Moon ${moonN.sign} in ${moonN.nak}. Today Jupiter sits in your house ${jup.house} and Saturn in your house ${sat.house}. ${ss.active?ss.phase+'.':'Sade Sati is not active.'}</p>
   <div class="chips"><span class="chip"><span class="dot c"></span><span class="ev-c">Engine: birth positions, dasha, aspects</span></span><span class="chip"><span class="dot w"></span><span class="ev-w">Field: today's positions and Panchanga</span></span></div></div>
  ${armStage()}
  <div class="grid2">
   <div class="panel"><div class="ey" style="display:flex;justify-content:space-between;gap:8px;flex-wrap:wrap;margin-bottom:12px"><h3 style="margin:0">Rashi chart</h3><div style="display:flex;gap:6px;flex-wrap:wrap"><div class="seg" role="group" aria-label="Layout"><button id="jyNorth" aria-pressed="${JYPREF.layout==='north'}">North</button><button id="jySouth" aria-pressed="${JYPREF.layout==='south'}">South</button></div><div class="seg" role="group" aria-label="Show"><button id="jyNatal" aria-pressed="${JYPREF.show==='natal'}">Birth</button><button id="jyBoth" aria-pressed="${JYPREF.show==='both'}">Both</button><button id="jyToday" aria-pressed="${JYPREF.show==='today'}">Today</button></div></div></div>
    <div class="bg-wrap" id="jyChart">${JYPREF.layout==='north'?northChart(list):southChart(list)}</div>
    <div class="legend"><span><i style="background:var(--gold-warm)"></i>Birth (engine)</span><span><i style="background:var(--emerald)"></i>${fmtDate(r.d)} (field)</span><span>ʳ retrograde</span></div><p class="meta" style="margin:8px 0 0">Lagna ${RASHI[LAGNA_SIGN]} ${fmtDeg(LAGNA_SID%30)} from the Swiss Ephemeris at birth. Tap a graha.</p></div>
   <div style="display:grid;gap:var(--s3);align-content:start;min-width:0">
    <div class="panel"><h3>Panchanga · ${fmtDate(r.d)} · 06:00 IST</h3><dl class="kv"><dt>Tithi</dt><dd><b>${P.tithi}</b> · ${Math.round(P.tithiFrac*100)}% elapsed</dd><dt>Nakshatra</dt><dd><b>${P.nak}</b> pada ${P.pada} · Moon in ${P.moonSign}</dd><dt>Yoga</dt><dd>${P.yoga}</dd><dt>Karana</dt><dd>${P.karana}</dd><dt>Vara</dt><dd>${P.vara}</dd></dl><p class="meta" style="margin:8px 0 0">Field figures from the day's Sun and Moon. The daily engine call will replace them.</p></div>
    <div class="panel"><h3>Birth Panchanga · engine</h3><dl class="kv"><dt>Tithi</dt><dd><b>${ENG.panchanga.result.tithi_name}</b></dd><dt>Nakshatra</dt><dd><b>${ENG.panchanga.result.nakshatra_name}</b></dd><dt>Yoga</dt><dd>${ENG.panchanga.result.yoga_name}</dd><dt>Karana</dt><dd>${ENG.panchanga.result.karana_name}</dd><dt>Vara</dt><dd>${ENG.panchanga.result.vara_name}</dd></dl></div>
    <div class="panel"><h3>Sade Sati</h3><p style="margin:0 0 6px"><b class="${ss.active?'ev-u':'ev-c'}">${ss.active?'Active':'Not active'}</b> on ${fmtDate(r.d)}: Saturn in ${ss.saturnSign}, Moon in ${ss.moonSign}.${ss.dhaiya?' '+ss.dhaiya+'.':''}</p><p class="meta" style="margin:0">Engine on ${istDate(ENG.transits.at)}: ${TR.sade_sati.is_active?'active':'inactive'}, Saturn ${TR.sade_sati.saturn_sign}. Next phase begins when Saturn enters Leo.</p></div>
   </div></div>
  <div><div class="divider"><span></span></div><h2>Dasha</h2><div id="jyDasha">${dashaPanel()}</div></div>
  <div class="grid2">
   <div class="panel"><h3>Sky today, sidereal</h3><div class="tbl"><table><thead><tr><th>Graha</th><th>Rashi</th><th>Nakshatra</th><th>House</th></tr></thead><tbody>${T.filter(p=>!['Uranus','Neptune','Pluto'].includes(p.body)).map(p=>`<tr class="click" data-k="transit" data-b="${p.body}"><td><span class="sym">${SYM[p.body]||''}</span>${p.body}${p.retro?' ʳ':''}</td><td class="num">${p.sign} ${fmtDeg(p.deg)}</td><td>${p.nak} ${p.pada}</td><td class="num">${p.house}</td></tr>`).join('')}</tbody></table></div></div>
   <div class="panel"><h3>Transit aspects to the birth chart</h3><div class="tbl"><table><thead><tr><th>Transit</th><th>Aspect</th><th>Natal</th><th>Orb</th></tr></thead><tbody>${(isToday||state.date===istDate(ENG.transits.at)?TR.aspects.filter(a=>a.orb<=3).map(a=>`<tr><td>${a.transiting_planet}</td><td>${a.aspect_type}</td><td>${a.natal_planet}</td><td class="num">${a.orb.toFixed(2)}°</td></tr>`):aspToday.map(x=>`<tr><td>${x.t.body}</td><td>${x.a.name}</td><td>${x.n.body}</td><td class="num">${x.a.orb.toFixed(2)}°</td></tr>`)).join('')||'<tr><td colspan="4" class="muted">None within orb.</td></tr>'}</tbody></table></div><p class="meta" style="margin:8px 0 0">${isToday?'Engine aspects for '+istStamp(ENG.transits.at)+', period quality '+TR.period_quality+'.':'Field aspects within 2°, conjunction, opposition, square and trine.'}</p></div>
  </div></section>`;
  const set=(k,v)=>{JYPREF[k]=v;try{localStorage.setItem('wa-jy',JSON.stringify(JYPREF))}catch(e){}rJyotish()};
  $('#jyNorth').onclick=()=>set('layout','north');$('#jySouth').onclick=()=>set('layout','south');
  $('#jyNatal').onclick=()=>set('show','natal');$('#jyBoth').onclick=()=>set('show','both');$('#jyToday').onclick=()=>set('show','today');
  $('#main').querySelectorAll('[data-b]').forEach(el=>el.addEventListener('click',()=>showGraha(el.dataset.k,el.dataset.b,r)));
  const wireDasha=()=>{$('#jyDasha').querySelectorAll('[data-md]').forEach(b=>b.onclick=()=>{jyState.md=b.dataset.md;jyState.ad=null;$('#jyDasha').innerHTML=dashaPanel();wireDasha();armSync()});$('#jyDasha').querySelectorAll('[data-ad]').forEach(b=>b.onclick=()=>{jyState.ad=b.dataset.ad;$('#jyDasha').innerHTML=dashaPanel();wireDasha();armSync()})};
  wireDasha();
  window.onArmPick=()=>{$('#jyDasha').innerHTML=dashaPanel();wireDasha()};
  const ah=$('#armHost');if(ah){if(K3.ok()){try{const {inst,fresh}=K3.mount('jyotish',ah,h=>arm3d(h));if(!fresh)inst.update()}catch(e){console.warn('armillary',e);K3.release();ah.remove()}}else ah.remove()}
}

function armSync(){const o=K3.current('jyotish');if(o)o.sync()}
function armStage(){const V=ENG.vimshottari.result,c=V.current_period;
  const nx=(V.upcoming_transitions||[]).find(u=>u.date.slice(0,10)>state.date);
  const rd=(k,v)=>`<div class="readout"><span class="k">${k}</span><span class="v">${v}</span></div>`;
  const ad=SKY.ad.find(a=>a[2]<=state.date&&state.date<a[3]);let pd=null,pdS=null,pdE=null;if(ad){ad[4].forEach((p,i)=>{if(p[1]<=state.date){pd=p[0];pdS=p[1];pdE=i+1<ad[4].length?ad[4][i+1][1]:ad[3]}})}
  return `<div class="stage arm-stage" id="armHost" style="border:1px solid var(--rule);border-radius:4px">
   <div class="hud arm-hud">${rd('Mahadasha · engine',`<span class="g">${DASHA_SYM[c.mahadasha.planet]} ${c.mahadasha.planet}</span> · ${istDate(c.mahadasha.start)} to ${istDate(c.mahadasha.end)}`)}${ad?rd('Antardasha',`${DASHA_SYM[ad[1]]} ${ad[1]} · to ${fmtDate(ad[3])}`):''}${pd?rd('Pratyantardasha',`${DASHA_SYM[pd]} ${pd} · ${fmtDate(pdS)} to ${fmtDate(pdE)}`):''}</div>
   ${nx?`<div class="hud arm-hud-r"><div class="shift"><span class="k">Next shift · engine</span><span class="d">${istDate(nx.date)}</span><span class="v">${DASHA_SYM[nx.from_planet]||''} ${nx.from_planet} → ${DASHA_SYM[nx.to_planet]||''} ${nx.to_planet} · ${cap(nx.type)}</span></div></div>`:''}
   <div class="hud arm-hud-b"><span class="readout"><span class="k">Rings</span><span class="v">Outer mahadasha · middle antardasha · inner pratyantardasha</span></span><span class="k">Tap a ring to drill in</span></div></div>`}
