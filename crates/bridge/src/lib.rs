use std::fmt::Write;
use wit_bindgen_core::{
    abi::{AbiVariant, WasmType},
    make_external_symbol, uwriteln,
    wit_parser::{self, Function, Resolve, TypeOwner, WorldId, WorldKey},
    Source, WorldGenerator,
};

#[derive(Default)]
struct Bridge {
    src: Source,
    opts: Opts,
}

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "clap", derive(clap::Args))]
pub struct Opts {
    /// Output bridge code for webassembly micro runtime
    #[cfg_attr(feature = "clap", arg(long))]
    wamr: bool,
    /// w2c2 Instance name (derived from wasm file)
    #[cfg_attr(feature = "clap", arg(long, default_value_t = String::default()))]
    instance: String,
    /// w2c2 Include name
    #[cfg_attr(feature = "clap", arg(long, default_value_t = String::default()))]
    include: String,
}

impl Opts {
    pub fn build(&self) -> Box<dyn WorldGenerator> {
        let mut r = Bridge::default();
        r.opts = self.clone();
        Box::new(r)
    }
}

impl WorldGenerator for Bridge {
    fn preprocess(&mut self, resolve: &Resolve, world: WorldId) {
        let world = &resolve.worlds[world];
        let name = if self.opts.instance.is_empty() {
            world.name.clone()
        } else {
            self.opts.instance.clone()
        };
        let include = if self.opts.include.is_empty() {
            name.clone() + ".h"
        } else {
            self.opts.include.clone()
        };
        uwriteln!(
            self.src,
            r#"
        #include <stdint.h>
        #include <stdio.h>
        #include "{include}"
        
        static {name}Instance* instance;
        static {name}Instance app_instance;
        
        void trap(Trap trap) {{
            abort();
        }}
        
        {name}Instance* get_app() {{
            if (!instance) {{
                {name}Instantiate(&app_instance, NULL);
                instance = &app_instance;
            }}
            return instance;
        }}
        "#
        );
    }

    fn import_interface(
        &mut self,
        resolve: &wit_parser::Resolve,
        name: &WorldKey,
        iface: wit_parser::InterfaceId,
        _files: &mut wit_bindgen_core::Files,
    ) -> anyhow::Result<()> {
        let world = match name {
            WorldKey::Name(n) => n.clone(),
            WorldKey::Interface(i) => resolve.interfaces[*i].name.clone().unwrap_or_default(),
        };
        uwriteln!(self.src, "// Import IF {world}");

        let mut gen = self.interface(resolve);
        for (_name, func) in resolve.interfaces[iface].functions.iter() {
            gen.generate_function(func, &TypeOwner::Interface(iface), AbiVariant::GuestImport);
        }
        Ok(())
    }

    fn export_interface(
        &mut self,
        resolve: &wit_parser::Resolve,
        name: &WorldKey,
        iface: wit_parser::InterfaceId,
        _files: &mut wit_bindgen_core::Files,
    ) -> anyhow::Result<()> {
        let world = match name {
            WorldKey::Name(n) => n.clone(),
            WorldKey::Interface(i) => resolve.interfaces[*i].name.clone().unwrap_or_default(),
        };
        uwriteln!(self.src, "// Export IF {world}");

        let mut gen = self.interface(resolve);
        for (_name, func) in resolve.interfaces[iface].functions.iter() {
            gen.generate_function(func, &TypeOwner::Interface(iface), AbiVariant::GuestExport);
        }
        Ok(())
    }

    fn import_funcs(
        &mut self,
        resolve: &wit_parser::Resolve,
        worldid: wit_parser::WorldId,
        funcs: &[(&str, &wit_parser::Function)],
        _files: &mut wit_bindgen_core::Files,
    ) {
        let world = &resolve.worlds[worldid];
        uwriteln!(self.src, "// Import Funcs {}", world.name);
        let mut gen = self.interface(resolve);
        for (_name, func) in funcs.iter() {
            gen.generate_function(func, &TypeOwner::World(worldid), AbiVariant::GuestImport);
        }
    }

