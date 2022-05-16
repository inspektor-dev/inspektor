---
sidebar_position: 1
title: Introduction
---

Hello all, 

**Inspektor** is an access control layer for all your data sources. It acts as guardian and enforces access policies to all your data sources. 

It helps organizations in securing their data assets and obtaining data compliance.

With Inspektor, you can leverage open policy and GitOps to enforce policies. By having features like **policy as a code** and **GitOps**, Inspektor is a first-class citizen for your modern cloud-native workloads.

Inspektor is designed to work with all databases such as Postgres, MySQL, and MongoDB. 

The access policies are defined using OPA (open policy agent). Since policies are written in OPA, you can write policies such as granting access to the support engineer only if a support ticket is assigned.

Go to the official documentation to learn more about OPA.


## Use Cases

- Standardise your ad hoc data access.
- Create access credentials in no time for your dev team to debug.
- Manage all your data policies in a centralized place and avoid managing data policies in silos.
- Protect PPI data of your customers while collaborating.
- Avoid dangerous commands like DELETE,UPDATE accidentally.

## How it works

Inspektor has two components: 
1. The **Dataplane** and, <br/>
2. The **Controlplane**.

Let us understand what those 2 words actually means, 

**Controlplane:** The control plane is the part of a network that controls how data packets are forwarded — meaning how data is sent from one place to another. The process of creating a routing table, for example, is considered part of the control plane. 

Routers use various protocols to identify network paths, and they store these paths in routing tables.

and...

**Dataplane:** In contrast to the control plane, which determines how packets should be forwarded, the data plane actually forwards the packets. The data plane is also called the forwarding plane.

Think of the control plane as being like the stoplights that operate at the intersections of a city. Meanwhile, the data plane (or the forwarding plane) is more like the cars that drive on the roads, stop at the intersections, and obey the stoplights.


The **dataplane** deployed along with your data service as a **sidecar**, to **intercept** all the network traffic to your data service to enforce access policies.

![Inspektor design](../static/img/inspektordesign.png)

The **controlplane** acts as a management service to dynamically configure all your dataplane to enforce policies.

<br/>

**Supported Data Source**
- Postgres

**Planned Data Sources**
- Snowflake
- MongoDB
- MySQL
- S3

### Tech Stack 
- Languages: Rust and Go
- UI: Vue
- Policy: Open Policy Agent (OPA)

### Team

**Balaji Jinnah (poonai)** | [Twitter](https://twitter.com/poonai_) | [Rant](https://poonai.github.io) | [Discord](https://discord.gg/YxZbDJHTxf) 
- Lead Project Creator and Maintainer.

<br/>

**Priyansh**
- disturbs the maintainer.

