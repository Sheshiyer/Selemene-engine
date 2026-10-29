/* ---------- Engines: the eight Selemene engines this key can run, each figure traced to its response ---------- */
const ENGINE_META={
  'human-design':['Human Design','Type, profile, authority, channels from the Swiss Ephemeris'],
  'gene-keys':['Gene Keys','Activation spheres, derived from the Human Design activations'],
  'numerology':['Numerology','Pythagorean and Chaldean figures from name and birth date'],
  'panchanga':['Panchanga','The five limbs of the birth day'],
  'vimshottari':['Vimshottari','Mahadasha, antardasha and pratyantardasha, from the birth Moon'],
  'transits':['Transits','Sidereal aspects of the sky to the birth chart'],
  'vedic-clock':['Vedic Clock','Hora, choghadiya, dosha and organ at the moment of the call'],
  'biorhythm':['Biorhythm','Seven cycles counted from birth'],
  'biofield':['Biofield',''],'nadabrahman':['Nadabrahman',''],'raaga':['Raaga',''],'i-ching':['I Ching',''],'sacred-geometry':['Sacred Geometry','']};
const ENGINE_ORDER=['human-design','gene-keys','panchanga','vimshottari','transits','vedic-clock','biorhythm','numerology'];
const LOCKED_ORDER=['biofield','nadabrahman','raaga','i-ching','sacred-geometry'];
function istStamp(iso){if(!iso)return '';const d=new Date(iso);if(isNaN(d))return iso;const t=new Date(d.getTime()+5.5*36e5);return `${t.getUTCDate()} ${MON[t.getUTCMonth()]} ${t.getUTCFullYear()} ${String(t.getUTCHours()).padStart(2,'0')}:${String(t.getUTCMinutes()).padStart(2,'0')} IST`}
function istDate(iso){return iso?fmtDate(new Date(new Date(iso).getTime()+5.5*36e5).toISOString().slice(0,10)):''}
const cap=s=>String(s).replace(/_/g,' ').replace(/^\w/,c=>c.toUpperCase());
const E=id=>(ENG[id]&&ENG[id].result)||null;
function engEyebrow(id){const e=ENG[id];return `<div class="ey"><span class="eyebrow">${ENGINE_META[id][0]}</span><span class="tag c">computed</span></div>`}
function engFoot(id){const e=ENG[id];return `<span class="meta">${e.backend||'engine'} · v${e.version||''} · ${istStamp(e.at)}</span>`}

