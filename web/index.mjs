import init, {} from "./wasm/rs_wgpu_cube.js";

let options = {
    cache: "no-cache",
    headers: {
        'Content-Type': 'text/plain'
    },
};
let a = await fetch("./shaders/vertex.wgsl", options);
let b = await fetch("./shaders/fragment.wgsl", options);
document.getElementById("s_vertex").innerHTML = await a.text();
document.getElementById("s_fragment").innerHTML = await b.text();

init().then(() => {
    console.log("WASM Loaded");
});