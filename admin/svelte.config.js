import adapter from '@sveltejs/adapter-node'; // Change from adapter-auto
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  
  kit: {
    adapter: adapter(),
    
    // Add your path aliases here
    alias: {
      '$lib': './src/lib',
      '$lib/*': './src/lib/*'
    }
  },
  compilerOptions: {
    // Add any additional compiler options here
    // For example, you can enable strict mode
    runes: true
  }
};

export default config;
