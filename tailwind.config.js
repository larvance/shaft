/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        obsidian: {
          950: "#08090a",
          900: "#0f1012",
          850: "#121417",
          800: "#15171a",
          700: "#1c1e22",
          600: "#272a30",
          500: "#363a43",
        },
        brand: {
          DEFAULT: "#5e6ad2",
          light: "#818cf8",
          dark: "#4338ca",
          glow: "rgba(94, 106, 210, 0.4)",
        },
        hairline: "rgba(255, 255, 255, 0.08)",
      },
      fontFamily: {
        sans: [
          "Inter",
          "-apple-system",
          "BlinkMacSystemFont",
          "Segoe UI",
          "Roboto",
          "sans-serif",
        ],
        mono: [
          "JetBrains Mono",
          "ui-monospace",
          "SFMono-Regular",
          "Menlo",
          "monospace",
        ],
      },
      letterSpacing: {
        tighter: "-0.04em",
        tight: "-0.025em",
      },
    },
  },
  plugins: [],
};
