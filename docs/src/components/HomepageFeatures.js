import React, {Fragment} from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import styles from './HomepageFeatures.module.css';
import GITOPS from "../../static/img/inspektor/gitops.png"


const FeatureList = [
  {
    title: 'OPEN POLICY AGENT LOG',
    Svg: require('@site/static/img/inspektor/policy_agent.svg').default,
    description: "Search-as-you-type returns answers in less than 50 milliseconds. That's faster than the blink of an eye!",
  },
  {
    title: 'SSO LOGIN',
    Svg: require('@site/static/img/inspektor/sso_login.svg').default,
    description: "Search-as-you-type returns answers in less than 50 milliseconds. That's faster than the blink of an eye!",
  },
  {
    title: 'GRANULAR ACCESS LEVEL',
    Svg: require('@site/static/img/inspektor/granular_access.svg').default,
    description: "Search-as-you-type returns answers in less than 50 milliseconds. That's faster than the blink of an eye!",
  },
  {
    title: 'ENRICHED ACCESS LOG',
    Svg: require('@site/static/img/inspektor/access_log.svg').default,
    description: "Search-as-you-type returns answers in less than 50 milliseconds. That's faster than the blink of an eye!",
  },
  {
    title: 'ANAMOLY DETECTION',
    Svg: require('@site/static/img/inspektor/anamoly_detection.svg').default,
    description: "Search-as-you-type returns answers in less than 50 milliseconds. That's faster than the blink of an eye!",
  },
  {
    title: 'GITOPS',
    // Svg: require('@site/static/img/inspektor/gitops.svg').default,
    imageSrc: GITOPS,
    description: "Search-as-you-type returns answers in less than 50 milliseconds. That's faster than the blink of an eye!",
  },
];

function Feature({ Svg, title, description, imageSrc }) {
  return (
    <div className={styles.feature}>
      <div className="text--center">
        {Svg ? <Svg className={styles.featureSvg} role="img" /> : <img className={styles.featureSvg} src={imageSrc}/>}
      </div>
      <div className="text--center padding-horiz--md">
        <h3 className={styles.titleStyles}>{title}</h3>
        <p>{description}</p>
      </div>
    </div>
  );
}

function VideoSection({ }) {
  return (
    <div className={styles.videoSection}>
      <div className={styles.videoCol1}>
        <h3 style={{ fontSize: "2rem" }}>Data Access Simplified</h3>
        <p>Database administrators can manage their access to their sql or no sql database from a centralized tool. Inspektor also helps teams audit database access using its enriched access log. So, admins never miss on who queried what.</p>
      </div>
      <div className={styles.videoCol2}>
        <iframe width="100%" height="400" src="https://www.youtube.com/embed/E7X5-mGRKro" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
      </div>
    </div>
  )
}

function BottomHero() {
  return (
    <div className={styles.bottomHero}>
      <div className={styles.bottomHeroContainer}>
        <h1 style={{ color: "white"}}>Get Ready to Started. It's Fast & Easy.</h1>
        <p>Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et olore magna aliqua. Risus commodo viverra.</p>
        <Link className='button button--secondary' style={{color: "#7230FF", width: "10rem"}} to="https://discord.com/invite/YxZbDJHTxf">
         <div className='hero-button-text'> <img src="img/inspektor/discord_icon_blue.svg"/>&nbsp;Discord</div>
        </Link>
      </div>
    </div>
  )
}

export default function HomepageFeatures() {
  return (
    <Fragment>
      <section className={styles.features}>
        <div className="container">
          <h1 className={styles.featureHeading}>Why Inspektor</h1>
          <div className={styles.featureRow}>
            {FeatureList.map((props, idx) => (
              <Feature key={idx} {...props} />
            ))}
          </div>
          <VideoSection />
        </div>
      </section>
      <BottomHero />
    </Fragment>

  );
}
