import swisseph as swe, json, datetime as dt
F=swe.FLG_MOSEPH|swe.FLG_SPEED
SEQ=[41,19,13,49,30,55,37,63,22,36,25,17,21,51,42,3,27,24,2,23,8,20,16,35,45,12,15,52,39,53,62,56,31,33,7,4,29,59,40,64,47,6,46,18,48,57,32,50,28,44,1,43,14,34,9,5,26,11,10,58,38,54,61,60]
def gl(lon):
    off=(lon-302.0)%360; i=int(off//5.625); return SEQ[i], int((off-i*5.625)//0.9375)+1
CEN={'Head':[64,61,63],'Ajna':[47,24,4,17,43,11],'Throat':[62,23,56,35,12,45,33,8,31,20,16],'G':[1,13,25,46,2,15,10,7],'Heart':[21,40,26,51],'Spleen':[48,57,44,50,32,28,18],'SolarPlexus':[36,22,37,6,49,55,30],'Sacral':[5,14,29,59,9,3,42,27,34],'Root':[53,60,52,19,39,41,58,38,54]}
G2C={g:c for c,gs in CEN.items() for g in gs}
CH=[(1,8),(2,14),(3,60),(4,63),(5,15),(6,59),(7,31),(9,52),(10,20),(10,34),(10,57),(11,56),(12,22),(13,33),(16,48),(17,62),(18,58),(19,49),(20,34),(20,57),(21,45),(23,43),(24,61),(25,51),(26,44),(27,50),(28,38),(29,46),(30,41),(32,54),(34,57),(35,36),(37,40),(39,55),(42,53),(47,64)]
MOT={'Heart','SolarPlexus','Sacral','Root'}
c=json.load(open('chart.json'))
natal_pts=[]
for side in ('personality','design'):
    for k,v in c[side].items(): natal_pts.append((side[0].upper(),k,v['lon'],v['gate'],v['line']))
natal=set(p[3] for p in natal_pts)
B=[('Sun',swe.SUN),('Moon',swe.MOON),('Mercury',swe.MERCURY),('Venus',swe.VENUS),('Mars',swe.MARS),('Jupiter',swe.JUPITER),('Saturn',swe.SATURN),('Uranus',swe.URANUS),('Neptune',swe.NEPTUNE),('Pluto',swe.PLUTO),('NorthNode',swe.TRUE_NODE)]
ORDER=['Sun','Earth','Moon','Mercury','Venus','Mars','Jupiter','Saturn','Uranus','Neptune','Pluto','NorthNode','SouthNode']
def pos(j):
    out={}
    for n,p in B:
        x=swe.calc_ut(j,p,F)[0]; out[n]=(x[0],x[3])
    out['Earth']=((out['Sun'][0]+180)%360,0); out['SouthNode']=((out['NorthNode'][0]+180)%360,out['NorthNode'][1])
    return out
def analyse(gates_tr):
    allg=natal|set(gates_tr)
    chans=[]
    for a,b in CH:
        if a in allg and b in allg:
            src='natal' if a in natal and b in natal else ('transit' if (a in gates_tr and b in gates_tr and not (a in natal or b in natal)) else 'bridge')
            if a in gates_tr and b in gates_tr and (a in natal or b in natal) and not(a in natal and b in natal): src='bridge'
            if a in gates_tr and b in gates_tr and not(a in natal) and not(b in natal): src='transit'
            chans.append([f"{a}-{b}",src])
    # connectivity
    adj={k:set() for k in CEN}
    for ch,_ in chans:
        a,b=map(int,ch.split('-')); adj[G2C[a]].add(G2C[b]); adj[G2C[b]].add(G2C[a])
    defined=[k for k in CEN if adj[k]]
    seen=set(); comps=[]
    for k in defined:
        if k in seen: continue
        st=[k]; comp=set()
        while st:
            x=st.pop()
            if x in comp: continue
            comp.add(x); st+=list(adj[x])
        seen|=comp; comps.append(comp)
    m2t=any('Throat' in cc and cc&MOT for cc in comps)
    return chans,defined,len(comps),m2t
start=dt.datetime(2026,10,30); end=dt.datetime(2030,12,31)
days=[]; d=start
prev=None
while d<=end:
    j=swe.julday(d.year,d.month,d.day,0.5)  # 06:00 IST
    P=pos(j); row={'d':d.strftime('%Y-%m-%d'),'p':[]}
    gates=[]
    for n in ORDER:
        lon,sp=P[n]; g,l=gl(lon); row['p'].append([g,l,1 if sp<0 and n not in('Earth',) else 0,round(lon,2)])
        if n!='Moon': gates.append(g)
    chans,defined,ncomp,m2t=analyse(gates)
    row['ch']=chans; row['cen']=defined; row['def']=ncomp; row['m2t']=1 if m2t else 0
    # contacts within 1 deg (slow+fast excluding Moon)
    con=[]
    for i,n in enumerate(ORDER):
        if n in('Moon',): continue
        lon=P[n][0]
        for side,k,nl,ng,nline in natal_pts:
            if abs(((lon-nl+180)%360)-180)<1.0: con.append([n,side,k,round(abs(((lon-nl+180)%360)-180),2)])
    row['con']=con
    days.append(row); d+=dt.timedelta(days=1)
# events
def dstr(j):
    y,m,dd,h=swe.revjul(j+5.5/24); return f"{y}-{m:02d}-{dd:02d}"
ev=[]
j0=swe.julday(2026,10,25,0); j1=swe.julday(2031,1,1,0)
step=0.25
slow=[('Mars',swe.MARS),('Jupiter',swe.JUPITER),('Saturn',swe.SATURN),('Uranus',swe.URANUS),('Neptune',swe.NEPTUNE),('Pluto',swe.PLUTO),('NorthNode',swe.TRUE_NODE)]
stat=[('Mercury',swe.MERCURY),('Venus',swe.VENUS),('Mars',swe.MARS),('Jupiter',swe.JUPITER),('Saturn',swe.SATURN),('Uranus',swe.URANUS),('Neptune',swe.NEPTUNE),('Pluto',swe.PLUTO)]
prevg={};prevs={};prevx={}
key_pts=[p for p in natal_pts]
j=j0
while j<j1:
    for n,p in set(slow+stat):
        x=swe.calc_ut(j,p,F)[0]; lon,sp=x[0],x[3]
        if (n,p) in slow:
            g=gl(lon)[0]
            if n in prevg and prevg[n]!=g:
                lbl=f"{n} enters Gate {g}" if n!='NorthNode' else f"Nodes enter Gates {g}/{gl((lon+180)%360)[0]}"
                ev.append([dstr(j),'ingress',lbl+(" · your natal gate" if g in natal else ""),n,g])
            prevg[n]=g
        if n!='NorthNode':
            if n in prevs and prevs[n]*sp<0: ev.append([dstr(j),'station',f"{n} stations {'retrograde' if sp<0 else 'direct'} in Gate {gl(lon)[0]}",n,gl(lon)[0]])
            prevs[n]=sp
        # conjunctions to natal points
        pts=key_pts
        for side,k,nl,ng,nline in pts:
            if n in('Mercury','Venus') and k not in('Sun','Earth','NorthNode','SouthNode','Jupiter'): continue
            x2=((lon-nl+180)%360)-180; kk=(n,side,k)
            if kk in prevx and prevx[kk]*x2<0 and abs(x2)<3:
                ev.append([dstr(j),'contact',f"{n} exact on your {'Personality' if side=='P' else 'Design'} {k} ({ng}.{nline})",n,ng])
            prevx[kk]=x2
    j+=step
# Sun on natal Sun/Earth (solar return etc.)
for side,k,nl,ng,nline in natal_pts:
    if k in ('Sun','Earth') :
        jj=j0
        while True:
            try: jx=swe.solcross_ut(nl,jj,swe.FLG_MOSEPH)
            except Exception: break
            if jx>j1: break
            ev.append([dstr(jx),'contact',f"Sun exact on your {'Personality' if side=='P' else 'Design'} {k} ({ng}.{nline})",'Sun',ng]); jj=jx+10
jj=j0
while True:
    r=swe.sol_eclipse_when_glob(jj,swe.FLG_MOSEPH); t=r[1][0]
    if t>j1: break
    lon=swe.calc_ut(t,swe.SUN,F)[0][0]; g,l=gl(lon); ev.append([dstr(t),'eclipse',f"Solar eclipse in Gate {g}.{l}"+(" · your natal gate" if g in natal else ""),'Sun',g]); jj=t+10
jj=j0
while True:
    r=swe.lun_eclipse_when(jj,swe.FLG_MOSEPH); t=r[1][0]
    if t>j1: break
    lon=swe.calc_ut(t,swe.MOON,F)[0][0]; g,l=gl(lon); ev.append([dstr(t),'eclipse',f"Lunar eclipse in Gate {g}.{l}"+(" · your natal gate" if g in natal else ""),'Moon',g]); jj=t+10
# dasha antardasha
for a,b,lbl in [("2026-09-13","2028-10-31","Jupiter / Jupiter"),("2028-10-31","2031-05-15","Jupiter / Saturn")]:
    ev.append([a,'dasha',f"Vimshottari antardasha begins: {lbl}",'Jupiter',0])
# channel windows from daily data (bridge/transit channels involving non-Moon)
ev.sort()
ev=[e for e in ev if e[0]>='2026-10-25']
json.dump({'natal':natal_pts,'days':days,'events':ev,'order':ORDER},open('sky.json','w'),separators=(',',':'))
import os; print(len(days),len(ev),os.path.getsize('sky.json'))
print(days[0])
for e in ev[:40]: print(e)
