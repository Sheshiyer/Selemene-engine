# Kubernetes Deployment Guide

## Prerequisites

- Kubernetes 1.25+
- kubectl configured
- Helm 3.0+ (optional, for cert-manager)
- Persistent volume provisioner (for storage)

## Directory Structure

```
k8s/
├── kustomization.yaml
├── base/
│   ├── namespace.yaml
│   ├── configmap.yaml
│   ├── secret.yaml
│   ├── deployment.yaml
│   ├── service.yaml
│   └── ingress.yaml
├── redis/
│   ├── deployment.yaml
│   └── service.yaml
├── postgres/
│   ├── deployment.yaml
│   ├── service.yaml
│   └── pvc.yaml
├── ts-engines/
│   ├── deployment.yaml
│   └── service.yaml
└── cert-manager/
    └── certificate.yaml
```

---

## Quick Start

### 1. Apply All Resources

```bash
kubectl apply -k k8s/
```

### 2. Verify Deployment

```bash
# Check pods
kubectl get pods -n noesis

# Check services
kubectl get svc -n noesis

# View logs
kubectl logs -f deployment/noesis-api -n noesis
```

### 3. Access API

```bash
# Port forward for testing
kubectl port-forward svc/noesis-api 8080:8080 -n noesis

# Test
curl http://localhost:8080/health
```

---

## Base Resources

### Namespace

```yaml
# k8s/base/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: noesis
  labels:
    app.kubernetes.io/name: noesis
```

### ConfigMap

```yaml
# k8s/base/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: noesis-config
  namespace: noesis
data:
  RUST_LOG: "info"
  SERVER_HOST: "0.0.0.0"
  SERVER_PORT: "8080"
  CACHE_L1_SIZE: "268435456"
  CACHE_L1_TTL: "3600"
  CACHE_L2_TTL: "86400"
  SWISS_EPHEMERIS_PATH: "/app/data/ephemeris"
  TS_ENGINES_URL: "http://ts-engines:3001"
  ENABLE_METRICS: "true"
  LOG_FORMAT: "json"
```

### Secret

```yaml
# k8s/base/secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: noesis-secrets
  namespace: noesis
type: Opaque
stringData:
  JWT_SECRET: "your-secure-jwt-secret-here"
  DATABASE_URL: "postgresql://noesis:password@postgres:5432/noesis"
  REDIS_URL: "redis://redis:6379"
```

### Deployment

```yaml
# k8s/base/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: noesis-api
  namespace: noesis
spec:
  replicas: 3
  selector:
    matchLabels:
      app: noesis-api
  template:
    metadata:
      labels:
        app: noesis-api
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/metrics"
    spec:
      containers:
      - name: noesis-api
        image: noesis/selemene-engine:2.0.0
        ports:
        - containerPort: 8080
          name: http
        envFrom:
        - configMapRef:
            name: noesis-config
        - secretRef:
            name: noesis-secrets
        volumeMounts:
        - name: ephemeris-data
          mountPath: /app/data/ephemeris
          readOnly: true
        resources:
          requests:
            cpu: "500m"
            memory: "512Mi"
          limits:
            cpu: "2000m"
            memory: "4Gi"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: ephemeris-data
        persistentVolumeClaim:
          claimName: ephemeris-pvc
```

### Service

```yaml
# k8s/base/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: noesis-api
  namespace: noesis
spec:
  type: ClusterIP
  ports:
  - port: 8080
    targetPort: 8080
    protocol: TCP
    name: http
  selector:
    app: noesis-api
```

### Ingress

```yaml
# k8s/base/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: noesis-ingress
  namespace: noesis
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
  - hosts:
    - api.noesis.example.com
    secretName: noesis-tls
  rules:
  - host: api.noesis.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: noesis-api
            port:
              number: 8080
```

---

## Redis Deployment

