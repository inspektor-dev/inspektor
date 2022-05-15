---

sidebar_position: 3

title: Design

---


**Inspektor** is an access control layer for all your data sources. It acts as guardian and enforces access policies to all your data sources. 

With Inspektor, you can leverage open policy and GitOps to enforce policies. By having features like policy as a code and GitOps, Inspektor is a first-class citizen for your modern cloud-native workloads.

## How it works

Inspektor has two components: the **dataplane** and the **controlplane**.

The **dataplane** deployed along with your data service as a **sidecar**, to **intercept** all the network traffic to your data service to enforce access policies.

![Inspektor design](../static/img/inspektordesign.png)

The **controlplane** acts as a management service to dynamically configure all your dataplane to enforce policies.