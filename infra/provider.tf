variable "aws_region" {
  type = string
  default = "ap-northeast-2"
}

provider "aws" {
  region = var.aws_region
}

