export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        primary: 'var(--text-primary)',
        secondary: 'var(--text-secondary)',
        accent: 'rgb(var(--accent-rgb))',
        sidebar: 'var(--sidebar-bg)',
        card: 'var(--card-bg)',
        border: {
          DEFAULT: 'var(--border-color)',
          subtle: 'var(--border-color)',
        }
      },
      typography: {
        // 
      },
      spacing: {
        // ...
      },
      backgroundSize: ({ theme }) => ({
        auto: 'auto',
        cover: 'cover',
        contain: 'contain',
        ...theme('spacing')
      })
    },
  },
  plugins: [],
};