    fn export_funcs(
        &mut self,
        resolve: &wit_parser::Resolve,
        worldid: wit_parser::WorldId,
        funcs: &[(&str, &wit_parser::Function)],
        _files: &mut wit_bindgen_core::Files,
    ) -> anyhow::Result<()> {
        let world = &resolve.worlds[worldid];
        uwriteln!(self.src, "// Export Funcs {}", world.name);
        let mut gen = self.interface(resolve);
        for (_name, func) in funcs.iter() {
            gen.generate_function(func, &TypeOwner::World(worldid), AbiVariant::GuestExport);
        }
        Ok(())
    }

    fn import_types(
        &mut self,
        resolve: &wit_parser::Resolve,
        world: wit_parser::WorldId,
        _types: &[(&str, wit_parser::TypeId)],
        _files: &mut wit_bindgen_core::Files,
    ) {
        let world = &resolve.worlds[world];
        uwriteln!(self.src, "// Import Types {}", world.name);
    }

    fn finish(
        &mut self,
        resolve: &wit_parser::Resolve,
        world: wit_parser::WorldId,
        files: &mut wit_bindgen_core::Files,
    ) -> anyhow::Result<()> {
        let world = &resolve.worlds[world];
        files.push(&format!("{}_bridge.c", world.name), self.src.as_bytes());
        Ok(())
    }
}

impl Bridge {
    fn interface<'a>(&'a mut self, resolve: &'a Resolve) -> BridgeInterfaceGenerator<'a> {
        BridgeInterfaceGenerator { gen: self, resolve }
    }

    fn wasm_type(&self, ty: WasmType, _var: TypeVariant) -> String {
        match ty {
            WasmType::I32 => todo!(),
            WasmType::I64 => todo!(),
            WasmType::F32 => todo!(),
            WasmType::F64 => todo!(),
            WasmType::Pointer => todo!(),
            WasmType::PointerOrI64 => todo!(),
            WasmType::Length => todo!(),
        }
    }

    fn func_name(
        &self,
        resolve: &Resolve,
        func: &Function,
        owner: &TypeOwner,
        variant: AbiVariant,
    ) -> String {
        let module_name = match owner {
            TypeOwner::World(_) => todo!(),
            TypeOwner::Interface(i) => resolve.interfaces[*i].name.clone().unwrap_or_default(),
            TypeOwner::None => todo!(),
        };
        make_external_symbol(&module_name, &func.name, variant)
    }
}

struct BridgeInterfaceGenerator<'a> {
    gen: &'a mut Bridge,
    resolve: &'a Resolve,
}

enum TypeVariant {
    W2C2,
    Native,
}

impl<'a> BridgeInterfaceGenerator<'a> {
    fn generate_function(&mut self, func: &Function, owner: &TypeOwner, variant: AbiVariant) {
        uwriteln!(self.gen.src, "// Func {} {:?}", func.name, variant);
        let result_var = match variant {
            AbiVariant::GuestImport => TypeVariant::W2C2,
            AbiVariant::GuestExport => TypeVariant::Native,
            AbiVariant::GuestImportAsync => todo!(),
            AbiVariant::GuestExportAsync => todo!(),
            AbiVariant::GuestExportAsyncStackful => todo!(),
        };
        let signature = self.resolve.wasm_signature(variant, func);
        let return_via_pointer = signature.retptr;
        let is_export = matches!(variant, AbiVariant::GuestExport);
        if is_export {
            self.gen
                .src
                .push_str(r#"__attribute__ ((visibility ("default"))) "#);
        }
        let res = if signature.results.is_empty() || return_via_pointer {
            "void".into()
        } else {
            self.gen.wasm_type(signature.results[0], result_var)
        };
        self.gen.src.push_str(&res);
        self.gen.src.push_str(" ");
        let fname = self.gen.func_name(self.resolve, func, owner, variant);
        self.gen.src.push_str(&fname);
    }
}
