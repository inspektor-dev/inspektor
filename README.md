# Inspektor
[![Rust](https://github.com/poonai/inspektor/actions/workflows/rust.yml/badge.svg)](https://github.com/poonai/inspektor/actions/workflows/rust.yml)

**Inspektor** is a access control layer for all your data sources. It act as gaurdian to your all datasources and enforces access polices to all your data sources. 

With Inspektor, you can leverage open policy and GitOps to enforce polices. By having features like policy as a code and GitOps, Inspektor is a first class citizen for your mordern cloud native workloads.

## How it works

Inspektor has two components: the **data plane** and the **control plane**.

The **data plane** deployed along with your data service as a **sidecar**, to **intercept** all the network traffic to your data service to enforce access
policies.


The **control plane** act an management service to dynamically configure all your data plane to enforce policies.

## Supported Data Source
 - Postgres
 
## Planned Data Sources
 - MongoDB
 - MySQL
 - S3