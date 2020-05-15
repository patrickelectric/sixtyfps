import { getOptions } from 'loader-utils';
import * as webpack from 'webpack';
import * as crypto from 'crypto';
import * as child_process from 'child_process';
import * as fs from 'fs';
export const Greeter = (name: string) => `Hello ${name}`;
export default function loader(this: webpack.loader.LoaderContext, source: string): string {

    let compiler_path = __dirname + "/../../../../target/debug/sixtyfps-wasm-compiler";

    //const options = getOptions(this);
    console.log(`Compiling ${this.resourcePath} with ${compiler_path} to WASM`);
    console.log(`root context = ${this.rootContext}`);

    let build_dir = this.rootContext + "/wasm-build-cache";

    let input_dir = crypto.createHash("sha1").update(this.resourcePath).digest('hex');

    let wasm_output_dir = build_dir + "/" + input_dir;

    fs.mkdirSync(wasm_output_dir, { recursive: true });

    console.log(`Running compiler with output directory ${wasm_output_dir}`)
    let proc = child_process.spawnSync(compiler_path, [this.resourcePath, wasm_output_dir]);

    return `import {JSComponent} from '${wasm_output_dir + "/pkg/wasmwrapper.js"}';
    export default JSComponent;
`;
}
