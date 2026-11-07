/* ---------- Today: one day witnessed through every engine ----------
   Two instruments side by side (the Human Design loom and the Vedic day dial), then a lens matrix that
   sets each engine's birth figure beside this day's figure and names where they meet, then a ribbon that
   overlays the engines across thirty-one days. Every figure carries its source. */
const LOOMPREF=(()=>{const d={planets:true,keys:false};try{return Object.assign(d,JSON.parse(localStorage.getItem('wa-loom')||'{}'))}catch(e){return d}})();
const BIRTH_NAK=11,BIRTH_MOON_SIGN=5;
const PROV={e:['c','Engine'],m:['w','Field · engine method'],f:['n','Field']};
const istHourNow=()=>{const t=new Date(Date.now()+5.5*36e5);return t.getUTCHours()+t.getUTCMinutes()/60};
const BIRTH_CLOCK=(()=>{try{const C=DL.clock('1991-08-13');return {C,A:DL.at(C,13+31/60)}}catch(e){return null}})();
function transitAspects(r){const out=[];const asp=[['Conjunction',0,1.5],['Sextile',60,1.5],['Square',90,1.5],['Trine',120,1.5],['Opposition',180,1.5]];
  const T=tPos(r).filter(t=>!['Earth','SouthNode'].includes(t.body)),N=NAT.filter(n=>n.side==='P'&&!['Earth','SouthNode'].includes(n.body));
  for(const t of T)for(const n of N){const d=angDiff(t.lon,n.lon);for(const [nm,a,o] of asp){if(Math.abs(d-a)<=o)out.push({t:t.body,n:n.body,name:nm,orb:Math.abs(d-a)})}}
  return out.sort((a,b)=>a.orb-b.orb)}
