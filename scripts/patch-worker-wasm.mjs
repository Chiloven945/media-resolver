import { writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const output = resolve(process.argv[2] || "worker/pkg/engine.js");
const source = `import * as imports from "./engine_bg.js";
import wkmod from "./engine_bg.wasm";
import * as nodemod from "./engine_bg.wasm";

if (typeof process !== "undefined" && process.release?.name === "node") {
    imports.__wbg_set_wasm(nodemod);
} else {
    const instance = new WebAssembly.Instance(wkmod, {
        "./engine_bg.js": imports
    });
    imports.__wbg_set_wasm(instance.exports);
}

export * from "./engine_bg.js";
`;

await writeFile(output, source, "utf8");
console.log(`Patched wasm-bindgen Worker glue: ${output}`);
