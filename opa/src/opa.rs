use anyhow::Error;
use std::ffi::{CStr, VaList};
use std::fmt;
use std::ops;
use wasmtime::{ Engine, Extern, Func, Instance, Memory, MemoryType, Module, Store, Trap, TypedFunc, AsContextMut};


/// OpenPolicy is used to load the compiled wasm module in Rust runtime. 
pub struct OpenPolicy {
    // shared memory between opa wasm and rust runtime.
    memory: Memory,
    // global ctx of wasm policy. TODO: can be reused. 
    engine: Engine,
    // store is the state of the current execution.
    store: Store<()>,
    // loaded wasm module TODO: can be reused.
    module: Module,
    // wasm module with all the neccessary import.
    instance: Instance,
    // expored wasm function from the policy.
    exported_fn: OpaExportFn
}

impl OpenPolicy {
    // from takes opa wasm as a input and returns the OpenPolicy. Throws error if it not able to set the
    // import or not able to get the export.
    pub fn from(wasm_byte_code: Vec<u8>) -> Result<OpenPolicy, Error> {
        let engine = Engine::default();
        let mut store = Store::new(&engine, ());
        // minimum memory is set to 5. Because it same as c example on the opa document.
        let memory_type = MemoryType::new(5, None);
        let memory = Memory::new(&mut store, memory_type)?;
        // imports required by the opa wasm policy
        let imports = get_opa_imports(&mut store, memory.clone());
        let module = Module::from_binary(&engine, &wasm_byte_code[..])?;
        let instance = Instance::new(&mut store, &module, &imports)?;
        // get all the exported fuction from the policy
        let exported_fn = OpaExportFn::new(&mut store, &instance)?;
        Ok(OpenPolicy {
            memory: memory,
            engine: engine,
            store: store,
            module: module,
            instance: instance,
            exported_fn: exported_fn
        })
    }
}

// get_opa_imports returns all the imports are required by the opa wasm policy
fn get_opa_imports<S: AsContextMut>(mut store: S, memory: Memory) ->[Extern;7]{
    let imports = [
        // respective import function: opa_abort, opa_println
            Extern::Func(Func::wrap(
                &mut store,
                |id: i32, _ctx: i32| -> Result<i32, Trap> {
                    return Err(Trap::new(format!(
                        "builtin fuction for id {:?} is not implemented",
                        id
                    )));
                },
            )),
            // respective import function: opa_builtin0	
            Extern::Func(Func::wrap(
                &mut store,
                |id: i32, _ctx: i32, _a: i32| -> Result<i32, Trap> {
                    return Err(Trap::new(format!(
                        "builtin fuction for id {:?} is not implemented",
                        id
                    )));
                },
            )),
            // respective import function: opa_builtin1
            Extern::Func(Func::wrap(
                &mut store,
                |id: i32, _ctx: i32, _a: i32, _b: i32| -> Result<i32, Trap> {
                    return Err(Trap::new(format!(
                        "builtin fuction for id {:?} is not implemented",
                        id
                    )));
                },
            )),
            // respective import function: opa_builtin2
            Extern::Func(Func::wrap(
                &mut store,
                |id: i32, _ctx: i32, _a: i32, _b: i32, _c: i32| -> Result<i32, Trap> {
                    return Err(Trap::new(format!(
                        "builtin fuction for id {:?} is not implemented",
                        id
                    )));
                },
            )),
            // respective import function: opa_builtin3	
            Extern::Func(Func::wrap(
                &mut store,
                |id: i32, _ctx: i32, _a: i32, _b: i32, _c: i32,_d: i32| -> Result<i32, Trap> {
                    return Err(Trap::new(format!(
                        "builtin fuction for id {:?} is not implemented",
                        id
                    )));
                },
            )),
            // respective import function: opa_builtin4
            Extern::Func(Func::wrap(&mut store, | _addr: i32| -> Result<i32, Trap> {
                return Err(Trap::new(format!(
                    "print and abort function not implement",
                )));
            })),
            // shared memory between opa webassembly and rust.
            Extern::Memory(memory),
        ];
        return imports
}

// OpaExporFn contains all the required exported function from opa wasm build.
struct OpaExportFn {
    eval: TypedFunc<(i32, i32, i32, i32, i32, i32,i32), i32>,
    malloc: TypedFunc<i32, i32>,
    json_parse: TypedFunc<(i32, i32), i32>,
    free: TypedFunc<i32, ()>,
    get_heap_ptr: TypedFunc<(), i32>,
    set_heap_ptr: TypedFunc<i32, ()>
}

impl OpaExportFn {
    fn new<S: AsContextMut>(mut store:S, instance:&Instance) -> Result<OpaExportFn, anyhow::Error>{
        // one shot opa policy execution
        let eval = instance.get_typed_func::<(i32, i32, i32, i32, i32, i32,i32), i32, _>(&mut store, "opa_eval")?;
        // malloc will allocate required memory for the given size and return the starting address of the allocated object
        let malloc = instance.get_typed_func::<i32, i32, _>(&mut store, "opa_malloc")?;
        // json_parse will parse the json str in wasm memory and gives the address of the serialized json object.
        let json_parse = instance.get_typed_func::<(i32, i32), i32, _>(&mut store, "opa_json_parse")?;
        // free will de-allocate the allocated object. inverse of malloc.
        let free = instance.get_typed_func::<i32, (), _>(&mut store, "opa_free")?;
        // heap_ptr_get will give the current heap ptr of the execution
        let heap_ptr_get = instance.get_typed_func::<(), i32, _>(&mut store, "opa_heap_ptr_get")?;
        // heap_ptr_get is used to set the heap ptr. from what point we want to resume the execution.
        let heap_ptr_set = instance.get_typed_func::<i32,(), _>(&mut store, "opa_heap_ptr_set")?;
        Ok(OpaExportFn{
            eval: eval,
            malloc: malloc,
            json_parse: json_parse,
            free: free,
            get_heap_ptr: heap_ptr_get,
            set_heap_ptr: heap_ptr_set,
        })
    }
}