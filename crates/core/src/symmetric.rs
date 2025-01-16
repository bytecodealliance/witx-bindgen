// helper functions for symmetric ABI

use wit_parser::{Resolve, Type, TypeDefKind};

// figure out whether deallocation is needed in the caller
fn needs_dealloc2(resolve: &Resolve, tp: &Type) -> bool {
    match tp {
        Type::Bool
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::S8
        | Type::S16
        | Type::S32
        | Type::S64
        | Type::F32
        | Type::F64
        | Type::Char => false,
        Type::String => true,
        Type::Id(id) => match &resolve.types[*id].kind {
            TypeDefKind::Enum(_) => false,
            TypeDefKind::Record(r) => r.fields.iter().any(|f| needs_dealloc2(resolve, &f.ty)),
            TypeDefKind::Resource => false,
            TypeDefKind::Handle(_) => false,
            TypeDefKind::Flags(_) => false,
            TypeDefKind::Tuple(t) => t.types.iter().any(|f| needs_dealloc2(resolve, f)),
            TypeDefKind::Variant(_) => todo!(),
            TypeDefKind::Option(tp) => needs_dealloc2(resolve, tp),
            TypeDefKind::Result(r) => {
                r.ok.as_ref()
                    .map_or(false, |tp| needs_dealloc2(resolve, tp))
                    || r.err
                        .as_ref()
                        .map_or(false, |tp| needs_dealloc2(resolve, tp))
            }
            TypeDefKind::List(_l) => true,
            TypeDefKind::Future(_) => todo!(),
            TypeDefKind::Stream(_) => todo!(),
            TypeDefKind::ErrorContext => false,
            TypeDefKind::Type(tp) => needs_dealloc2(resolve, tp),
            TypeDefKind::Unknown => false,
        },
    }
}

pub fn needs_dealloc(resolve: &Resolve, args: &[(String, Type)]) -> bool {
    for (_n, t) in args {
        if needs_dealloc2(resolve, t) {
            return true;
        }
    }
    return false;
}

fn has_non_canonical_list2(resolve: &Resolve, ty: &Type, maybe: bool) -> bool {
    match ty {
        Type::Bool
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::S8
        | Type::S16
        | Type::S32
        | Type::S64
        | Type::F32
        | Type::F64
        | Type::Char
        | Type::String => false,
        Type::Id(id) => match &resolve.types[*id].kind {
            TypeDefKind::Record(r) => r
                .fields
                .iter()
                .any(|field| has_non_canonical_list2(resolve, &field.ty, maybe)),
            TypeDefKind::Resource | TypeDefKind::Handle(_) | TypeDefKind::Flags(_) => false,
            TypeDefKind::Tuple(t) => t
                .types
                .iter()
                .any(|ty| has_non_canonical_list2(resolve, ty, maybe)),
            TypeDefKind::Variant(var) => var.cases.iter().any(|case| {
                case.ty
                    .as_ref()
                    .map_or(false, |ty| has_non_canonical_list2(resolve, ty, maybe))
            }),
            TypeDefKind::Enum(_) => false,
            TypeDefKind::Option(ty) => has_non_canonical_list2(resolve, ty, maybe),
            TypeDefKind::Result(res) => {
                res.ok
                    .as_ref()
                    .map_or(false, |ty| has_non_canonical_list2(resolve, ty, maybe))
                    || res
                        .err
                        .as_ref()
                        .map_or(false, |ty| has_non_canonical_list2(resolve, ty, maybe))
            }
            TypeDefKind::List(ty) => {
                if maybe {
                    true
                } else {
                    has_non_canonical_list2(resolve, ty, true)
                }
            }
            TypeDefKind::Future(_) | TypeDefKind::Stream(_) | TypeDefKind::ErrorContext => false,
            TypeDefKind::Type(ty) => has_non_canonical_list2(resolve, ty, maybe),
            TypeDefKind::Unknown => false,
        },
    }
}

// fn has_non_canonical_list(resolve: &Resolve, results: &Results) -> bool {
//     match results {
//         Results::Named(vec) => vec
//             .iter()
//             .any(|(_, ty)| has_non_canonical_list2(resolve, ty, false)),
//         Results::Anon(one) => has_non_canonical_list2(resolve, &one, false),
//     }
// }

pub fn has_non_canonical_list(resolve: &Resolve, args: &[(String, Type)]) -> bool {
    args.iter()
        .any(|(_, ty)| has_non_canonical_list2(resolve, ty, false))
}

fn has_non_canonical_list_rust2(resolve: &Resolve, ty: &Type) -> bool {
    match ty {
        Type::Bool
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::S8
        | Type::S16
        | Type::S32
        | Type::S64
        | Type::F32
        | Type::F64
        | Type::Char
        | Type::String => false,
        Type::Id(id) => match &resolve.types[*id].kind {
            TypeDefKind::Record(r) => r
                .fields
                .iter()
                .any(|field| has_non_canonical_list_rust2(resolve, &field.ty)),
            TypeDefKind::Resource | TypeDefKind::Handle(_) | TypeDefKind::Flags(_) => false,
            TypeDefKind::Tuple(t) => t
                .types
                .iter()
                .any(|ty| has_non_canonical_list_rust2(resolve, ty)),
            TypeDefKind::Variant(var) => var.cases.iter().any(|case| {
                case.ty
                    .as_ref()
                    .map_or(false, |ty| has_non_canonical_list_rust2(resolve, ty))
            }),
            TypeDefKind::Enum(_) => false,
            TypeDefKind::Option(ty) => has_non_canonical_list_rust2(resolve, ty),
            TypeDefKind::Result(res) => {
                res.ok
                    .as_ref()
                    .map_or(false, |ty| has_non_canonical_list_rust2(resolve, ty))
                    || res
                        .err
                        .as_ref()
                        .map_or(false, |ty| has_non_canonical_list_rust2(resolve, ty))
            }
            TypeDefKind::List(_ty) => true,
            TypeDefKind::Future(_) | TypeDefKind::Stream(_) | TypeDefKind::ErrorContext => false,
            TypeDefKind::Type(ty) => has_non_canonical_list_rust2(resolve, ty),
            TypeDefKind::Unknown => false,
        },
    }
}

pub fn has_non_canonical_list_rust(resolve: &Resolve, args: &[(String, Type)]) -> bool {
    args.iter()
        .any(|(_, ty)| has_non_canonical_list_rust2(resolve, ty))
}
