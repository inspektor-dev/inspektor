use anyhow::Error;
use wasmtime::{
    AsContextMut, Engine, Extern, Func, Instance, Memory, MemoryType, Module, Store, Trap,
    TypedFunc,
};

/// OpenPolicy is used to load the compiled wasm module in Rust runtime.
pub struct OpenPolicy {
    // global ctx of wasm policy. TODO: can be reused.
    engine: Engine,
    // loaded wasm module TODO: can be reused.
    module: Module,
    // wasm module with all the neccessary import.
    instance: Instance,
    // heap_ptr_addr hold the start addrs of the heap.
    heap_ptr_addr: i32,
    // runtime hold all current wasm opa runtime.
    runtime: OpaRuntime,
}

impl OpenPolicy {
    // from takes opa wasm as a input and returns the OpenPolicy. Throws error if it not able to set the
    // import or not able to get the export.
    pub fn new(wasm_byte_code: Vec<u8>) -> Result<OpenPolicy, Error> {
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
        let runtime = OpaRuntime::new(store, &instance, memory)?;

        Ok(OpenPolicy {
            engine: engine,
            module: module,
            instance: instance,
            runtime: runtime,
            heap_ptr_addr: 0,
        })
    }

    pub fn eval(&mut self, input: &[u8]) -> Result<Vec<u8>, Error> {
        // provide empty data.
        let data_addr = self.runtime.write_json(b"{}")?;
        let heap_ptr = self.runtime.get_current_heap_ptr()?;
        // set the json input the into opa wasm runtime.
        let input_addr = heap_ptr;
        let heap_ptr = self.runtime.direct_write_str(input, heap_ptr)?;
        self.runtime.eval(data_addr, input_addr, input.len() as i32, heap_ptr)
    }
}

// get_opa_imports returns all the imports are required by the opa wasm policy
fn get_opa_imports<S: AsContextMut>(mut store: S, memory: Memory) -> [Extern; 7] {
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
            |id: i32, _ctx: i32, _a: i32, _b: i32, _c: i32, _d: i32| -> Result<i32, Trap> {
                return Err(Trap::new(format!(
                    "builtin fuction for id {:?} is not implemented",
                    id
                )));
            },
        )),
        // respective import function: opa_builtin4
        Extern::Func(Func::wrap(&mut store, |_addr: i32| -> Result<(), Trap> {
            println!("abort called");
            return Err(Trap::new(
                format!("print and abort function not implement",),
            ));
        })),
        // shared memory between opa webassembly and rust.
        Extern::Memory(memory),
    ];
    return imports;
}

// OpaExporFn contains all the required exported function from opa wasm build.
struct OpaRuntime {
    eval: TypedFunc<(i32, i32, i32, i32, i32, i32, i32), i32>,
    opa_malloc: TypedFunc<i32, i32>,
    json_parse: TypedFunc<(i32, i32), i32>,
    free: TypedFunc<i32, ()>,
    get_heap_ptr: TypedFunc<(), i32>,
    set_heap_ptr: TypedFunc<i32, ()>,
    store: Store<()>,
    memory: Memory,
}

impl OpaRuntime {
    fn new(
        mut store: Store<()>,
        instance: &Instance,
        memory: Memory,
    ) -> Result<OpaRuntime, anyhow::Error> {
        // one shot opa policy execution
        let eval = instance.get_typed_func::<(i32, i32, i32, i32, i32, i32, i32), i32, _>(
            &mut store, "opa_eval",
        )?;
        // malloc will allocate required memory for the given size and return the starting address of the allocated object
        let malloc = instance.get_typed_func::<i32, i32, _>(&mut store, "opa_malloc")?;
        // json_parse will parse the json str in wasm memory and gives the address of the serialized json object.
        let json_parse =
            instance.get_typed_func::<(i32, i32), i32, _>(&mut store, "opa_json_parse")?;
        // free will de-allocate the allocated object. inverse of malloc.
        let free = instance.get_typed_func::<i32, (), _>(&mut store, "opa_free")?;
        // heap_ptr_get will give the current heap ptr of the execution
        let heap_ptr_get = instance.get_typed_func::<(), i32, _>(&mut store, "opa_heap_ptr_get")?;
        // heap_ptr_get is used to set the heap ptr. from what point we want to resume the execution.
        let heap_ptr_set = instance.get_typed_func::<i32, (), _>(&mut store, "opa_heap_ptr_set")?;
        Ok(OpaRuntime {
            eval: eval,
            opa_malloc: malloc,
            json_parse: json_parse,
            free: free,
            get_heap_ptr: heap_ptr_get,
            set_heap_ptr: heap_ptr_set,
            store: store,
            memory: memory,
        })
    }

    fn write_json(&mut self, val: &[u8]) -> Result<i32, Error> {
        let addr = self.write_str(val)?;
        let mut store = self.store.as_context_mut();
        // now parse the string json to opa internal serialized json.
        let input_val_addr = self
            .json_parse
            .call(&mut store, (addr , (val.len()) as i32))?;
        Ok(input_val_addr)
    }

    fn write_str(&mut self, val: &[u8]) -> Result<i32, Error> {
        let mut store = self.store.as_context_mut();
        // reserve extra one byte for the null termination.
        let addr = self.opa_malloc.call(&mut store, (val.len() ) as i32)? as usize;

        self.memory.write(&mut store, addr, val)?;
      //  self.memory.write(&mut store, addr + val.len() + 1, &[0])?;
        Ok(addr as i32)
    }

    fn direct_write_str(&mut self, val: &[u8], offset: i32) -> Result<i32, Error> {
        let mut store = self.store.as_context_mut();
        self.memory.write(&mut store, offset as usize, val)?;
        self.memory
            .write(&mut store, (offset as usize) + val.len() + 1, &[0])?;
        Ok(offset + val.len() as i32 )
    }

    fn eval(
        &mut self,
        data_addr: i32,
        input_str_addr: i32,
        input_len: i32,
        heap_ptr: i32
    ) -> Result<Vec<u8>, Error> {
        println!("eval heap ptr {:?}", heap_ptr);
        let mut store = self.store.as_context_mut();
        let output_addr = self.eval.call(
            &mut store,
            (0, 0, data_addr, input_str_addr, input_len , heap_ptr, 0),
        )?;
        // let's read the data from the output addrs
        let data = &self.memory.data(&store)[output_addr as usize..];
        // output is string so size can be determined by the null termination.
        let len = data.iter().position(|b| *b == 0);
        if len.is_none() {
            return Err(anyhow::anyhow!("invalid output data"));
        }
        let output = data[..len.unwrap()].to_vec();
        Ok(output)
    }

    fn get_current_heap_ptr(&mut self) -> Result<i32, Error> {
        let mut store = self.store.as_context_mut();
        let ptr = self.get_heap_ptr.call(&mut store, ())?;
        Ok(ptr)
    }
}