function todayLens(r,h){
  const R=dayReading(r),P=panchangaOf(r),B=DL.bio(r.d),N=DL.personal(r.d),VD=DL.vedic(r.d,h),C=VD.C,A=DL.at(C,VD.H);
  const moonNak=Math.floor(r.sid[0]/(360/27)),moonSign=Math.floor(r.sid[0]/30);const tara=DL.tara(moonNak,BIRTH_NAK),cb=DL.chandra(moonSign,BIRTH_MOON_SIGN);
  const ad=SKY.ad.find(a=>a[2]<=r.d&&r.d<a[3]);let pd=null,pdS=null,pdE=null;if(ad)ad[4].forEach((p,i)=>{if(p[1]<=r.d){pd=p[0];pdS=p[1];pdE=i+1<ad[4].length?ad[4][i+1][1]:ad[3]}});
  const lordAt=pl=>{const n=NATAL_SID.find(x=>x.body===pl);return n?`${n.sign}, house ${n.house}`:''};
  const tp=tPos(r),byGate={};SPHERES.forEach(sp=>{const n=NAT.find(x=>x.side===sp.src[0]&&x.body===sp.src[1]);(byGate[n.gate]=byGate[n.gate]||[]).push(sp.name)});
  const touched=Object.entries(byGate).map(([g,names])=>{const by=tp.filter(t=>t.body!=='Moon'&&t.gate===+g).map(t=>PNAME(t.body));return by.length?`${by.join(' and ')} in key ${g}, your ${names.join(' and ')}`:null}).filter(Boolean);
  const TS=todaySid(r),jup=TS.find(p=>p.body==='Jupiter'),sat=TS.find(p=>p.body==='Saturn'),ss=sadeSati(r.sid[3]);
  const fromMoon=p=>((Math.floor(p.lon/30)-BIRTH_MOON_SIGN+12)%12)+1;
  const asp=transitAspects(r).slice(0,4);
  const nm=ENG.numerology&&ENG.numerology.result;const core=nm?{['Life Path']:nm.life_path.value,Expression:nm.expression.value,['Soul Urge']:nm.soul_urge.value,Personality:nm.personality.value,Birthday:nm.birthday.value}:{};
  const echo=Object.entries(core).filter(([k,v])=>v===N.day||v===N.year).map(([k,v])=>`${v===N.day?'personal day':'personal year'} ${v} repeats your ${k}`);
  const crit=DL.CYC.filter(([k])=>B[k].critical).map(([k])=>k);
  const bc=BIRTH_CLOCK;
  return engineDay([
    {id:'human-design',name:'Human Design',src:'e',birth:'Generator 2/4 · Sacral · wide split · channels 23-43 and 42-53',day:`${defLabel(r.def)} definition${r.m2t?' · motor to Throat':''} · ${r.cen.length} of 9 centers`,meet:R.bridges.length?`Bridged by the sky: ${R.bridges.join(', ')}. ${R.decl}`:R.decl,link:'#design'},
    {id:'gene-keys',name:'Gene Keys',src:'e',birth:"Life's Work 4.2 · Evolution 49.2 · Radiance 23.4 · Purpose 43.4",day:`Sun key ${R.sun.gate}: ${GATES[R.sun.gate][2]} → ${GATES[R.sun.gate][3]} → ${GATES[R.sun.gate][4]} · Earth key ${R.earth.gate}`,meet:touched.length?touched.join('. ')+'.':'No transit sits in your eleven sphere keys.',link:'#keys',daySrc:'f'},
    {id:'panchanga',name:'Panchanga',src:'e',birth:'Chaturthi (Shukla) · Uttara Phalguni · Siddha · Vishti · Mangalavara',day:`${P.tithi} · ${P.nak} ${P.pada} · ${P.yoga} · ${P.karana}`,meet:`Tara ${tara.n} of 9 counted from your birth star: ${tara.name}, the star of ${tara.key} in the classical count. The Moon stands in house ${cb} from your birth Moon.`,link:'#jyotish',daySrc:'f'},
    {id:'vimshottari',name:'Vimshottari',src:'e',birth:'Birth Moon in Uttara Phalguni · Sun dasha at birth',day:ad?`${ad[0]} / ${ad[1]} / ${pd}`:'—',meet:pd?`${pd} runs ${fmtDate(pdS)} to ${fmtDate(pdE)}, ${Math.max(0,Math.round((Date.parse(pdE)-Date.parse(r.d))/864e5))} days remain. ${pd} sits in ${lordAt(pd)} at birth.`:'',link:'#jyotish',daySrc:'m'},
    {id:'transits',name:'Transits',src:'e',birth:'Moon Virgo 9°49\' · Jupiter Cancer 29°46\' exalted · Saturn Capricorn 8°31\'',day:`Jupiter in house ${jup.house} (${fromMoon(jup)} from Moon) · Saturn in house ${sat.house} (${fromMoon(sat)} from Moon) · Sade Sati ${ss.active?'active':'not active'}`,meet:asp.length?asp.map(a=>`${a.t} ${a.name.toLowerCase()} natal ${a.n} ${a.orb.toFixed(1)}°`).join(' · '):'No transit within 1.5° of a natal point.',link:'#sky',daySrc:'m'},
    {id:'numerology',name:'Numerology',src:'e',birth:nm?`Life Path ${nm.life_path.value} · Expression ${nm.expression.value} · Soul Urge ${nm.soul_urge.value} · Personality ${nm.personality.value}`:'',day:`Personal year ${N.year} · month ${N.month} · day ${N.day} (${DL.NUM_KEY[N.day]})`,meet:echo.length?cap(echo.join('; '))+'.':`Personal day ${N.day} touches none of your core numbers.`,daySrc:'f'},
    {id:'biorhythm',name:'Biorhythm',src:'e',birth:'Six cycles start at zero on 13 Aug 1991',day:DL.CYC.slice(0,3).map(([k])=>`${cap(k)} ${Math.round(B[k].pct)}% ${B[k].phase.toLowerCase()}`).join(' · '),meet:`Day ${B.days}. ${crit.length?'Crossing zero: '+crit.join(', ')+'.':'No cycle crosses zero.'} Passion ${Math.round(B.passion)}, mastery ${Math.round(B.mastery)}, wisdom ${Math.round(B.wisdom)}.`,daySrc:'m'},
    {id:'vedic-clock',name:'Vedic Clock',src:'e',birth:bc?`Born in ${bc.A.hora.lord} hora · ${bc.A.chog.name} choghadiya · ${bc.A.organ.organ} time`:'',day:`${DL.fmtH(h)}: ${A.hora.lord} hora · ${A.chog.name} (${A.chog.meaning}) · ${A.organ.organ} · ${A.dosha.dosha}`,meet:bc&&A.hora.lord===bc.A.hora.lord?`The hora lord matches your birth hora (${bc.A.hora.lord}).`:`Day lord ${C.lord}. Rahu Kaal ${DL.fmtH(C.rahu.s)}–${DL.fmtH(C.rahu.e)}. Abhijit ${DL.fmtH(C.abhijit.s)}–${DL.fmtH(C.abhijit.e)}.`,daySrc:'m'}], r);
}

