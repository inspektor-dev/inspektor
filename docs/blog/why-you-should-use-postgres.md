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

Initially, I was more comfortable with `MongoDB` than `SQL` because `NodeJS` and `mongoose` was easy to use as a university kid. I got familiar to postgres once I took the real job. After that I never went back to any other databases for my projects, it's always been postgres ❤️.

**here is my mental model, why I choose Postgres for all my projects?** 

- tested and proven
- third party extension
- JSON for other needs
- tooling around around postgres 
- community 

## tested and proven

Postgres is been developed for more than 2 decades and used by many big giants. It's basic  human tendency to avoid risk and use the well-tested solution. Check the below screenshot, how instagram engineer flexing his postgres usage. 

![postgres at instagram](/img/postgresinstagram.png)

## Extension ecosystem

Postgres allows developers to extends it's capabilites by writing an extensions, since some get into some unique usecase which can't be solved by general purpose database.

**I used pg_cron extenstion to solve my unique usecase myself**

> I wanted to do historic aggregation of a numeric column. Usual scenario would be building a ETL pipeline, but I found a solution using `pg_cron`. You can check this [link](https://hashnode.com/post/how-to-use-pgcron-in-postgres-to-do-historic-aggregation-ckzcsfi150ffzxts12eqegiq5) to know the entire story. 

Fellow OSS engineers have opensourced their extensions for the community to use. Here are some of my favourite extensions: 

- [**zomboDB**](https://github.com/zombodb/zombodb) integrates elastic search with Postgres for full text search. 
- [**pg_cron**](https://github.com/citusdata/pg_cron) cron jobs inside postgres
- [**pg_storm**](https://github.com/heterodb/pg-strom) accelerate analytics query performance by offloading analytics computation to GPU

If you don't find any extenstion for your usecase, you can always write your own extension. 
Now you can write extension in rust as well using [`pgx`](https://github.com/tcdi/pgx) crate.

## JSON for other needs

Usual question that comes while choosing Postgres is that we can't store complex relationship. But unknow fact to most of the developers is that, postgres let developers to store and query JSON data. 

![image of an elephant](/img/postgresjson.png)

## Tooling around Postgres

Having a good database alone don't solve the problem, there are other scenarios we need to consider. For eg: backup, runnnig an HA setup. Postgres have all sort of tooling to run a production database.

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

Not just community have precense in different social channel, community is friendly and helps you instantly if you come across any issue. 

## Closing Notes

Postgres is not just about the database, the kind of ecosystem, development, and research it poses is unimaginable. I want to end the essay by saying

> Postgres doing its justification for its elephant mascot
