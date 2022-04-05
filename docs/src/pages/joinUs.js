import React from "react"
import Layout from '@theme/Layout';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';


const JoinUs = () => {
    const { siteConfig } = useDocusaurusContext();

    return (
        <Layout
            title={`Hello from ${siteConfig.title}`}
            description="Description will go into a meta tag in <head />">
            <main>
                <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "50vh" }}>
                    <h3># Page under construction</h3>
                </div>
            </main>
        </Layout>
    )
}

export default JoinUs