/* ---------- The engine-held day: Selemene computes it and keeps it in readings ---------- */
const istHM=iso=>{const t=new Date(Date.parse(iso)+5.5*36e5);return `${String(t.getUTCHours()).padStart(2,'0')}:${String(t.getUTCMinutes()).padStart(2,'0')}`};
const istISOd=iso=>new Date(Date.parse(iso)+5.5*36e5).toISOString().slice(0,10);
/* Where an engine day is stored, its figures replace the field's for the five day engines. */
function engineDay(rows,r){const D=dayOf(r.d);if(!D)return rows;const at=D.moment?` at ${istHM(D.moment)} IST`:'';
  const P=dayEng(r.d,'panchanga'),V=dayEng(r.d,'vimshottari'),T=dayEng(r.d,'transits'),B=dayEng(r.d,'biorhythm'),C=dayEng(r.d,'vedic-clock');
  return rows.map(x=>{
    if(x.id==='panchanga'&&P){const ni=NAKS.indexOf(P.nakshatra_name);const tara=ni>=0?DL.tara(ni,BIRTH_NAK):null;
      return {...x,day:`${P.tithi_name} · ${P.nakshatra_name} · ${P.yoga_name} · ${P.karana_name} · ${P.vara_name}${at}`,daySrc:'e',meet:tara?`Tara ${tara.n} of 9 counted from your birth star: ${tara.name}, the star of ${tara.key} in the classical count.`:x.meet}}
    if(x.id==='vimshottari'&&V&&V.current_period){const c=V.current_period,pe=c.pratyantardasha;const left=Math.max(0,Math.round((Date.parse(pe.end)-Date.parse(D.moment||r.d))/864e5));
      const nx=(V.upcoming_transitions||[])[0];
      return {...x,day:`${c.mahadasha.planet} / ${c.antardasha.planet} / ${pe.planet}`,daySrc:'e',meet:`${pe.planet} runs to ${fmtDate(istISOd(pe.end))}, ${left} days remain.${nx?` Next: ${nx.to_planet} ${String(nx.type||'').toLowerCase()} on ${fmtDate(istISOd(nx.date))}.`:''}`}}
    if(x.id==='transits'&&T&&T.aspects){const tight=T.aspects.filter(a=>a.orb<=1.5).sort((a,b)=>a.orb-b.orb).slice(0,4);
      return {...x,daySrc:'e',day:`${x.day.replace(/ · Sade Sati.*$/,'')} · Sade Sati ${T.sade_sati&&T.sade_sati.is_active?'active':'not active'} · period ${String(T.period_quality||'').toLowerCase()}`,
        meet:tight.length?tight.map(a=>`${a.transiting_planet} ${a.aspect_type.toLowerCase()} natal ${a.natal_planet} ${a.orb.toFixed(2)}°`).join(' · ')+' (sidereal, engine)':'No transit within 1.5° of a natal point (engine).'}}
    if(x.id==='biorhythm'&&B&&B.physical){const cyc=['physical','emotional','intellectual'];
      return {...x,daySrc:'e',day:cyc.map(k=>`${cap(k)} ${Math.round(B[k].percentage)}% ${String(B[k].phase).toLowerCase()}`).join(' · '),
        meet:`Day ${B.days_alive}. ${(B.critical_days||[]).length?'Critical days ahead: '+B.critical_days.map(d=>fmtDate(d)).join(', ')+'.':'No critical day in the next week.'} Energy ${Math.round(B.overall_energy)}.`}}
    if(x.id==='vedic-clock'&&C&&C.hora){return {...x,meet:`${x.meet} Engine${at}: ${C.hora.ruler} hora · ${C.choghadiya?C.choghadiya.name:''}.`}}
    return x})}
/* No-prediction guardrail, ported from packages/witness-pipeline rubric (negation-aware per clause) */
const GUARD=[/guarantee/i,/investment advice/i,/will become rich/i,/definitely inherit/i,/will marry/i,/inevitable/i,/must divorce/i,/spouse will be/i,/diagnos/i,/treatment/i,/\bcure\b/i,/will develop/i,/you have .*disease/i,/will have children/i,/infertile/i,/parent will die/i,/child will/i,/\bdestined\b/i,/\bwill happen\b/i,/\bis going to happen\b/i];
const CLAUSE=/[.!?;:\n]|\b(?:but|however|yet|although|though)\b/gi,NEG=/\b(?:not|no|nothing|never|none|nor|neither|cannot|without|rather than)\b|n't\b/i;
function guardFails(text){const out=[];for(const re of GUARD){const g=new RegExp(re.source,'gi');for(const m of String(text).matchAll(g)){const before=text.slice(0,m.index);let cs=0;for(const b of before.matchAll(CLAUSE))cs=b.index+b[0].length;if(!NEG.test(before.slice(cs))){out.push(m[0]);break}}}return out}
function dyadPanel(D){const w=D.witness;if(!w){const why=(D.missing||[]).find(m=>m.part==='witness-dyad');
    return `<div class="panel dyad"><h3>Witness dyad<span class="sub">not stored for this day</span></h3><p class="muted" style="margin:0">${esc(why?why.reason:'No dyad was stored for this day.')}</p></div>`}
  const fails=guardFails([w.aletheios,w.pichet,w.synthesis,w.witness_question].join(' \n'));
  if(fails.length)return `<div class="panel dyad"><h3>Witness dyad<span class="sub">withheld</span></h3><p class="muted" style="margin:0">The stored dyad for this day used forecast or diagnostic language (${esc(fails.join(', '))}), which the field does not show. The engine's per-engine witness prompts remain in the lens matrix.</p></div>`;
  return `<div class="panel dyad"><h3>Witness dyad · ${fmtDate(D.date)}<span class="sub">${w.llm_powered?'Language model':'Rule-based'} · ${(w.engines_used||[]).join(', ')}</span></h3>
   <div class="dyad-grid"><div class="dyad-voice al"><div class="dv-h">${icon('sacred-geometry',22)}<b>Aletheios</b><span>names</span></div><p>${esc(w.aletheios)}</p></div>
    <div class="dyad-voice pi"><div class="dv-h">${icon('witness',22)}<b>Pichet</b><span>embodies</span></div><p>${esc(w.pichet)}</p></div></div>
   <p class="dyad-syn">${esc(w.synthesis)}</p><p class="dyad-q">${esc(w.witness_question)}</p>
   <p class="meta" style="margin:0"><span class="state w">Witness</span> Stored by the Selemene engine for ${D.moment?istHM(D.moment)+' IST':'this day'}, with no biofield capture. Checked against the field's no-prediction guardrail.</p></div>`}