/* Summaries on the cards */
function engCard(id){
  const r=E(id);if(!r)return '';let body='';
  switch(id){
    case 'human-design':{const agree=NAT.filter(n=>{const a=r[(n.side==='P'?'personality':'design')+'_activations'][n.body.toLowerCase()];return a&&a.gate===n.gate&&a.line===n.line}).length;
      body=`<div class="big">${r.hd_type} · ${r.profile}</div><div class="rows"><span>Authority <b>${r.authority}</b></span><span>Definition <b>${r.definition}</b></span><span>Channels <b>${r.active_channels.join(', ')}</b></span><span>Centers <b>${r.defined_centers.map(c=>c.replace('SolarPlexus','Solar Plexus')).join(', ')}</b></span><span><span class="ev-c">Agrees with the field on ${agree} of ${NAT.length} activations</span></span></div>`;break}
    case 'gene-keys':{const S=r.activation_spheres;const K=Object.fromEntries(r.active_keys.map(k=>[k.source,k]));
      body=`<div class="rows">${[['lifes_work',"Life's Work"],['evolution','Evolution'],['radiance','Radiance'],['purpose','Purpose']].map(([k,n])=>{const s=S[k],key=K[s.source]||{};return `<span>${n} <b>${s.key_number}.${s.line}</b> · ${key.shadow||''} → ${key.gift||''} → ${key.siddhi||''}</span>`}).join('')}</div><p class="meta" style="margin:0">Four spheres from the engine. Venus and Pearl sequences come from the field mapping in the Keys room.</p>`;break}
    case 'numerology':{body=`<div class="rows">${[['life_path','Life Path'],['expression','Expression'],['soul_urge','Soul Urge'],['personality','Personality'],['birthday','Birthday'],['chaldean_name','Chaldean name']].map(([k,n])=>`<span>${n} <b>${r[k].value}${r[k].is_master?' master':''}</b> · ${r[k].meaning.replace(/ \(master\)/,'')}</span>`).join('')}</div>`;break}
    case 'panchanga':{body=`<div class="big">${r.tithi_name}</div><div class="rows"><span>Nakshatra <b>${r.nakshatra_name}</b> · ${r.nakshatra_value.toFixed(2)}</span><span>Yoga <b>${r.yoga_name}</b></span><span>Karana <b>${r.karana_name}</b></span><span>Vara <b>${r.vara_name}</b></span><span>Moon <b>${RASHI[Math.floor(r.lunar_longitude/30)]} ${fmtDeg(r.lunar_longitude%30)}</b> sidereal</span></div>`;break}
    case 'vimshottari':{const c=r.current_period,u=r.upcoming_transitions[0];
      body=`<div class="big">${c.mahadasha.planet} / ${c.antardasha.planet} / ${c.pratyantardasha.planet}</div><div class="rows"><span>Mahadasha <b>${c.mahadasha.planet}</b> · ${istDate(c.mahadasha.start)} to ${istDate(c.mahadasha.end)}</span><span>Antardasha <b>${c.antardasha.planet}</b> · to ${istDate(c.antardasha.end)}</span><span>Pratyantardasha <b>${c.pratyantardasha.planet}</b> · to ${istDate(c.pratyantardasha.end)}</span>${u?`<span>Next <b>${u.from_planet} → ${u.to_planet}</b> ${cap(u.type)} · ${istDate(u.date)}</span>`:''}</div>`;break}
    case 'transits':{const tight=r.aspects.filter(a=>a.orb<=1).slice(0,5);
      body=`<div class="big">${r.period_quality}</div><div class="rows"><span>Sade Sati <b>${r.sade_sati.is_active?'active':'inactive'}</b> · Moon ${r.sade_sati.moon_sign}, Saturn ${r.sade_sati.saturn_sign}</span><span>Retrograde <b>${r.retrograde_planets.join(', ')}</b></span>${tight.map(a=>`<span>${a.transiting_planet} ${a.aspect_type.toLowerCase()} natal ${a.natal_planet} <b>${a.orb.toFixed(2)}°</b> · ${a.nature}</span>`).join('')}</div>`;break}
    case 'vedic-clock':{body=`<div class="big">${r.hora.ruler} hora · ${r.choghadiya.name}</div><div class="rows"><span>Hora <b>${r.hora.ruler}</b> · ${r.hora.start.slice(11,16)} to ${r.hora.end.slice(11,16)} · ${r.hora.is_favorable?'favourable':'neutral'}</span><span>Choghadiya <b>${r.choghadiya.name}</b> · ${r.choghadiya.meaning}</span><span>Dosha <b>${r.current_dosha.dosha}</b> · organ ${r.current_organ.organ} (${r.current_organ.element})</span><span>Sunrise <b>${r.vedic_day.sunrise.slice(11,16)}</b> · sunset <b>${r.vedic_day.sunset.slice(11,16)}</b> · ${r.vedic_day.weekday}</span></div>`;break}
    case 'biorhythm':{const C=['physical','emotional','intellectual','intuitive','aesthetic','spiritual'];
      body=`<div class="big">${Math.round(r.overall_energy)}% energy</div><div class="bars">${C.map(c=>`<div><span>${cap(c)}</span><i><b style="width:${Math.max(2,Math.round(r[c].percentage))}%;background:${r[c].is_critical?'var(--terracotta)':r[c].value>=0?'var(--emerald)':'var(--violet-glow)'}"></b></i><span>${Math.round(r[c].percentage)}%</span></div>`).join('')}</div><span class="meta">Day ${r.days_alive} · ${r.target_date} · critical: ${r.critical_days.length?r.critical_days.join(', '):'none'}</span>`;break}
  }
  return `<button class="eng" data-eng="${id}">${engEyebrow(id)}<h3>${ENGINE_META[id][1]}</h3>${body}${ENG[id].prompt?`<p class="q" style="margin:0">${esc(ENG[id].prompt)}</p>`:''}${engFoot(id)}</button>`;
}

