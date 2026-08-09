export default defineNuxtConfig({
    ssr: false,
    devtools: { enabled: false },

    $development: {
        devtools: { enabled: true }
    },

    modules: ["@nuxt/ui"],

    css: ["~/assets/css/main.css"],

    fonts: {
        provider: "bunny"
    },

    app: {
        baseURL: "/",
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
            buildHash: "development",
            resolverEndpoint: ""
        }
    },

    typescript: {
        strict: true,
        typeCheck: false
    },

    compatibilityDate: "2026-08-01"
});
