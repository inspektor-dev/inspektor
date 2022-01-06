import React from 'react';
import clsx from 'clsx';
import styles from './HomepageFeatures.module.css';

const FeatureList = [
  {
    title: 'Policy as Code',
    Svg: require('../../static/img/openpolicyagent-icon.svg').default,
    description: (
      <>
        Inspektor let's the user to define the access policy as code. It use 
        CNCF open policy agent as it's default policy language. 
      </>
    ),
  },
  {
    title: 'GitOps',
    Svg: require('../../static/img/Octocat.svg').default,
    description: (
      <>
        Inspektor has a first class integration with github. As soon as policy author,
        pushes the policy to github, inspektor magically fetches the updated
        policy and configure all the dataplane dynamically.
      </>
    ),
  },
  {
    title: 'Centralized Access Control',
    Svg: require('../../static/img/pairprogramming.svg').default,
    description: (
      <>
        Inspektor allows users to enforce access policy across all the data source.
        Currently, Inspektor supports postgres, more integration will be added soon.
      </>
    ),
  },
];

function Feature({Svg, title, description}) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center">
        <Svg className={styles.featureSvg} alt={title} />
      </div>
      <div className="text--center padding-horiz--md">
        <h3>{title}</h3>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures() {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
