const defaultTheme = require('tailwindcss/defaultTheme');

module.exports = {
  content: [
    "./ui/src/**/*.rs",
    "./index.html"
  ],
  theme: {
    extend: {
      fontFamily: {
        display: ['"Oxanium"', ...defaultTheme.fontFamily.sans],
        sans: ['"Sora"', ...defaultTheme.fontFamily.sans],
      },
      colors: {
        background: "#210F37",
        surface: "#121829",
        primary: "#00FFC3",
        accent:  "#E55934",
        text: {
          DEFAULT: "#E4E4E4",
          muted: "#9CA3AF",
        },
        neutral: {
          dark: "#1A1423",
          light: "#E4E0E7",
        },
      },
    },
  },
  plugins: [],
}
