import swisseph as swe
F=swe.FLG_MOSEPH|swe.FLG_SPEED
def d(j):
    y,m,dd,h=swe.revjul(j); return f"{y}-{m:02d}-{dd:02d}"
now=swe.julday(2026,10,29,22.7)
def crossings(body,target,a,b):
    out=[];prev=None
    for i in range(int(a),int(b)):
        j=now+i; lon=swe.calc_ut(j,body,F)[0][0]; x=((lon-target+180)%360)-180
        if prev is not None and prev*x<0 and abs(x)<5: out.append(d(j))
        prev=x
    return out
def stations(body,a,b):
    out=[];prev=None
    for i in range(a,b):
        j=now+i; s=swe.calc_ut(j,body,F)[0][3]
        if prev is not None and prev*s<0: out.append((d(j),'R' if s<0 else 'D'))
        prev=s
    return out
print("Jupiter x natal Jupiter 143.51",crossings(swe.JUPITER,143.51,-60,300))
print("Jupiter x natal Sun 140.09",crossings(swe.JUPITER,140.09,-60,300))
print("Mars x natal Sun 140.09",crossings(swe.MARS,140.09,-30,300))
print("Mars x natal Jupiter",crossings(swe.MARS,143.51,-30,300))
print("Pluto x P Saturn 302.25",crossings(swe.PLUTO,302.25,-1500,800))
print("Pluto x D Saturn 306.83",crossings(swe.PLUTO,306.83,-100,3500))
print("Pluto gate41 entry ~",crossings(swe.PLUTO,302.0,-1500,100))
print("Mercury x natal Pluto 227.63",crossings(swe.MERCURY,227.63,-30,60))
for b,n in [(swe.MERCURY,'Mercury'),(swe.VENUS,'Venus'),(swe.MARS,'Mars'),(swe.JUPITER,'Jupiter'),(swe.SATURN,'Saturn')]:
    print(n,"stations",stations(b,-40,250))
print("NN x natal Earth 320.09 (49)",crossings(swe.TRUE_NODE,320.09,-10,400))
print("SN: NN x 140.09+180 same thing")
# eclipses next 12 months
j=now
for k in range(4):
    r=swe.sol_eclipse_when_glob(j,swe.FLG_MOSEPH); t=r[1][0]
    lon=swe.calc_ut(t,swe.SUN,F)[0][0]; print("Solar eclipse",d(t),round(lon,2)); j=t+10
j=now
for k in range(3):
    r=swe.lun_eclipse_when(j,swe.FLG_MOSEPH); t=r[1][0]
    lon=swe.calc_ut(t,swe.MOON,F)[0][0]; print("Lunar eclipse",d(t),round(lon,2)); j=t+10
# mean node check
b=swe.julday(1991,8,13,8+1/60)
print("true/mean node birth",swe.calc_ut(b,swe.TRUE_NODE,F)[0][0],swe.calc_ut(b,swe.MEAN_NODE,F)[0][0])
dj=swe.julday(1991,5,13,8.48)
print("true/mean node design",swe.calc_ut(dj,swe.TRUE_NODE,F)[0][0],swe.calc_ut(dj,swe.MEAN_NODE,F)[0][0])
