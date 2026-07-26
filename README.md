# CarCanSim CI/CD Pipeline & Kubernetes Infrastructure

This repository contains the configuration and infrastructure setup for **CarCanSim**, a telemetry and OBD-II simulation project built with Rust, containerized with Docker, and deployed via Kubernetes with an automated Jenkins CI/CD pipeline triggered by Gitea.

---

## 🚀 Architecture Overview

* **Orchestration:** Kubernetes (via Docker Desktop)
* **Ingress Controller:** NGINX Ingress Controller (`kind` provider setup)
* **Version Control & Webhooks:** Gitea (`http://gitea.local`)
* **CI/CD Server:** Jenkins (`http://jenkins.local`)
* **Monitoring:** Prometheus & Grafana (`http://grafana.local`)

---

## 🛠️ Setup & Deployment Guide

Follow these steps to deploy and configure the infrastructure from scratch.

### 1. Cluster Prerequisites & RBAC Permissions

Grant the default service account administrative rights within the default namespace so Jenkins can manage deployments:

```bash

kubectl create rolebinding jenkins-admin-binding --clusterrole=admin --serviceaccount=default:default --namespace=default

```

### 2. Install NGINX Ingress Controller

Deploy the NGINX Ingress controller to manage local domain routing (.local):

```bash

kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/main/deploy/static/provider/kind/deploy.yaml
```

### 3. Deploy Infrastructure and Applications

The deployment files are separated to prevent Jenkins from restarting itself during application updates:

Deploy Jenkins Infrastructure (CI/CD Server):

```bash
kubectl apply -f k8s/jenkins/
```

> Deploy Applications & Monitoring (Gitea, Grafana, Prometheus, Telemetry App, and Ingress):

```bash
kubectl apply -f k8s/
```

> Note: kubectl apply -f k8s/ is used specifically for reloads in Gitea, Prometheus, Grafana, and the core app.

## 🔑 Post-Deployment Configuration

Retrieve Jenkins Admin Password
To unlock Jenkins upon first launch, run the following command to print the initial password:

```bash
kubectl exec deploy/jenkins -c jenkins -- cat /var/jenkins_home/secrets/initialAdminPassword
```

Configure Gitea Webhook
To trigger the Jenkins pipeline automatically on every git push, configure a Webhook in Gitea pointing to the following target URL structure:

```plaintext
http://<jenkins-acc-name>:<jenkins-api-token>@jenkins:8080/job/carcansim-pipeline/build?token=<jenkins-pipeline-secret>
```
