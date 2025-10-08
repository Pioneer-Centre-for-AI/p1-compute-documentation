// @ts-check
import starlight from '@astrojs/starlight';
import { defineConfig } from 'astro/config';

import react from '@astrojs/react';

import starlightThemeNova from 'starlight-theme-nova'

import tailwindcss from '@tailwindcss/vite';

import expressiveCode from 'astro-expressive-code';

export default defineConfig({
    site: 'https://hpc.aicentre.dk',
    base: '/',
    integrations: [
        expressiveCode(),
        starlight({
            plugins: [
                starlightThemeNova(),
            ],
            title: 'P1 HPC Documentation',
            customCss: [
                './src/styles/global.css',
                './src/styles/custom.css',
            ],
            social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/Pioneer-Centre-for-AI/p1-compute-documentation' }],
            logo: {
                light: './src/assets/logos/PCAI_Solo_RGB.svg',
                dark: './src/assets/logos/PCAI_Solo_White.svg',
            },
            head: [
                {
                    tag: 'meta',
                    attrs: {
                        name: "author",
                        content: "Benjamin Starostka Jakobsen",
                    }
                },
                {
                    tag: 'link',
                    attrs: {
                        rel: 'icon',
                        href: './src/assets/logos/PCAI_Solo_RGB.svg',
                    }
                },
            ],
            sidebar: [
                { label: "About", link: '/about/' },
                {
                    label: 'Clusters',
                    items: [
                        { label: 'DTU HPC', slug: 'clusters/dtu-hpc' },
                        { label: 'NGC HPC', slug: 'clusters/ngc-hpc', badge: { text: 'WAITING LIST', variant: 'caution' } },
                        { label: 'DCAI Gefion', slug: 'clusters/dcai-gefion', badge: { text: 'INTEREST LIST', variant: 'caution' } },
                    ],
                },
                {
                    label: 'Guides',
                    autogenerate: { directory: 'guides' },
                },
                {
                    label: 'Reference',
                    items: [
                        { label: 'Learning Resources', slug: 'reference/learning-resources' },
                    ],
                },
                { label: 'Terms and Conditions', slug: 'terms-and-conditions' },
                { label: "Contribute", link: '/contribute/' },
                { label: 'Computing (new tab)', link: 'https://www.aicentre.dk/computing', attrs: { target: '_blank' } },
            ],
        }),
        react()],
    vite: {
        define: {
        },

        plugins: [tailwindcss()],
    },
});
