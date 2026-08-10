const runtimeEnvironment = (
        globalThis as {
            process?: {
                env?: Record<string, string | undefined>;
            };
        }
).process?.env ?? {};

const buildHash = runtimeEnvironment.NUXT_PUBLIC_BUILD_HASH || "development";
const resolverEndpoint = runtimeEnvironment.NUXT_PUBLIC_RESOLVER_ENDPOINT || "";

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
            buildHash,
            resolverEndpoint
        }
    },

    typescript: {
        strict: true,
        typeCheck: false
    },

    compatibilityDate: "2026-08-01"
});
