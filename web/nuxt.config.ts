const baseURL = process.env.NUXT_APP_BASE_URL || "/";

export default defineNuxtConfig({
    ssr: false,
    devtools: { enabled: process.env.NODE_ENV !== "production" },

    modules: ["@nuxt/ui"],

    css: ["~/assets/css/main.css"],

    fonts: {
        provider: 'bunny'
    },

    app: {
        baseURL,
        head: {
            title: "Media Resolver",
            meta: [
                {
                    name: "description",
                    content: "Resolve supported links into usable resources."
                },
                { name: "color-scheme", content: "light dark" }
            ]
        }
    },

    runtimeConfig: {
        public: {
            buildHash: process.env.GITHUB_SHA?.slice(0, 7) || "development"
        }
    },

    typescript: {
        strict: true,
        typeCheck: false
    },

    hooks: {
        "prerender:routes"({ routes }) {
            routes.clear();
        }
    },

    compatibilityDate: "2026-08-01"
});
