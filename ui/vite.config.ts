import { defineConfig, loadEnv } from 'vite';
import solid from 'vite-plugin-solid';
import federation from '@originjs/vite-plugin-federation';

export default defineConfig(({ mode }) => {
  // Load env file based on `mode` in the current working directory.
  const env = loadEnv(mode, process.cwd(), '');
  const host = env.TAURI_DEV_HOST;

  return {
    define: {
      // Provide an explicit app-level constant derived from an env var.
      __APP_ENV__: JSON.stringify(env.APP_ENV),
    },

    plugins: [
      solid(),
      federation({
        name: 'phoenix-host',
        // Configure remotes here when you have them.
        // Example:
        // remotes: {
        //   remoteApp: 'http://localhost:5001/assets/remoteEntry.js',
        // },
        shared: ['solid-js'],
      }),
    ],

    clearScreen: false,

    server: {
      port: env.APP_PORT ? Number(env.APP_PORT) : 1420,
      strictPort: true,
      host: host || false,
      hmr: host
        ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
        : undefined,
      watch: {
        ignored: ["**/src-tauri/**"],
      },
    },

    envPrefix: ['VITE_', 'TAURI_ENV_*'],

    build: {
      // Tauri uses Chromium on Windows and WebKit on macOS and Linux
      target: env.TAURI_PLATFORM === 'windows' ? 'chrome105' : 'safari13',

      // don't minify for debug builds
      minify: !env.TAURI_ENV_DEBUG ? 'esbuild' : false,
      // produce sourcemaps for debug builds
      sourcemap: !!env.TAURI_ENV_DEBUG,
    },
  };
});
