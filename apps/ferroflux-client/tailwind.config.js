/** @type {import('tailwindcss').Config} */
export default {
    content: ['./src/**/*.{html,js,svelte,ts}'],
    theme: {
        extend: {
            colors: {
                // Semantic colors for the app - Dark Mode optimized
                bg: {
                    DEFAULT: '#191919', // Main background
                    sidebar: '#202020', // Sidebar background
                    hover: '#2C2C2C',   // Hover state
                    active: '#37352F',  // Active/Selected state
                    input: '#252525',   // Input fields
                },
                border: {
                    DEFAULT: '#2F2F2F',
                    subtle: '#252525',
                    active: '#454545'
                },
                text: {
                    DEFAULT: '#EFEFEF', // Primary text
                    muted: '#9B9B9B',   // Secondary text
                    subtle: '#6B6B6B',  // Tertiary text
                },
                // Functional colors
                brand: {
                    DEFAULT: '#FF5C5C', // Accent
                    hover: '#FF7C7C'
                },
                status: {
                    success: '#4CAF50',
                    warning: '#FFC107',
                    error: '#F44336'
                }
            },
            fontFamily: {
                sans: ['Inter', 'ui-sans-serif', 'system-ui', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'Helvetica Neue', 'Arial', 'sans-serif'],
                mono: ['JetBrains Mono', 'ui-monospace', 'SFMono-Regular', 'Menlo', 'Monaco', 'Consolas', 'Liberation Mono', 'Courier New', 'monospace'],
            }
        },
    },
    plugins: [],
}