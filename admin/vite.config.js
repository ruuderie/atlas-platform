import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    port: 5150,
    host: '0.0.0.0',
    https: process.env.USE_HTTPS === 'true',
    fs: {
      // Allow serving files from one level up to the project root
      allow: ['..']
    },
    proxy: {
      // Add proxy configuration if needed
    },
    allowedHosts: [
      'localhost',
      'admin.rustsveltebusinessdirectory.orb.local',
      '.orb.local'
    ]
  },
  logLevel: 'info', // Changed from debug to info for more consistent logging
  clearScreen: false, // Prevent Vite from clearing the console
  build: {
    minify: false, // Disable minification to keep console.log statements
    sourcemap: true, // Generate source maps for better debugging
  },
  define: {
    'process.env.NODE_ENV': JSON.stringify(process.env.NODE_ENV || 'development'),
    'process.env.DEBUG': JSON.stringify('*'), // Add this to enable all debug logs
  },
  optimizeDeps: {
    // Force esbuild to use a specific version
    esbuildOptions: {
      // This helps ensure esbuild version consistency
      preserveSymlinks: true
    }
  },
  resolve: {
    alias: {
      '@': '/src',
      '@components': '/src/components'
    }
  }
});
