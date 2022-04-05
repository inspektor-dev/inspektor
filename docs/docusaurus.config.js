// @ts-check
// Note: type annotations allow type checking and IDEs autocompletion

const lightCodeTheme = require('prism-react-renderer/themes/github');
const darkCodeTheme = require('prism-react-renderer/themes/dracula');

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'Inspektor',
  tagline: 'Open Policy for your data layer',
  url: 'https://your-docusaurus-test-site.com',
  baseUrl: '/',
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',
  favicon: 'img/favicon.ico',
  organizationName: 'poonai', // Usually your GitHub org/user name.
  projectName: 'inspektor', // Usually your repo name.

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
            'https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/',
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
      announcementBar: {
        id: "support",
        backgroundColor: "#7230FF",
        textColor: "#fff",
        isCloseable: false,
        content: `⭐️ If you like Inspektor, give it a star on <a target="_blank" rel="noopener noreferrer" href="https://github.com/poonai/inspektor">GitHub</a>`,
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
          {to: '/', label: 'Home', position: 'right', className: 'navbar-item'},
          {
            type: 'doc',
            docId: 'intro',
            position: 'right',
            label: 'Docs',
            className: 'navbar-item'
          },
          // {to: '/about', label: 'About', position: 'right', className: 'navbar-item'},
          // {to: '/joinus', label: 'Join us', position: 'right', className: 'navbar-item'},
          // {
          //   href: 'https://github.com/facebook/docusaurus',
          //   label: 'GitHub',
          //   position: 'right',
          // },
        ],
      },
      footer: {
        style: 'light',
        links: [
          {
            title: 'Docs',
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
          // {
          //   title: 'More',
          //   items: [
          //     {
          //       label: 'Blog',
          //       to: '/blog',
          //     },
          //     {
          //       label: 'GitHub',
          //       href: 'https://github.com/poonai/inspektor',
          //     },
          //   ],
          // },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} Inspektor. All Rights Reserved`,
      },
      prism: {
        theme: lightCodeTheme,
        darkTheme: darkCodeTheme,
      },
    }),
};

module.exports = config;
