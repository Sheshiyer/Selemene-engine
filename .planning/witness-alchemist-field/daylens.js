/* ---------- Day lenses: deterministic engine methods recomputed for any date ----------
   Each function reproduces a Selemene engine's published method so the field can witness dates the
   stored responses do not cover. Each is checked against the stored response for its call date (see
   DL.verify), and the page labels these figures "field · engine method" rather than "engine". */
const DL=(()=>{
  const rad=Math.PI/180,deg=180/Math.PI;
  const BIRTH={date:'1991-08-13',lat:12.9716,lon:77.5946,tz:5.5};
  const dms=s=>Date.parse(s+'T00:00:00Z');
  const addD=(s,n)=>new Date(dms(s)+n*864e5).toISOString().slice(0,10);
  /* Sun times: NOAA solar position, altitude -0.833°, as the Vedic Clock engine reports (sun_event_method: noaa-solar-position) */
  function solar(jd){const T=(jd-2451545)/36525;const L0=((280.46646+T*(36000.76983+T*.0003032))%360+360)%360,M=357.52911+T*(35999.05029-.0001537*T),e=.016708634-T*(.000042037+.0000001267*T);
    const C=Math.sin(M*rad)*(1.914602-T*(.004817+.000014*T))+Math.sin(2*M*rad)*(.019993-.000101*T)+Math.sin(3*M*rad)*.000289;const om=125.04-1934.136*T;const lam=L0+C-.00569-.00478*Math.sin(om*rad);
    const eps0=23+(26+(21.448-T*(46.815+T*(.00059-T*.001813)))/60)/60,eps=eps0+.00256*Math.cos(om*rad);const dec=Math.asin(Math.sin(eps*rad)*Math.sin(lam*rad));
    const y=Math.tan(eps*rad/2)**2;const eqt=4*deg*(y*Math.sin(2*L0*rad)-2*e*Math.sin(M*rad)+4*e*y*Math.sin(M*rad)*Math.cos(2*L0*rad)-.5*y*y*Math.sin(4*L0*rad)-1.25*e*e*Math.sin(2*M*rad));return {dec,eqt}}
  function sunTimes(date,lat,lon,tz){lat=lat==null?BIRTH.lat:lat;lon=lon==null?BIRTH.lon:lon;tz=tz==null?BIRTH.tz:tz;
    const ev=(sign)=>{let t=sign<0?6:18;for(let i=0;i<3;i++){const jd=dms(date)/864e5+2440587.5+(t-tz)/24;const {dec,eqt}=solar(jd);const ha=Math.acos(Math.cos(90.833*rad)/(Math.cos(lat*rad)*Math.cos(dec))-Math.tan(lat*rad)*Math.tan(dec))*deg;
      const noon=720-4*lon-eqt+tz*60;t=(noon+sign*4*ha)/60}return t};return {rise:ev(-1),set:ev(1)}}  // decimal hours, local
  const fmtH=h=>{const s=Math.round(((h%24)+24)%24*3600);return `${String(Math.floor(s/3600)).padStart(2,'0')}:${String(Math.floor(s%3600/60)).padStart(2,'0')}`};
  const fmtHs=h=>{const s=Math.round(((h%24)+24)%24*3600);return `${fmtH(h)}:${String(s%60).padStart(2,'0')}`};
  /* The Vedic day runs sunrise to next sunrise; weekday is the civil weekday of its sunrise */
  const WEEK_LORD=['Sun','Moon','Mars','Mercury','Jupiter','Venus','Saturn'];
  const CHALDEAN=['Saturn','Jupiter','Mars','Sun','Venus','Mercury','Moon'];
  const HORA_KEY={Sun:'vitality, authority, recognition',Moon:'feeling, care, the public',Mars:'effort, competition, cutting',Mercury:'speech, trade, study',Jupiter:'teaching, counsel, ritual',Venus:'beauty, pleasure, agreement',Saturn:'labour, structure, endings'};
  const CHOG_DAY=['Udveg','Char','Labh','Amrit','Kaal','Shubh','Rog'],CHOG_NIGHT=['Shubh','Amrit','Char','Rog','Kaal','Labh','Udveg'];
  const CHOG_START_DAY={0:'Udveg',1:'Amrit',2:'Rog',3:'Labh',4:'Shubh',5:'Char',6:'Kaal'},CHOG_START_NIGHT={0:'Shubh',1:'Char',2:'Kaal',3:'Udveg',4:'Amrit',5:'Rog',6:'Labh'};
  const CHOG={Udveg:['Sun','inauspicious','Anxiety'],Char:['Venus','neutral','Movement'],Labh:['Mercury','good','Gain'],Amrit:['Moon','good','Nectar'],Kaal:['Saturn','inauspicious','Loss'],Shubh:['Jupiter','good','Auspicious'],Rog:['Mars','inauspicious','Disease']};
  const ORGANS=[['Gallbladder','Wood',23],['Liver','Wood',1],['Lung','Metal',3],['Large intestine','Metal',5],['Stomach','Earth',7],['Spleen','Earth',9],['Heart','Fire',11],['Small intestine','Fire',13],['Bladder','Water',15],['Kidney','Water',17],['Pericardium','Fire',19],['Triple burner','Fire',21]];
  const DOSHAS=[['Kapha',6],['Pitta',10],['Vata',14],['Kapha',18],['Pitta',22],['Vata',2]];
  const RAHU=[8,2,7,5,6,4,3],YAMA=[5,4,3,2,1,7,6],GULI=[7,6,5,4,3,2,1]; // eighth of daylight, by weekday Sun..Sat
  function clock(date){const wd=new Date(dms(date)).getUTCDay();const t=sunTimes(date),n=sunTimes(addD(date,1));const rise=t.rise,set=t.set,next=n.rise+24;
    const dl=(set-rise)/12,nl=(next-set)/12;const lord0=WEEK_LORD[wd],ci=CHALDEAN.indexOf(lord0);
    const horas=[...Array(24)].map((_,i)=>{const s=i<12?rise+i*dl:set+(i-12)*nl,e=s+(i<12?dl:nl);const lord=CHALDEAN[(ci+i)%7];return {n:i+1,lord,s,e,period:i<12?'day':'night',key:HORA_KEY[lord]}});
    const dp=(set-rise)/8,np=(next-set)/8;const d0=CHOG_DAY.indexOf(CHOG_START_DAY[wd]),n0=CHOG_NIGHT.indexOf(CHOG_START_NIGHT[wd]);
    const chog=[...Array(16)].map((_,i)=>{const day=i<8;const name=day?CHOG_DAY[(d0+i)%7]:CHOG_NIGHT[(n0+i-8)%7];const s=day?rise+i*dp:set+(i-8)*np;return {n:(day?i:i-8)+1,name,ruler:CHOG[name][0],nature:CHOG[name][1],meaning:CHOG[name][2],s,e:s+(day?dp:np),period:day?'day':'night'}});
    const part=k=>({s:rise+(k-1)*dp,e:rise+k*dp});
    const mid=(rise+set)/2,muh=(set-rise)/15;
    return {date,weekday:wd,lord:lord0,rise,set,next,horas,chog,rahu:part(RAHU[wd]),yama:part(YAMA[wd]),gulika:part(GULI[wd]),abhijit:{s:mid-muh/2,e:mid+muh/2},
      organs:ORGANS.map(([o,el,h])=>({organ:o,element:el,s:h,e:h+2})),doshas:DOSHAS.map(([d,h])=>({dosha:d,s:h,e:h+4}))}}
  function vedic(date,h){const C=clock(date);if(h<C.rise){const P=clock(addD(date,-1));return {C:P,H:h+24,prev:true}}return {C,H:h,prev:false}}
  function at(C,h){const H=h<C.rise?h+24:h;const f=a=>a.find(x=>x.s<=H&&H<x.e);const oh=((h%24)+24)%24;
    return {hora:f(C.horas),chog:f(C.chog),organ:C.organs.find(o=>{const s=o.s%24,e=o.e%24;return s<e?(s<=oh&&oh<e):(oh>=s||oh<e)}),dosha:C.doshas.find(o=>{const s=o.s%24,e=o.e%24;return s<e?(s<=oh&&oh<e):(oh>=s||oh<e)})}}
  /* Biorhythm: the engine's six sine cycles counted from birth, and its three composites */
  const CYC=[['physical',23],['emotional',28],['intellectual',33],['intuitive',38],['aesthetic',43],['spiritual',53]];
  // ported from crates/engine-biorhythm/src/calculator.rs: phase_label, is_critical_day, days_until_peak, days_until_critical
  const isCrit=(n,P)=>Math.abs(Math.sin(2*Math.PI*n/P))<Math.abs(Math.sin(2*Math.PI*1/P));
  const untilPeak=(n,P)=>{const ph=(n%P)/P,dist=ph<=.25?.25-ph:1.25-ph;const k=Math.ceil(dist*P);return k===0?P:k};
  const untilCrit=(n,P)=>{const ph=(n%P)/P;let m=Infinity;for(const t of [.5,1]){if(ph<t){const k=Math.ceil((t-ph)*P);if(k>0&&k<m)m=k}}return m===Infinity?Math.ceil((1-ph)*P):m};
  const phaseOf=(v,n,P)=>isCrit(n,P)?'Critical':v>.95?'Peak':v<-.95?'Low':Math.cos(2*Math.PI*n/P)>0?'Rising':'Falling';
  const daysAlive=date=>Math.round((dms(date)-dms(BIRTH.date))/864e5);
  function bio(date){const d=daysAlive(date);const out={days:d};
    CYC.forEach(([k,P])=>{const v=Math.sin(2*Math.PI*d/P);out[k]={value:v,pct:(v+1)/2*100,cycle_day:((d%P)+P)%P,phase:phaseOf(v,d,P),critical:isCrit(d,P),until_peak:untilPeak(d,P),until_critical:untilCrit(d,P),period:P}});
    out.passion=(out.physical.pct+out.emotional.pct)/2;out.mastery=(out.physical.pct+out.intellectual.pct)/2;out.wisdom=(out.emotional.pct+out.intellectual.pct)/2;
    out.energy=CYC.reduce((a,[k])=>a+out[k].pct,0)/6;return out}
  function criticalDays(date,window){const d=daysAlive(date),out=[];for(let o=1;o<=window;o++){const n=d+o;if(isCrit(n,23)||isCrit(n,28)||isCrit(n,33))out.push(addD(date,o))}return out}
  /* Numerology: personal year, month and day by the Pythagorean method, masters kept */
  const red=n=>{while(n>9&&n!==11&&n!==22&&n!==33)n=String(n).split('').reduce((a,b)=>a+ +b,0);return n};
  const digits=s=>String(s).split('').reduce((a,b)=>a+ +b,0);
  function personal(date){const [y,m,d]=date.split('-').map(Number);const py=red(red(8)+red(13)+red(digits(y)));const pm=red(py+red(m));const pd=red(pm+red(digits(d)));return {year:py,month:pm,day:pd}}
  const NUM_KEY={1:'beginning, initiative',2:'pairing, patience',3:'expression, play',4:'structure, work',5:'change, freedom',6:'care, responsibility',7:'study, solitude',8:'power, harvest',9:'completion, release',11:'insight, illumination',22:'master building',33:'master teaching'};
  /* Tarabala and Chandrabala: today's Moon counted from the birth Moon */
  const TARA=['Janma','Sampat','Vipat','Kshema','Pratyari','Sadhaka','Vadha','Mitra','Param Mitra'];
  const TARA_KEY=['birth','wealth','hazard','wellbeing','obstruction','accomplishment','harm','friendship','great friendship'];
  function tara(moonNak,birthNak){const k=((moonNak-birthNak+27)%27)%9;return {n:k+1,name:TARA[k],key:TARA_KEY[k]}}
  function chandra(moonSign,birthSign){return ((moonSign-birthSign+12)%12)+1}
  /* Verification against the stored responses: returns rows of [check, field, engine, ok] */
  function verify(ENGR){const rows=[];
    const vc=ENGR['vedic-clock']&&ENGR['vedic-clock'].result;
    if(vc){const hm=s=>{const t=s.slice(11,19).split(':').map(Number);return t[0]+t[1]/60+t[2]/3600};
      const civ=new Date(Date.parse(vc.calculated_for)+5.5*36e5).toISOString(),cd=civ.slice(0,10),ch=hm(civ);const V=vedic(cd,ch),C=V.C,d=vc.vedic_day.date;
      rows.push([`Vedic day at ${cd} ${fmtH(ch)}`,C.date,d,C.date===d]);
      const er=hm(vc.vedic_day.sunrise),es=hm(vc.vedic_day.sunset);rows.push(['Sunrise '+d,fmtHs(C.rise),fmtHs(er),Math.abs(C.rise-er)<2/60]);rows.push(['Sunset '+d,fmtHs(C.set),fmtHs(es),Math.abs(C.set-es)<2/60]);
      const A=at(C,V.H);rows.push(['Hora at call',`${A.hora.lord} (${A.hora.n})`,`${vc.hora.ruler} (${vc.hora.number})`,A.hora.lord===vc.hora.ruler&&A.hora.n===vc.hora.number]);
      rows.push(['Hora window',`${fmtH(A.hora.s)}–${fmtH(A.hora.e)}`,`${vc.hora.start.slice(11,16)}–${vc.hora.end.slice(11,16)}`,fmtH(A.hora.s)===vc.hora.start.slice(11,16)]);
      rows.push(['Choghadiya at call',A.chog.name,vc.choghadiya.name,A.chog.name===vc.choghadiya.name]);rows.push(['Choghadiya window',`${fmtH(A.chog.s)}–${fmtH(A.chog.e)}`,`${vc.choghadiya.start.slice(11,16)}–${vc.choghadiya.end.slice(11,16)}`,fmtH(A.chog.s)===vc.choghadiya.start.slice(11,16)]);
      rows.push(['Organ at call',A.organ.organ,vc.current_organ.organ,A.organ.organ.replace(/ /g,'').toLowerCase()===vc.current_organ.organ.toLowerCase()]);rows.push(['Dosha at call',A.dosha.dosha,vc.current_dosha.dosha,A.dosha.dosha===vc.current_dosha.dosha])}
    const br=ENGR.biorhythm&&ENGR.biorhythm.result;
    if(br){const B=bio(br.target_date);rows.push(['Biorhythm days alive',String(B.days),String(br.days_alive),B.days===br.days_alive]);
      CYC.forEach(([k])=>{const e=br[k],f=B[k];rows.push([`Biorhythm ${k}`,`${f.pct.toFixed(2)}% · ${f.phase}${f.critical?' · critical':''} · peak ${f.until_peak}d · zero ${f.until_critical}d`,`${e.percentage.toFixed(2)}% · ${e.phase}${e.is_critical?' · critical':''} · peak ${e.days_until_peak}d · zero ${e.days_until_critical}d`,
        Math.abs(f.pct-e.percentage)<.01&&f.phase===e.phase&&f.critical===e.is_critical&&f.until_peak===e.days_until_peak&&f.until_critical===e.days_until_critical])});
      rows.push(['Biorhythm energy',B.energy.toFixed(2),br.overall_energy.toFixed(2),Math.abs(B.energy-br.overall_energy)<.01]);
      const cd=criticalDays(br.target_date,(br.forecast||[]).length||7);rows.push(['Biorhythm critical days',cd.join(', ')||'none',(br.critical_days||[]).join(', ')||'none',cd.join()===(br.critical_days||[]).join()])}
    return rows}
  return {sunTimes,clock,vedic,at,bio,criticalDays,personal,tara,chandra,verify,fmtH,fmtHs,NUM_KEY,CHOG,HORA_KEY,CYC,TARA,addD};
})();
if(typeof module!=='undefined')module.exports=DL;