/* Full detail in the drawer */
function kv(obj,keys){return `<dl class="kv">${(keys||Object.keys(obj)).filter(k=>obj[k]!==undefined&&obj[k]!==null&&typeof obj[k]!=='object').map(k=>`<dt>${cap(k)}</dt><dd>${esc(obj[k])}</dd>`).join('')}</dl>`}
function showEngine(id){
  const r=E(id),e=ENG[id];let html='';
  switch(id){
    case 'human-design':{html=`<div class="tbl"><table><thead><tr><th>Body</th><th>Personality</th><th>Design</th></tr></thead><tbody>${['sun','earth','moon','northnode','southnode','mercury','venus','mars','jupiter','saturn','uranus','neptune','pluto'].map(b=>{const p=r.personality_activations[b],d=r.design_activations[b];return `<tr><td>${cap(b.replace('northnode','North Node').replace('southnode','South Node'))}</td><td class="num">${p.gate}.${p.line} · ${fmtLon(p.longitude)}</td><td class="num">${d.gate}.${d.line} · ${fmtLon(d.longitude)}</td></tr>`}).join('')}</tbody></table></div>`;break}
    case 'gene-keys':{html=`<div class="list">${r.active_keys.map(k=>`<button class="item" data-gate="${k.key_number}"><span class="k">${k.key_number}.${k.line}</span><span><b>${cap(k.source.replace(/([A-Z])/g,' $1').trim())}</b><small>${k.shadow} → ${k.gift} → ${k.siddhi} · partner ${k.programming_partner}</small></span></button>`).join('')}</div><p class="meta">${esc(r.meaning_source_quality.note||'')}</p>`;break}
    case 'numerology':{html=`<div class="list">${Object.entries(r).map(([k,v])=>`<div class="item" style="cursor:default"><span class="k">${v.value}</span><span><b>${cap(k)}${v.is_master?' · master':''}</b><small>${v.meaning} · reduction ${v.reduction_chain.join(' → ')}</small></span></div>`).join('')}</div>`;break}
    case 'panchanga':{html=kv(r,['tithi_name','tithi_value','nakshatra_name','nakshatra_value','yoga_name','yoga_value','karana_name','karana_value','vara_name','lunar_longitude','solar_longitude','julian_day','position_source']);break}
    case 'vimshottari':{const T=r.timeline.mahadashas,c=r.current_period,P=r.period_enrichment;
      html=`<h3>Current period</h3>${kv({mahadasha:`${c.mahadasha.planet} · ${istDate(c.mahadasha.start)} to ${istDate(c.mahadasha.end)}`,antardasha:`${c.antardasha.planet} · ${istDate(c.antardasha.start)} to ${istDate(c.antardasha.end)}`,pratyantardasha:`${c.pratyantardasha.planet} · ${istDate(c.pratyantardasha.start)} to ${istDate(c.pratyantardasha.end)}`})}
      <h3>Upcoming transitions</h3><div class="list">${r.upcoming_transitions.map(u=>`<div class="item" style="cursor:default"><span class="k">${istDate(u.date)}</span><span><b>${u.from_planet} → ${u.to_planet}</b><small>${cap(u.type)} · in ${u.days_until} days</small></span></div>`).join('')}</div>
      <h3>Mahadashas</h3><div class="tbl"><table><thead><tr><th>Planet</th><th>From</th><th>To</th><th>Years</th></tr></thead><tbody>${T.map(m=>`<tr${m.planet===c.mahadasha.planet&&m.start_date===c.mahadasha.start?' style="color:var(--gold)"':''}><td>${m.planet}</td><td class="num">${istDate(m.start_date)}</td><td class="num">${istDate(m.end_date)}</td><td class="num">${m.duration_years.toFixed(2)}</td></tr>`).join('')}</tbody></table></div>
      <h3>Themes named by the engine</h3><p class="muted">${P.mahadasha_themes.join(', ')}. Life areas: ${P.life_areas.join(', ')}. Watch: ${P.challenges.join(', ').toLowerCase()}.</p>`;break}
    case 'transits':{html=`<p class="muted">Sidereal (Lahiri). Period quality: ${r.period_quality}.</p><div class="tbl"><table><thead><tr><th>Transit</th><th>Aspect</th><th>Natal</th><th>Orb</th><th>Nature</th></tr></thead><tbody>${r.aspects.map(a=>`<tr><td>${a.transiting_planet}</td><td>${a.aspect_type}${a.is_applying?' ↗':''}</td><td>${a.natal_planet}</td><td class="num">${a.orb.toFixed(2)}°</td><td>${a.nature}</td></tr>`).join('')}</tbody></table></div>
      <h3>Sky at the call</h3><div class="tbl"><table><thead><tr><th>Planet</th><th>Sign</th><th>Degree</th></tr></thead><tbody>${r.transit_positions.map(p=>`<tr><td>${p.planet}${p.is_retrograde?' R':''}</td><td>${p.sign}</td><td class="num">${fmtDeg(p.degree_in_sign)}</td></tr>`).join('')}</tbody></table></div>`;break}
    case 'vedic-clock':{html=`<p class="muted">A snapshot for ${istStamp(r.calculated_for)}. This engine answers for the moment it is called.</p>${kv({hora:`${r.hora.ruler} (${r.hora.number}) · ${r.hora.start.slice(11,16)} to ${r.hora.end.slice(11,16)}`,choghadiya:`${r.choghadiya.name} · ${r.choghadiya.ruler} · ${r.choghadiya.meaning}`,dosha:r.current_dosha.dosha,organ:`${r.current_organ.organ} · ${r.current_organ.element} · ${r.current_organ.time_window}`,sunrise:r.vedic_day.sunrise.slice(11,16),sunset:r.vedic_day.sunset.slice(11,16),next_sunrise:r.vedic_day.next_sunrise.slice(11,16)})}
      <h3>Upcoming transitions</h3><div class="list">${r.upcoming_transitions.map(u=>`<div class="item" style="cursor:default"><span class="k">${u.time}</span><span><b>${esc(u.description)}</b></span></div>`).join('')}</div><h3>Synthesis</h3><p class="muted">${esc(r.synthesis)}</p>`;break}
    case 'biorhythm':{const C=['physical','emotional','intellectual','intuitive','aesthetic','spiritual'];
      html=`<div class="tbl"><table><thead><tr><th>Cycle</th><th>Now</th><th>Phase</th><th>Peak in</th><th>Critical in</th></tr></thead><tbody>${C.map(c=>`<tr><td>${cap(c)}</td><td class="num">${Math.round(r[c].percentage)}%</td><td>${r[c].phase}</td><td class="num">${r[c].days_until_peak} d</td><td class="num">${r[c].days_until_critical} d</td></tr>`).join('')}</tbody></table></div>
      <h3>Seven days</h3><div class="tbl"><table><thead><tr><th>Date</th><th>Phys</th><th>Emo</th><th>Int</th><th>Energy</th></tr></thead><tbody>${r.forecast.map(f=>`<tr><td class="num">${f.date}</td><td class="num">${Math.round(f.physical)}</td><td class="num">${Math.round(f.emotional)}</td><td class="num">${Math.round(f.intellectual)}</td><td class="num">${Math.round(f.overall_energy)}</td></tr>`).join('')}</tbody></table></div>${kv({passion:r.passion.toFixed(1),wisdom:r.wisdom.toFixed(1),mastery:r.mastery.toFixed(1)})}`;break}
  }
  openDrawer(`<div class="eyebrow">Selemene · ${ENGINE_META[id][0]}</div><h2>${ENGINE_META[id][1]}</h2>${e.prompt?`<p class="q">${esc(e.prompt)}</p>`:''}${html}<p class="meta" style="margin-top:12px">${engFoot(id)}</p>`);
  wire($('#drawerBody'));
}

