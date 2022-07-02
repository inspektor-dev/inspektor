---
title: 900,000 instances of Kubernetes EXPOSED! 
description: An analysis from cybersecurity firm Cyble has found over 900,000 Kubernetes (K8s) exposed across the internet and thus vulnerable to malicious scans and/or data-exposing cyberattacks
slug: kubernetes-breach
authors:
  - name: zriyansh
    title: Contributor
    url: https://twitter.com/priyanskhodiyar
    image_url: https://pbs.twimg.com/profile_images/1523748501414637568/BcE2tR0z_400x400.jpg
tags: [kubernetes]
image: /img/k8.png
hide_table_of_contents: false
---
![banner for blog](/img/k8.png)

Official link - https://blog.cyble.com/2022/06/27/exposed-kubernetes-clusters/

An analysis from cybersecurity firm Cyble has found over 900,000 Kubernetes (K8s) exposed across the internet and thus vulnerable to malicious scans and/or data-exposing cyberattacks

Out of all those 900,000
With error code 403 - most of the misconfigured Kubernetes instances, indicates they are safe from breach. 

With error code 401 - nearly 5,000 instances were found, that indicates unauthorized requests.

With error code 200 - 799 Kubernetes instances were found which could be exploited to obtain Kubernetes Dashboard access without the need for a password.

Many of the misconfigured clusters spotted by cybersecurity researchers were due to the use of default settings.

To avoid misconfigurations, Cyble said companies should keep Kubernetes updated to the latest version and remove debugging tools from production containers.

The results show a massive 900,000 Kubernetes servers, with 
- 65% of them (585,000) being located in the United States, 
- 14% in China, 
- 9% in Germany, 
- 6% each in Netherlands and Ireland

The below queries can be used to find exposure for Kubernetes instances exposed over the internet:
- KubernetesDashboard
- Kubernetes-master
- Kubernetes
- Kube
- K8
- Favicon:2130463260, -1203021870

Our friends at Telegram had a deep conversation, here's an attached screenshot of the same.

![screenshot](/img/ss.png)


This is all, install Inspektor and protect your Data(bases).