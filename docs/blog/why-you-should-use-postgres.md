---
title: Why you should use postgres as a primary database?
description: Find out why postgres is so popular and it's used an a primary database
slug: why-postgres-is-popular
authors:
  - name: Poonai
    title: Maintainer of inspektor
    url: https://twitter.com/poonai_
    image_url: https://i.imgur.com/RNM7R6Q.jpg
tags: [postgres,extensions]
image: /img/elephant.jpg
hide_table_of_contents: false
---
![image of an elephant](/img/elephant.jpg)

As a university student, I was more familiar with `NodeJS` and`mongoose` than `SQL`, so I went with `MongoDB`, instead of `SQL`. When I started working in the real world, I became familiar with postgres. Postgres has always been my go-to database for my projects since then ❤️.

**here is my mental model, why I choose Postgres for all my projects?** 

- **tested and proven**
- **third party extension**
- **JSON for other needs**
- **tooling around around postgres** 
- **community** 

## tested and proven

For more than two decades, Postgres has been developed and is now being used by many large corporations. It's a basic human instinct to stick with tried-and-true methods. Take a look at the screenshot below to see how an instagram engineer flexing his postgres usage.

![postgres at instagram](/img/postgresinstagram.png)

## extension ecosystem

Postgres allows developers to extends it's capabilites by writing an extensions, since some unique usecase can't be solved by general purpose database.

**I used pg_cron extenstion to solve my unique usecase myself**

> I wanted to do historic aggregation of a numeric column. Usual scenario would be building a ETL pipeline, but I found a solution using `pg_cron`. You can check this [link](https://hashnode.com/post/how-to-use-pgcron-in-postgres-to-do-historic-aggregation-ckzcsfi150ffzxts12eqegiq5) to know the entire story. 

Fellow OSS engineers have opensourced their extensions for the community to use. Here are some of my favourite extensions: 

- [**zomboDB**](https://github.com/zombodb/zombodb) integrates elastic search with Postgres for full text search. 
- [**pg_cron**](https://github.com/citusdata/pg_cron) cron jobs inside postgres
- [**pg_storm**](https://github.com/heterodb/pg-strom) accelerate analytics query performance by offloading analytics computation to GPU

you can always write your own extension, if you don't find extenstion for your usecase. 
Now you can write extension in rust as well using [`pgx`](https://github.com/tcdi/pgx)

## JSON for other needs

Usual question that comes while choosing Postgres is that, can we store complex relationship?. But unknow fact to most of the developers is that, postgres let developers to store and query JSON data.

![postgres json tweet](/img/postgresjson.png)

## Tooling around Postgres

Having a good database alone won't solve the problem, there are other scenarios that we need to consider. For eg: backup, runnnig an HA setup. Postgres have all sort of tooling to run a production database.

- [**patroni**](https://github.com/zalando/patroni) - running a HA postgres on k8s
- [**kubesdb**](https://kubedb.com/) - running postgres on k8s
- [**dexter**](https://github.com/ankane/dexter) - automatic indexer to optimize db query  performance
- [**timescale**](https://github.com/timescale/timescaledb) - turn your postgres into timeseries database
- [**supabase**](https://github.com/supabase/supabase) - instant graphql api from postgres databases


## Community

postgres community is very welcoming and have precense in all the popular social communities: 
- [IRC](https://www.postgresql.org/community/irc/)
- [SLACK](https://postgres-slack.herokuapp.com/)
- [DISCORD](https://www.reddit.com/r/PostgreSQL/comments/ie8h3z/postgres_discord_server/)


Ofc, you can join [our community](https://t.co/NWnxhxsIx7) as well to talk about postgres :P 

Not only does the community have a presence on various social media platforms, but it is also friendly and helps you instantly if you come across any issue.

## Closing Notes

Postgres isn't just a database; it's an entire ecosystem of development, research, and innovation that's impossible to fathom. I want to end the essay by saying

> Postgres doing its justification for its elephant mascot
