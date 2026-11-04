import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { readdirSync } from 'node:fs';
import { defineConfig } from 'vite';

const galleryDirs = readdirSync(new URL('./pkgs/gallery', import.meta.url), { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name);

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
  define: { __GALLERY_DIRS__: JSON.stringify(galleryDirs) }
});
