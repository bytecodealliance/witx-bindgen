use anyhow::Result;
use heck::{ToLowerCamelCase, ToShoutySnakeCase, ToUpperCamelCase};
use indexmap::IndexMap;
use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
    iter, mem,
    ops::Deref,
};
use wit_bindgen_core::{
    abi::{self, AbiVariant, Bindgen, Bitcast, Instruction, LiftLower, WasmType},
    wit_parser::LiveTypes,
    Direction,
};
use wit_bindgen_core::{
    uwrite, uwriteln,
    wit_parser::{
        Docs, Enum, Flags, FlagsRepr, Function, FunctionKind, Handle, Int, InterfaceId, Record,
        Resolve, Result_, SizeAlign, Tuple, Type, TypeDefKind, TypeId, TypeOwner, Variant, WorldId,
        WorldKey,
    },
    Files, InterfaceGenerator as _, Ns, WorldGenerator,
};
use wit_component::{StringEncoding, WitPrinter};
use wit_parser::{Alignment, ArchitectureSize};
mod csproj;
pub use csproj::CSProject;

//TODO remove unused
const CSHARP_IMPORTS: &str = "\
using System;
using System.Runtime.CompilerServices;
using System.Collections;
using System.Runtime.InteropServices;
using System.Text;
using System.Collections.Generic;
using System.Diagnostics;
using System.Diagnostics.CodeAnalysis;
";

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "clap", derive(clap::Args))]
pub struct Opts {
    #[cfg_attr(feature = "clap", arg(long, default_value_t = StringEncoding::default()))]
    pub string_encoding: StringEncoding,

    /// Whether or not to generate a stub class for exported functions
    #[cfg_attr(feature = "clap", arg(long))]
    pub generate_stub: bool,

    // TODO: This should only temporarily needed until mono and native aot aligns.
    #[cfg_attr(feature = "clap", arg(short, long, value_enum))]
    pub runtime: CSharpRuntime,

    /// Use the `internal` access modifier by default instead of `public`
    #[cfg_attr(feature = "clap", arg(long))]
    pub internal: bool,

    /// Skip generating `cabi_realloc`, `WasmImportLinkageAttribute`, and component type files
    #[cfg_attr(feature = "clap", arg(long))]
    pub skip_support_files: bool,
}

impl Opts {
    pub fn build(&self) -> Box<dyn WorldGenerator> {
        Box::new(CSharp {
            opts: self.clone(),
            ..CSharp::default()
        })
    }
}

#[derive(Clone)]
struct ResourceInfo {
    module: String,
    name: String,
    docs: Docs,
    direction: Direction,
}

impl ResourceInfo {
    /// Returns the name of the exported implementation of this resource.
    ///
    /// The result is only valid if the resource is actually being exported by the world.
    fn export_impl_name(&self) -> String {
        format!(
            "{}Impl.{}",
            CSharp::get_class_name_from_qualified_name(&self.module)
                .1
                .strip_prefix("I")
                .unwrap()
                .to_upper_camel_case(),
            self.name.to_upper_camel_case()
        )
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum CSharpRuntime {
    #[default]
    NativeAOT,
    Mono,
}

struct InterfaceFragment {
    csharp_src: String,
    csharp_interop_src: String,
    stub: String,
}

pub struct InterfaceTypeAndFragments {
    is_export: bool,
    interface_fragments: Vec<InterfaceFragment>,
}

impl InterfaceTypeAndFragments {
    pub fn new(is_export: bool) -> Self {
        InterfaceTypeAndFragments {
            is_export: is_export,
            interface_fragments: Vec::<InterfaceFragment>::new(),
        }
    }
}

/// Indicates if we are generating for functions in an interface or free standing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionLevel {
    Interface,
    FreeStanding,
}

#[derive(Default)]
pub struct CSharp {
    opts: Opts,
    name: String,
    return_area_size: ArchitectureSize,
    return_area_align: Alignment,
    tuple_counts: HashSet<usize>,
    needs_result: bool,
    needs_option: bool,
    needs_interop_string: bool,
    needs_export_return_area: bool,
    needs_rep_table: bool,
    needs_wit_exception: bool,
    interface_fragments: HashMap<String, InterfaceTypeAndFragments>,
    world_fragments: Vec<InterfaceFragment>,
    sizes: SizeAlign,
    interface_names: HashMap<InterfaceId, String>,
    anonymous_type_owners: HashMap<TypeId, TypeOwner>,
    all_resources: HashMap<TypeId, ResourceInfo>,
    world_resources: HashMap<TypeId, ResourceInfo>,
    import_funcs_called: bool,
}

impl CSharp {
    fn access_modifier(&self) -> &'static str {
        if self.opts.internal {
            "internal"
        } else {
            "public"
        }
    }

    fn qualifier(&self) -> String {
        let world = self.name.to_upper_camel_case();
        format!("{world}World.")
    }

    fn interface<'a>(
        &'a mut self,
        resolve: &'a Resolve,
        name: &'a str,
        direction: Direction,
    ) -> InterfaceGenerator<'a> {
        InterfaceGenerator {
            src: String::new(),
            csharp_interop_src: String::new(),
            stub: String::new(),
            gen: self,
            resolve,
            name,
            direction,
        }
    }

    // returns the qualifier and last part
    fn get_class_name_from_qualified_name(qualified_type: &str) -> (String, String) {
        let parts: Vec<&str> = qualified_type.split('.').collect();
        if let Some(last_part) = parts.last() {
            let mut qualifier = qualified_type.strip_suffix(last_part);
            if qualifier.is_some() {
                qualifier = qualifier.unwrap().strip_suffix(".");
            }
            (qualifier.unwrap_or("").to_string(), last_part.to_string())
        } else {
            (String::new(), String::new())
        }
    }
}

impl WorldGenerator for CSharp {
    fn preprocess(&mut self, resolve: &Resolve, world: WorldId) {
        let name = &resolve.worlds[world].name;
        self.name = name.to_string();
        self.sizes.fill(resolve);
    }

    fn import_interface(
        &mut self,
        resolve: &Resolve,
        key: &WorldKey,
        id: InterfaceId,
        _files: &mut Files,
    ) -> Result<()> {
        let name = interface_name(self, resolve, key, Direction::Import);
        self.interface_names.insert(id, name.clone());
        let mut gen = self.interface(resolve, &name, Direction::Import);

        let mut old_resources = mem::take(&mut gen.gen.all_resources);
        gen.types(id);
        let new_resources = mem::take(&mut gen.gen.all_resources);
        old_resources.extend(new_resources.clone());
        gen.gen.all_resources = old_resources;

        for (resource, funcs) in by_resource(
            resolve.interfaces[id]
                .functions
                .iter()
                .map(|(k, v)| (k.as_str(), v)),
            new_resources.keys().copied(),
        ) {
            if let Some(resource) = resource {
                gen.start_resource(resource, Some(key));
            }

            let import_module_name = &resolve.name_world_key(key);
            for func in funcs {
                gen.import(import_module_name, func);
            }

            if resource.is_some() {
                gen.end_resource();
            }
        }

        // for anonymous types
        gen.define_interface_types(id);

        gen.add_interface_fragment(false);

        Ok(())
    }

    fn import_funcs(
        &mut self,
        resolve: &Resolve,
        world: WorldId,
        funcs: &[(&str, &Function)],
        _files: &mut Files,
    ) {
        self.import_funcs_called = true;

        let name = &format!("{}-world", resolve.worlds[world].name).to_upper_camel_case();
        let name = &format!("{name}.I{name}");
        let mut gen = self.interface(resolve, name, Direction::Import);

        for (resource, funcs) in by_resource(
            funcs.iter().copied(),
            gen.gen.world_resources.keys().copied(),
        ) {
            if let Some(resource) = resource {
                gen.start_resource(resource, None);
            }

            for func in funcs {
                gen.import("$root", func);
            }

            if resource.is_some() {
                gen.end_resource();
            }
        }

        gen.add_world_fragment();
    }

    fn export_interface(
        &mut self,
        resolve: &Resolve,
        key: &WorldKey,
        id: InterfaceId,
        _files: &mut Files,
    ) -> Result<()> {
        let name = interface_name(self, resolve, key, Direction::Export);
        self.interface_names.insert(id, name.clone());
        let mut gen = self.interface(resolve, &name, Direction::Export);

        let mut old_resources = mem::take(&mut gen.gen.all_resources);
        gen.types(id);
        let new_resources = mem::take(&mut gen.gen.all_resources);
        old_resources.extend(new_resources.clone());
        gen.gen.all_resources = old_resources;

        for (resource, funcs) in by_resource(
            resolve.interfaces[id]
                .functions
                .iter()
                .map(|(k, v)| (k.as_str(), v)),
            new_resources.keys().copied(),
        ) {
            if let Some(resource) = resource {
                gen.start_resource(resource, Some(key));
            }

            for func in funcs {
                gen.export(func, Some(key));
            }

            if resource.is_some() {
                gen.end_resource();
            }
        }

        // for anonymous types
        gen.define_interface_types(id);

        gen.add_interface_fragment(true);
        Ok(())
    }

    fn export_funcs(
        &mut self,
        resolve: &Resolve,
        world: WorldId,
        funcs: &[(&str, &Function)],
        _files: &mut Files,
    ) -> Result<()> {
        let name = &format!("{}-world", resolve.worlds[world].name).to_upper_camel_case();
        let name = &format!("{name}.I{name}");
        let mut gen = self.interface(resolve, name, Direction::Export);

        for (resource, funcs) in by_resource(funcs.iter().copied(), iter::empty()) {
            if let Some(resource) = resource {
                gen.start_resource(resource, None);
            }

            for func in funcs {
                gen.export(func, None);
            }

            if resource.is_some() {
                gen.end_resource();
            }
        }

        gen.add_world_fragment();
        Ok(())
    }

    fn import_types(
        &mut self,
        resolve: &Resolve,
        world: WorldId,
        types: &[(&str, TypeId)],
        _files: &mut Files,
    ) {
        let name = &format!("{}-world", resolve.worlds[world].name).to_upper_camel_case();
        let name = &format!("{name}.I{name}");
        let mut gen = self.interface(resolve, name, Direction::Import);

        let mut old_resources = mem::take(&mut gen.gen.all_resources);
        for (ty_name, ty) in types {
            gen.define_type(ty_name, *ty);
        }
        let new_resources = mem::take(&mut gen.gen.all_resources);
        old_resources.extend(new_resources.clone());
        gen.gen.all_resources = old_resources;
        gen.gen.world_resources = new_resources;

        gen.add_world_fragment();
    }

