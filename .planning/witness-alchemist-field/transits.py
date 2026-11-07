import swisseph as swe, json
exec(open('compute.py').read().split("P=[")[0])  # SEQ, gl
F=swe.FLG_MOSEPH|swe.FLG_SPEED
CH=[(1,8),(2,14),(3,60),(4,63),(5,15),(6,59),(7,31),(9,52),(10,20),(10,34),(10,57),(11,56),(12,22),(13,33),(16,48),(17,62),(18,58),(19,49),(20,34),(20,57),(21,45),(23,43),(24,61),(25,51),(26,44),(27,50),(28,38),(29,46),(30,41),(32,54),(34,57),(35,36),(37,40),(39,55),(42,53),(47,64)]
c=json.load(open('chart.json'))
natal={v['gate'] for s in ('personality','design') for v in c[s].values()}
print("natal gates",sorted(natal))
tr={k:v['gate'] for k,v in c['transit'].items() if k!='Moon'}
print("transit gates",tr)
for a,b in CH:
    for x,y in ((a,b),(b,a)):
        hits=[k for k,g in tr.items() if g==x]
        if hits and y in natal: print(f"Transit completes {a}-{b}: {hits} in {x} + natal {y}")
        if hits and y in tr.values() and x<y: print(f"Transit-only channel {a}-{b}")
# mean node variable check
birth=swe.julday(1991,8,13,8+1/60)
import math
d=json.load(open('chart.json'))
for lbl in ('personality','design'):
    jd=birth if lbl=='personality' else None
mn=swe.calc_ut(birth,swe.MEAN_NODE,F)[0][0]; print("P mean node",gl(mn))
# design jd recompute
sun=swe.calc_ut(birth,swe.SUN,F)[0][0]; t=(sun-88)%360; jd=birth-89
for _ in range(50):
    s=swe.calc_ut(jd,swe.SUN,F)[0][0]; jd+=(((t-s+180)%360)-180)/0.9856
print("D mean node",gl(swe.calc_ut(jd,swe.MEAN_NODE,F)[0][0]))
def dstr(j):
    y,m,dd,h=swe.revjul(j); return f"{y}-{m:02d}-{dd:02d}"
now=swe.julday(2026,10,29,22.7)
def track(body,name,days=420,step=1):
    prev=None; start=now-days//2
    for i in range(0,days):
        j=start+i*step; lon=swe.calc_ut(j,body,F)[0][0]; g=gl(lon)[:2]
        if g!=prev: print(f"  {name} enters {g[0]}.{g[1]} on {dstr(j)}"); prev=g
print("\nNodes gate changes"); 
prev=None
for i in range(-200,400,1):
    j=now+i; g=gl(swe.calc_ut(j,swe.TRUE_NODE,F)[0][0])[0]
    if g!=prev: print("  NN gate",g,"SN gate",gl((swe.calc_ut(j,swe.TRUE_NODE,F)[0][0]+180)%360)[0],"from",dstr(j)); prev=g
for body,name in [(swe.JUPITER,'Jupiter'),(swe.MARS,'Mars'),(swe.SATURN,'Saturn'),(swe.PLUTO,'Pluto'),(swe.URANUS,'Uranus'),(swe.NEPTUNE,'Neptune')]:
    print(name); prev=None
    for i in range(-120,420,1):
        j=now+i; g=gl(swe.calc_ut(j,body,F)[0][0])[0]
        if g!=prev: print(f"   gate {g} from {dstr(j)}"); prev=g
# Jupiter stations & return
print("\nJupiter lon around")
for i in range(-60,330,10):
    j=now+i; x=swe.calc_ut(j,swe.JUPITER,F)[0]; print(dstr(j),round(x[0],2),'R' if x[3]<0 else '')
