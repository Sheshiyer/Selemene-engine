import swisseph as swe, json, math
swe.set_ephe_path(None)
SEQ=[41,19,13,49,30,55,37,63,22,36,25,17,21,51,42,3,27,24,2,23,8,20,16,35,45,12,15,52,39,53,62,56,31,33,7,4,29,59,40,64,47,6,46,18,48,57,32,50,28,44,1,43,14,34,9,5,26,11,10,58,38,54,61,60]
def gl(lon):
    off=(lon-302.0)%360; i=int(off//5.625); r=off-i*5.625
    line=int(r//0.9375)+1; r2=r-(line-1)*0.9375
    color=int(r2//0.15625)+1; r3=r2-(color-1)*0.15625
    tone=int(r3//(0.15625/6))+1; r4=r3-(tone-1)*(0.15625/6)
    base=int(r4//(0.15625/30))+1
    return SEQ[i],line,color,tone,base
P=[('Sun',swe.SUN),('Moon',swe.MOON),('NorthNode',swe.TRUE_NODE),('Mercury',swe.MERCURY),('Venus',swe.VENUS),('Mars',swe.MARS),('Jupiter',swe.JUPITER),('Saturn',swe.SATURN),('Uranus',swe.URANUS),('Neptune',swe.NEPTUNE),('Pluto',swe.PLUTO)]
SIGNS=['Ari','Tau','Gem','Can','Leo','Vir','Lib','Sco','Sag','Cap','Aqu','Pis']
def fmt(lon): return f"{SIGNS[int(lon//30)]} {lon%30:5.2f}"
def chart(jd):
    out={}
    for n,p in P:
        x=swe.calc_ut(jd,p,swe.FLG_MOSEPH|swe.FLG_SPEED)[0]; out[n]={'lon':x[0],'speed':x[3]}
    out['Earth']={'lon':(out['Sun']['lon']+180)%360,'speed':0}
    out['SouthNode']={'lon':(out['NorthNode']['lon']+180)%360,'speed':0}
    return out
birth=swe.julday(1991,8,13,8+1/60)  # 13:31 IST
sun=swe.calc_ut(birth,swe.SUN,swe.FLG_MOSEPH)[0][0]
target=(sun-88)%360
jd=birth-89
for _ in range(60):
    s=swe.calc_ut(jd,swe.SUN,swe.FLG_MOSEPH)[0][0]; d=((target-s+180)%360)-180
    if abs(d)<1e-7: break
    jd+=d/0.9856
design=jd
now=swe.julday(2026,10,29,22+42/60)
res={}
for lbl,j in [('personality',birth),('design',design),('transit',now)]:
    c=chart(j); res[lbl]={}
    y,m,d,h=swe.revjul(j); print(f"\n== {lbl} {y}-{m:02d}-{d:02d} {h:.3f}UT")
    for n,v in c.items():
        g=gl(v['lon']); r='R' if v['speed']<0 else ' '
        res[lbl][n]=dict(lon=v['lon'],gate=g[0],line=g[1],color=g[2],tone=g[3],base=g[4],retro=v['speed']<0)
        print(f"{n:10s} {fmt(v['lon'])} {r} {g[0]}.{g[1]}  c{g[2]} t{g[3]} b{g[4]}")
json.dump(res,open('/home/claude/hd/chart.json','w'),indent=1)
# Vedic: sidereal Lahiri moon, asc
swe.set_sid_mode(swe.SIDM_LAHIRI)
flags=swe.FLG_SIDEREAL|swe.FLG_MOSEPH
mo=swe.calc_ut(birth,swe.MOON,flags)[0][0]
print("\nSidereal Moon",fmt(mo))
cusps,ascmc=swe.houses_ex(birth,12.97,77.59,b'W',flags)
print("Sidereal Asc",fmt(ascmc[0]))
for n,p in P[:11]:
    print('sid',n,fmt(swe.calc_ut(birth,p,flags)[0][0]))
# Vimshottari
lords=['Ketu','Venus','Sun','Moon','Mars','Rahu','Jupiter','Saturn','Mercury']
yrs=[7,20,6,10,7,18,16,19,17]
nak=mo/(360/27); ni=int(nak); frac=nak-ni; li=ni%9
start=birth-frac*yrs[li]*365.2425
t=start; seq=[]
for k in range(9):
    L=(li+k)%9; dur=yrs[L]*365.2425
    seq.append((lords[L],t,t+dur)); t+=dur
def dt(j):
    y,m,d,_=swe.revjul(j); return f"{y}-{m:02d}-{d:02d}"
print("\nMoon nakshatra index",ni,"lord",lords[li])
for L,a,b in seq: print("MD",L,dt(a),dt(b))
# antardashas of current MD
for L,a,b in seq:
    if a<=now<b:
        Li=lords.index(L); t=a
        for k in range(9):
            A=(Li+k)%9; dur=(b-a)*yrs[A]/120
            mark='<-- now' if t<=now<t+dur else ''
            print("  AD",L,"/",lords[A],dt(t),dt(t+dur),mark); t+=dur
# Astrocartography MC lines (geo longitude where planet culminates) at natal moment
gst=swe.sidtime(birth)*15
print("\nMC/IC lines (east longitude)")
for n,p in P[:11]:
    eq=swe.calc_ut(birth,p,swe.FLG_EQUATORIAL|swe.FLG_MOSEPH)[0]
    mc=((eq[0]-gst+180)%360)-180; ic=((mc+360)%360)-180
    print(f"{n:8s} MC {mc:7.2f}  IC {ic:7.2f}")