    fn finish(&mut self, resolve: &Resolve, id: WorldId, files: &mut Files) -> Result<()> {
        if !self.import_funcs_called {
            // Ensure that we emit type declarations for any top-level imported resource types:
            self.import_funcs(resolve, id, &[], files);
        }

        let world = &resolve.worlds[id];
        let world_namespace = self.qualifier();
        let world_namespace = world_namespace.strip_suffix(".").unwrap();
        let namespace = format!("{world_namespace}");
        let name = world.name.to_upper_camel_case();

        let version = env!("CARGO_PKG_VERSION");
        let header = format!(
            "\
            // Generated by `wit-bindgen` {version}. DO NOT EDIT!
            // <auto-generated />
            #nullable enable
            "
        );
        let mut src = String::new();
        src.push_str(&header);

        let access = self.access_modifier();

        uwrite!(
            src,
            "{CSHARP_IMPORTS}

             namespace {world_namespace} {{

             {access} interface I{name}World {{
            "
        );

        src.push_str(
            &self
                .world_fragments
                .iter()
                .map(|f| f.csharp_src.deref())
                .collect::<Vec<_>>()
                .join("\n"),
        );

        let mut producers = wasm_metadata::Producers::empty();
        producers.add(
            "processed-by",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
        );

        src.push_str("}\n");

        if self.needs_result {
            uwrite!(
                src,
                r#"

                {access} readonly struct None {{}}

                [StructLayout(LayoutKind.Sequential)]
                {access} readonly struct Result<Ok, Err>
                {{
                    {access} readonly byte Tag;
                    private readonly object value;

                    private Result(byte tag, object value)
                    {{
                        Tag = tag;
                        this.value = value;
                    }}

                    {access} static Result<Ok, Err> ok(Ok ok)
                    {{
                        return new Result<Ok, Err>(OK, ok!);
                    }}

                    {access} static Result<Ok, Err> err(Err err)
                    {{
                        return new Result<Ok, Err>(ERR, err!);
                    }}

                    {access} bool IsOk => Tag == OK;
                    {access} bool IsErr => Tag == ERR;

                    {access} Ok AsOk
                    {{
                        get
                        {{
                            if (Tag == OK)
                                return (Ok)value;
                            else
                                throw new ArgumentException("expected OK, got " + Tag);
                        }}
                    }}

                    {access} Err AsErr
                    {{
                        get
                        {{
                            if (Tag == ERR)
                                return (Err)value;
                            else
                                throw new ArgumentException("expected ERR, got " + Tag);
                        }}
                    }}

                    {access} const byte OK = 0;
                    {access} const byte ERR = 1;
                }}
                "#,
            )
        }

        if self.needs_option {
            uwrite!(
                src,
                r#"

                {access} class Option<T> {{
                    private static Option<T> none = new ();

                    private Option()
                    {{
                        HasValue = false;
                    }}

                    {access} Option(T v)
                    {{
                        HasValue = true;
                        Value = v;
                    }}

                    {access} static Option<T> None => none;

                    [MemberNotNullWhen(true, nameof(Value))]
                    {access} bool HasValue {{ get; }}

                    {access} T? Value {{ get; }}
                }}
                "#,
            )
        }

        if self.needs_interop_string {
            uwrite!(
                src,
                r#"
                {access} static class InteropString
                {{
                    internal static IntPtr FromString(string input, out int length)
                    {{
                        var utf8Bytes = Encoding.UTF8.GetBytes(input);
                        length = utf8Bytes.Length;
                        var gcHandle = GCHandle.Alloc(utf8Bytes, GCHandleType.Pinned);
                        return gcHandle.AddrOfPinnedObject();
                    }}
                }}
                "#,
            )
        }

        if self.needs_wit_exception {
            uwrite!(
                src,
                r#"
                {access} class WitException: Exception {{
                    {access} object Value {{ get; }}
                    {access} uint NestingLevel {{ get; }}

                    {access} WitException(object v, uint level)
                    {{
                        Value = v;
                        NestingLevel = level;
                    }}
                }}
                "#,
            )
        }

        // Declare a statically-allocated return area, if needed. We only do
        // this for export bindings, because import bindings allocate their
        // return-area on the stack.
        if self.needs_export_return_area {
            let mut ret_area_str = String::new();

            let (array_size, element_type) =
                dotnet_aligned_array(self.return_area_size, self.return_area_align);
            uwrite!(
                ret_area_str,
                "
                {access} static class InteropReturnArea
                {{
                    [InlineArray({0})]
                    [StructLayout(LayoutKind.Sequential, Pack = {1})]
                    internal struct ReturnArea
                    {{
                        private {2} buffer;

                        internal unsafe nint AddressOfReturnArea()
                        {{
                            return (nint)Unsafe.AsPointer(ref buffer);
                        }}
                    }}

                    [ThreadStatic]
                    [FixedAddressValueType]
                    internal static ReturnArea returnArea = default;
                }}
                ",
                array_size,
                self.return_area_align,
                element_type
            );

            src.push_str(&ret_area_str);
        }

        if self.needs_rep_table {
            src.push_str("\n");
            src.push_str(include_str!("RepTable.cs"));
        }

        if !&self.world_fragments.is_empty() {
            src.push_str("\n");

            src.push_str("namespace exports {\n");
            src.push_str(&format!("{access} static class {name}World\n"));
            src.push_str("{");

            for fragment in &self.world_fragments {
                src.push_str("\n");

                src.push_str(&fragment.csharp_interop_src);
            }
            src.push_str("}\n");
            src.push_str("}\n");
        }

        src.push_str("\n");

        src.push_str("}\n");

        files.push(&format!("{name}.cs"), indent(&src).as_bytes());

        let generate_stub = |name: String, files: &mut Files, stubs: Stubs| {
            let (stub_namespace, interface_or_class_name) =
                CSharp::get_class_name_from_qualified_name(&name);

            let stub_class_name = format!(
                "{}Impl",
                match interface_or_class_name.starts_with("I") {
                    true => interface_or_class_name
                        .strip_prefix("I")
                        .unwrap()
                        .to_string(),
                    false => interface_or_class_name.clone(),
                }
            );

            let stub_file_name = match stub_namespace.len() {
                0 => stub_class_name.clone(),
                _ => format!("{stub_namespace}.{stub_class_name}"),
            };

            let (fragments, fully_qualified_namespace) = match stubs {
                Stubs::World(fragments) => {
                    let fully_qualified_namespace = format!("{namespace}");
                    (fragments, fully_qualified_namespace)
                }
                Stubs::Interface(fragments) => {
                    let fully_qualified_namespace = format!("{stub_namespace}");
                    (fragments, fully_qualified_namespace)
                }
            };

            if fragments.iter().all(|f| f.stub.is_empty()) {
                return;
            }

            let body = fragments
                .iter()
                .map(|f| f.stub.deref())
                .collect::<Vec<_>>()
                .join("\n");

            let body = format!(
                "{header}
                 {CSHARP_IMPORTS}

                 namespace {fully_qualified_namespace};

                 {access} partial class {stub_class_name} : {interface_or_class_name} {{
                    {body}
                 }}
                "
            );

            files.push(&format!("{stub_file_name}.cs"), indent(&body).as_bytes());
        };

        if self.opts.generate_stub {
            generate_stub(
                format!("I{name}World"),
                files,
                Stubs::World(&self.world_fragments),
            );
        }

        if !self.opts.skip_support_files {
            //TODO: This is currently needed for mono even if it's built as a library.
            if self.opts.runtime == CSharpRuntime::Mono {
                files.push(
                    &format!("MonoEntrypoint.cs",),
                    indent(&format!(
                        r#"
                        {access} class MonoEntrypoint() {{
                            {access} static void Main() {{
                            }}
                        }}
                        "#
                    ))
                    .as_bytes(),
                );
            }

            // For the time being, we generate both a .wit file and a .o file to
            // represent the component type.  Newer releases of the .NET runtime
            // will be able to use the former, but older ones will need the
            // latter.
            //
            // TODO: stop generating the .o file once a new-enough release is
            // available for us to test using only the .wit file.

            {
                // When generating a WIT file, we first round-trip through the
                // binary encoding.  This has the effect of flattening any
                // `include`d worlds into the specified world and excluding
                // unrelated worlds, ensuring the output WIT contains no extra
                // information beyond what the binary representation contains.
                //
                // This is important because including more than one world in
                // the output would make it ambigious, and since this file is
                // intended to be used non-interactively at link time, the
                // linker will have no additional information to resolve such
                // ambiguity.
                let (resolve, world) =
                    wit_parser::decoding::decode_world(&wit_component::metadata::encode(
                        &resolve,
                        id,
                        self.opts.string_encoding,
                        None,
                    )?)?;
                let pkg = resolve.worlds[world].package.unwrap();

                files.push(
                    &format!("{world_namespace}_component_type.wit"),
                    WitPrinter::default()
                        .emit_docs(false)
                        .print(
                            &resolve,
                            pkg,
                            &resolve
                                .packages
                                .iter()
                                .filter_map(|(id, _)| if id == pkg { None } else { Some(id) })
                                .collect::<Vec<_>>(),
                        )?
                        .as_bytes(),
                );
            }

            // TODO: remove when we switch to dotnet 9
            let mut wasm_import_linakge_src = String::new();

            uwrite!(
                wasm_import_linakge_src,
                r#"{header}
                #if !NET9_0_OR_GREATER
                // temporarily add this attribute until it is available in dotnet 9
                namespace System.Runtime.InteropServices
                {{
                    internal partial class WasmImportLinkageAttribute : Attribute {{}}
                }}
                #endif
                "#,
            );
            files.push(
                &format!("{world_namespace}_wasm_import_linkage_attribute.cs"),
                indent(&wasm_import_linakge_src).as_bytes(),
            );
        }

        for (full_name, interface_type_and_fragments) in &self.interface_fragments {
            let fragments = &interface_type_and_fragments.interface_fragments;

            let (namespace, interface_name) =
                &CSharp::get_class_name_from_qualified_name(full_name);

            // C#
            let body = fragments
                .iter()
                .map(|f| f.csharp_src.deref())
                .collect::<Vec<_>>()
                .join("\n");

            if body.len() > 0 {
                let body = format!(
                    "{header}
                    {CSHARP_IMPORTS}

                    namespace {namespace};

                    {access} interface {interface_name} {{
                        {body}
                    }}
                    "
                );

                files.push(&format!("{full_name}.cs"), indent(&body).as_bytes());
            }

            // C# Interop
            let body = fragments
                .iter()
                .map(|f| f.csharp_interop_src.deref())
                .collect::<Vec<_>>()
                .join("\n");

            let class_name = interface_name.strip_prefix("I").unwrap();
            let body = format!(
                "{header}
                {CSHARP_IMPORTS}

                namespace {namespace}
                {{
                  {access} static class {class_name}Interop {{
                      {body}
                  }}
                }}
                "
            );

            files.push(
                &format!("{namespace}.{class_name}Interop.cs"),
                indent(&body).as_bytes(),
            );

            if interface_type_and_fragments.is_export && self.opts.generate_stub {
                generate_stub(full_name.to_string(), files, Stubs::Interface(fragments));
            }
        }

        Ok(())
    }
}

struct InterfaceGenerator<'a> {
    src: String,
    csharp_interop_src: String,
    stub: String,
    gen: &'a mut CSharp,
    resolve: &'a Resolve,
    name: &'a str,
    direction: Direction,
}

impl InterfaceGenerator<'_> {
    fn define_interface_types(&mut self, id: InterfaceId) {
        let mut live = LiveTypes::default();
        live.add_interface(self.resolve, id);
        self.define_live_types(live, id);
    }

    //TODO: we probably need this for anonymous types outside of an interface...
    // fn define_function_types(&mut self, funcs: &[(&str, &Function)]) {
    //     let mut live = LiveTypes::default();
    //     for (_, func) in funcs {
    //         live.add_func(self.resolve, func);
    //     }
    //     self.define_live_types(live);
    // }

    fn define_live_types(&mut self, live: LiveTypes, id: InterfaceId) {
        let mut type_names = HashMap::new();

        for ty in live.iter() {
            // just create c# types for wit anonymous types
            let type_def = &self.resolve.types[ty];
            if type_names.contains_key(&ty) || type_def.name.is_some() {
                continue;
            }

            let typedef_name = self.type_name(&Type::Id(ty));

            let prev = type_names.insert(ty, typedef_name.clone());
            assert!(prev.is_none());

            // workaround for owner not set on anonymous types, maintain or own map to the owner
            self.gen
                .anonymous_type_owners
                .insert(ty, TypeOwner::Interface(id));

            self.define_anonymous_type(ty, &typedef_name)
        }
    }

    fn define_anonymous_type(&mut self, type_id: TypeId, typedef_name: &str) {
        let type_def = &self.resolve().types[type_id];
        let kind = &type_def.kind;

        // TODO Does c# need this exit?
        // // skip `typedef handle_x handle_y` where `handle_x` is the same as `handle_y`
        // if let TypeDefKind::Handle(handle) = kind {
        //     let resource = match handle {
        //         Handle::Borrow(id) | Handle::Own(id) => id,
        //     };
        //     let origin = dealias(self.resolve, *resource);
        //     if origin == *resource {
        //         return;
        //     }
        // }

        //TODO: what other TypeDefKind do we need here?
        match kind {
            TypeDefKind::Tuple(t) => self.type_tuple(type_id, typedef_name, t, &type_def.docs),
            TypeDefKind::Option(t) => self.type_option(type_id, typedef_name, t, &type_def.docs),
            TypeDefKind::Record(t) => self.type_record(type_id, typedef_name, t, &type_def.docs),
            TypeDefKind::List(t) => self.type_list(type_id, typedef_name, t, &type_def.docs),
            TypeDefKind::Variant(t) => self.type_variant(type_id, typedef_name, t, &type_def.docs),
            TypeDefKind::Result(t) => self.type_result(type_id, typedef_name, t, &type_def.docs),
            TypeDefKind::Handle(_) => {
                // Handles don't require a separate definition beyond what we already define for the corresponding
                // resource types.
            }
            _ => unreachable!(),
        }
    }

    fn qualifier(&self, when: bool, ty: &TypeId) -> String {
        // anonymous types dont get an owner from wit-parser, so assume they are part of an interface here.
        let owner = if let Some(owner_type) = self.gen.anonymous_type_owners.get(ty) {
            *owner_type
        } else {
            let type_def = &self.resolve.types[*ty];
            type_def.owner
        };

        let global_prefix = self.global_if_user_type(&Type::Id(*ty));

        if let TypeOwner::Interface(id) = owner {
            if let Some(name) = self.gen.interface_names.get(&id) {
                if name != self.name {
                    return format!("{global_prefix}{name}.");
                }
            }
        }

        if when {
            let name = self.name;
            format!("{global_prefix}{name}.")
        } else {
            String::new()
        }
    }

    fn add_interface_fragment(self, is_export: bool) {
        self.gen
            .interface_fragments
            .entry(self.name.to_string())
            .or_insert_with(|| InterfaceTypeAndFragments::new(is_export))
            .interface_fragments
            .push(InterfaceFragment {
                csharp_src: self.src,
                csharp_interop_src: self.csharp_interop_src,
                stub: self.stub,
            });
    }

    fn add_world_fragment(self) {
        self.gen.world_fragments.push(InterfaceFragment {
            csharp_src: self.src,
            csharp_interop_src: self.csharp_interop_src,
            stub: self.stub,
        });
    }

    fn import(&mut self, import_module_name: &str, func: &Function) {
        let (camel_name, modifiers) = match &func.kind {
            FunctionKind::Freestanding | FunctionKind::Static(_) => {
                (func.item_name().to_upper_camel_case(), "static")
            }
            FunctionKind::Method(_) => (func.item_name().to_upper_camel_case(), ""),
            FunctionKind::Constructor(id) => {
                (self.gen.all_resources[id].name.to_upper_camel_case(), "")
            }
        };

        let access = self.gen.access_modifier();

        let extra_modifiers = extra_modifiers(func, &camel_name);

        let interop_camel_name = func.item_name().to_upper_camel_case();

        let sig = self.resolve.wasm_signature(AbiVariant::GuestImport, func);

        let wasm_result_type = match &sig.results[..] {
            [] => "void",
            [result] => wasm_type(*result),
            _ => unreachable!(),
        };

        let (result_type, results) = if let FunctionKind::Constructor(_) = &func.kind {
            (String::new(), Vec::new())
        } else {
            match func.results.len() {
                0 => ("void".to_string(), Vec::new()),
                1 => {
                    let (payload, results) = payload_and_results(
                        self.resolve,
                        *func.results.iter_types().next().unwrap(),
                    );
                    (
                        if let Some(ty) = payload {
                            self.gen.needs_result = true;
                            self.type_name_with_qualifier(&ty, true)
                        } else {
                            "void".to_string()
                        },
                        results,
                    )
                }
                _ => {
                    let types = func
                        .results
                        .iter_types()
                        .map(|ty| self.type_name_with_qualifier(ty, true))
                        .collect::<Vec<_>>()
                        .join(", ");
                    (format!("({})", types), Vec::new())
                }
            }
        };

        let wasm_params = sig
            .params
            .iter()
            .enumerate()
            .map(|(i, param)| {
                let ty = wasm_type(*param);
                format!("{ty} p{i}")
            })
            .collect::<Vec<_>>()
            .join(", ");

        let mut bindgen = FunctionBindgen::new(
            self,
            &func.item_name(),
            &func.kind,
            func.params
                .iter()
                .enumerate()
                .map(|(i, (name, _))| {
                    if i == 0 && matches!(&func.kind, FunctionKind::Method(_)) {
                        "this".to_owned()
                    } else {
                        name.to_csharp_ident()
                    }
                })
                .collect(),
            results,
        );

        abi::call(
            bindgen.gen.resolve,
            AbiVariant::GuestImport,
            LiftLower::LowerArgsLiftResults,
            func,
            &mut bindgen,
        );

        let src = bindgen.src;

        let params = func
            .params
            .iter()
            .skip(if let FunctionKind::Method(_) = &func.kind {
                1
            } else {
                0
            })
            .map(|param| {
                let ty = self.type_name_with_qualifier(&param.1, true);
                let param_name = &param.0;
                let param_name = param_name.to_csharp_ident();
                format!("{ty} {param_name}")
            })
            .collect::<Vec<_>>()
            .join(", ");

        let import_name = &func.name;

        let target = if let FunctionKind::Freestanding = &func.kind {
            &mut self.csharp_interop_src
        } else {
            &mut self.src
        };

        uwrite!(
            target,
            r#"
            internal static class {interop_camel_name}WasmInterop
            {{
                [DllImport("{import_module_name}", EntryPoint = "{import_name}"), WasmImportLinkage]
                internal static extern {wasm_result_type} wasmImport{interop_camel_name}({wasm_params});
            "#
        );

        uwrite!(
            target,
            r#"
            }}
            "#,
        );

        uwrite!(
            target,
            r#"
                {access} {extra_modifiers} {modifiers} unsafe {result_type} {camel_name}({params})
                {{
                    {src}
                    //TODO: free alloc handle (interopString) if exists
                }}
            "#
        );
    }

    fn export(&mut self, func: &Function, interface_name: Option<&WorldKey>) {
        let (camel_name, modifiers) = match &func.kind {
            FunctionKind::Freestanding | FunctionKind::Static(_) => {
                (func.item_name().to_upper_camel_case(), "static abstract")
            }
            FunctionKind::Method(_) => (func.item_name().to_upper_camel_case(), ""),
            FunctionKind::Constructor(id) => {
                (self.gen.all_resources[id].name.to_upper_camel_case(), "")
            }
        };

        let extra_modifiers = extra_modifiers(func, &camel_name);

        let sig = self.resolve.wasm_signature(AbiVariant::GuestExport, func);

        let (result_type, results) = if let FunctionKind::Constructor(_) = &func.kind {
            (String::new(), Vec::new())
        } else {
            match func.results.len() {
                0 => ("void".to_owned(), Vec::new()),
                1 => {
                    let (payload, results) = payload_and_results(
                        self.resolve,
                        *func.results.iter_types().next().unwrap(),
                    );
                    (
                        if let Some(ty) = payload {
                            self.gen.needs_result = true;
                            self.type_name(&ty)
                        } else {
                            "void".to_string()
                        },
                        results,
                    )
                }
                _ => {
                    let types = func
                        .results
                        .iter_types()
                        .map(|ty| self.type_name(ty))
                        .collect::<Vec<String>>()
                        .join(", ");
                    (format!("({}) ", types), Vec::new())
                }
            }
        };

        let mut bindgen = FunctionBindgen::new(
            self,
            &func.item_name(),
            &func.kind,
            (0..sig.params.len()).map(|i| format!("p{i}")).collect(),
            results,
        );

        abi::call(
            bindgen.gen.resolve,
            AbiVariant::GuestExport,
            LiftLower::LiftArgsLowerResults,
            func,
            &mut bindgen,
        );

        assert!(!bindgen.needs_cleanup_list);

        let src = bindgen.src;

        let vars = bindgen
            .resource_drops
            .iter()
            .map(|(t, v)| format!("{t}? {v} = null;"))
            .collect::<Vec<_>>()
            .join(";\n");

        let wasm_result_type = match &sig.results[..] {
            [] => "void",
            [result] => wasm_type(*result),
            _ => unreachable!(),
        };

        let wasm_params = sig
            .params
            .iter()
            .enumerate()
            .map(|(i, param)| {
                let ty = wasm_type(*param);
                format!("{ty} p{i}")
            })
            .collect::<Vec<_>>()
            .join(", ");

        let params = func
            .params
            .iter()
            .skip(if let FunctionKind::Method(_) = &func.kind {
                1
            } else {
                0
            })
            .map(|(name, ty)| {
                let ty = self.type_name(ty);
                let name = name.to_csharp_ident();
                format!("{ty} {name}")
            })
            .collect::<Vec<String>>()
            .join(", ");

        let interop_name = format!("wasmExport{}", func.name.to_upper_camel_case());
        let core_module_name = interface_name.map(|s| self.resolve.name_world_key(s));
        let export_name = func.core_export_name(core_module_name.as_deref());
        let access = self.gen.access_modifier();

        uwrite!(
            self.csharp_interop_src,
            r#"
            [UnmanagedCallersOnly(EntryPoint = "{export_name}")]
            {access} static unsafe {wasm_result_type} {interop_name}({wasm_params}) {{
                {vars}
                {src}
            }}
            "#
        );

        if !sig.results.is_empty() {
            uwrite!(
                self.csharp_interop_src,
                r#"
                [UnmanagedCallersOnly(EntryPoint = "cabi_post_{export_name}")]
                {access} static void cabi_post_{interop_name}({wasm_result_type} returnValue) {{
                    Console.WriteLine("TODO: cabi_post_{export_name}");
                }}
                "#
            );
        }

        if !matches!(&func.kind, FunctionKind::Constructor(_)) {
            uwrite!(
                self.src,
                r#"{extra_modifiers} {modifiers} {result_type} {camel_name}({params});

            "#
            );
        }

        if self.gen.opts.generate_stub {
            let sig = self.sig_string(func, true);

            uwrite!(
                self.stub,
                r#"
                {sig} {{
                    throw new NotImplementedException();
                }}
                "#
            );
        }
    }

    fn type_name(&mut self, ty: &Type) -> String {
        self.type_name_with_qualifier(ty, false)
    }

    // We use a global:: prefix to avoid conflicts with namespace clashes on partial namespace matches
    fn global_if_user_type(&self, ty: &Type) -> String {
        match ty {
            Type::Id(id) => {
                let ty = &self.resolve.types[*id];
                match &ty.kind {
                    TypeDefKind::Option(_ty) => "".to_owned(),
                    TypeDefKind::Result(_result) => "".to_owned(),
                    TypeDefKind::List(_list) => "".to_owned(),
                    TypeDefKind::Tuple(_tuple) => "".to_owned(),
                    TypeDefKind::Type(inner_type) => self.global_if_user_type(inner_type),
                    _ => "global::".to_owned(),
                }
            }
            _ => "".to_owned(),
        }
    }

    fn type_name_with_qualifier(&mut self, ty: &Type, qualifier: bool) -> String {
        match ty {
            Type::Bool => "bool".to_owned(),
            Type::U8 => "byte".to_owned(),
            Type::U16 => "ushort".to_owned(),
            Type::U32 => "uint".to_owned(),
            Type::U64 => "ulong".to_owned(),
            Type::S8 => "sbyte".to_owned(),
            Type::S16 => "short".to_owned(),
            Type::S32 => "int".to_owned(),
            Type::S64 => "long".to_owned(),
            Type::F32 => "float".to_owned(),
            Type::F64 => "double".to_owned(),
            Type::Char => "uint".to_owned(),
            Type::String => "string".to_owned(),
            Type::Id(id) => {
                let ty = &self.resolve.types[*id];
                match &ty.kind {
                    TypeDefKind::Type(ty) => self.type_name_with_qualifier(ty, qualifier),
                    TypeDefKind::List(ty) => {
                        if is_primitive(ty) {
                            format!("{}[]", self.type_name(ty))
                        } else {
                            format!("List<{}>", self.type_name_with_qualifier(ty, qualifier))
                        }
                    }
                    TypeDefKind::Tuple(tuple) => {
                        let count = tuple.types.len();
                        self.gen.tuple_counts.insert(count);

                        let params = match count {
                            0 => String::new(),
                            1 => self
                                .type_name_with_qualifier(tuple.types.first().unwrap(), qualifier),
                            _ => format!(
                                "({})",
                                tuple
                                    .types
                                    .iter()
                                    .map(|ty| self.type_name_with_qualifier(ty, qualifier))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        };

                        params
                    }
                    TypeDefKind::Option(base_ty) => {
                        self.gen.needs_option = true;
                        let nesting = if let Type::Id(id) = base_ty {
                            matches!(&self.resolve.types[*id].kind, TypeDefKind::Option(_))
                        } else {
                            false
                        };
                        let base_ty = self.type_name_with_qualifier(base_ty, qualifier);
                        if nesting {
                            format!("Option<{base_ty}>")
                        } else {
                            format!("{base_ty}?")
                        }
                    }
                    TypeDefKind::Result(result) => {
                        self.gen.needs_result = true;
                        let mut name = |ty: &Option<Type>| {
                            ty.as_ref()
                                .map(|ty| self.type_name_with_qualifier(ty, qualifier))
                                .unwrap_or_else(|| "None".to_owned())
                        };
                        let ok = name(&result.ok);
                        let err = name(&result.err);

                        format!("Result<{ok}, {err}>")
                    }
                    TypeDefKind::Handle(handle) => {
                        let (Handle::Own(id) | Handle::Borrow(id)) = handle;
                        self.type_name_with_qualifier(&Type::Id(*id), qualifier)
                    }
                    _ => {
                        if let Some(name) = &ty.name {
                            format!(
                                "{}{}",
                                self.qualifier(qualifier, id),
                                name.to_upper_camel_case()
                            )
                        } else {
                            unreachable!("todo: {ty:?}")
                        }
                    }
                }
            }
        }
    }

    fn print_docs(&mut self, docs: &Docs) {
        if let Some(docs) = &docs.contents {
            let lines = docs
                .trim()
                .replace("<", "&lt;")
                .replace(">", "&gt;")
                .lines()
                .map(|line| format!("* {line}"))
                .collect::<Vec<_>>()
                .join("\n");

            uwrite!(
                self.src,
                "
                /**
                 {lines}
                 */
                "
            )
        }
    }

    fn non_empty_type<'a>(&self, ty: Option<&'a Type>) -> Option<&'a Type> {
        if let Some(ty) = ty {
            let id = match ty {
                Type::Id(id) => *id,
                _ => return Some(ty),
            };
            match &self.resolve.types[id].kind {
                TypeDefKind::Type(t) => self.non_empty_type(Some(t)).map(|_| ty),
                TypeDefKind::Record(r) => (!r.fields.is_empty()).then_some(ty),
                TypeDefKind::Tuple(t) => (!t.types.is_empty()).then_some(ty),
                _ => Some(ty),
            }
        } else {
            None
        }
    }

    fn start_resource(&mut self, id: TypeId, key: Option<&WorldKey>) {
        let access = self.gen.access_modifier();
        let qualified = self.type_name_with_qualifier(&Type::Id(id), true);
        let info = &self.gen.all_resources[&id];
        let name = info.name.clone();
        let upper_camel = name.to_upper_camel_case();
        let docs = info.docs.clone();
        self.print_docs(&docs);

        match self.direction {
            Direction::Import => {
                let module_name = key
                    .map(|key| self.resolve.name_world_key(key))
                    .unwrap_or_else(|| "$root".into());

                uwriteln!(
                    self.src,
                    r#"
                    {access} class {upper_camel}: IDisposable {{
                        internal int Handle {{ get; set; }}

                        {access} readonly record struct THandle(int Handle);

                        {access} {upper_camel}(THandle handle) {{
                            Handle = handle.Handle;
                        }}

                        public void Dispose() {{
                            Dispose(true);
                            GC.SuppressFinalize(this);
                        }}

                        [DllImport("{module_name}", EntryPoint = "[resource-drop]{name}"), WasmImportLinkage]
                        private static extern void wasmImportResourceDrop(int p0);

                        protected virtual void Dispose(bool disposing) {{
                            if (Handle != 0) {{
                                wasmImportResourceDrop(Handle);
                                Handle = 0;
                            }}
                        }}

                        ~{upper_camel}() {{
                            Dispose(false);
                        }}
                    "#
                );
            }
            Direction::Export => {
                let prefix = key
                    .map(|s| format!("{}#", self.resolve.name_world_key(s)))
                    .unwrap_or_else(String::new);

                uwrite!(
                    self.csharp_interop_src,
                    r#"
                    [UnmanagedCallersOnly(EntryPoint = "{prefix}[dtor]{name}")]
                    {access} static unsafe void wasmExportResourceDtor{upper_camel}(int rep) {{
                        var val = ({qualified}) {qualified}.repTable.Remove(rep);
                        val.Handle = 0;
                        // Note we call `Dispose` here even though the handle has already been disposed in case
                        // the implementation has overridden `Dispose(bool)`.
                        val.Dispose();
                    }}
                    "#
                );

                let module_name = key
                    .map(|key| format!("[export]{}", self.resolve.name_world_key(key)))
                    .unwrap_or_else(|| "[export]$root".into());

                // The ergonomics of exported resources are not ideal, currently. Implementing such a resource
                // requires both extending a class and implementing an interface. The reason for the class is to
                // allow implementers to inherit code which tracks and disposes of the resource handle; the reason
                // for the interface is to express the API contract which the implementation must fulfill,
                // including static functions.
                //
                // We could remove the need for the class (and its `IDisposable` implementation) entirely if we
                // were to dispose of the handle immediately when lifting an owned handle, in which case we would
                // be left with nothing to keep track of or dispose later. However, we keep the handle alive in
                // case we want to give ownership back to the host again, in which case we'll be able to reuse the
                // same handle instead of calling `[resource-new]` to allocate a new one. Whether this optimization
                // is worth the trouble is open to debate, but we currently consider it a worthwhile tradeoff.
                //
                // Note that applications which export resources are relatively rare compared to those which only
                // import them, so in practice most developers won't encounter any of this anyway.
                uwriteln!(
                    self.src,
                    r#"
                    {access} abstract class {upper_camel}: IDisposable {{
                        internal static RepTable<{upper_camel}> repTable = new ();
                        internal int Handle {{ get; set; }}

                        public void Dispose() {{
                            Dispose(true);
                            GC.SuppressFinalize(this);
                        }}

                        internal static class WasmInterop {{
                            [DllImport("{module_name}", EntryPoint = "[resource-drop]{name}"), WasmImportLinkage]
                            internal static extern void wasmImportResourceDrop(int p0);

                            [DllImport("{module_name}", EntryPoint = "[resource-new]{name}"), WasmImportLinkage]
                            internal static extern int wasmImportResourceNew(int p0);

                            [DllImport("{module_name}", EntryPoint = "[resource-rep]{name}"), WasmImportLinkage]
                            internal static extern int wasmImportResourceRep(int p0);
                        }}

                        protected virtual void Dispose(bool disposing) {{
                            if (Handle != 0) {{
                                var handle = Handle;
                                Handle = 0;
                                WasmInterop.wasmImportResourceDrop(handle);
                            }}
                        }}

                        ~{upper_camel}() {{
                            Dispose(false);
                        }}
                    }}

                    {access} interface I{upper_camel} {{
                    "#
                );

                if self.gen.opts.generate_stub {
                    let super_ = self.type_name_with_qualifier(&Type::Id(id), true);
                    let interface = {
                        let split = super_.split('.').collect::<Vec<_>>();
                        split
                            .iter()
                            .map(|&v| v.to_owned())
                            .take(split.len() - 1)
                            .chain(split.last().map(|v| format!("I{v}")))
                            .collect::<Vec<_>>()
                            .join(".")
                    };

                    uwriteln!(
                        self.stub,
                        r#"
                        {access} class {upper_camel}: {super_}, {interface} {{
                        "#
                    );
                }
            }
        };
    }

    fn end_resource(&mut self) {
        if self.direction == Direction::Export && self.gen.opts.generate_stub {
            uwriteln!(
                self.stub,
                "
                }}
                "
            );
        }

        uwriteln!(
            self.src,
            "
            }}
            "
        );
    }

    fn sig_string(&mut self, func: &Function, qualifier: bool) -> String {
        let result_type = if let FunctionKind::Constructor(_) = &func.kind {
            String::new()
        } else {
            match func.results.len() {
                0 => "void".into(),
                1 => {
                    let (payload, _) = payload_and_results(
                        self.resolve,
                        *func.results.iter_types().next().unwrap(),
                    );
                    if let Some(ty) = payload {
                        self.gen.needs_result = true;
                        self.type_name_with_qualifier(&ty, qualifier)
                    } else {
                        "void".to_string()
                    }
                }
                count => {
                    self.gen.tuple_counts.insert(count);
                    format!(
                        "({})",
                        func.results
                            .iter_types()
                            .map(|ty| self.type_name_with_qualifier(ty, qualifier))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
        };

        let params = func
            .params
            .iter()
            .skip(if let FunctionKind::Method(_) = &func.kind {
                1
            } else {
                0
            })
            .map(|(name, ty)| {
                let ty = self.type_name_with_qualifier(ty, qualifier);
                let name = name.to_csharp_ident();
                format!("{ty} {name}")
            })
            .collect::<Vec<_>>()
            .join(", ");

        let (camel_name, modifiers) = match &func.kind {
            FunctionKind::Freestanding | FunctionKind::Static(_) => {
                (func.item_name().to_upper_camel_case(), "static")
            }
            FunctionKind::Method(_) => (func.item_name().to_upper_camel_case(), ""),
            FunctionKind::Constructor(id) => {
                (self.gen.all_resources[id].name.to_upper_camel_case(), "")
            }
        };

        let access = self.gen.access_modifier();

        format!("{access} {modifiers} {result_type} {camel_name}({params})")
    }
}

impl<'a> wit_bindgen_core::InterfaceGenerator<'a> for InterfaceGenerator<'a> {
    fn resolve(&self) -> &'a Resolve {
        self.resolve
    }

    fn type_record(&mut self, _id: TypeId, name: &str, record: &Record, docs: &Docs) {
        let access = self.gen.access_modifier();

        self.print_docs(docs);

        let name = name.to_upper_camel_case();

        let parameters = record
            .fields
            .iter()
            .map(|field| {
                format!(
                    "{} {}",
                    self.type_name(&field.ty),
                    field.name.to_csharp_ident()
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        let assignments = record
            .fields
            .iter()
            .map(|field| {
                let name = field.name.to_csharp_ident();
                format!("this.{name} = {name};")
            })
            .collect::<Vec<_>>()
            .join("\n");

        let fields = if record.fields.is_empty() {
            format!("{access} const {name} INSTANCE = new {name}();")
        } else {
            record
                .fields
                .iter()
                .map(|field| {
                    format!(
                        "{access} readonly {} {};",
                        self.type_name(&field.ty),
                        field.name.to_csharp_ident()
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        uwrite!(
            self.src,
            "
            {access} class {name} {{
                {fields}

                {access} {name}({parameters}) {{
                    {assignments}
                }}
            }}
            "
        );
    }

    fn type_flags(&mut self, _id: TypeId, name: &str, flags: &Flags, docs: &Docs) {
        self.print_docs(docs);

        let name = name.to_upper_camel_case();

        let enum_elements = flags
            .flags
            .iter()
            .enumerate()
            .map(|(i, flag)| {
                let flag_name = flag.name.to_shouty_snake_case();
                let suffix = if matches!(flags.repr(), FlagsRepr::U32(2)) {
                    "UL"
                } else {
                    ""
                };
                format!("{flag_name} = 1{suffix} << {i},")
            })
            .collect::<Vec<_>>()
            .join("\n");

        let enum_type = match flags.repr() {
            FlagsRepr::U32(2) => ": ulong",
            FlagsRepr::U16 => ": ushort",
            FlagsRepr::U8 => ": byte",
            _ => "",
        };

        let access = self.gen.access_modifier();

        uwrite!(
            self.src,
            "
            {access} enum {name} {enum_type} {{
                {enum_elements}
            }}
            "
        );
    }

    fn type_tuple(&mut self, id: TypeId, _name: &str, _tuple: &Tuple, _docs: &Docs) {
        self.type_name(&Type::Id(id));
    }

    fn type_variant(&mut self, _id: TypeId, name: &str, variant: &Variant, docs: &Docs) {
        self.print_docs(docs);

        let name = name.to_upper_camel_case();
        let tag_type = int_type(variant.tag());
        let access = self.gen.access_modifier();

        let constructors = variant
            .cases
            .iter()
            .map(|case| {
                let case_name = case.name.to_csharp_ident();
                let tag = case.name.to_shouty_snake_case();
                let (parameter, argument) = if let Some(ty) = self.non_empty_type(case.ty.as_ref())
                {
                    (
                        format!("{} {case_name}", self.type_name(ty)),
                        case_name.deref(),
                    )
                } else {
                    (String::new(), "null")
                };

                format!(
                    "{access} static {name} {case_name}({parameter}) {{
                         return new {name}({tag}, {argument});
                     }}
                    "
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let accessors = variant
            .cases
            .iter()
            .filter_map(|case| {
                self.non_empty_type(case.ty.as_ref()).map(|ty| {
                    let case_name = case.name.to_upper_camel_case();
                    let tag = case.name.to_shouty_snake_case();
                    let ty = self.type_name(ty);
                    format!(
                        r#"{access} {ty} As{case_name}
                        {{
                            get
                            {{
                                if (Tag == {tag})
                                    return ({ty})value!;
                                else
                                    throw new ArgumentException("expected {tag}, got " + Tag);
                            }}
                        }}
                        "#
                    )
                })
            })
            .collect::<Vec<_>>()
            .join("\n");

        let tags = variant
            .cases
            .iter()
            .enumerate()
            .map(|(i, case)| {
                let tag = case.name.to_shouty_snake_case();
                format!("{access} const {tag_type} {tag} = {i};")
            })
            .collect::<Vec<_>>()
            .join("\n");

        uwrite!(
            self.src,
            "
            {access} class {name} {{
                {access} readonly {tag_type} Tag;
                private readonly object? value;

                private {name}({tag_type} tag, object? value) {{
                    this.Tag = tag;
                    this.value = value;
                }}

                {constructors}
                {accessors}
                {tags}
            }}
            "
        );
    }

    fn type_option(&mut self, id: TypeId, _name: &str, _payload: &Type, _docs: &Docs) {
        self.type_name(&Type::Id(id));
    }

    fn type_result(&mut self, id: TypeId, _name: &str, _result: &Result_, _docs: &Docs) {
        self.type_name(&Type::Id(id));
    }

    fn type_enum(&mut self, _id: TypeId, name: &str, enum_: &Enum, docs: &Docs) {
        self.print_docs(docs);

        let name = name.to_upper_camel_case();

        let cases = enum_
            .cases
            .iter()
            .map(|case| case.name.to_shouty_snake_case())
            .collect::<Vec<_>>()
            .join(", ");

        let access = self.gen.access_modifier();

        uwrite!(
            self.src,
            "
            {access} enum {name} {{
                {cases}
            }}
            "
        );
    }

    fn type_alias(&mut self, id: TypeId, _name: &str, _ty: &Type, _docs: &Docs) {
        self.type_name(&Type::Id(id));
    }

    fn type_list(&mut self, id: TypeId, _name: &str, _ty: &Type, _docs: &Docs) {
        self.type_name(&Type::Id(id));
    }

    fn type_builtin(&mut self, _id: TypeId, _name: &str, _ty: &Type, _docs: &Docs) {
        unimplemented!();
    }

    fn type_resource(&mut self, id: TypeId, name: &str, docs: &Docs) {
        // Here we just record information about the resource; we don't actually emit any code until we're ready to
        // visit any functions associated with the resource (e.g. in CSharp::import_interface, etc.).
        self.gen
            .all_resources
            .entry(id)
            .or_insert_with(|| ResourceInfo {
                module: self.name.to_owned(),
                name: name.to_owned(),
                docs: docs.clone(),
                direction: Direction::Import,
            })
            .direction = self.direction;
    }
}

enum Stubs<'a> {
    World(&'a Vec<InterfaceFragment>),
    Interface(&'a Vec<InterfaceFragment>),
}

struct Block {
    body: String,
    results: Vec<String>,
    element: String,
    base: String,
}

struct Cleanup {
    address: String,
}

struct BlockStorage {
    body: String,
    element: String,
    base: String,
    cleanup: Vec<Cleanup>,
}

struct FunctionBindgen<'a, 'b> {
    gen: &'b mut InterfaceGenerator<'a>,
    func_name: &'b str,
    kind: &'b FunctionKind,
    params: Box<[String]>,
    results: Vec<TypeId>,
    src: String,
    locals: Ns,
    block_storage: Vec<BlockStorage>,
    blocks: Vec<Block>,
    payloads: Vec<String>,
    needs_cleanup_list: bool,
    cleanup: Vec<Cleanup>,
    import_return_pointer_area_size: ArchitectureSize,
    import_return_pointer_area_align: Alignment,
    fixed: usize, // Number of `fixed` blocks that need to be closed.
    resource_drops: Vec<(String, String)>,
}

impl<'a, 'b> FunctionBindgen<'a, 'b> {
    fn new(
        gen: &'b mut InterfaceGenerator<'a>,
        func_name: &'b str,
        kind: &'b FunctionKind,
        params: Box<[String]>,
        results: Vec<TypeId>,
    ) -> FunctionBindgen<'a, 'b> {
        let mut locals = Ns::default();
        // Ensure temporary variable names don't clash with parameter names:
        for param in &params[..] {
            locals.tmp(param);
        }

        Self {
            gen,
            func_name,
            kind,
            params,
            results,
            src: String::new(),
            locals,
            block_storage: Vec::new(),
            blocks: Vec::new(),
            payloads: Vec::new(),
            needs_cleanup_list: false,
            cleanup: Vec::new(),
            import_return_pointer_area_size: Default::default(),
            import_return_pointer_area_align: Default::default(),
            fixed: 0,
            resource_drops: Vec::new(),
        }
    }

    fn lower_variant(
        &mut self,
        cases: &[(&str, Option<Type>)],
        lowered_types: &[WasmType],
        op: &str,
        results: &mut Vec<String>,
    ) {
        let blocks = self
            .blocks
            .drain(self.blocks.len() - cases.len()..)
            .collect::<Vec<_>>();

        let payloads = self
            .payloads
            .drain(self.payloads.len() - cases.len()..)
            .collect::<Vec<_>>();

        let lowered = lowered_types
            .iter()
            .map(|_| self.locals.tmp("lowered"))
            .collect::<Vec<_>>();

        results.extend(lowered.iter().cloned());

        let declarations = lowered
            .iter()
            .zip(lowered_types)
            .map(|(lowered, ty)| format!("{} {lowered};", wasm_type(*ty)))
            .collect::<Vec<_>>()
            .join("\n");

        let cases = cases
            .iter()
            .zip(blocks)
            .zip(payloads)
            .enumerate()
            .map(
                |(i, (((name, ty), Block { body, results, .. }), payload))| {
                    let payload = if let Some(ty) = self.gen.non_empty_type(ty.as_ref()) {
                        let ty = self.gen.type_name_with_qualifier(ty, true);
                        let name = name.to_upper_camel_case();

                        format!("{ty} {payload} = {op}.As{name};")
                    } else {
                        String::new()
                    };

                    let assignments = lowered
                        .iter()
                        .zip(&results)
                        .map(|(lowered, result)| format!("{lowered} = {result};\n"))
                        .collect::<Vec<_>>()
                        .concat();

                    format!(
                        "case {i}: {{
                         {payload}
                         {body}
                         {assignments}
                         break;
                     }}"
                    )
                },
            )
            .collect::<Vec<_>>()
            .join("\n");

        uwrite!(
            self.src,
            r#"
            {declarations}

            switch ({op}.Tag) {{
                {cases}

                default: throw new ArgumentException($"invalid discriminant: {{{op}}}");
            }}
            "#
        );
    }

    fn lift_variant(
        &mut self,
        ty: &Type,
        cases: &[(&str, Option<Type>)],
        op: &str,
        results: &mut Vec<String>,
    ) {
        let blocks = self
            .blocks
            .drain(self.blocks.len() - cases.len()..)
            .collect::<Vec<_>>();
        let ty = self.gen.type_name_with_qualifier(ty, true);
        //let ty = self.gen.type_name(ty);
        let generics_position = ty.find('<');
        let lifted = self.locals.tmp("lifted");

        let cases = cases
            .iter()
            .zip(blocks)
            .enumerate()
            .map(|(i, ((case_name, case_ty), Block { body, results, .. }))| {
                let payload = if self.gen.non_empty_type(case_ty.as_ref()).is_some() {
                    results.into_iter().next().unwrap()
                } else if generics_position.is_some() {
                    if let Some(ty) = case_ty.as_ref() {
                        format!("{}.INSTANCE", self.gen.type_name_with_qualifier(ty, true))
                    } else {
                        format!("new global::{}None()", self.gen.gen.qualifier())
                    }
                } else {
                    String::new()
                };

                let method = case_name.to_csharp_ident();

                let call = if let Some(position) = generics_position {
                    let (ty, generics) = ty.split_at(position);
                    format!("{ty}{generics}.{method}")
                } else {
                    format!("{ty}.{method}")
                };

                format!(
                    "case {i}: {{
                         {body}
                         {lifted} = {call}({payload});
                         break;
                     }}"
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        uwrite!(
            self.src,
            r#"
            {ty} {lifted};

            switch ({op}) {{
                {cases}

                default: throw new ArgumentException($"invalid discriminant: {{{op}}}");
            }}
            "#
        );

        results.push(lifted);
    }
}

impl Bindgen for FunctionBindgen<'_, '_> {
    type Operand = String;

    fn emit(
        &mut self,
        _resolve: &Resolve,
        inst: &Instruction<'_>,
        operands: &mut Vec<String>,
        results: &mut Vec<String>,
    ) {
        match inst {
            Instruction::GetArg { nth } => results.push(self.params[*nth].clone()),
            Instruction::I32Const { val } => results.push(val.to_string()),
            Instruction::ConstZero { tys } => results.extend(tys.iter().map(|ty| {
                match ty {
                    WasmType::I32 => "0",
                    WasmType::I64 => "0L",
                    WasmType::F32 => "0.0F",
                    WasmType::F64 => "0.0D",
                    WasmType::Pointer => "0",
                    WasmType::PointerOrI64 => "0L",
                    WasmType::Length => "0",
                }
                .to_owned()
            })),
            Instruction::I32Load { offset }
            | Instruction::PointerLoad { offset }
            | Instruction::LengthLoad { offset } => results.push(format!("BitConverter.ToInt32(new Span<byte>((void*)({} + {offset}), 4))",operands[0])),
            Instruction::I32Load8U { offset } => results.push(format!("new Span<byte>((void*)({} + {offset}), 1)[0]",operands[0])),
            Instruction::I32Load8S { offset } => results.push(format!("(sbyte)new Span<byte>((void*)({} + {offset}), 1)[0]",operands[0])),
            Instruction::I32Load16U { offset } => results.push(format!("BitConverter.ToUInt16(new Span<byte>((void*)({} + {offset}), 2))",operands[0])),
            Instruction::I32Load16S { offset } => results.push(format!("BitConverter.ToInt16(new Span<byte>((void*)({} + {offset}), 2))",operands[0])),
            Instruction::I64Load { offset } => results.push(format!("BitConverter.ToInt64(new Span<byte>((void*)({} + {offset}), 8))",operands[0])),
            Instruction::F32Load { offset } => results.push(format!("BitConverter.ToSingle(new Span<byte>((void*)({} + {offset}), 4))",operands[0])),
            Instruction::F64Load { offset } => results.push(format!("BitConverter.ToDouble(new Span<byte>((void*)({} + {offset}), 8))",operands[0])),
            Instruction::I32Store { offset }
            | Instruction::PointerStore { offset }
            | Instruction::LengthStore { offset } => uwriteln!(self.src, "BitConverter.TryWriteBytes(new Span<byte>((void*)({} + {offset}), 4), unchecked((int){}));", operands[1], operands[0]),
            Instruction::I32Store8 { offset } => uwriteln!(self.src, "*(byte*)({} + {offset}) = (byte){};", operands[1], operands[0]),
            Instruction::I32Store16 { offset } => uwriteln!(self.src, "BitConverter.TryWriteBytes(new Span<byte>((void*)({} + {offset}), 2), (short){});", operands[1], operands[0]),
            Instruction::I64Store { offset } => uwriteln!(self.src, "BitConverter.TryWriteBytes(new Span<byte>((void*)({} + {offset}), 8), unchecked((long){}));", operands[1], operands[0]),
            Instruction::F32Store { offset } => uwriteln!(self.src, "BitConverter.TryWriteBytes(new Span<byte>((void*)({} + {offset}), 4), unchecked((float){}));", operands[1], operands[0]),
            Instruction::F64Store { offset } => uwriteln!(self.src, "BitConverter.TryWriteBytes(new Span<byte>((void*)({} + {offset}), 8), unchecked((double){}));", operands[1], operands[0]),

            Instruction::I64FromU64 => results.push(format!("unchecked((long)({}))", operands[0])),
            Instruction::I32FromChar => results.push(format!("((int){})", operands[0])),
            Instruction::I32FromU32 => results.push(format!("unchecked((int)({}))", operands[0])),
            Instruction::U8FromI32 => results.push(format!("((byte){})", operands[0])),
            Instruction::S8FromI32 => results.push(format!("((sbyte){})", operands[0])),
            Instruction::U16FromI32 => results.push(format!("((ushort){})", operands[0])),
            Instruction::S16FromI32 => results.push(format!("((short){})", operands[0])),
            Instruction::U32FromI32 => results.push(format!("unchecked((uint)({}))", operands[0])),
            Instruction::U64FromI64 => results.push(format!("unchecked((ulong)({}))", operands[0])),
            Instruction::CharFromI32 => results.push(format!("unchecked((uint)({}))", operands[0])),

            Instruction::I64FromS64
            | Instruction::I32FromU16
            | Instruction::I32FromS16
            | Instruction::I32FromU8
            | Instruction::I32FromS8
            | Instruction::I32FromS32
            | Instruction::F32FromCoreF32
            | Instruction::CoreF32FromF32
            | Instruction::CoreF64FromF64
            | Instruction::F64FromCoreF64
            | Instruction::S32FromI32
            | Instruction::S64FromI64 => results.push(operands[0].clone()),

            Instruction::Bitcasts { casts } => {
                results.extend(casts.iter().zip(operands).map(|(cast, op)| perform_cast(op, cast)))
            }

            Instruction::I32FromBool => {
                results.push(format!("({} ? 1 : 0)", operands[0]));
            }
            Instruction::BoolFromI32 => results.push(format!("({} != 0)", operands[0])),

            Instruction::FlagsLower {
                flags,
                name: _,
                ty: _,
            } => {
                if flags.flags.len() > 32 {
                    results.push(format!(
                        "unchecked((int)(((long){}) & uint.MaxValue))",
                        operands[0].to_string()
                    ));
                    results.push(format!(
                        "unchecked(((int)((long){} >> 32)))",
                        operands[0].to_string()
                    ));
                } else {
                    results.push(format!("(int){}", operands[0].to_string()));
                }
            }

            Instruction::FlagsLift { flags, name, ty } => {
                let qualified_type_name = format!(
                    "{}{}",
                    self.gen.qualifier(true, ty),
                    name.to_string().to_upper_camel_case()
                );
                if flags.flags.len() > 32 {
                    results.push(format!(
                        "({})(unchecked((uint)({})) | (ulong)(unchecked((uint)({}))) << 32)",
                        qualified_type_name,
                        operands[0].to_string(),
                        operands[1].to_string()
                    ));
                } else {
                    results.push(format!("({})({})", qualified_type_name, operands[0]))
                }
            }

            Instruction::RecordLower { record, .. } => {
                let op = &operands[0];
                for f in record.fields.iter() {
                    results.push(format!("{}.{}", op, f.name.to_csharp_ident()));
                }
            }
            Instruction::RecordLift { ty, name, .. } => {
                let qualified_type_name = format!(
                    "{}{}",
                    self.gen.qualifier(true, ty),
                    name.to_string().to_upper_camel_case()
                );
                let mut result = format!("new {} (\n", qualified_type_name);

                result.push_str(&operands.join(", "));
                result.push_str(")");

                results.push(result);
            }
            Instruction::TupleLift { .. } => {
                let mut result = String::from("(");

                uwriteln!(result, "{}", operands.join(", "));

                result.push_str(")");
                results.push(result);
            }

            Instruction::TupleLower { tuple, ty: _ } => {
                let op = &operands[0];
                match tuple.types.len() {
                    1 => results.push(format!("({})", op)),
                    _ => {
                        for i in 0..tuple.types.len() {
                            results.push(format!("{}.Item{}", op, i + 1));
                        }
                    }
                }
            }

            Instruction::VariantPayloadName => {
                let payload = self.locals.tmp("payload");
                results.push(payload.clone());
                self.payloads.push(payload);
            }

            Instruction::VariantLower {
                variant,
                results: lowered_types,
                ..
            } => self.lower_variant(
                &variant
                    .cases
                    .iter()
                    .map(|case| (case.name.deref(), case.ty))
                    .collect::<Vec<_>>(),
                lowered_types,
                &operands[0],
                results,
            ),

            Instruction::VariantLift { variant, ty, .. } => self.lift_variant(
                &Type::Id(*ty),
                &variant
                    .cases
                    .iter()
                    .map(|case| (case.name.deref(), case.ty))
                    .collect::<Vec<_>>(),
                &operands[0],
                results,
            ),

            Instruction::OptionLower {
                results: lowered_types,
                payload,
                ..
            } => {
                let some = self.blocks.pop().unwrap();
                let none = self.blocks.pop().unwrap();
                let some_payload = self.payloads.pop().unwrap();
                let none_payload = self.payloads.pop().unwrap();

                let lowered = lowered_types
                    .iter()
                    .map(|_| self.locals.tmp("lowered"))
                    .collect::<Vec<_>>();

                results.extend(lowered.iter().cloned());

                let declarations = lowered
                    .iter()
                    .zip(lowered_types.iter())
                    .map(|(lowered, ty)| format!("{} {lowered};", wasm_type(*ty)))
                    .collect::<Vec<_>>()
                    .join("\n");

                let op = &operands[0];

                let nesting = if let Type::Id(id) = payload {
                    matches!(&self.gen.resolve.types[*id].kind, TypeDefKind::Option(_))
                } else {
                    false
                };

                let mut block = |ty: Option<&Type>, Block { body, results, .. }, payload, nesting| {
                    let payload = if let Some(ty) = self.gen.non_empty_type(ty) {
                        let ty = self.gen.type_name_with_qualifier(ty, true);
                        if nesting {
                            format!("var {payload} = {op}.Value;")
                        } else {
                            format!("var {payload} = ({ty}) {op};")
                        }
                    } else {
                        String::new()
                    };

                    let assignments = lowered
                        .iter()
                        .zip(&results)
                        .map(|(lowered, result)| format!("{lowered} = {result};\n"))
                        .collect::<Vec<_>>()
                        .concat();

                    format!(
                        "{payload}
                         {body}
                         {assignments}"
                    )
                };

                let none = block(None, none, none_payload, nesting);
                let some = block(Some(payload), some, some_payload, nesting);

                let test = if nesting {
                    ".HasValue"
                } else {
                    " != null"
                };

                uwrite!(
                    self.src,
                    r#"
                    {declarations}

                    if ({op}{test}) {{
                        {some}
                    }} else {{
                        {none}
                    }}
                    "#
                );
            }

            Instruction::OptionLift { payload, ty } => {
                let some = self.blocks.pop().unwrap();
                let _none = self.blocks.pop().unwrap();

                let ty = self.gen.type_name_with_qualifier(&Type::Id(*ty), true);
                let lifted = self.locals.tmp("lifted");
                let op = &operands[0];

                let nesting = if let Type::Id(id) = payload {
                    matches!(&self.gen.resolve.types[*id].kind, TypeDefKind::Option(_))
                } else {
                    false
                };

                let payload = if self.gen.non_empty_type(Some(*payload)).is_some() {
                    some.results.into_iter().next().unwrap()
                } else {
                    "null".into()
                };

                let some = some.body;

                let (none_value, some_value) = if nesting {
                    (format!("{ty}.None"), format!("new ({payload})"))
                } else {
                    ("null".into(), payload)
                };

                uwrite!(
                    self.src,
                    r#"
                    {ty} {lifted};

                    switch ({op}) {{
                        case 0: {{
                            {lifted} = {none_value};
                            break;
                        }}

                        case 1: {{
                            {some}
                            {lifted} = {some_value};
                            break;
                        }}

                        default: throw new ArgumentException("invalid discriminant: " + ({op}));
                    }}
                    "#
                );

                results.push(lifted);
            }

            Instruction::ResultLower {
                results: lowered_types,
                result,
                ..
            } => self.lower_variant(
                &[("ok", result.ok), ("err", result.err)],
                lowered_types,
                &operands[0],
                results,
            ),

            Instruction::ResultLift { result, ty } => self.lift_variant(
                &Type::Id(*ty),
                &[("ok", result.ok), ("err", result.err)],
                &operands[0],
                results,
            ),

            Instruction::EnumLower { .. } => results.push(format!("(int){}", operands[0])),

            Instruction::EnumLift { ty, .. } => {
                let t = self.gen.type_name_with_qualifier(&Type::Id(*ty), true);
                let op = &operands[0];
                results.push(format!("({}){}", t, op));

                // uwriteln!(
                //    self.src,
                //    "Debug.Assert(Enum.IsDefined(typeof({}), {}));",
                //    t,
                //    op
                // );
            }

            Instruction::ListCanonLower { element, realloc } => {
                let list = &operands[0];
                let (_size, ty) = list_element_info(element);

                match self.gen.direction {
                    Direction::Import => {
                        let buffer: String = self.locals.tmp("buffer");
                        uwrite!(
                            self.src,
                            "
                            void* {buffer} = stackalloc {ty}[({list}).Length];
                            {list}.AsSpan<{ty}>().CopyTo(new Span<{ty}>({buffer}, {list}.Length));
                            "
                        );
                        results.push(format!("(int){buffer}"));
                        results.push(format!("({list}).Length"));
                    }
                    Direction::Export => {
                        let address = self.locals.tmp("address");
                        let buffer = self.locals.tmp("buffer");
                        let gc_handle = self.locals.tmp("gcHandle");
                        let size = self.gen.gen.sizes.size(element).size_wasm32();
                        uwrite!(
                            self.src,
                            "
                            byte[] {buffer} = new byte[({size}) * {list}.Length];
                            Buffer.BlockCopy({list}.ToArray(), 0, {buffer}, 0, ({size}) * {list}.Length);
                            var {gc_handle} = GCHandle.Alloc({buffer}, GCHandleType.Pinned);
                            var {address} = {gc_handle}.AddrOfPinnedObject();
                            "
                        );

                        if realloc.is_none() {
                            self.cleanup.push(Cleanup {
                                address: gc_handle.clone(),
                            });
                        }
                        results.push(format!("((IntPtr)({address})).ToInt32()"));
                        results.push(format!("{list}.Length"));
                    }
                }
            }

            Instruction::ListCanonLift { element, .. } => {
                let (_, ty) = list_element_info(element);
                let array = self.locals.tmp("array");
                let address = &operands[0];
                let length = &operands[1];

                uwrite!(
                    self.src,
                    "
                    var {array} = new {ty}[{length}];
                    new Span<{ty}>((void*)({address}), {length}).CopyTo(new Span<{ty}>({array}));
                    "
                );

                results.push(array);
            }

            Instruction::StringLower { realloc } => {
                let op = &operands[0];
                let interop_string = self.locals.tmp("interopString");
                let result_var = self.locals.tmp("result");
                uwriteln!(
                    self.src,
                    "
                    var {result_var} = {op};
                    IntPtr {interop_string} = InteropString.FromString({result_var}, out int length{result_var});"
                );

                if realloc.is_none() {
                    results.push(format!("{interop_string}.ToInt32()"));
                } else {
                    results.push(format!("{interop_string}.ToInt32()"));
                }
                results.push(format!("length{result_var}"));

                self.gen.gen.needs_interop_string = true;
            }

            Instruction::StringLift { .. } => results.push(format!(
                "Encoding.UTF8.GetString((byte*){}, {})",
                operands[0], operands[1]
            )),

            Instruction::ListLower { element, realloc } => {
                let Block {
                    body,
                    results: block_results,
                    element: block_element,
                    base,
                } = self.blocks.pop().unwrap();
                assert!(block_results.is_empty());

                let list = &operands[0];
                let size = self.gen.gen.sizes.size(element).size_wasm32();
                let ty = self.gen.type_name_with_qualifier(element, true);
                let index = self.locals.tmp("index");

                let buffer: String = self.locals.tmp("buffer");
                let gc_handle = self.locals.tmp("gcHandle");
                let address = self.locals.tmp("address");

                uwrite!(
                    self.src,
                    "
                    byte[] {buffer} = new byte[{size} * {list}.Count];
                    var {gc_handle} = GCHandle.Alloc({buffer}, GCHandleType.Pinned);
                    var {address} = {gc_handle}.AddrOfPinnedObject();

                    for (int {index} = 0; {index} < {list}.Count; ++{index}) {{
                        {ty} {block_element} = {list}[{index}];
                        int {base} = (int){address} + ({index} * {size});
                        {body}
                    }}
                    "
                );

                if realloc.is_none() {
                    self.cleanup.push(Cleanup {
                        address: gc_handle.clone(),
                    });
                }

                results.push(format!("(int){address}"));
                results.push(format!("{list}.Count"));
            }

            Instruction::ListLift { element, .. } => {
                let Block {
                    body,
                    results: block_results,
                    base,
                    ..
                } = self.blocks.pop().unwrap();
                let address = &operands[0];
                let length = &operands[1];
                let array = self.locals.tmp("array");
                let ty = self.gen.type_name_with_qualifier(element, true);
                let size = self.gen.gen.sizes.size(element).size_wasm32();
                let index = self.locals.tmp("index");

                let result = match &block_results[..] {
                    [result] => result,
                    _ => todo!("result count == {}", results.len()),
                };

                uwrite!(
                    self.src,
                    "
                    var {array} = new List<{ty}>({length});
                    for (int {index} = 0; {index} < {length}; ++{index}) {{
                        nint {base} = {address} + ({index} * {size});
                        {body}
                        {array}.Add({result});
                    }}
                    "
                );

                results.push(array);
            }

            Instruction::IterElem { .. } => {
                results.push(self.block_storage.last().unwrap().element.clone())
            }

            Instruction::IterBasePointer => {
                results.push(self.block_storage.last().unwrap().base.clone())
            }

            Instruction::CallWasm { sig, .. } => {
                let assignment = match &sig.results[..] {
                    [_] => {
                        let result = self.locals.tmp("result");
                        let assignment = format!("var {result} = ");
                        results.push(result);
                        assignment
                    }

                    [] => String::new(),

                    _ => unreachable!(),
                };

                let func_name = self.func_name.to_upper_camel_case();

                let operands = operands.join(", ");

                uwriteln!(
                    self.src,
                    "{assignment} {func_name}WasmInterop.wasmImport{func_name}({operands});"
                );
            }

            Instruction::CallInterface { func } => {
                let module = self.gen.name;
                let func_name = self.func_name.to_upper_camel_case();
                let interface_name = CSharp::get_class_name_from_qualified_name(module).1;

                let class_name_root = interface_name
                    .strip_prefix("I")
                    .unwrap()
                    .to_upper_camel_case();

                let mut oper = String::new();

                for (i, param) in operands.iter().enumerate() {
                    if i == 0 && matches!(self.kind, FunctionKind::Method(_)) {
                        continue;
                    }

                    oper.push_str(&format!("({param})"));

                    if i < operands.len() && operands.len() != i + 1 {
                        oper.push_str(", ");
                    }
                }

                match self.kind {
                    FunctionKind::Freestanding | FunctionKind::Static(_) | FunctionKind::Method(_) => {
                        let target = match self.kind {
                            FunctionKind::Static(id) => self.gen.gen.all_resources[id].export_impl_name(),
                            FunctionKind::Method(_) => operands[0].clone(),
                            _ => format!("{class_name_root}Impl")
                        };

                        match func.results.len() {
                            0 => uwriteln!(self.src, "{target}.{func_name}({oper});"),
                            1 => {
                                let ret = self.locals.tmp("ret");
                                let ty = self.gen.type_name_with_qualifier(
                                    func.results.iter_types().next().unwrap(),
                                    true
                                );
                                uwriteln!(self.src, "{ty} {ret};");
                                let mut cases = Vec::with_capacity(self.results.len());
                                let mut oks = Vec::with_capacity(self.results.len());
                                let mut payload_is_void = false;
                                for (index, ty) in self.results.iter().enumerate() {
                                    let TypeDefKind::Result(result) = &self.gen.resolve.types[*ty].kind else {
                                        unreachable!();
                                    };
                                    let err_ty = if let Some(ty) = result.err {
                                        self.gen.type_name_with_qualifier(&ty, true)
                                    } else {
                                        "None".to_owned()
                                    };
                                    let ty = self.gen.type_name_with_qualifier(&Type::Id(*ty), true);
                                    let head = oks.concat();
                                    let tail = oks.iter().map(|_| ")").collect::<Vec<_>>().concat();
                                    cases.push(
                                        format!(
                                            "\
                                            case {index}: {{
                                                ret = {head}{ty}.err(({err_ty}) e.Value){tail};
                                                break;
                                            }}
                                            "
                                        )
                                    );
                                    oks.push(format!("{ty}.ok("));
                                    payload_is_void = result.ok.is_none();
                                }
                                if !self.results.is_empty() {
                                    self.src.push_str("try {\n");
                                }
                                let head = oks.concat();
                                let tail = oks.iter().map(|_| ")").collect::<Vec<_>>().concat();
                                let val = if payload_is_void {
                                    uwriteln!(self.src, "{target}.{func_name}({oper});");
                                    "new None()".to_owned()
                                } else {
                                    format!("{target}.{func_name}({oper})")
                                };
                                uwriteln!(
                                    self.src,
                                    "{ret} = {head}{val}{tail};"
                                );
                                if !self.results.is_empty() {
                                    self.gen.gen.needs_wit_exception = true;
                                    let cases = cases.join("\n");
                                    uwriteln!(
                                        self.src,
                                        r#"}} catch (WitException e) {{
                                            switch (e.NestingLevel) {{
                                                {cases}

                                                default: throw new ArgumentException($"invalid nesting level: {{e.NestingLevel}}");
                                            }}
                                        }}
                                        "#
                                    );
                                }
                                results.push(ret);
                            }
                            _ => {
                                let ret = self.locals.tmp("ret");
                                uwriteln!(
                                    self.src,
                                    "var {ret} = {target}.{func_name}({oper});"
                                );
                                let mut i = 1;
                                for _ in func.results.iter_types() {
                                    results.push(format!("{ret}.Item{i}"));
                                    i += 1;
                                }
                            }
                        }
                    }
                    FunctionKind::Constructor(id) => {
                        let target = self.gen.gen.all_resources[id].export_impl_name();
                        let ret = self.locals.tmp("ret");
                        uwriteln!(self.src, "var {ret} = new {target}({oper});");
                        results.push(ret);
                    }
                }

                for (_,  drop) in &self.resource_drops {
                    uwriteln!(self.src, "{drop}?.Dispose();");
                }
            }

            Instruction::Return { amt: _, func } => {
                for Cleanup { address } in &self.cleanup {
                    uwriteln!(self.src, "{address}.Free();");
                }

                if !matches!((self.gen.direction, self.kind), (Direction::Import, FunctionKind::Constructor(_))) {
                    match func.results.len() {
                        0 => (),
                        1 => {
                            let mut payload_is_void = false;
                            let mut previous = operands[0].clone();
                            let mut vars = Vec::with_capacity(self.results.len());
                            if let Direction::Import = self.gen.direction {
                                for ty in &self.results {
                                    vars.push(previous.clone());
                                    let tmp = self.locals.tmp("tmp");
                                    uwrite!(
                                        self.src,
                                        "\
                                        if ({previous}.IsOk) {{
                                        var {tmp} = {previous}.AsOk;
                                    "
                                    );
                                    previous = tmp;
                                    let TypeDefKind::Result(result) = &self.gen.resolve.types[*ty].kind else {
                                        unreachable!();
                                    };
                                    payload_is_void = result.ok.is_none();
                                }
                            }
                            uwriteln!(self.src, "return {};", if payload_is_void { "" } else { &previous });
                            for (level, var) in vars.iter().enumerate().rev() {
                                self.gen.gen.needs_wit_exception = true;
                                uwrite!(
                                    self.src,
                                    "\
                                    }} else {{
                                        throw new WitException({var}.AsErr!, {level});
                                    }}
                                    "
                                );
                            }
                        }
                        _ => {
                            let results = operands.join(", ");
                            uwriteln!(self.src, "return ({results});")
                        }
                    }

                    // Close all the fixed blocks.
                    for _ in 0..self.fixed {
                        uwriteln!(self.src, "}}");
                    }
                }
            }

            Instruction::Malloc { .. } => unimplemented!(),

            Instruction::GuestDeallocate { .. } => {
                uwriteln!(self.src, r#"Console.WriteLine("TODO: deallocate buffer for indirect parameters");"#);
            }

            Instruction::GuestDeallocateString => {
                uwriteln!(self.src, r#"Console.WriteLine("TODO: deallocate buffer for string");"#);
            }

            Instruction::GuestDeallocateVariant { .. } => {
                uwriteln!(self.src, r#"Console.WriteLine("TODO: deallocate buffer for variant");"#);
            }

            Instruction::GuestDeallocateList { .. } => {
                uwriteln!(self.src, r#"Console.WriteLine("TODO: deallocate buffer for list");"#);
            }

            Instruction::HandleLower {
                handle,
                ..
            } => {
                let (Handle::Own(ty) | Handle::Borrow(ty)) = handle;
                let is_own = matches!(handle, Handle::Own(_));
                let handle = self.locals.tmp("handle");
                let id = dealias(self.gen.resolve, *ty);
                let ResourceInfo { direction, .. } = &self.gen.gen.all_resources[&id];
                let op = &operands[0];

                uwriteln!(self.src, "var {handle} = {op}.Handle;");

                match direction {
                    Direction::Import => {
                        if is_own {
                            uwriteln!(self.src, "{op}.Handle = 0;");
                        }
                    }
                    Direction::Export => {
                        self.gen.gen.needs_rep_table = true;
                        let local_rep = self.locals.tmp("localRep");
			let export_name = self.gen.gen.all_resources[&id].export_impl_name();
                        if is_own {
                            // Note that we set `{op}.Handle` to zero below to ensure that application code doesn't
                            // try to use the instance while the host has ownership.  We'll set it back to non-zero
                            // if and when the host gives ownership back to us.
                            uwriteln!(
                                self.src,
                                "if ({handle} == 0) {{
                                     var {local_rep} = {export_name}.repTable.Add({op});
                                     {handle} = {export_name}.WasmInterop.wasmImportResourceNew({local_rep});
                                 }}
                                 {op}.Handle = 0;
                                 "
                            );
                        } else {
                            uwriteln!(
                                self.src,
                                "if ({handle} == 0) {{
                                     var {local_rep} = {export_name}.repTable.Add({op});
                                     {handle} = {export_name}.WasmInterop.wasmImportResourceNew({local_rep});
                                     {op}.Handle = {handle};
                                 }}"
                            );
                        }
                    }
                }
                results.push(format!("{handle}"));
            }

            Instruction::HandleLift {
                handle,
                ..
            } => {
                let (Handle::Own(ty) | Handle::Borrow(ty)) = handle;
                let is_own = matches!(handle, Handle::Own(_));
                let mut resource = self.locals.tmp("resource");
                let id = dealias(self.gen.resolve, *ty);
                let ResourceInfo { direction, .. } = &self.gen.gen.all_resources[&id];
                let op = &operands[0];

                match direction {
                    Direction::Import => {
			let import_name = self.gen.type_name_with_qualifier(&Type::Id(id), true);

                        if let FunctionKind::Constructor(_) = self.kind {
                            resource = "this".to_owned();
                            uwriteln!(self.src,"{resource}.Handle = {op};");
                        } else {
                            let var = if is_own { "var" } else { "" };
                            uwriteln!(
                                self.src,
                                "{var} {resource} = new {import_name}(new {import_name}.THandle({op}));"
                            );
                        }
                        if !is_own {
                            self.resource_drops.push((import_name, resource.clone()));
                        }
                    }
                    Direction::Export => {
                        self.gen.gen.needs_rep_table = true;

			let export_name = self.gen.gen.all_resources[&id].export_impl_name();
                        if is_own {
                            uwriteln!(
                                self.src,
                                "var {resource} = ({export_name}) {export_name}.repTable.Get\
				     ({export_name}.WasmInterop.wasmImportResourceRep({op}));
                                 {resource}.Handle = {op};"
                            );
                        } else {
                            uwriteln!(self.src, "var {resource} = ({export_name}) {export_name}.repTable.Get({op});");
                        }
                    }
                }
                results.push(resource);
            }
        }
    }

    fn return_pointer(&mut self, size: ArchitectureSize, align: Alignment) -> String {
        let ptr = self.locals.tmp("ptr");

        // Use a stack-based return area for imports, because exports need
        // their return area to be live until the post-return call.
        match self.gen.direction {
            Direction::Import => {
                self.import_return_pointer_area_size =
                    self.import_return_pointer_area_size.max(size);
                self.import_return_pointer_area_align =
                    self.import_return_pointer_area_align.max(align);
                let (array_size, element_type) = dotnet_aligned_array(
                    self.import_return_pointer_area_size,
                    self.import_return_pointer_area_align,
                );
                let ret_area = self.locals.tmp("retArea");
                let ret_area_byte0 = self.locals.tmp("retAreaByte0");
                uwrite!(
                    self.src,
                    "
                    var {2} = new {0}[{1}];
                    fixed ({0}* {3} = &{2}[0])
                    {{
                        var {ptr} = (nint){3};
                    ",
                    element_type,
                    array_size,
                    ret_area,
                    ret_area_byte0
                );
                self.fixed = self.fixed + 1;

                return format!("{ptr}");
            }
            Direction::Export => {
                self.gen.gen.return_area_size = self.gen.gen.return_area_size.max(size);
                self.gen.gen.return_area_align = self.gen.gen.return_area_align.max(align);

                uwrite!(
                    self.src,
                    "
                    var {ptr} = InteropReturnArea.returnArea.AddressOfReturnArea();
                    "
                );
                self.gen.gen.needs_export_return_area = true;

                return format!("{ptr}");
            }
        }
    }

    fn push_block(&mut self) {
        self.block_storage.push(BlockStorage {
            body: mem::take(&mut self.src),
            element: self.locals.tmp("element"),
            base: self.locals.tmp("basePtr"),
            cleanup: mem::take(&mut self.cleanup),
        });
    }

    fn finish_block(&mut self, operands: &mut Vec<String>) {
        let BlockStorage {
            body,
            element,
            base,
            cleanup,
        } = self.block_storage.pop().unwrap();

        if !self.cleanup.is_empty() {
            //self.needs_cleanup_list = true;

            for Cleanup { address } in &self.cleanup {
                uwriteln!(self.src, "{address}.Free();");
            }
        }

        self.cleanup = cleanup;

        self.blocks.push(Block {
            body: mem::replace(&mut self.src, body),
            results: mem::take(operands),
            element: element,
            base: base,
        });
    }

    fn sizes(&self) -> &SizeAlign {
        &self.gen.gen.sizes
    }

    fn is_list_canonical(&self, _resolve: &Resolve, element: &Type) -> bool {
        is_primitive(element)
    }
}

// We cant use "StructLayout.Pack" as dotnet will use the minimum of the type and the "Pack" field,
// so for byte it would always use 1 regardless of the "Pack".
fn dotnet_aligned_array(
    array_size: ArchitectureSize,
    required_alignment: Alignment,
) -> (usize, String) {
    match required_alignment.align_wasm32() {
        1 => {
            return (array_size.size_wasm32(), "byte".to_owned());
        }
        2 => {
            return ((array_size.size_wasm32() + 1) / 2, "ushort".to_owned());
        }
        4 => {
            return ((array_size.size_wasm32() + 3) / 4, "uint".to_owned());
        }
        8 => {
            return ((array_size.size_wasm32() + 7) / 8, "ulong".to_owned());
        }
        _ => todo!("unsupported return_area_align {}", required_alignment),
    }
}

fn perform_cast(op: &String, cast: &Bitcast) -> String {
    match cast {
        Bitcast::I32ToF32 => format!("BitConverter.Int32BitsToSingle({op})"),
        Bitcast::I64ToF32 => format!("BitConverter.Int32BitsToSingle((int){op})"),
        Bitcast::F32ToI32 => format!("BitConverter.SingleToInt32Bits({op})"),
        Bitcast::F32ToI64 => format!("BitConverter.SingleToInt32Bits({op})"),
        Bitcast::I64ToF64 => format!("BitConverter.Int64BitsToDouble({op})"),
        Bitcast::F64ToI64 => format!("BitConverter.DoubleToInt64Bits({op})"),
        Bitcast::I32ToI64 => format!("(long) ({op})"),
        Bitcast::I64ToI32 => format!("(int) ({op})"),
        Bitcast::I64ToP64 => format!("{op}"),
        Bitcast::P64ToI64 => format!("{op}"),
        Bitcast::LToI64 | Bitcast::PToP64 => format!("(long) ({op})"),
        Bitcast::I64ToL | Bitcast::P64ToP => format!("(int) ({op})"),
        Bitcast::I32ToP
        | Bitcast::PToI32
        | Bitcast::I32ToL
        | Bitcast::LToI32
        | Bitcast::LToP
        | Bitcast::PToL
        | Bitcast::None => op.to_owned(),
        Bitcast::Sequence(sequence) => {
            let [first, second] = &**sequence;
            perform_cast(&perform_cast(op, first), second)
        }
    }
}

fn int_type(int: Int) -> &'static str {
    match int {
        Int::U8 => "byte",
        Int::U16 => "ushort",
        Int::U32 => "uint",
        Int::U64 => "ulong",
    }
}

fn wasm_type(ty: WasmType) -> &'static str {
    match ty {
        WasmType::I32 => "int",
        WasmType::I64 => "long",
        WasmType::F32 => "float",
        WasmType::F64 => "double",
        WasmType::Pointer => "nint",
        WasmType::PointerOrI64 => "long",
        WasmType::Length => "int",
    }
}

fn list_element_info(ty: &Type) -> (usize, &'static str) {
    match ty {
        Type::S8 => (1, "sbyte"),
        Type::S16 => (2, "short"),
        Type::S32 => (4, "int"),
        Type::S64 => (8, "long"),
        Type::U8 => (1, "byte"),
        Type::U16 => (2, "ushort"),
        Type::U32 => (4, "uint"),
        Type::U64 => (8, "ulong"),
        Type::F32 => (4, "float"),
        Type::F64 => (8, "double"),
        _ => unreachable!(),
    }
}

fn indent(code: &str) -> String {
    let mut indented = String::with_capacity(code.len());
    let mut indent = 0;
    let mut was_empty = false;
    for line in code.trim().lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if was_empty {
                continue;
            }
            was_empty = true;
        } else {
            was_empty = false;
        }

        if trimmed.starts_with('}') {
            indent -= 1;
        }
        if !trimmed.is_empty() {
            indented.extend(iter::repeat(' ').take(indent * 4));
            indented.push_str(trimmed);
        }
        if trimmed.ends_with('{') {
            indent += 1;
        }
        indented.push('\n');
    }
    indented
}

fn interface_name(
    csharp: &mut CSharp,
    resolve: &Resolve,
    name: &WorldKey,
    direction: Direction,
) -> String {
    let pkg = match name {
        WorldKey::Name(_) => None,
        WorldKey::Interface(id) => {
            let pkg = resolve.interfaces[*id].package.unwrap();
            Some(resolve.packages[pkg].name.clone())
        }
    };

    let name = match name {
        WorldKey::Name(name) => name.to_upper_camel_case(),
        WorldKey::Interface(id) => resolve.interfaces[*id]
            .name
            .as_ref()
            .unwrap()
            .to_upper_camel_case(),
    };

    let namespace = match &pkg {
        Some(name) => {
            let mut ns = format!(
                "{}.{}.",
                name.namespace.to_csharp_ident(),
                name.name.to_csharp_ident()
            );

            if let Some(version) = &name.version {
                let v = version
                    .to_string()
                    .replace('.', "_")
                    .replace('-', "_")
                    .replace('+', "_");
                ns = format!("{}v{}.", ns, &v);
            }
            ns
        }
        None => String::new(),
    };

    let world_namespace = &csharp.qualifier();

    format!(
        "{}wit.{}.{}I{name}",
        world_namespace,
        match direction {
            Direction::Import => "imports",
            Direction::Export => "exports",
        },
        namespace
    )
}

fn is_primitive(ty: &Type) -> bool {
    matches!(
        ty,
        Type::U8
            | Type::S8
            | Type::U16
            | Type::S16
            | Type::U32
            | Type::S32
            | Type::U64
            | Type::S64
            | Type::F32
            | Type::F64
    )
}

trait ToCSharpIdent: ToOwned {
    fn to_csharp_ident(&self) -> Self::Owned;
}

impl ToCSharpIdent for str {
    fn to_csharp_ident(&self) -> String {
        // Escape C# keywords
        // Source: https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/

        //TODO: Repace with actual keywords
        match self {
            "abstract" | "as" | "base" | "bool" | "break" | "byte" | "case" | "catch" | "char"
            | "checked" | "class" | "const" | "continue" | "decimal" | "default" | "delegate"
            | "do" | "double" | "else" | "enum" | "event" | "explicit" | "extern" | "false"
            | "finally" | "fixed" | "float" | "for" | "foreach" | "goto" | "if" | "implicit"
            | "in" | "int" | "interface" | "internal" | "is" | "lock" | "long" | "namespace"
            | "new" | "null" | "object" | "operator" | "out" | "override" | "params"
            | "private" | "protected" | "public" | "readonly" | "ref" | "return" | "sbyte"
            | "sealed" | "short" | "sizeof" | "stackalloc" | "static" | "string" | "struct"
            | "switch" | "this" | "throw" | "true" | "try" | "typeof" | "uint" | "ulong"
            | "unchecked" | "unsafe" | "ushort" | "using" | "virtual" | "void" | "volatile"
            | "while" => format!("@{self}"),
            _ => self.to_lower_camel_case(),
        }
    }
}

/// Group the specified functions by resource (or `None` for freestanding functions).
///
/// The returned map is constructed by iterating over `funcs`, then iterating over `all_resources`, thereby
/// ensuring that even resources with no associated functions will be represented in the result.
fn by_resource<'a>(
    funcs: impl Iterator<Item = (&'a str, &'a Function)>,
    all_resources: impl Iterator<Item = TypeId>,
) -> IndexMap<Option<TypeId>, Vec<&'a Function>> {
    let mut by_resource = IndexMap::<_, Vec<_>>::new();
    for (_, func) in funcs {
        by_resource
            .entry(match &func.kind {
                FunctionKind::Freestanding => None,
                FunctionKind::Method(resource)
                | FunctionKind::Static(resource)
                | FunctionKind::Constructor(resource) => Some(*resource),
            })
            .or_default()
            .push(func);
    }
    for id in all_resources {
        by_resource.entry(Some(id)).or_default();
    }
    by_resource
}

/// Dereference any number `TypeDefKind::Type` aliases to retrieve the target type.
fn dealias(resolve: &Resolve, mut id: TypeId) -> TypeId {
    loop {
        match &resolve.types[id].kind {
            TypeDefKind::Type(Type::Id(that_id)) => id = *that_id,
            _ => break id,
        }
    }
}

fn payload_and_results(resolve: &Resolve, ty: Type) -> (Option<Type>, Vec<TypeId>) {
    fn recurse(resolve: &Resolve, ty: Type, results: &mut Vec<TypeId>) -> Option<Type> {
        if let Type::Id(id) = ty {
            if let TypeDefKind::Result(result) = &resolve.types[id].kind {
                results.push(id);
                if let Some(ty) = result.ok {
                    recurse(resolve, ty, results)
                } else {
                    None
                }
            } else {
                Some(ty)
            }
        } else {
            Some(ty)
        }
    }

    let mut results = Vec::new();
    let payload = recurse(resolve, ty, &mut results);
    (payload, results)
}

fn extra_modifiers(func: &Function, name: &str) -> &'static str {
    if let FunctionKind::Method(_) = &func.kind {
        // Avoid warnings about name clashes.
        //
        // TODO: add other `object` method names here
        if name == "GetType" {
            return "new";
        }
    }

    ""
}
