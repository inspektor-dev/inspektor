<p align="center">
  <img src="docs/inspektor.png" alt="Inspektor" width="600" height="200" />
</p>

<h1 align="center">Inspektor</h1>
<p align="center">
  <a href="https://github.com/poonai/inspektor/actions/workflows/rust.yml"><img src="https://github.com/poonai/inspektor/actions/workflows/rust.yml/badge.svg" alt="Build Status"></a>
      <a href="https://github.com/poonai/inspektor/blob/main/LICENSE"><img src="https://img.shields.io/github/license/poonai/inspektor" alt="Apache 2 License" height="22"/></a>
<a href="https://discord.gg/YxZbDJHTxf"><img src="https://img.shields.io/discord/870545680463187989.svg" alt="discord badge" height="22"/></a>

      

</p>

<p align="center">
  <img src="docs/static/img/inspektordesign.png" alt="Inspektor design"  height="700" />
</p>

**Inspektor** is an access control layer for all your data sources. It acts as a guardian and enforces access policies to all your data sources. 

With Inspektor, you can leverage open policy and GitOps to enforce polices. By having features like policy as a code and GitOps, Inspektor is a first class citizen for your mordern cloud-native workloads.

If you have ideas 🧵 for improving inspektor, please join [Inspektor Discord](https://discord.gg/YxZbDJHTxf) or visit the [GitHub Discussion](https://github.com/poonai/inspektor/discussions)

⭐ If you find this project interesting, please consider starring the project on GitHub⭐

## Use Cases
- Create access credentials in no time for your dev team to debug
- Manage all your data policies in a centralized place and avoid managing data policies in silos
- Protect PPI data of your customers while collaborating
- Avoid dangerous commands like `DELETE`,`UPDATE` accidentally. 

## How it works

Inspektor has two components: the **data plane** and the **control plane**.

The **data plane** deployed along with your data service as a **sidecar**, to **intercept** all the network traffic to your data service to enforce access
policies.

The **control plane** acts as an management service to dynamically configure all your data plane to enforce policies.

## Supported Data Source
 - Postgres
 
## Planned Data Sources
 - Snowflake
 - MongoDB
 - MYSQL
 - S3
