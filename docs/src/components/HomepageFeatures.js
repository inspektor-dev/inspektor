import React, {Fragment} from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import styles from './HomepageFeatures.module.css';
import GITOPS from "../../static/img/inspektor/gitops.png"


const FeatureList = [
  {
    title: 'OPEN POLICY AGENT',
    Svg: require('@site/static/img/inspektor/policy_agent.svg').default,
    description: "Inspektor by default support rego language by open policy agent. So, you can write polices as code",
  },
  {
    title: 'SSO LOGIN',
    Svg: require('@site/static/img/inspektor/sso_login.svg').default,
    description: "Use your existing SSO logins to access your databases. Credentials automatically created and deleted as person join or leaves the organization",
  },
  {
    title: 'GRANULAR ACCESS CONTROL',
    Svg: require('@site/static/img/inspektor/granular_access.svg').default,
    description: "Use column level access level to protect your customers PII data such as SSN, date of birth and address",
  },
  {
    title: 'ENRICHED ACCESS LOG',
    Svg: require('@site/static/img/inspektor/access_log.svg').default,
    description: "Inspektor gives enriched access log to figure out who did what and when. This helps security team to investigate database access pattern",
  },
  {
    title: 'ANOMALY DETECTION',
    Svg: require('@site/static/img/inspektor/anamoly_detection.svg').default,
    description: "Inspektor continously monitor database access and alerts if it find any anolmaly",
  },
  {
    title: 'GITOPS',
    // Svg: require('@site/static/img/inspektor/gitops.svg').default,
    imageSrc: GITOPS,
    description: "Inspektor syncs policies from github. So you can use your existing workflow to review and approve PR. As soon as code merges, the polices will be updated",
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
        <p>Database administrators can manage their access to their sql or no sql database from a centralized tool. Inspektor also helps teams to audit their database access using enriched access log. So, admins never miss on who queried what.</p>
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
        <h1 style={{ color: "white"}}>Inspektor helps to secure your databases. Join the community</h1>
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
