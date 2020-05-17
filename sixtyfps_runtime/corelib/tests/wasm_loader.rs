extern crate wasm_bindgen_test;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

use parity_wasm::builder;
use parity_wasm::elements;
use parity_wasm::elements::Instruction;

use js_sys::{Object, Reflect, WebAssembly};

use wasm_bindgen_futures::{spawn_local, JsFuture};

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

fn create_function() -> builder::FunctionDefinition {
    let fun = builder::FunctionBuilder::new()
        .main()
        .signature()
        .build()
        .body()
        .with_instructions(elements::Instructions::new(vec![
            Instruction::GetGlobal(0),
            Instruction::I32Const(50),
            Instruction::I32Add,
            Instruction::SetGlobal(1),
            Instruction::End,
        ]))
        .build()
        .build();
    fun
}

fn build_module() -> Vec<u8> {
    let mut module = builder::module()
        .import()
        .module("env")
        .field("my_import")
        .external()
        .global(elements::ValueType::I32, false)
        .build()
        .global()
        .mutable()
        .value_type()
        .i32()
        .init_expr(Instruction::GetGlobal(0))
        .build()
        .export()
        .field("my_value")
        .internal()
        .global(1)
        .build();

    module.push_function(create_function());

    // And finally we finish our module builder to produce actual
    // wasm module.
    let module = module.build();

    parity_wasm::serialize(module).unwrap()
}

async fn test_loader_async() -> Result<(), JsValue> {
    let module_buffer = build_module();

    let import = Object::new();
    let env = Object::new();
    Reflect::set(&env, &"my_import".into(), &JsValue::from(100i32))?;
    Reflect::set(&import, &"env".into(), &env)?;

    let instantiation_value =
        JsFuture::from(WebAssembly::instantiate_buffer(module_buffer.as_slice(), &import)).await?;
    console_log!("here");

    let instance: WebAssembly::Instance =
        Reflect::get(&instantiation_value, &"instance".into())?.dyn_into()?;

    let exports = instance.exports();

    let js_global = Reflect::get(&exports, &"my_value".into())?;
    let glob_v = Reflect::get(&js_global, &"value".into())?;

    assert_eq!(glob_v, 42i32);
    Ok(())
}

#[wasm_bindgen_test]
fn test_loader() {
    spawn_local(async {
        test_loader_async().await.unwrap();
    });
}
