use wit_bindgen_core::wit_parser::{Function, Resolve, Results, Type, TypeDefKind};

#[derive(Debug, Default)]
pub struct WamrSig {
    wamr_types: String,
    wamr_result: String,
}

impl ToString for WamrSig {
    fn to_string(&self) -> String {
        "(".to_string() + &self.wamr_types + ")" + &self.wamr_result
    }
}

fn push_wamr(ty: &Type, resolve: &Resolve, params_str: &mut String) {
    match ty {
        Type::Bool
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::S8
        | Type::S16
        | Type::S32
        | Type::Char => {
            params_str.push('i');
        }
        Type::U64 | Type::S64 => {
            params_str.push('I');
        }
        Type::F32 => {
            params_str.push('f');
        }
        Type::F64 => {
            params_str.push('F');
        }
        Type::String => {
            params_str.push_str("$~");
        }
        Type::Id(id) => match &resolve.types[*id].kind {
            TypeDefKind::Type(t) => push_wamr(t, resolve, params_str),
            TypeDefKind::Record(_r) => {
                params_str.push_str("R");
            }
            TypeDefKind::Flags(_) => params_str.push_str("L"),
            TypeDefKind::Tuple(_) => params_str.push_str("T"),
            TypeDefKind::Variant(_) => params_str.push_str("V"),
            TypeDefKind::Enum(_e) => {
                params_str.push_str("i");
            }
            TypeDefKind::Option(_) => params_str.push_str("O"),
            TypeDefKind::Result(_) => params_str.push_str("R"),
            TypeDefKind::List(_t) => {
                params_str.push_str("*~");
            }
            TypeDefKind::Future(_) => todo!(),
            TypeDefKind::Stream(_) => todo!(),
            TypeDefKind::Unknown => todo!(),
            TypeDefKind::Resource => {
                params_str.push('i');
            }
            TypeDefKind::Handle(_h) => {
                params_str.push('i');
            }
            TypeDefKind::ErrorContext => todo!(),
        },
    }
}

fn wamr_add_result(sig: &mut WamrSig, resolve: &Resolve, ty: &Type) {
    match ty {
        Type::Bool
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::S8
        | Type::S16
        | Type::S32
        | Type::Char => {
            sig.wamr_result = "i".into();
        }
        Type::S64 | Type::U64 => {
            sig.wamr_result = "I".into();
        }
        Type::F32 => {
            sig.wamr_result = "f".into();
        }
        Type::F64 => {
            sig.wamr_result = "F".into();
        }
        Type::String => {
            sig.wamr_types.push('*');
        }
        Type::Id(id) => match &resolve.types[*id].kind {
            TypeDefKind::Record(_r) => sig.wamr_types.push('R'),
            TypeDefKind::Flags(fl) => {
                sig.wamr_types
                    .push(if fl.flags.len() > 32 { 'I' } else { 'i' })
            }
            TypeDefKind::Tuple(_) => sig.wamr_result.push('i'),
            TypeDefKind::Variant(_) => {
                sig.wamr_types.push('*');
            }
            TypeDefKind::Enum(_e) => {
                sig.wamr_types.push('*');
            }
            TypeDefKind::Option(_o) => {
                sig.wamr_types.push('*');
            }
            TypeDefKind::Result(_) => {
                sig.wamr_types.push('*');
            }
            TypeDefKind::List(_) => {
                sig.wamr_types.push('*');
            }
            TypeDefKind::Future(_) => todo!(),
            TypeDefKind::Stream(_) => todo!(),
            TypeDefKind::Type(ty) => wamr_add_result(sig, resolve, &ty),
            TypeDefKind::Unknown => todo!(),
            TypeDefKind::Resource => {
                // resource-rep is returning a pointer
                // perhaps i???
                sig.wamr_result = "*".into();
            }
            TypeDefKind::Handle(_h) => {
                sig.wamr_result = "i".into();
            }
            TypeDefKind::ErrorContext => todo!(),
        },
    }
}

pub fn wamr_signature(resolve: &Resolve, func: &Function) -> WamrSig {
    let mut result = WamrSig::default();
    for (_name, param) in func.params.iter() {
        push_wamr(param, resolve, &mut result.wamr_types);
    }
    match &func.results {
        Results::Named(p) => {
            if !p.is_empty() {
                // assume a pointer
                result.wamr_types.push('*');
            }
        }
        Results::Anon(e) => wamr_add_result(&mut result, resolve, e),
    }
    result
}
