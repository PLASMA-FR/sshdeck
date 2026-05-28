import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'SSHDeck',
  description: 'Termius + Yazi for your terminal. A clean local-first SSH command center built in Rust.',
  base: '/sshdeck/',
  cleanUrls: true,
  lastUpdated: true,
  head: [
    ['meta', { name: 'theme-color', content: '#05070b' }],
    ['meta', { property: 'og:title', content: 'SSHDeck' }],
    ['meta', { property: 'og:description', content: 'Termius + Yazi for your terminal. No cloud. No account. No Electron.' }],
    ['meta', { property: 'og:type', content: 'website' }]
  ],
  themeConfig: {
    logo: '/logo.svg',
    siteTitle: 'SSHDeck',
    nav: [
      { text: 'Guide', link: '/docs/getting-started' },
      { text: 'Files', link: '/docs/sshdeck-files' },
      { text: 'Security', link: '/docs/security' },
      { text: 'Roadmap', link: '/docs/roadmap' }
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/PLASMA-FR/sshdeck' }
    ],
    sidebar: {
      '/docs/': [
        {
          text: 'Start',
          items: [
            { text: 'Docs home', link: '/docs/' },
            { text: 'Getting started', link: '/docs/getting-started' },
            { text: 'Installation', link: '/docs/installation' },
            { text: 'Quickstart', link: '/docs/quickstart' }
          ]
        },
        {
          text: 'Core workflows',
          items: [
            { text: 'Host management', link: '/docs/host-management' },
            { text: 'Importing SSH config', link: '/docs/importing-ssh-config' },
            { text: 'Keyboard shortcuts', link: '/docs/keyboard-shortcuts' },
            { text: 'Mouse support', link: '/docs/mouse-support' },
            { text: 'SSHDeck Files', link: '/docs/sshdeck-files' },
            { text: 'File transfers', link: '/docs/file-transfers' },
            { text: 'Safe remote editing', link: '/docs/safe-remote-editing' },
            { text: 'Tunnels', link: '/docs/tunnels' },
            { text: 'Remote commands', link: '/docs/remote-commands' },
            { text: 'Health checks', link: '/docs/health-checks' }
          ]
        },
        {
          text: 'Reference',
          items: [
            { text: 'Configuration', link: '/docs/configuration' },
            { text: 'Themes', link: '/docs/themes' },
            { text: 'Security', link: '/docs/security' },
            { text: 'Troubleshooting', link: '/docs/troubleshooting' },
            { text: 'Roadmap', link: '/docs/roadmap' },
            { text: 'Contributing', link: '/docs/contributing' },
            { text: 'FAQ', link: '/docs/faq' }
          ]
        }
      ]
    },
    search: { provider: 'local' },
    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2026 SSHDeck contributors'
    }
  }
})
