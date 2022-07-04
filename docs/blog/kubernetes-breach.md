---
title: decrypting recent security incidents
description: Recent days, we have witnessed lot of security incidents. This post covers some of the major incidents and what are those learnings.
slug: decrypting-recent-security-incidents
authors:
  - name: zriyansh
    title: Contributor
    url: https://twitter.com/priyanskhodiyar
    image_url: https://pbs.twimg.com/profile_images/1523748501414637568/BcE2tR0z_400x400.jpg
tags: [kubernetes]
image: /img/hacking.jpg
hide_table_of_contents: false
---
![banner for blog](/img/hacking.jpg)

Most of us do silly mistakes mostly due to lethargy or some are genuine. But the price we pay is huge because some of the damages are irreversible. 
But exposing nearly a million Kubernetes clusters seems a little serious and made me revisit some of the security issues in recent days.

# Recent Security issues

**1. Cybel Claim**

[Cyble claims](https://blog.cyble.com/2022/06/27/exposed-kubernetes-clusters/) that they found 900,000 Kubernetes cluster API servers exposed to the public internet due to misconfiguration, but only 799 of them can be exploited, where the intruder can get the access of the entire cluster.

![country-wise distribution of exposed k8s cluster](/img/k8.png)

**2. Okta breach**

Okta is a public traded identity management company that was breached by a hacking group called Lapsus$. As per the investigation, it turned out to be hackers able to get VPN keys of support engineers by social engineering.

The most disturbing part is that the hackers are inside the network for more than 1 month without anyone's notice. 

![mockery on the state of the industry](/img/ss.png)

**3. Log4J**

If you are coming from the Java world, you wouldn't have missed the log4j vulnerability. Log4j vulnerability will allow the attacker to run the code whatever they want and gain access to the system. I'm still not sure how many of the folks would have upgraded the version or if this vulnerability is still living in a not upgraded system.

![meme on cloudflare](/img/log4j.jpeg)

## Learnings

From all those mentioned cases, it's clear that if we have followed best practices we could have avoided most of those instances.

- Network policies wouldn't have exposed Kubernetes clusters.
- Security education and basic monitoring wouldn't have let Lapsus$ hack the system for almost 1 month.
- Continuous package scanning to avoid supply chain would make the developers upgrade the system at the right time.

The present time itself is full of challenges in the world of network security. Everything seems to indicate that the number of threats that users will have to face will continue to grow, so now more than ever, having a good security policy and protection is essential.


We covered only a fraction of security incidents, we have curated the list of leaks around the world in the [github repo](https://github.com/inspektor-dev/awesome-data-leak). The list might be a shock to you by seeing your day-to-day tech companies on the [list](https://github.com/inspektor-dev/awesome-data-leak).

Let me leave you with a question **"Does Remote work lack IAM & cybersecurity oversight?"**

Chao!
