import "vuetify/styles";
import "@mdi/font/css/materialdesignicons.css";
import { createVuetify } from "vuetify";
import * as components from "vuetify/components";
import * as directives from "vuetify/directives";

export const vuetify = createVuetify({
  components,
  directives,
  theme: {
    defaultTheme: "light",
    themes: {
      light: {
        dark: false,
        colors: {
          primary: "#4A7CFF",
          secondary: "#7A8BA5",
          background: "#F4F7FB",
          surface: "#FCFEFF",
          "surface-elevated": "#FFFFFF",
          glass: "#FFFFFF",
          "glass-border": "#DCE6F2",
          success: "#2EA26B",
          warning: "#D99A33",
          error: "#D85F6B",
          info: "#5B8AF0",
        },
      },
      dark: {
        dark: true,
        colors: {
          primary: "#7AA6FF",
          secondary: "#90A0B8",
          background: "#0E1726",
          surface: "#131F30",
          "surface-elevated": "#1A2840",
          glass: "#18273A",
          "glass-border": "#2D4460",
          success: "#58C48A",
          warning: "#F3BF65",
          error: "#FF8C99",
          info: "#7AA6FF",
        },
      },
    },
  },
  defaults: {
    VBtn: {
      variant: "flat",
      rounded: "lg",
    },
    VTextField: {
      variant: "outlined",
      density: "comfortable",
      rounded: "lg",
    },
    VSelect: {
      variant: "outlined",
      density: "comfortable",
      rounded: "lg",
    },
    VSlider: { density: "compact", trackSize: 4 },
    VCard: { flat: true, rounded: "xl" },
    VChip: { rounded: "pill" },
    VAlert: { rounded: "xl", variant: "tonal" },
    VProgressLinear: { rounded: true, height: 4 },
  },
});
