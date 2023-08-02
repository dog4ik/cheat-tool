/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{js,jsx,ts,tsx}"],
  theme: {
    extend: {
      animation: {
        "fade-in": "fadein 0.2s ease-in-out",
        "fade-out": "fadeout 0.2s ease-in-out",
        "push-in-fade-out": "push-in-fade-out 0.5s ease-in-out",
        blob: "blob 7s infinite",
        "slide-from-top": "slide-from-top 0.2s ease-in-out",
        "slide-from-botttom": "slide-from-botttom 0.2s ease-in-out",
        "slide-from-left": "slide-from-left 0.2s ease-in-out",
        "slide-from-right": "slide-from-right 0.2s ease-in-out",
      },
      keyframes: {
        fadein: {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
        fadeout: {
          "0%": { opacity: "1" },
          "100%": { opacity: "0" },
        },
        "push-in-fade-out": {
          "0%": { opacity: "0.8", scale: "0.5" },
          "100%": { opacity: "0", scale: "1" },
        },
        blob: {
          "0%": {
            transform: "translate(0px, 0px) scale(1)",
          },
          "33%": {
            transform: "translate(10px, -10px) scale(1.2)",
          },
          "66%": {
            transform: "translate(-10px, 10px) scale(1)",
          },
          "100%": {
            transform: "tranlate(0px, 0px) scale(1.2)",
          },
        },
        "slide-from-top": {
          "0%": {
            transform: "translateY(-100%)",
          },
          "100%": {
            transform: "translateY(0%)",
          },
        },
        "slide-from-botttom": {
          "0%": {
            transform: "translateY(100%)",
          },
          "100%": {
            transform: "translateY(0%)",
          },
        },
        "slide-from-left": {
          "0%": {
            transform: "translateX(-100%)",
          },
          "100%": {
            transform: "translateX(0%)",
          },
        },
        "slide-from-right": {
          "0%": {
            transform: "translateX(100%)",
          },
          "100%": {
            transform: "translateX(0%)",
          },
        },
      },
    },
  },
  plugins: [],
};
