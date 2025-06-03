/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./ui/src/**/*.rs",
    "./ui/index.html"
  ],
  theme: {
    extend: {
      colors: {
        primary: "#0A6460",
        accent:  "#E55934",
        neutral: {
          light: "#F2EFE9",
          dark: "#1D1128",
        },
        shadow: {
          light: "#D9DBE1",
          dark: "#9EA2B0",
        }
      },
    },
  },
  plugins: [],
}
