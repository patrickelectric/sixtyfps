import("../hello.60").then(module => console.log(`GOT: ${module.foo.foo}`)).catch(console.error())
