const {JSDOM}=require('jsdom');const fs=require('fs');
const html=fs.readFileSync('witness-alchemist-field.html','utf8');
const dom=new JSDOM(html,{runScripts:'dangerously',url:'https://x.test/#sky',pretendToBeVisual:true});
const w=dom.window;w.scrollTo=()=>{};w.matchMedia=()=>({matches:false});
w.addEventListener('error',e=>console.log('ERR',e.message));
setTimeout(()=>{
 for(const r of ['sky','today','design','keys','engines','jyotish','timeline','map','folio']){w.location.hash='#'+r;}
 const check=r=>new Promise(res=>{w.location.hash='#'+r;setTimeout(()=>{const m=w.document.getElementById('main');console.log(r,m.innerHTML.length,m.textContent.slice(0,140).replace(/\s+/g,' '));res()},50)});
 (async()=>{for(const r of ['sky','today','design','keys','engines','jyotish','timeline','map','folio'])await check(r);
  // click tests
  w.location.hash='#today';await new Promise(r=>setTimeout(r,50));
  w.document.querySelector('[data-ch]').click();console.log('drawer',w.document.getElementById('drawerBody').textContent.slice(0,120));
  w.document.querySelector('.gate-dot').dispatchEvent(new w.Event('click'));
  w.document.getElementById('nextDay').click();console.log('date',w.document.getElementById('dateIn').value);
  w.location.hash='#map';await new Promise(r=>setTimeout(r,50));
  w.document.querySelector('.city').dispatchEvent(new w.MouseEvent('click',{bubbles:true}));console.log('city',w.document.getElementById('drawerBody').textContent.slice(0,300));
  w.location.hash='#timeline';await new Promise(r=>setTimeout(r,50));w.document.querySelector('[data-ad]').click();console.log('ad ok');
  w.location.hash='#design';await new Promise(r=>setTimeout(r,50));w.document.querySelector('[data-chap]').click();console.log('chap',w.document.getElementById('drawerBody').textContent.slice(0,100));
  w.location.hash='#sky';await new Promise(r=>setTimeout(r,50));console.log('wheel',!!w.document.querySelector('svg.wheel'),'bodies',w.document.querySelectorAll('svg.wheel .pl').length);
  w.document.querySelector('svg.wheel circle.pl').dispatchEvent(new w.Event('click'));console.log('body',w.document.getElementById('drawerBody').textContent.slice(0,120));
  w.document.getElementById('skyVedic').click();await new Promise(r=>setTimeout(r,30));console.log('vedic',w.document.getElementById('skyVedic').getAttribute('aria-pressed'),w.document.querySelector('svg.wheel text').textContent);
  w.document.getElementById('skyGates').click();await new Promise(r=>setTimeout(r,30));console.log('gates',w.document.querySelectorAll('svg.wheel text').length);
  w.location.hash='#today';await new Promise(r=>setTimeout(r,80));console.log('today lenses',w.document.querySelectorAll('.lens-row:not(.lens-head)').length,'ribbon days',w.document.querySelectorAll('.rhit').length,'dial',!!w.document.querySelector('#dialHost svg'));
  {const h=w.document.getElementById('dialH');h.value=13.5;h.dispatchEvent(new w.Event('input'));console.log('dial @13:30',w.document.getElementById('dialRead').textContent);w.document.getElementById('dialCheck').click();console.log('method check',w.document.getElementById('drawerBody').textContent.match(/\d+ of \d+ match/)[0]);w.document.getElementById('ovKeys').click();console.log('overlay pref',w.localStorage.getItem('wa-loom'))}
  w.location.hash='#engines';await new Promise(r=>setTimeout(r,50));console.log('engines cells',w.document.querySelectorAll('.hexcell').length,'index',w.document.querySelectorAll('.eng-index button').length);
  for(const b of [...w.document.querySelectorAll('.eng-index button')]){b.click();const rd=w.document.getElementById('engRead');const op=w.document.getElementById('engOpen');if(op)op.click();console.log(' ',b.textContent.trim().padEnd(16),'readout',rd.textContent.length,op?'drawer '+w.document.getElementById('drawerBody').textContent.length:'')}
  w.location.hash='#jyotish';await new Promise(r=>setTimeout(r,50));console.log('jy chart',!!w.document.querySelector('#jyChart svg'),'grahas',w.document.querySelectorAll('#jyChart .pl').length,'md',w.document.querySelectorAll('[data-md]').length,'ad',w.document.querySelectorAll('[data-ad]').length);
  w.document.querySelector('#jyChart .pl').dispatchEvent(new w.Event('click'));console.log('graha',w.document.getElementById('drawerBody').textContent.slice(0,100));
  w.document.getElementById('jyNorth').click();await new Promise(r=>setTimeout(r,30));console.log('north',w.document.querySelector('#jyChart svg').getAttribute('aria-label'));
  w.document.querySelector('[data-md]').click();console.log('md0 ad rows',w.document.querySelectorAll('[data-ad]').length);
  w.document.querySelectorAll('[data-md]')[4].click();w.document.querySelectorAll('[data-ad]')[1].click();console.log('jup/sat pd rows',w.document.querySelectorAll('#jyDasha .grid3 > div:nth-child(3) .item').length);

 })();
},300);
