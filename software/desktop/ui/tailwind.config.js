/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{js,jsx,ts,tsx}", "./public/index.html"],
  theme: {
    extend: {
      colors: {
        accent: "#B56650",  // dusty terracotta
        ink: "#3B2C24",
        paper: "#FAF7F2",
        muted: "#8C7663"
      }
    }
  },
  plugins: []
};
