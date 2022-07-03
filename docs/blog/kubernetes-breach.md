---
title: silly misconfiguration made 900,000 kubernetes cluster public.  
description: In recent days
slug: missconfiguration-made-kubernetes-cluster-public
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

Most of us do silly mistakes mostly due to lethargic or some are geninue. But the price we pay is huge because some of the damaage are irreversiable. 
But exposing million kuebernetes cluster seems little serious and made me to revisit some of the
security issues in recent days.

# Recent Security issues

**Cybel claim**

[Cyble claims](https://blog.cyble.com/2022/06/27/exposed-kubernetes-clusters/) that they found 900,000 kubernetes cluster api server exposed to the public internet, but only 799 of them can be exploited, where the intruder can get the access of the entire cluster.

![country wise distribution of exposed k8s cluster](/img/k8.png)

**Okta breach**

Okta is a public traded identity management company was breached by hacking group called Lapsus$. As per the investigation, it turned to be hackers able to get vpn keys of support engineers by social engineering.

The most disturbing part is that the hackers are inside the network for more than 1 months without anyone's notice. 

![mockery on state of industry](/img/ss.png)

**Log4J**

If you are coming from java world, you wouldn't have missed log4j vulnerbility. Log4j vulnerbility will allow the attacker to run the code whatever they want and gain access to the system. I'm still not sure how many of the folks would of upgraded the version or this vulnerbility is still living in a not upgraded system.

![meme on cloudflare](/img/log4j.jpeg)

## Learnings

From those all those metioned cases, it's clear that if we have followed best practices we could have avoided most of those instance.

- Network polices wouldn't have exposed kubernetes clusters
- Security education and basic monitoring wouldn't have let Lapsus$ to hack the system for almost 1 month
- Continous package scanning to avoid supply chain would made the developers to upgrade the system at right time


We covered only fraction of security insidents, we have curated the list of leaks around the world in the [github repo](https://github.com/inspektor-dev/awesome-data-leak). The list might be a shock to you by seeing your day to day tech companies on the [list](https://github.com/inspektor-dev/awesome-data-leak).