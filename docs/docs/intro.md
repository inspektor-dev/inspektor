---
sidebar_position: 1
title: Tutorial
---

# Introduction

Inpektor helps you to enfore access control for all your data sources. 

In this tutorial, you'll be downloading and running the sample docker-compose setup and configure it to enforce access policy on postgres database for different user group. 

# Prerequisite
 - docker
 - docker-compose

## Demo Inspektor setup.

Copy the given give yaml file and paste it to `docker-compose.yaml`

```yaml

version: '3.5'

services:
  postgres:
    container_name: postgres_container
    image: postgres
    environment:
        POSTGRES_USER: "debuggeruser"
        POSTGRES_PASSWORD: "debuggerpassword"
    volumes:
       - postgres:/data/postgres
    ports:
      - "5432:5432"
    restart: unless-stopped
volumes: 
    postgres:
```