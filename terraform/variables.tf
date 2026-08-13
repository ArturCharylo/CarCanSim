variable "resource_group_name" {
  type        = string
  default     = "rg-carcansim-dev"
  description = "Resource group name in Azure"
}

variable "location" {
  type        = string
  default     = "swedencentral"
  description = "Region Azure"
}

variable "acr_name" {
  type        = string
  default     = "acrcarcansim" 
  description = "Container Registry name (ACR)"
}

variable "container_app_name" {
  type        = string
  default     = "carcansim-telemetry"
  description = "App name in Azure Container Apps"
}

variable "azure_subscription_id" { 
  type = string 
}

variable "azure_tenant_id" { 
  type = string 
}

variable "azure_client_id" { 
  type = string 
}

variable "azure_client_secret" { 
  type      = string 
  sensitive = true
}