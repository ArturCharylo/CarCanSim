# CarCanSim CI/CD Pipeline & Cloud/Kubernetes Infrastructure

This repository contains the configuration, Infrastructure as Code (IaC), and automated multi-stage CI/CD deployment pipelines for **CarCanSim** — a vehicle telemetry and CAN/OBD-II simulation application built with Rust, containerized with Docker, and deployable to both a local **Kubernetes cluster (Kind)** and **Microsoft Azure Cloud (Azure Container Apps)** via **Terraform**.

---

## 🚀 Architecture & Deployment Targets

CarCanSim supports hybrid deployment setups and continuous deployment workflows:

1. **Cloud Serverless Deployment (Microsoft Azure via Terraform & Azure DevOps):**
   * **IaC Engine:** Terraform (`>= 1.3.0`, `hashicorp/azurerm ~> 3.0`)
   * **Compute:** Azure Container Apps (ACA) (Serverless scaling from 0 to 3 replicas)
   * **Container Registry:** Azure Container Registry (ACR) (`Basic` SKU)
   * **CI/CD Pipeline:** Azure DevOps Multi-Stage YAML Pipeline with Self-Hosted Windows Agent (`LAPTOP_ARTUR`)
   * **Observability & Logs:** Azure Log Analytics Workspace & Prometheus `/metrics` endpoint
   * **Security:** Non-interactive Service Principal authentication (`client_id` & `client_secret`) with Variable Groups isolation

2. **Local Development Cluster (Kubernetes / Kind & Jenkins / ArgoCD GitOps):**
   * **Orchestration:** Kubernetes (via Kind / Docker Desktop)
   * **GitOps & Ingress:** ArgoCD & NGINX Ingress Controller
   * **Self-Hosted CI/CD & Git:** Jenkins (`http://jenkins.local`) & Gitea (`http://gitea.local`)
   * **Local Monitoring Stack:** Prometheus & Grafana (`http://grafana.local`)

---

## 🔄 Automated CI/CD Pipeline (Azure DevOps)

The deployment pipeline is defined in `azure-pipeline.yml` and consists of two automated stages:

* **Stage 1: LocalTestAndBuild:**
  * Builds the Docker image locally on the Self-Hosted agent (`telemetry-app:latest`).
  * Loads the Docker image directly into the local Kind cluster (`carcansim-cluster`).
* **Stage 2: DeployToAzure:**
  * Runs automatically on branch `main` after Stage 1 passes.
  * Builds and tags the image with dynamic `$(Build.BuildId)` and `:latest`.
  * Authenticates non-interactively using Service Principal credentials from Azure DevOps Variable Group (`carcansim-variables`).
  * Pushes images to Azure Container Registry (ACR).
  * Executes a zero-downtime rolling update on the Azure Container App revision via Azure CLI (`az containerapp update`).

---

## ☁️ Option A: Microsoft Azure Cloud Deployment (Terraform)

### 1. Prerequisites

* Terraform `>= 1.3.0` installed
* Docker Desktop running locally (WSL2 Backend)
* Microsoft Azure Subscription & an Azure Service Principal with `Contributor` role

### 2. Configure Credentials (terraform.tfvars)

Navigate to the `terraform/` directory and populate your `terraform.tfvars` file (strictly excluded via `.gitignore`):

```hcl
azure_subscription_id = "<YOUR_AZURE_SUBSCRIPTION_ID>"
azure_tenant_id       = "<YOUR_AZURE_TENANT_ID>"
azure_client_id       = "<YOUR_SERVICE_PRINCIPAL_APP_CLIENT_ID>"
azure_client_secret   = "<YOUR_SERVICE_PRINCIPAL_CLIENT_SECRET>"

resource_group_name   = "rg-carcansim"
location              = "swedencentral"
acr_name              = "acrcarcansimartur123"
container_app_name    = "app-carcansim"
```

### 3. Initialize & Deploy Base Infrastructure

```bash
cd terraform
terraform init
terraform apply
```

### 4. Build & Push Docker Image to Azure Container Registry (ACR)

From the project root directory:

```bash
docker build -t acrcarcansimartur123.azurecr.io/carcansim:latest .
az login --service-principal -u "<YOUR_AZURE_CLIENT_ID>" -p "<YOUR_AZURE_CLIENT_SECRET>" --tenant "<YOUR_AZURE_TENANT_ID>"
az acr login --name acrcarcansimartur123
docker push acrcarcansimartur123.azurecr.io/carcansim:latest
```

### 5. Finalize Container App Deployment

Run terraform apply once more in the `terraform/` directory to spin up the Azure Container App revision:

```bash
cd terraform
terraform apply
```

The output provides the public HTTPS URL to access the application:

```plaintext
acr_login_server = "acrcarcansimartur123.azurecr.io"
app_url          = "https://app-carcansim.<unique-hash>.swedencentral.azurecontainerapps.io"
```

Access telemetry and metrics at: `https://<APP_URL>/metrics`

---

## 💻 Option B: Local Kubernetes Setup (Kind + Jenkins / ArgoCD)

### 1. Cluster Prerequisites & Kind Setup

```bash
kind create cluster --name carcansim-cluster
```

Grant administrative RBAC permissions to the default service account:

```bash
kubectl create rolebinding jenkins-admin-binding --clusterrole=admin --serviceaccount=default:default --namespace=default
```

### 2. Install Ingress & GitOps Controllers

* NGINX Ingress Controller:

```bash
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/main/deploy/static/provider/kind/deploy.yaml
```

* ArgoCD GitOps Engine:

```bash
kubectl create namespace argocd
kubectl apply -n argocd --server-side --force-conflicts -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml
```

### 3. Deploy Local Applications & Observability

* Deploy Jenkins Infrastructure:

```bash
kubectl apply -f k8s/jenkins/
```

* Deploy Applications & Monitoring Stack (Gitea, Grafana, Prometheus, Telemetry App, Ingress):

```bash
kubectl apply -f k8s/
```

### 4. Post-Deployment Verification

* Retrieve Jenkins initial admin password:

```bash
kubectl exec deploy/jenkins -c jenkins -- cat /var/jenkins_home/secrets/initialAdminPassword
```

* Retrieve ArgoCD initial admin password:

```bash
kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath="{.data.password}"
```

---

## 🧹 Teardown / Cost Management (Azure)

To completely destroy all cloud resources and maintain a zero-cost baseline on your Azure subscription:

```bash
cd terraform
terraform destroy -auto-approve
```