function rEngines(){
  const ok=ENGINE_ORDER.filter(E),locked=LOCKED_ORDER.filter(id=>ENG[id]&&!ENG[id].result);
  $('#main').innerHTML=`<section class="room">
  <div class="hero"><div class="epithet">Engine pool · natal · ${istDate(ENG['human-design']&&ENG['human-design'].at)}</div><p class="decl">${ok.length} engines. One chart.</p>
   <p class="lede">Each figure below is read straight from a Selemene response for your birth data. Nothing is interpolated. Tap an engine for its full return; the Vedic Clock and Biorhythm answer for the moment they were called.</p>
   <div class="chips"><span class="chip"><span class="dot c"></span><span class="ev-c">Computed by engine</span></span><span class="chip"><span class="dot u"></span><span class="ev-u">${locked.length} locked on this key</span></span><span class="chip">selemene.tryambakam.space</span></div></div>
  <div class="eng-grid">${ok.map(engCard).join('')}</div>
  ${locked.length?`<div><div class="divider"><span></span></div><h2>Locked on this key</h2><div class="eng-grid">${locked.map(id=>`<div class="eng locked"><div class="ey"><span class="eyebrow">${ENGINE_META[id][0]}</span><span class="tag" style="color:var(--ev-unresolved);border-color:rgba(198,93,59,.5)">${ENG[id].status||403}</span></div><p class="muted" style="margin:0">${esc(ENG[id].error||'ENGINE_SCOPE_FORBIDDEN').replace(/_/g,' ').toLowerCase()}. Opens when the key tier allows.</p></div>`).join('')}</div></div>`:''}
  </section>`;
  $('#main').querySelectorAll('[data-eng]').forEach(b=>b.onclick=()=>showEngine(b.dataset.eng));
}
