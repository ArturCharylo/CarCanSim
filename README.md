# CarCanSim CI/CD Pipeline & Cloud/Kubernetes Infrastructure

This repository contains the configuration, Infrastructure as Code (IaC), and deployment pipelines for **CarCanSim** — a telemetry and CAN/OBD-II simulation project built with Rust, containerized with Docker, and deployable both to a local **Kubernetes cluster** and to **Microsoft Azure Cloud** via **Terraform**.

---

## 🚀 Architecture & Deployment Options

CarCanSim supports two deployment targets:

1. **Cloud Production Deployment (Microsoft Azure via Terraform):**
   * **IaC Engine:** Terraform (`>= 1.3.0`, `hashicorp/azurerm ~> 3.0`)
   * **Compute:** Azure Container Apps (ACA) (Serverless container scaling 0–3 replicas)
   * **Container Registry:** Azure Container Registry (ACR) (`Basic` SKU)
   * **Observability & Logs:** Azure Log Analytics Workspace & Prometheus `/metrics`
   * **Security:** Non-interactive Service Principal authentication & strict `.tfvars` isolation

2. **Local Development Cluster (Kubernetes & Jenkins):**
   * **Orchestration:** Kubernetes (via Docker Desktop / Kind)
   * **Ingress Controller:** NGINX Ingress Controller
   * **Version Control & Webhooks:** Gitea (`http://gitea.local`)
   * **CI/CD Server:** Jenkins (`http://jenkins.local`)
   * **Monitoring:** Prometheus & Grafana (`http://grafana.local`)

---

## ☁️ Option A: Microsoft Azure Cloud Deployment (Terraform)

### 1. Prerequisites

* Terraform installed
* Docker Desktop running locally
* Microsoft Azure Subscription & an Azure Service Principal with Contributor role

### 2. Configure Credentials (terraform.tfvars)

Navigate to the `terraform/` directory and configure your `terraform.tfvars` file:

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
docker build -t <ACR_NAME>.azurecr.io/carcansim:latest .
echo "<YOUR_AZURE_CLIENT_SECRET>" | docker login <ACR_NAME>.azurecr.io -u <YOUR_AZURE_CLIENT_ID> --password-stdin
docker push <ACR_NAME>.azurecr.io/carcansim:latest
```

### 5. Finalize Container App Deployment

Run terraform apply once more in the `terraform/` directory to spin up the Azure Container App revision:

```bash
cd terraform
terraform apply
```

The output will provide the public HTTPS URL to access the application:

```Plaintext
acr_login_server = "acrcarcansimartur123.azurecr.io"
app_url          = "https://app-carcansim.<unique-hash>.swedencentral.azurecontainerapps.io"
```

Access telemetry and metrics at: `https://<APP_URL>/metrics`

---

## 💻 Option B: Local Kubernetes & Jenkins Setup

Follow these steps to deploy and configure the local infrastructure from scratch.

### 1. Cluster Prerequisites & RBAC Permissions

Grant the default service account administrative rights within the default namespace:

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

* Deploy Jenkins Infrastructure (CI/CD Server):

  ```bash
  kubectl apply -f k8s/jenkins/
  ```

* Deploy Applications & Monitoring (Gitea, Grafana, Prometheus, Telemetry App, and Ingress):

  ```bash
  kubectl apply -f k8s/
  ```

### 4. Post-Deployment Configuration

#### Retrieve Jenkins Admin Password

To unlock Jenkins upon first launch, run:

```Plaintext
kubectl exec deploy/jenkins -c jenkins -- cat /var/jenkins_home/secrets/initialAdminPassword
```

#### Configure Gitea Webhook

To trigger the Jenkins pipeline automatically on every git push, configure a Webhook in Gitea pointing to:

```bash
http://<jenkins-acc-name>:<jenkins-api-token>@jenkins:8080/job/carcansim-pipeline/build?token=<jenkins-pipeline-secret>
```

---

## 🧹 Teardown / Cost Management (Azure)

To completely remove cloud resources and avoid unnecessary compute/storage costs:

```bash
cd terraform
terraform destroy
```
