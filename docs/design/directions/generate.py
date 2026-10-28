#!/usr/bin/env python3
"""Generate the four compass-direction SVGs (gold line on void, vesica frame, emerald lens, stroke-draw loop).
Usage: python3 generate.py [out_dir]   (defaults to this folder). Also writes *-static.svg without the animation."""
import math, sys, os, re
D = sys.argv[1] if len(sys.argv) > 1 else os.path.dirname(os.path.abspath(__file__))
GOLD="#C5A017"; VOID="#070B1D"; EM="#10B5A7"; W=1024; C=512
def vesica():
    r=420; d=300; h=math.sqrt(r*r-(d/2)**2)
    return f'<path class="frame" pathLength="1" d="M {C} {C-h:.1f} A {r} {r} 0 0 1 {C} {C+h:.1f} A {r} {r} 0 0 1 {C} {C-h:.1f} Z"/>'
def polyline(pts, cls="l", close=True):
    d="M "+" L ".join(f"{x:.1f} {y:.1f}" for x,y in pts)+(" Z" if close else "")
    return f'<path class="{cls}" pathLength="1" d="{d}"/>'
def circle(cx,cy,r,cls="l"): return f'<circle class="{cls}" pathLength="1" cx="{cx:.1f}" cy="{cy:.1f}" r="{r:.1f}"/>'
def head(title):
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {W}" width="{W}" height="{W}" role="img" aria-label="{title}">
<title>{title}</title>
<style>
.l,.frame{{fill:none;stroke:{GOLD};stroke-width:1.6;stroke-linecap:round;stroke-linejoin:round;vector-effect:non-scaling-stroke}}
.frame{{stroke-width:1.2;opacity:.7}}
.d{{fill:{GOLD}}}
.lens{{fill:{EM}}}
.glow{{fill:{EM};opacity:.35}}
@media (prefers-reduced-motion: no-preference){{
 .l,.frame{{stroke-dasharray:1;stroke-dashoffset:1;animation:draw 9s cubic-bezier(.19,1,.22,1) infinite}}
 .frame{{animation-delay:0s}}
 .d{{opacity:0;animation:dot 9s ease-out infinite}}
 .lens,.glow{{transform-origin:{C}px {C}px;animation:lens 9s ease-in-out infinite}}
 @keyframes draw{{0%{{stroke-dashoffset:1;opacity:1}}58%{{stroke-dashoffset:0;opacity:1}}92%{{stroke-dashoffset:0;opacity:1}}100%{{stroke-dashoffset:0;opacity:0}}}}
 @keyframes dot{{0%,50%{{opacity:0}}62%{{opacity:1}}92%{{opacity:1}}100%{{opacity:0}}}}
 @keyframes lens{{0%{{opacity:.15;transform:scale(.6)}}60%{{opacity:1;transform:scale(1)}}92%{{opacity:1;transform:scale(1)}}100%{{opacity:.15;transform:scale(.6)}}}}
}}
</style>
<rect width="{W}" height="{W}" fill="{VOID}"/>
'''
def delays(svg):
    out=[]; k=0
    for line in svg.split("\n"):
        if 'class="l"' in line:
            line=line.replace('class="l"', f'class="l" style="animation-delay:{0.25+k*0.035:.3f}s"'); k+=1
        out.append(line)
    return "\n".join(out)
def lens(r=12): return f'<circle class="glow" cx="{C}" cy="{C}" r="{r*2.6}"/><circle class="lens" cx="{C}" cy="{C}" r="{r}"/>'
def write(name, s):
    s=delays(s+"</svg>"); open(f"{D}/{name}.svg","w").write(s)
    open(f"{D}/{name}-static.svg","w").write(re.sub(r"@media \(prefers-reduced-motion: no-preference\)\{.*?\n\}\n","",s,flags=re.S))
# STABILIZE: convergence
s=head("STABILIZE · convergence")+vesica()
for arm in range(6):
    pts=[]
    for i in range(181):
        t=i/180; th=arm*math.pi/3+t*2.2*math.pi; r=250*(1-t)**1.15+6
        pts.append((C+r*math.cos(th),C+r*math.sin(th)))
    s+=polyline(pts,"l",False)
    for i in (30,70,110):
        t=i/180; th=arm*math.pi/3+t*2.2*math.pi; r=250*(1-t)**1.15+6
        s+=f'<circle class="d" cx="{C+r*math.cos(th):.1f}" cy="{C+r*math.sin(th):.1f}" r="3"/>'
write("convergence", s+circle(C,C,262)+lens())
# HEAL: sri yantra
s=head("HEAL · sri yantra")+vesica(); R=240
for ay,by,hw in [(-0.95,0.62,0.78),(-0.62,0.40,0.62),(-0.40,0.18,0.52),(-0.18,0.0,0.28),(0.95,-0.62,0.78),(0.70,-0.40,0.66),(0.48,-0.18,0.52),(0.30,0.0,0.36)]:
    s+=polyline([(C,C+ay*R),(C-hw*R,C+by*R),(C+hw*R,C+by*R)])
s+=circle(C,C,R*1.02)+circle(C,C,R*1.18)
for n,rad in ((8,R*1.10),(16,R*1.26)):
    for k in range(n):
        a=2*math.pi*k/n; a2=2*math.pi*(k+1)/n; am=(a+a2)/2
        p0=(C+rad*0.93*math.cos(a),C+rad*0.93*math.sin(a)); p1=(C+rad*1.08*math.cos(am),C+rad*1.08*math.sin(am)); p2=(C+rad*0.93*math.cos(a2),C+rad*0.93*math.sin(a2))
        s+=f'<path class="l" pathLength="1" d="M {p0[0]:.1f} {p0[1]:.1f} Q {p1[0]:.1f} {p1[1]:.1f} {p2[0]:.1f} {p2[1]:.1f}"/>'
write("sri-yantra", s+lens(9))
# CREATE: metatron
s=head("CREATE · metatron")+vesica(); r=46; pts=[(C,C)]
for ring in (2,4):
    for k in range(6):
        a=math.pi/6+k*math.pi/3; pts.append((C+ring*r*math.cos(a),C+ring*r*math.sin(a)))
s+=polyline([(C+5.4*r*math.cos(math.pi/6+k*math.pi/3),C+5.4*r*math.sin(math.pi/6+k*math.pi/3)) for k in range(6)])
for i in range(13):
    for j in range(i+1,13): s+=polyline([pts[i],pts[j]],"l",False)
for x,y in pts: s+=circle(x,y,r)
for x,y in pts[1:]: s+=f'<circle class="d" cx="{x:.1f}" cy="{y:.1f}" r="3"/>'
write("metatron", s+lens(10))
# MUTATE: compass
s=head("MUTATE · compass")+vesica()
for rad in (250,205,120): s+=circle(C,C,rad)
for k in range(32):
    a=2*math.pi*k/32; r1=250-(14 if k%4==0 else 7)
    s+=polyline([(C+250*math.cos(a),C+250*math.sin(a)),(C+r1*math.cos(a),C+r1*math.sin(a))],"l",False)
for k in range(8):
    a=-math.pi/2+k*math.pi/4; L=238 if k%2==0 else 150; w=22 if k%2==0 else 14
    tip=(C+L*math.cos(a),C+L*math.sin(a)); l=(C+w*math.cos(a+math.pi/2),C+w*math.sin(a+math.pi/2)); rr=(C+w*math.cos(a-math.pi/2),C+w*math.sin(a-math.pi/2))
    s+=polyline([tip,l,(C,C),rr])+polyline([tip,(C,C)],"l",False)
for k in range(4):
    a=-math.pi/2+k*math.pi/2; s+=f'<circle class="d" cx="{C+250*math.cos(a):.1f}" cy="{C+250*math.sin(a):.1f}" r="5"/>'
write("compass", s+lens(11))
print("wrote 4 directions to", D)