```yaml
# k8s/redis/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: redis
  namespace: noesis
spec:
  replicas: 1
  selector:
    matchLabels:
      app: redis
  template:
    metadata:
      labels:
        app: redis
    spec:
      containers:
      - name: redis
        image: redis:7-alpine
        ports:
        - containerPort: 6379
        resources:
          requests:
            cpu: "100m"
            memory: "128Mi"
          limits:
            cpu: "500m"
            memory: "1Gi"
        volumeMounts:
        - name: redis-data
          mountPath: /data
      volumes:
      - name: redis-data
        persistentVolumeClaim:
          claimName: redis-pvc
---
apiVersion: v1
kind: Service
metadata:
  name: redis
  namespace: noesis
spec:
  ports:
  - port: 6379
  selector:
    app: redis
```

---

## PostgreSQL Deployment

```yaml
# k8s/postgres/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: postgres
  namespace: noesis
spec:
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:16-alpine
        ports:
        - containerPort: 5432
        env:
        - name: POSTGRES_DB
          value: noesis
        - name: POSTGRES_USER
          value: noesis
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: noesis-secrets
              key: POSTGRES_PASSWORD
        volumeMounts:
        - name: postgres-data
          mountPath: /var/lib/postgresql/data
        resources:
          requests:
            cpu: "250m"
            memory: "256Mi"
          limits:
            cpu: "1000m"
            memory: "2Gi"
      volumes:
      - name: postgres-data
        persistentVolumeClaim:
          claimName: postgres-pvc
```

---

## TypeScript Engines

```yaml
# k8s/ts-engines/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ts-engines
  namespace: noesis
spec:
  replicas: 2
  selector:
    matchLabels:
      app: ts-engines
  template:
    metadata:
      labels:
        app: ts-engines
    spec:
      containers:
      - name: ts-engines
        image: noesis/ts-engines:2.0.0
        ports:
        - containerPort: 3001
        resources:
          requests:
            cpu: "200m"
            memory: "256Mi"
          limits:
            cpu: "1000m"
            memory: "1Gi"
        livenessProbe:
          httpGet:
            path: /health
            port: 3001
          initialDelaySeconds: 10
          periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: ts-engines
  namespace: noesis
spec:
  ports:
  - port: 3001
  selector:
    app: ts-engines
```

---

## Kustomization

```yaml
# k8s/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

namespace: noesis

resources:
- base/namespace.yaml
- base/configmap.yaml
- base/secret.yaml
- base/deployment.yaml
- base/service.yaml
- base/ingress.yaml
- redis/
- postgres/
- ts-engines/

images:
- name: noesis/selemene-engine
  newTag: 2.0.0
- name: noesis/ts-engines
  newTag: 2.0.0
```

---

## Horizontal Pod Autoscaler

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: noesis-api-hpa
  namespace: noesis
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: noesis-api
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

---

## Network Policies

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: noesis-network-policy
  namespace: noesis
spec:
  podSelector:
    matchLabels:
      app: noesis-api
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: redis
    ports:
    - protocol: TCP
      port: 6379
  - to:
    - podSelector:
        matchLabels:
          app: postgres
    ports:
    - protocol: TCP
      port: 5432
  - to:
    - podSelector:
        matchLabels:
          app: ts-engines
    ports:
    - protocol: TCP
      port: 3001
```

---

## Production Checklist

- [ ] TLS certificates configured (cert-manager)
- [ ] Secrets stored securely (external secrets, Vault)
- [ ] Resource limits defined
- [ ] HPA configured
- [ ] Network policies applied
- [ ] Monitoring configured (Prometheus/Grafana)
- [ ] Logging configured (Loki/ELK)
- [ ] Backup strategy for PostgreSQL
- [ ] Ephemeris data volume populated
- [ ] Ingress configured with proper hostname

---

## Troubleshooting

### Pod Not Starting

```bash
kubectl describe pod <pod-name> -n noesis
kubectl logs <pod-name> -n noesis --previous
```

### Service Connectivity

```bash
kubectl run debug --rm -it --image=busybox -n noesis -- /bin/sh
# Inside pod:
wget -qO- http://noesis-api:8080/health
```

### Resource Issues

```bash
kubectl top pods -n noesis
kubectl describe node
```

---

**Last Updated**: 2026-01
