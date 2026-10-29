import swisseph as swe, json, math, datetime as dt
F=swe.FLG_MOSEPH
sky=json.load(open('sky.json'))
B=[('Sun',swe.SUN),('Moon',swe.MOON),('Mercury',swe.MERCURY),('Venus',swe.VENUS),('Mars',swe.MARS),('Jupiter',swe.JUPITER),('Saturn',swe.SATURN),('Uranus',swe.URANUS),('Neptune',swe.NEPTUNE),('Pluto',swe.PLUTO),('NorthNode',swe.TRUE_NODE)]
def wrap(x): return ((x+180)%360)-180
def mcs(j):
    gst=swe.sidtime(j)*15; out=[]
    for n,p in B:
        ra=swe.calc_ut(j,p,F|swe.FLG_EQUATORIAL)[0][0]; out.append(round(wrap(ra-gst),1))
    return out
swe.set_sid_mode(swe.SIDM_LAHIRI)
for day in sky['days']:
    y,m,d=map(int,day['d'].split('-')); j=swe.julday(y,m,d,0.5)
    day['mc']=mcs(j)
    day['sid']=[round(swe.calc_ut(j,p,F|swe.FLG_SIDEREAL)[0][0],1) for p in (swe.MOON,swe.SUN,swe.JUPITER,swe.SATURN,swe.TRUE_NODE,swe.MARS)]
    for p in day['p']: p[3]=round(p[3],1)
# natal astro lines
birth=swe.julday(1991,8,13,8+1/60); gst=swe.sidtime(birth)*15
lines={}
for n,p in B:
    ra,dec=swe.calc_ut(birth,p,F|swe.FLG_EQUATORIAL)[0][:2]
    mc=wrap(ra-gst); asc=[];dsc=[]
    for lat in range(-70,71,2):
        v=-math.tan(math.radians(lat))*math.tan(math.radians(dec))
        if abs(v)>1: continue
        h=math.degrees(math.acos(v))
        asc.append([lat,round(wrap(ra-h-gst),2)]); dsc.append([lat,round(wrap(ra+h-gst),2)])
    lines[n]={'mc':round(mc,2),'ic':round(wrap(mc+180),2),'asc':asc,'dsc':dsc,'dec':round(dec,2)}
# Bangalore check
def interp(pl,lat):
    for a,b in zip(pl,pl[1:]):
        if a[0]<=lat<=b[0]:
            if abs(b[1]-a[1])>180: return None
            return a[1]+(b[1]-a[1])*(lat-a[0])/(b[0]-a[0])
for n,L in lines.items():
    for k in('asc','dsc'):
        x=interp(L[k],12.97)
        if x is not None and abs(wrap(x-77.59))<12: print('near BLR',n,k,round(x,2))
    for k in ('mc','ic'):
        if abs(wrap(L[k]-77.59))<12: print('near BLR',n,k,L[k])
# dasha periods (MD Jupiter + antardashas + pratyantardashas)
lords=['Ketu','Venus','Sun','Moon','Mars','Rahu','Jupiter','Saturn','Mercury'];yrs=[7,20,6,10,7,18,16,19,17]
Y=365.2425
mdStart=swe.julday(2026,9,13,0)  # approx from earlier calc
# recompute precisely
mo=swe.calc_ut(birth,swe.MOON,F|swe.FLG_SIDEREAL)[0][0]; nak=mo/(360/27); ni=int(nak); fr=nak-ni; li=ni%9
t=birth-fr*yrs[li]*Y; md=[]
for k in range(9):
    L=(li+k)%9; md.append((lords[L],t,t+yrs[L]*Y)); t+=yrs[L]*Y
def ds(j):
    y,m,d,_=swe.revjul(j+5.5/24); return f"{y}-{m:02d}-{d:02d}"
das=[]
for L,a,b in md:
    if L in('Rahu','Jupiter','Saturn'):
        Li=lords.index(L); t=a
        for k in range(9):
            A=(Li+k)%9; du=(b-a)*yrs[A]/120; t2=t
            pr=[]
            for q in range(9):
                Pq=(A+q)%9; pd=du*yrs[Pq]/120; pr.append([lords[Pq],ds(t2)]); t2+=pd
            das.append([L,lords[A],ds(t),ds(t+du),pr]); t+=du
sky['lines']=lines; sky['dasha']=[[L,ds(a),ds(b)] for L,a,b in md]; sky['ad']=[d for d in das if d[3]>='2026-01-01' and d[2]<='2031-12-31']
sky['land']=open('land.txt').read()
json.dump(sky,open('sky.json','w'),separators=(',',':'))
import os;print(os.path.getsize('sky.json'))
for a in sky['ad'][:6]: print(a[:4])
