/** @type {import('tailwindcss').Config} */
export default {
    content: ['./src/**/*.{html,js,svelte,ts}'],
    theme: {
        extend: {
            colors: {
                // Using the requested deep gray palette
                bg: '#0F0F0F',
            }
        },
    },
    plugins: [],
}
