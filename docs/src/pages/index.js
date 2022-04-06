import React, { useEffect } from 'react';
import clsx from 'clsx';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import styles from './index.module.css';
import HomepageFeatures from '@site/src/components/HomepageFeatures';

function HomepageHeader() {
  const { siteConfig } = useDocusaurusContext();
  
  return (
    <header className={clsx('hero ', styles.heroBanner)}>
      <div className="container hero-title" >
        <h1 className="hero-header">
          Centralised access control for all your databases
        </h1>
        <p className="hero__subtitle">
          Control your Policy as code. Monitor and secure your data layer in a centralized tool.
        </p>
        <div className={styles.buttons}>
          <Link
            className="button button--primary github-button-styles"
            to="https://github.com/poonai/inspektor">
            <div className='hero-button-text'>
              <img src='img/inspektor/github_icon.svg' />&nbsp;Github
            </div>
          </Link>
          <div onMouseEnter={() => {
            document.getElementById("discord-button-text").style.color = "white"
            document.getElementById("discord-img").setAttribute("src", "img/inspektor/discord_icon_white.svg")
          }}
            onMouseLeave={() => {
              document.getElementById("discord-button-text").style.color = "#FF7A00"
              document.getElementById("discord-img").setAttribute("src", "img/inspektor/discord_icon_default.svg")
            }}>
            <Link
              className="button button--warning button--outline discord-button-styles "
              to="https://discord.com/invite/YxZbDJHTxf">
              <div className='hero-button-text'>
                <img id="discord-img" src='img/inspektor/discord_icon_default.svg' />&nbsp;<span id="discord-button-text">Discord</span>
              </div>
            </Link>
          </div>

        </div>

        <img src='img/inspektor/inspektor_hero_image.svg'/>
      </div>
    </header>
  );
}

export default function Home() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      title={`Inspektor`}
      description="Access control for your database simplified">
      <HomepageHeader />
      <main>
        <HomepageFeatures />
      </main>
    </Layout>
  );
}
