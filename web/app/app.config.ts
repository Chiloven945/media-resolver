export default defineAppConfig({
    ui: {
        colors: {
            primary: "blue",
            secondary: "cyan",
            success: "emerald",
            info: "blue",
            warning: "amber",
            error: "red",
            neutral: "slate"
        },
        icons: {
            loading: "i-lucide-loader-circle",
            menu: "i-lucide-menu",
            close: "i-lucide-x"
        },
        button: {
            slots: {
                base: "min-h-11 font-medium lg:min-h-0"
            }
        },
        card: {
            slots: {
                root: "ring-1 ring-default/70 shadow-sm"
            }
        },
        input: {
            slots: {
                base: "transition-shadow"
            }
        },
        modal: {
            slots: {
                content: "ring-1 ring-default/80 shadow-2xl"
            }
        },
        drawer: {
            slots: {
                content: "ring-1 ring-default/80"
            }
        },
        dashboardPanel: {
            slots: {
                root: "bg-default/80 backdrop-blur-xl"
            }
        },
        dashboardNavbar: {
            slots: {
                root: "bg-default/75 backdrop-blur-xl border-default/70"
            }
        },
        badge: {
            slots: {
                base: "min-h-11 font-medium lg:min-h-0"
            }
        }
    }
});
