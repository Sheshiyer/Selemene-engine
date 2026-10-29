const topo=require('topojson-client');const w=require('./node_modules/world-atlas/land-110m.json');
const land=topo.feature(w,w.objects.land);
const X=l=>((l+180)/360*1000).toFixed(1), Y=p=>((90-p)/180*500).toFixed(1);
let d='';
for(const f of (land.type==='FeatureCollection'?land.features:[land])){
 const g=f.geometry; const polys=g.type==='Polygon'?[g.coordinates]:g.coordinates;
 for(const poly of polys) for(const ring of poly){
   let prev=null; ring.forEach((pt,i)=>{const jump=prev&&Math.abs(pt[0]-prev[0])>180; d+=(i===0||jump?'M':'L')+X(pt[0])+' '+Y(pt[1]); prev=pt}); d+='Z';}
}
require('fs').writeFileSync('land.txt',d);console.log(d.length);
