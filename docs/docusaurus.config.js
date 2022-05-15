// @ts-check
// Note: type annotations allow type checking and IDEs autocompletion

const lightCodeTheme = require('prism-react-renderer/themes/github');
const darkCodeTheme = require('prism-react-renderer/themes/dracula');

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'Inspektor',
  tagline: 'Centralised access control for all your databases',
  url: 'https://inspektor.cloud',
  baseUrl: '/',
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',
  favicon: 'img/favicon.ico',
  organizationName: 'poonai', // Usually your GitHub org/user name.
  projectName: 'inspektor', // Usually your repo name.
  plugins: [
    [
      '@docusaurus/plugin-google-gtag',
      {
        trackingID: 'G-5H72H5D4R5',
        anonymizeIP: true,
      },
    ],
  ],
  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          // Please change this to your repo.
          editUrl: 'https://github.com/poonai/inspektor/tree/main/docs',
        },
        blog: {
          showReadingTime: true,
          // Please change this to your repo.
          editUrl:
            'https://github.com/poonai/inspektor/tree/main/blog',
        },
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      metadata: [{property:"og:image", content:"/img/inspektorcover.png"}],
      // googleAnalytics: {
      //   trackingID: 'G-5H72H5D4R5',
      //   anonymizeIP: true,
      // },
      // gtag: {
      //   trackingID: 'G-5H72H5D4R5',
      //   anonymizeIP: true,
      // },
      announcementBar: {
        id: "support",
        backgroundColor: "#7230FF",
        textColor: "#fff",
        isCloseable: false,
        content: `If you like Inspektor, give us a star ⭐️ on <a target="_blank" rel="noopener noreferrer" href="https://github.com/poonai/inspektor">GitHub</a>`,
    },
      colorMode:{
        defaultMode: 'light',
        disableSwitch: true,
      },
      navbar: {
        title: 'Inspektor',
        hideOnScroll: true,
        logo: {
          alt: 'Inspektor logo',
          src: 'img/inspektor/inspektor-logo.svg',
        },
        items: [
          {
            to: '/', 
            label: 'Home', 
            position: 'right', 
            className: 'navbar-item'
          },
          {
            type: 'doc',
            docId: 'intro',
            position: 'right',
            label: 'Docs',
            className: 'navbar-item'
          },
          //  {to: '/about', 
          //  label: 'About', 
          //  position: 'right', 
          //  className: 'navbar-item'
          // },
          {
            to: 'blog', 
            label: 'Blog', 
            position: 'right', 
            className: 'navbar-item'
          },
          {
            href: 'https://github.com/poonai/inspektor',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'light',
        links: [
          {
            title: 'Inspektor',
            items: [
              {
                label: 'Docs',
                to: '/docs/intro',
              },
            ],
          },
          {
            title: 'Community',
            items: [
              // {
              //   label: 'Stack Overflow',
              //   href: 'https://stackoverflow.com/questions/tagged/docusaurus',
              // },
              {
                label: 'Discord',
                href: 'https://discord.gg/YxZbDJHTxf',
              },
              {
                label: 'Twitter',
                href: 'https://twitter.com/poonai_',
              },
            ],
          },
          {
            title: 'More',
            items: [
              {
                label: 'Blog',
                to: '/blog',
              },
              {
                label: 'GitHub',
                href: 'https://github.com/poonai/inspektor',
              },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} Inspektor. All Rights Reserved.`,
      },
      prism: {
        theme: lightCodeTheme,
        darkTheme: darkCodeTheme,
      },
    }),
};

module.exports = config;
