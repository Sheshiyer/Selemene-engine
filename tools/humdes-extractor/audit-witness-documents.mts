import {join,resolve} from 'node:path';import{pathToFileURL}from'node:url';import{createHash}from'node:crypto';
const root=process.env.WITNESS_NODE_PACKAGES;if(!root)throw Error('Bundled package root required');
const canvas=await import(pathToFileURL(join(root,'@napi-rs/canvas/index.js')).href);
Object.assign(globalThis,{DOMMatrix:canvas.DOMMatrix,Path2D:canvas.Path2D,ImageData:canvas.ImageData});
const pdfjs=await import(pathToFileURL(join(root,'pdfjs-dist/legacy/build/pdf.mjs')).href);
const JSZip=(await import(pathToFileURL(join(root,'jszip/lib/index.js')).href)).default;
const dir=resolve(process.argv[2]);const stem=process.argv[3];const mdPath=resolve(process.argv[4]);
const decode=(s:string)=>s.replace(/&amp;/g,'&').replace(/&lt;/g,'<').replace(/&gt;/g,'>').replace(/&quot;/g,'"').replace(/&apos;/g,"'");
const bytes=await Bun.file(join(dir,stem+'.docx')).arrayBuffer();const zip=await JSZip.loadAsync(bytes);const xml=await zip.file('word/document.xml')!.async('string');
const text=[...xml.matchAll(/<w:t(?:\s[^>]*)?>([\s\S]*?)<\/w:t>/g)].map(m=>decode(m[1])).join(' ');
const pdf=await pdfjs.getDocument({data:new Uint8Array(await Bun.file(join(dir,stem+'.pdf')).arrayBuffer()),useSystemFonts:true,disableFontFace:true}).promise;
let pdfText='';for(let i=1;i<=pdf.numPages;i++){const p=await pdf.getPage(i);pdfText+=(await p.getTextContent()).items.map((x:any)=>x.str??'').join(' ')+'\n';}
await Bun.write(join(dir,stem+'.pdf-text.txt'),pdfText);
const md=await Bun.file(mdPath).text();const chapters=[...md.matchAll(/^## (?:Opening|Ouverture|Part(?:ie)?\s+[IVX]+)\b.*$/gm)].map(m=>m[0].slice(3));
const normalize=(s:string)=>s.normalize('NFKC').replace(/[\s\u00ad]+/g,'').replace(/[’‘]/g,"'").replace(/[–—]/g,'-');const pnorm=normalize(pdfText),dnorm=normalize(text);
let prev=-1;const results=chapters.map(title=>{const n=normalize(title);const docx=dnorm.includes(n);const at=pnorm.indexOf(n);const pdfPresent=at>=0;const order=at>prev;prev=at;return{title,docx,pdf:pdfPresent,order};});
const tokens=(s:string)=>s.toLowerCase().normalize('NFKC').match(/[\p{L}\p{N}]+/gu)??[];const ds=tokens(text),ps=new Set(tokens(pdfText));const missing=[...new Set(ds.filter(t=>!ps.has(t)))];
const counts=(ts:string[])=>{const m=new Map<string,number>();for(const t of ts)m.set(t,(m.get(t)??0)+1);return m};
const visibleMd=md.replace(/\[([^\]]+)\]\([^)]+\)/g,'$1').split('\n').filter(l=>!/^\s*\|?[\s:|-]+\|\s*$/.test(l)&&!/^\s*-{3,}\s*$/.test(l)).map(l=>l.replace(/^\s*\d+\.\s+/,'').replace(/^\s*#{1,4}\s+/, '').replace(/^\s*[-*>]\s+/,'')).join('\n');
const mc=counts(tokens(visibleMd)),docCounts=counts(ds);const mdToDocxDeficits=[...mc].filter(([t,n])=>(docCounts.get(t)??0)<n).map(([token,markdown])=>({token,markdown,docx:docCounts.get(token)??0}));
const dc=counts(ds),rawPdfTokens=tokens(pdfText),normalizedPdfTokens:string[]=[],wrapJoins:any[]=[];
for(let i=0;i<rawPdfTokens.length;i++){
 let joined=false;
 for(let length=2;length<=4&&i+length<=rawPdfTokens.length;length++){
  const pieces=rawPdfTokens.slice(i,i+length),word=pieces.join('');
  if(dc.has(word)&&pieces.some(piece=>!dc.has(piece))){normalizedPdfTokens.push(word);wrapJoins.push({pieces,word});i+=length-1;joined=true;break;}
 }
 if(!joined)normalizedPdfTokens.push(rawPdfTokens[i]);
}
const pc=counts(normalizedPdfTokens);const countDeficits=[...dc].filter(([t,n])=>(pc.get(t)??0)<n).map(([token,docx])=>({token,docx,pdf:pc.get(token)??0}));
const hash=async(p:string)=>createHash('sha256').update(new Uint8Array(await Bun.file(p).arrayBuffer())).digest('hex');
const r={stem,sourceSha256:await hash(mdPath),docxSha256:await hash(join(dir,stem+'.docx')),pdfSha256:await hash(join(dir,stem+'.pdf')),pdfPages:pdf.numPages,chapters:results,docxTextTokens:ds.length,markdownToDocxTokenCountDeficits:mdToDocxDeficits,missingPdfDistinctTokens:missing,tableWrapReconstruction:wrapJoins,pdfTokenCountDeficits:countDeficits,structuralPass:results.length===12&&results.every(c=>c.docx&&c.pdf&&c.order)&&missing.length===0&&countDeficits.length===0&&mdToDocxDeficits.length===0,visualReview:'separate page inspection required'};
await Bun.write(join(dir,stem+'-artifact-audit.json'),JSON.stringify(r,null,2));console.log(JSON.stringify(r));await pdf.destroy();
