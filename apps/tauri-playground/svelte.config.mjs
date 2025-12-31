import adapter from '@sveltejs/adapter-static';

let preprocess;
try {
  const { vitePreprocess } = await import('@sveltejs/vite-plugin-svelte');
  preprocess = vitePreprocess();
} catch (e) {
  console.warn('Could not load vitePreprocess, likely an IDE error:', e);
}

// Trigger reload
/** @type {import('@sveltejs/kit').Config} */
const config = {
  // Consult https://svelte.dev/docs/kit/integrations#preprocessors
  // for more information about preprocessors
  preprocess,

  kit: {
    adapter: adapter({
      fallback: 'index.html',
    }),
  },
};

export default config;