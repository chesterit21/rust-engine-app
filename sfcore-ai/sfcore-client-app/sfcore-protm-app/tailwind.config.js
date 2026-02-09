
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'dark-bg': '#02040a', // Almost black, very deep blue hint
        'dark-card': '#0a0c14', // Slightly lighter for cards
        'neon-cyan': '#00F0FF',
        'neon-teal': '#2DE2E6',
        'crimson': '#DC143C',
        'dark-border': '#1e1e1e',
      }
    },
  },
  plugins: [],
}
