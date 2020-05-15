import("../hello.60").then(module => {

    let UI = module.default;
    let instance = new UI();

    instance.run();

}).catch(console.error())