function freshChip(d){const D=dayOf(d);if(!D)return `<span class="chip"><span class="dot u"></span><span class="ev-u">No engine day stored · field figures</span></span>`;
  const n=Object.keys(D.engines||{}).length;return `<span class="chip"><span class="dot c"></span><span class="ev-c">Engine day · ${n}/5 engines${D.witness?' · dyad':''} · ${D.anchor==='sunrise+10m'?'sunrise ':''}${D.moment?istHM(D.moment):''} IST</span></span>`}
/* Day checks: the stored engine day against the field's own figures for the same date */
function dayChecks(d){const D=dayOf(d);if(!D)return [];const rows=[];const r=rec(d),P=panchangaOf(r);
  const E=id=>dayEng(d,id);const p=E('panchanga'),b=E('biorhythm'),c=E('vedic-clock'),v=E('vimshottari');
  if(p){rows.push(['Tithi',P.tithi,p.tithi_name,P.tithi===p.tithi_name]);rows.push(['Nakshatra',P.nak,p.nakshatra_name,P.nak===p.nakshatra_name]);rows.push(['Yoga',P.yoga,p.yoga_name,P.yoga===p.yoga_name]);rows.push(['Karana',P.karana,p.karana_name,P.karana===p.karana_name]);
    const civ=new Date(Date.parse(D.moment)+5.5*36e5),h=civ.getUTCHours()+civ.getUTCMinutes()/60,V=DL.vedic(d,h);const vara=['Ravivara','Somavara','Mangalavara','Budhavara','Guruvara','Shukravara','Shanivara'][new Date(V.C.date+'T00:00:00Z').getUTCDay()];
    rows.push(['Vara (Vedic day at the moment)',vara,p.vara_name,p.vara_name.startsWith(vara)])}
  if(b){const f=DL.bio(d);['physical','emotional','intellectual','intuitive','aesthetic','spiritual'].forEach(k=>rows.push([`Biorhythm ${k}`,`${f[k].pct.toFixed(2)}% ${f[k].phase}`,`${b[k].percentage.toFixed(2)}% ${b[k].phase}`,Math.abs(f[k].pct-b[k].percentage)<.01&&f[k].phase===b[k].phase]))}
  if(c&&c.vedic_day){const C=DL.clock(c.vedic_day.date),er=c.vedic_day.sunrise.slice(11,19).split(':').map(Number),eh=er[0]+er[1]/60+er[2]/3600;rows.push(['Sunrise',DL.fmtHs(C.rise),c.vedic_day.sunrise.slice(11,19),Math.abs(C.rise-eh)<2/60]);
    const civ=new Date(Date.parse(c.calculated_for)+5.5*36e5),h=civ.getUTCHours()+civ.getUTCMinutes()/60+civ.getUTCSeconds()/3600,V=DL.vedic(civ.toISOString().slice(0,10),h),A=DL.at(V.C,V.H);rows.push(['Hora at the moment',A.hora.lord,c.hora.ruler,A.hora.lord===c.hora.ruler]);rows.push(['Choghadiya at the moment',A.chog.name,c.choghadiya.name,A.chog.name===c.choghadiya.name])}
  if(v&&v.current_period){const ad=SKY.ad.find(a=>a[2]<=d&&d<a[3]);let pd=null;if(ad)ad[4].forEach(x=>{if(x[1]<=d)pd=x[0]});const f=ad?`${ad[0]} / ${ad[1]} / ${pd}`:'—',e=`${v.current_period.mahadasha.planet} / ${v.current_period.antardasha.planet} / ${v.current_period.pratyantardasha.planet}`;rows.push(['Dasha',f,e,f===e])}
  return rows}
