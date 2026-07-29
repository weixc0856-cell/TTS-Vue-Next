import { createRouter, createWebHistory } from "vue-router";

const routes = [
  {
    path: "/",
    name: "text-to-speech",
    component: () => import("../views/TextToSpeech.vue"),
  },
  {
    path: "/batch",
    name: "batch-convert",
    component: () => import("../views/BatchConvert.vue"),
  },
  {
    path: "/practice",
    name: "practice-hub",
    component: () => import("../views/practice/PracticeHub.vue"),
  },
  {
    path: "/practice/shadowing",
    name: "shadowing",
    component: () => import("../components/practice/shadowing/ShadowingSession.vue"),
    props: (route: { query: Record<string, string | undefined> }) => ({
      exerciseId: route.query.exerciseId || null,
      text: route.query.text || null,
    }),
  },
  {
    path: "/practice/roleplay",
    name: "roleplay",
    component: () => import("../components/practice/roleplay/RoleplaySession.vue"),
    props: (route: { query: Record<string, string | undefined> }) => ({
      exerciseId: route.query.exerciseId || null,
    }),
  },
  {
    path: "/practice/pronunciation",
    name: "pronunciation",
    component: () => import("../components/practice/pronunciation/DrillSession.vue"),
  },
  {
    path: "/practice/history",
    name: "practice-history",
    component: () => import("../views/practice/PracticeHistory.vue"),
  },
  {
    path: "/practice/history/:id",
    name: "session-detail",
    component: () => import("../views/practice/SessionDetailView.vue"),
  },
  {
    path: "/settings",
    name: "settings",
    component: () => import("../views/Settings.vue"),
  },
];

export const router = createRouter({
  history: createWebHistory(),
  routes,
});
