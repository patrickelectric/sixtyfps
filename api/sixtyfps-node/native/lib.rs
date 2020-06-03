use neon::prelude::*;

struct WrappedComponentType(Option<std::rc::Rc<interpreter::ComponentDescription>>);
struct WrappedComponentBox(Option<corelib::abi::datastructures::ComponentBox>);

/// Load a .60 files.
///
/// The first argument of this finction is a string to the .60 file
///
/// The return value is a SixtyFpsComponentType
fn load(mut cx: FunctionContext) -> JsResult<JsValue> {
    let path = cx.argument::<JsString>(0)?.value();
    let path = std::path::Path::new(path.as_str());
    let source = std::fs::read_to_string(&path).or_else(|e| cx.throw_error(e.to_string()))?;
    let c = match interpreter::load(source.as_str(), &path) {
        Ok(c) => c,
        Err(diag) => {
            diag.print(source);
            return cx.throw_error("Compilation error");
        }
    };

    let mut obj = SixtyFpsComponentType::new::<_, JsValue, _>(&mut cx, std::iter::empty())?;
    cx.borrow_mut(&mut obj, |mut obj| obj.0 = Some(c));
    Ok(obj.as_value(&mut cx))
}

fn create<'cx>(
    cx: &mut CallContext<'cx, impl neon::object::This>,
    component_type: std::rc::Rc<interpreter::ComponentDescription>,
) -> JsResult<'cx, JsValue> {
    let component = component_type.clone().create();

    if let Some(args) = cx.argument_opt(0).and_then(|arg| arg.downcast::<JsObject>().ok()) {
        let properties = component_type.properties();
        for x in args.get_own_property_names(cx)?.to_vec(cx)? {
            let prop_name = x.to_string(cx)?.value();
            let ty = properties
                .get(&prop_name)
                .ok_or(())
                .or_else(|()| {
                    cx.throw_error(format!("Property {} not found in the component", prop_name))
                })?
                .clone();
            let value = args.get(cx, x)?;
            let value = to_eval_value(value, ty, cx)?;
            component_type
                .set_property(component.borrow(), prop_name.as_str(), value)
                .or_else(|_| cx.throw_error(format!("Cannot assign property")))?;
        }
    }

    let mut obj = SixtyFpsComponent::new::<_, JsValue, _>(cx, std::iter::empty())?;
    cx.borrow_mut(&mut obj, |mut obj| obj.0 = Some(component));
    Ok(obj.as_value(cx))
}

fn to_eval_value<'a>(
    val: Handle<JsValue>,
    ty: sixtyfps_compiler::typeregister::Type,
    cx: &mut impl Context<'a>,
) -> NeonResult<interpreter::Value> {
    use interpreter::Value;
    use sixtyfps_compiler::typeregister::Type;
    match ty {
        Type::Invalid | Type::Component(_) | Type::Builtin(_) | Type::Signal => {
            cx.throw_error("Cannot convert to a Sixtyfps property value")
        }
        Type::Float32 | Type::Int32 => {
            Ok(Value::Number(val.downcast_or_throw::<JsNumber, _>(cx)?.value()))
        }
        Type::String => Ok(Value::String(val.to_string(cx)?.value().as_str().into())),
        Type::Color => todo!(),
        Type::Image => todo!(),
        Type::Bool => Ok(Value::Bool(val.downcast_or_throw::<JsBoolean, _>(cx)?.value())),
    }
}

declare_types! {
    class SixtyFpsComponentType for WrappedComponentType {
        init(_) {
            Ok(WrappedComponentType(None))
        }
        method create(mut cx) {
            let this = cx.this();
            let ct = cx.borrow(&this, |x| x.0.clone());
            let ct = ct.ok_or(()).or_else(|()| cx.throw_error("Invalid type"))?;
            create(&mut cx, ct)
        }
        method name(mut cx) {
            let this = cx.this();
            let ct = cx.borrow(&this, |x| x.0.clone());
            let ct = ct.ok_or(()).or_else(|()| cx.throw_error("Invalid type"))?;
            Ok(cx.string(ct.id()).as_value(&mut cx))
        }
    }

    class SixtyFpsComponent for WrappedComponentBox {
        init(_) {
            Ok(WrappedComponentBox(None))
        }
        method show(mut cx) {
            let mut this = cx.this();
            // FIXME: is take() here the right choice?
            let component = cx.borrow_mut(&mut this, |mut x| x.0.take());
            let component = component.ok_or(()).or_else(|()| cx.throw_error("Invalid type"))?;
            gl::sixtyfps_runtime_run_component_with_gl_renderer(component.leak());
            // FIXME: leak (that's because we somehow need a static life time)
            Ok(JsUndefined::new().as_value(&mut cx))
        }
    }
}

register_module!(mut m, {
    m.export_function("load", load)?;
    Ok(())
});