function provTag(s){const [c,w]=PROV[s];return `<span class="state ${c}">${w}</span>`}
function lensMatrix(r,h){return `<div class="lens-grid" role="table" aria-label="Engines: birth, this day, where they meet">
  <div class="lens-row lens-head" role="row"><span role="columnheader">Engine</span><span role="columnheader">At birth</span><span role="columnheader">This day</span><span role="columnheader">Where they meet</span></div>
  ${todayLens(r,h).map((x,i)=>`<div class="lens-row" role="row"><span class="lens-eng" role="cell">${icon(x.id,28)}<b>${x.name}</b><small>${String(i+1).padStart(2,'0')}</small></span>
   <span role="cell"><em>${provTag(x.src)}</em>${esc(x.birth)}</span><span role="cell"><em>${provTag(x.daySrc||'f')}</em>${esc(x.day)}</span><span role="cell" class="lens-meet">${esc(x.meet)}${x.link?` <a href="${x.link}">Open</a>`:''}</span></div>`).join('')}</div>`}
/* The ribbon: thirty-one days, every engine on its own track */
const BIO_COL={physical:'#B08A18',emotional:'#C7607C',intellectual:'#4F7BF2'};
function ribbon(center){const N=31,half=15;let start=addDays(center,-half);if(start<DAYS[0].d)start=DAYS[0].d;
  const days=[...Array(N)].map((_,i)=>addDays(start,i)).filter(d=>d<=DAYS[DAYS.length-1].d);const W=980,L=118,cw=(W-L-84)/days.length;const X=i=>L+i*cw+cw/2;
  const tracks=[['Date',30],['Definition',26],['Moon',30],['Tithi',30],['Tara',26],['Biorhythm',120],['Events',26]];let y=8;const TY={};tracks.forEach(([k,hh])=>{TY[k]=[y,hh];y+=hh+6});const H=y+4;
  let s=`<svg viewBox="0 0 ${W} ${H}" class="ribbon" role="img" aria-label="Thirty-one day ribbon around ${fmtDate(center)}">`;
  tracks.forEach(([k,hh])=>{const [ty]=TY[k];s+=`<text x="0" y="${ty+hh/2}" class="rl" dominant-baseline="middle">${k.toUpperCase()}</text><line x1="${L-8}" x2="${W-8}" y1="${ty+hh+3}" y2="${ty+hh+3}" class="rg"/>`});
  const ci=days.indexOf(center);if(ci>=0)s+=`<rect x="${L+ci*cw}" y="2" width="${cw}" height="${H-4}" class="rsel" rx="2"/>`;
  days.forEach((d,i)=>{const r=rec(d),P=panchangaOf(r),x=X(i);const [dy]=TY.Date,[fy,fh]=TY.Definition,[my]=TY.Moon,[ty,th]=TY.Tithi,[ry]=TY.Tara;
    const dd=+d.slice(8);s+=`<text x="${x}" y="${dy+12}" class="rd${d===todayIST()?' now':''}" text-anchor="middle">${dd}</text>${dd===1||i===0?`<text x="${x}" y="${dy+26}" class="rm" text-anchor="middle">${MON[+d.slice(5,7)-1].toUpperCase()}</text>`:''}`;
    s+=`<rect x="${x-cw/2+1.5}" y="${fy+4}" width="${cw-3}" height="${fh-8}" rx="2" class="def${Math.min(r.def,4)}"/>`;if(r.def>=3)s+=`<text x="${x}" y="${fy+fh/2+3}" class="rt" text-anchor="middle">${r.def}</text>`;
    if(r.m2t)s+=`<circle cx="${x}" cy="${fy+fh/2}" r="2.6" class="m2t"/>`;
    s+=`<text x="${x}" y="${my+18}" class="rn" text-anchor="middle">${NAKS[Math.floor(r.sid[0]/(360/27))].slice(0,4)}</text>`;
    const el=((r.p[2][3]-r.p[0][3])%360+360)%360,ill=(1-Math.cos(el*Math.PI/180))/2,cx=x,cy=ty+th/2,rr=Math.min(8,cw*.3);const k=Math.cos(el*Math.PI/180),waxing=el<180;
    s+=`<circle cx="${cx}" cy="${cy}" r="${rr}" class="mo0"/><path d="M${cx} ${cy-rr}A${rr} ${rr} 0 0 ${waxing?1:0} ${cx} ${cy+rr}A${Math.abs(k)*rr} ${rr} 0 0 ${(waxing?k>0:k<0)?0:1} ${cx} ${cy-rr}Z" class="mo1"/>`;
    const tr=DL.tara(Math.floor(r.sid[0]/(360/27)),BIRTH_NAK);s+=`<text x="${x}" y="${ry+16}" class="rt" text-anchor="middle">${tr.n}</text>`;
    s+=`<rect x="${L+i*cw}" y="2" width="${cw}" height="${H-4}" fill="transparent" class="rhit" data-day="${d}"><title>${fmtDate(d)} · ${defLabel(r.def).toLowerCase()} definition · Moon ${NAKS[Math.floor(r.sid[0]/(360/27))]} · ${P.tithi} · Tara ${tr.n} ${tr.name} · biorhythm ${['physical','emotional','intellectual'].map(k=>k.slice(0,4)+' '+Math.round(DL.bio(d)[k].pct)+'%').join(', ')}</title></rect>`});
  // biorhythm lines: three series, one axis, a zero line, direct labels at the right
  const [by,bh]=TY.Biorhythm,BY=v=>by+bh/2-v*(bh/2-6);s+=`<line x1="${L-8}" x2="${W-8}" y1="${BY(0)}" y2="${BY(0)}" class="rz"/>`;
  ['physical','emotional','intellectual'].forEach(k=>{const pts=[];for(let i=0;i<=(days.length-1)*4;i++){const f=i/4,d0=Math.round((Date.parse(days[0])-Date.parse('1991-08-13'))/864e5)+f;const P=DL.CYC.find(c=>c[0]===k)[1];pts.push(`${(X(0)+f*cw).toFixed(1)},${BY(Math.sin(2*Math.PI*d0/P)).toFixed(1)}`)}
    s+=`<polyline points="${pts.join(' ')}" fill="none" stroke="${BIO_COL[k]}" stroke-width="2" stroke-linejoin="round" stroke-linecap="round"/>`;
    const last=DL.bio(days[days.length-1])[k].value;s+=`<text x="${X(days.length-1)+10}" y="${BY(last)}" class="rbl" dominant-baseline="middle">${k}</text>`});
  const [ey,eh]=TY.Events;EVENTS.filter(e=>e[0]>=days[0]&&e[0]<=days[days.length-1]).forEach(e=>{const i=days.indexOf(e[0]);if(i<0)return;s+=`<path d="M${X(i)} ${ey+4}l5 8-5 8-5-8z" class="ev ${e[1]}"><title>${fmtDate(e[0])} · ${esc(e[2])}</title></path>`});
  s+='</svg>';
  return `<div class="ribbon-wrap">${s}</div><div class="legend ribbon-legend"><span><i style="background:${BIO_COL.physical}"></i>Physical</span><span><i style="background:${BIO_COL.emotional}"></i>Emotional</span><span><i style="background:${BIO_COL.intellectual}"></i>Intellectual</span><span><i class="lg-def1"></i>Single definition</span><span><i class="lg-def2"></i>Split</span><span><i class="lg-def3"></i>Triple split</span><span><i class="lg-m2t"></i>Motor to Throat</span><span>Tara counted from Uttara Phalguni · tap a day</span></div>`}
