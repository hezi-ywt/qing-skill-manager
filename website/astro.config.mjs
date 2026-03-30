// @ts-check
import { defineConfig } from 'astro/config'
import tailwindcss from '@tailwindcss/vite'
import react from '@astrojs/react'
import sitemap from '@astrojs/sitemap'

export default defineConfig({
  site: 'https://hezi-ywt.github.io',
  base: '/qing-skill-manager',
  output: 'static',
  vite: {
    plugins: [tailwindcss()],
  },
  integrations: [
    react(),
    sitemap({
      filter: (page) => page !== 'https://hezi-ywt.github.io/qing-skill-manager/404',
    }),
  ],
})