function verifyDrawer(){const rows=DL.verify(ENG),day=dayChecks(state.date),D=dayOf(state.date);
  const table=rs=>`<div class="tbl"><table><thead><tr><th>Check</th><th>Field</th><th>Engine</th><th></th></tr></thead><tbody>${rs.map(r=>`<tr><td>${esc(r[0])}</td><td class="num">${esc(r[1])}</td><td class="num">${esc(r[2])}</td><td>${r[3]?'<span class="ev-c">match</span>':'<span class="ev-u">differs</span>'}</td></tr>`).join('')}</tbody></table></div>`;
  openDrawer(`<div class="eyebrow">Method check</div><h2>Field figures, engine methods</h2><p class="muted">The day dial and the biorhythm cycles are recomputed in the page so any date can be witnessed. Each method is checked here against the stored engine response for its call time.</p>
  ${table(rows)}<p class="meta" style="margin-top:12px">${rows.filter(r=>r[3]).length} of ${rows.length} match.</p>
  <h3 style="margin-top:20px">This day · ${fmtDate(state.date)}</h3>${day.length?`<p class="muted">The engine-held day${D&&D.moment?` (computed at ${istHM(D.moment)} IST)`:''} against the field's own figures. Panchanga names are compared at different moments (field 06:00 IST, engine after sunrise), so a boundary crossed in between shows as a difference.</p>${table(day)}<p class="meta" style="margin-top:12px">${day.filter(r=>r[3]).length} of ${day.length} match.</p>`:'<p class="muted">No engine day is stored for this date. Run sync_daily.py, or let the engine\'s daily field run.</p>'}`)}

function rToday(){
  const r=rec(state.date),R=dayReading(r),isToday=state.date===todayIST();const C0=DL.clock(state.date);
  const h=isToday?istHourNow():(DIAL.date===state.date&&DIAL.h!=null?DIAL.h:C0.rise+.25);const VD=DL.vedic(state.date,h),C=VD.C;DIAL.date=state.date;DIAL.h=h;
  const P=panchangaOf(r),A=DL.at(C,VD.H),tara=DL.tara(Math.floor(r.sid[0]/(360/27)),BIRTH_NAK);
  const ad=SKY.ad.find(a=>a[2]<=state.date&&state.date<a[3]);let pd=null;if(ad)ad[4].forEach(p=>{if(p[1]<=state.date)pd=p[0]});
  const next=EVENTS.filter(e=>e[0]>=state.date&&e[0]<=addDays(state.date,30)).slice(0,10);
  const rd=(k,v)=>`<div class="readout"><span class="k">${k}</span><span class="v">${v}</span></div>`;
  $('#main').innerHTML=`<section class="room">
  <div class="hero"><div class="epithet">${isToday?'The sky now':'Witnessing'} · ${fmtDate(state.date)} · ${P.vara.split(' ')[0]}</div>
   <h1 class="title" style="letter-spacing:.08em">${esc(R.decl)}</h1>
   <p class="lede">${esc(R.trav)} The Moon walks ${P.nak} in ${P.tithi.toLowerCase()}; ${ad?`the dasha runs ${ad[0]} / ${ad[1]}${pd?' / '+pd:''}`:''}.</p>
   <div class="chips"><span class="chip"><span class="dot ${r.def===1?'c':'g'}"></span>${defLabel(r.def)} definition</span><span class="chip"><span class="dot ${r.m2t?'c':'w'}"></span>${r.m2t?'Motor to Throat':'No motor to Throat'}</span><span class="chip">☽ ${P.nak} · pada ${P.pada}</span><span class="chip">${P.tithi}</span><span class="chip">Tara ${tara.n} · ${tara.name}</span><span class="chip">${SYM[A.hora.lord]} ${A.hora.lord} hora</span>${freshChip(state.date)}</div></div>
  <div class="today-inst">
   <div class="panel inst"><h3>Bodygraph · Human Design<span class="sub">Loom</span></h3>
    <div class="inst-bar"><div class="seg" role="group" aria-label="Overlays"><button id="ovPlanets" aria-pressed="${LOOMPREF.planets}">Planets</button><button id="ovKeys" aria-pressed="${LOOMPREF.keys}">Gene Keys</button></div><span class="meta">Tap a center or a gate</span></div>
    <div class="bg-wrap" id="bgToday"></div>${BG_LEGEND}</div>
   <div class="panel inst"><h3>Day dial · Vedic Clock<span class="sub">Hora · choghadiya · organ · dosha</span></h3>
    <div class="inst-bar"><div class="dial-read" id="dialRead"></div><button class="pill sm" id="dialCheck">Method check</button></div>
    <div class="stage dial-stage" id="dialHost"></div>
    <div class="scrub dial-scrub"><span class="lab" id="dialT">${DL.fmtH(h)}</span><input type="range" id="dialH" min="0" max="24" step="0.0833" value="${((h%24)+24)%24}" aria-label="Hour of the day"><span class="lab" style="text-align:right">IST</span></div>
    <p class="meta" style="margin:0" id="dialInfo"></p></div>
  </div>
  ${dayOf(state.date)?dyadPanel(dayOf(state.date)):''}
  <div class="panel"><h3>Lens matrix · eight engines, one day<span class="sub">Birth · this day · where they meet</span></h3><div id="lensM">${lensMatrix(r,h)}</div></div>
  <div class="panel"><h3>Ribbon · thirty-one days<span class="sub">Every engine on its own track</span></h3>${ribbon(state.date)}</div>
  <div class="grid2">
   <div class="panel"><h3>Channels active today</h3><div class="list">${r.ch.map(([k,t])=>`<button class="item" data-ch="${k}"><span class="tag ${t==='natal'?'natal':t==='bridge'?'c':'w'}">${t==='natal'?'yours':t==='bridge'?'bridged':'transit'}</span><span><b>${k} ${CHANNELS[k][0]}</b><small>${CH_NOTE[k]||CHANNELS[k][1]+' circuit, carried by the sky. Felt as background weather in your aura.'}</small></span></button>`).join('')}</div></div>
   <div style="display:grid;gap:var(--s3);align-content:start;min-width:0">
    ${r.con.length?`<div class="panel"><h3>Exact contacts (within 1°)</h3><div class="list">${r.con.map(c=>{const n=NAT.find(x=>x.side===c[1]&&x.body===c[2]);return `<button class="item" data-gate="${n.gate}"><span class="k">${SYM[c[0]]||''} ${c[3]}°</span><span><b>Transit ${PNAME(c[0])} on your ${c[1]==='P'?'Personality':'Design'} ${PNAME(c[2])}</b><small>Gate ${n.gate}.${n.line} ${GATES[n.gate][0]}: ${GATES[n.gate][3]}</small></span></button>`}).join('')}</div></div>`:''}
    <div class="panel"><h3>Next thirty days</h3><div>${next.length?next.map(evRow).join(''):'<p class="muted">No major events in the next thirty days.</p>'}</div><a class="pill sm" href="#timeline" style="margin-top:8px">Full timeline</a></div>
   </div></div>
  <div class="panel"><h3>The sky, gate by gate</h3><div class="tbl"><table><thead><tr><th>Body</th><th>Gate</th><th>Western</th><th>Vedic</th><th>Contact</th></tr></thead><tbody>${todaySidFull(r).map(t=>`<tr class="click" data-gate="${t.gate}"><td><span class="sym">${SYM[t.body]}</span>${PNAME(t.body)}</td><td class="num">${t.gate}.${t.line}${t.retro?' R':''}</td><td class="num">${fmtLon(t.lon)}</td><td class="num">${t.sid.sign} ${fmtDeg(t.sid.deg)} · ${t.sid.nak} ${t.sid.pada}</td><td>${NATG.has(t.gate)?'<span class="tag natal">your gate</span>':''}</td></tr>`).join('')}</tbody></table></div></div>
  </section>`;
  bodygraph($('#bgToday'),r);
  const lo=K3.current('today');if(lo&&lo.overlay)lo.overlay(LOOMPREF);
  // the dial gets its own stage alongside the loom
  const dh=$('#dialHost');
  if(K3.ok()){try{const {inst,fresh}=K3.mount('today',dh,el=>dial3d(el,state.date,h),'dial');if(!fresh)inst.update(state.date,h)}catch(e){console.warn('dial',e);dial2d(dh,state.date,h)}}else dial2d(dh,state.date,h);
  dialReadout(h);
  const setH=v=>{DIAL.h=v;$('#dialT').textContent=DL.fmtH(v);dialReadout(v);const di=K3.current('today','dial');if(di)di.setHour(v);else dial2d(dh,state.date,v);$('#lensM').innerHTML=lensMatrix(rec(state.date),v)};
  $('#dialH').oninput=e=>setH(+e.target.value);
  $('#dialCheck').onclick=verifyDrawer;
  const ov=(k,b)=>{LOOMPREF[k]=!LOOMPREF[k];try{localStorage.setItem('wa-loom',JSON.stringify(LOOMPREF))}catch(e){}b.setAttribute('aria-pressed',LOOMPREF[k]);const l=K3.current('today');if(l&&l.overlay)l.overlay(LOOMPREF)};
  $('#ovPlanets').onclick=e=>ov('planets',e.currentTarget);$('#ovKeys').onclick=e=>ov('keys',e.currentTarget);
  $('#main').querySelectorAll('.rhit').forEach(el=>el.addEventListener('click',()=>setDate(el.dataset.day)));
  wire($('#main'));}
function dialReadout(h){const V=DL.vedic(state.date,h),C=V.C,A=DL.at(C,V.H);const inf=$('#dialInfo');if(inf)inf.textContent=`${V.prev?'Before sunrise: Vedic day of '+fmtDate(C.date)+' · ':''}Sunrise ${DL.fmtH(C.rise)} · sunset ${DL.fmtH(C.set)} · Rahu Kaal ${DL.fmtH(C.rahu.s)}–${DL.fmtH(C.rahu.e)} · Abhijit ${DL.fmtH(C.abhijit.s)}–${DL.fmtH(C.abhijit.e)} · day lord ${C.lord}`;const rd=(k,v)=>`<div class="readout"><span class="k">${k}</span><span class="v">${v}</span></div>`;
  const el=$('#dialRead');if(el)el.innerHTML=rd('Hora',`<span class="g">${SYM[A.hora.lord]} ${A.hora.lord}</span> ${DL.fmtH(A.hora.s)}–${DL.fmtH(A.hora.e)}`)+rd('Choghadiya',`${A.chog.name} · ${A.chog.meaning}`)+rd('Organ · dosha',`${A.organ.organ} · ${A.dosha.dosha}`)}
function todaySidFull(r){const ay=ayanOf(r);return tPos(r).map(t=>({...t,sid:sidInfo(t.lon,ay)}))}
