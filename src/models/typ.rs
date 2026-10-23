use super::assembly::AsmType;
use la_arena::{Arena, Idx};
use rustc_hash::FxHashMap;
use std::fmt;

pub type TypeId = Idx<TypeKind>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Int,
    Long,
    UInt,
    ULong,
    Double,
    Pointer(TypeId),
    Func(Vec<TypeId>, TypeId),
    Array(TypeId, usize),
}

pub struct Types {
    arena: Arena<TypeKind>,
    interned: FxHashMap<TypeKind, TypeId>,
    pub int: TypeId,
    pub long: TypeId,
    pub uint: TypeId,
    pub ulong: TypeId,
    pub double: TypeId,
}

impl Types {
    pub fn new() -> Self {
        let mut arena = Arena::new();
        let mut interned = FxHashMap::default();
        let mut alloc = |kind: TypeKind| {
            let typ = arena.alloc(kind.clone());
            interned.insert(kind, typ);
            typ
        };
        let int = alloc(TypeKind::Int);
        let long = alloc(TypeKind::Long);
        let uint = alloc(TypeKind::UInt);
        let ulong = alloc(TypeKind::ULong);
        let double = alloc(TypeKind::Double);
        Self {
            arena,
            interned,
            int,
            long,
            uint,
            ulong,
            double,
        }
    }

    pub fn intern(&mut self, kind: TypeKind) -> TypeId {
        if let Some(&typ) = self.interned.get(&kind) {
            return typ;
        }
        let typ = self.arena.alloc(kind.clone());
        self.interned.insert(kind, typ);
        typ
    }

    pub fn kind(&self, typ: TypeId) -> &TypeKind {
        &self.arena[typ]
    }

    pub fn size(&self, typ: TypeId) -> Option<usize> {
        use TypeKind::*;
        match self.kind(typ) {
            Int | UInt => Some(4),
            Long | ULong | Double | Pointer(_) => Some(8),
            Array(element, len) => self.size(*element)?.checked_mul(*len),
            Func(_, _) => None,
        }
    }

    pub fn alignment(&self, typ: TypeId) -> Option<usize> {
        use TypeKind::*;
        match self.kind(typ) {
            Int | UInt => Some(4),
            Long | ULong | Double | Pointer(_) => Some(8),
            Array(element, _) => {
                if self.size(typ)? >= 16 {
                    Some(16)
                } else {
                    self.alignment(*element)
                }
            }
            Func(_, _) => None,
        }
    }

    pub fn pointer_scale(&self, typ: TypeId) -> usize {
        let TypeKind::Pointer(element) = self.kind(typ) else {
            unreachable!()
        };
        self.size(*element).unwrap()
    }

    pub fn is_signed(&self, typ: TypeId) -> bool {
        use TypeKind::*;
        match self.kind(typ) {
            Int | Long => true,
            UInt | ULong | Pointer(_) | Double => false,
            _ => unreachable!(),
        }
    }

    pub fn is_pointer(&self, typ: TypeId) -> bool {
        matches!(self.kind(typ), TypeKind::Pointer(_))
    }

    pub fn is_integer(&self, typ: TypeId) -> bool {
        matches!(
            self.kind(typ),
            TypeKind::Int | TypeKind::Long | TypeKind::UInt | TypeKind::ULong
        )
    }

    pub fn is_arithmetic(&self, typ: TypeId) -> bool {
        self.is_integer(typ) || typ == self.double
    }

    pub fn is_array(&self, typ: TypeId) -> bool {
        matches!(self.kind(typ), TypeKind::Array(_, _))
    }

    pub fn to_asm_type(&self, typ: TypeId) -> AsmType {
        use TypeKind::*;
        match self.kind(typ) {
            Int | UInt => AsmType::Longword,
            Long | ULong | Pointer(_) => AsmType::Quadword,
            Double => AsmType::Double,
            Array(_, _) => {
                AsmType::ByteArray(self.size(typ).unwrap(), self.alignment(typ).unwrap())
            }
            Func(_, _) => unreachable!(),
        }
    }

    pub fn display(&self, typ: TypeId) -> TypeDisplay<'_> {
        TypeDisplay { types: self, typ }
    }
}

pub struct TypeDisplay<'a> {
    types: &'a Types,
    typ: TypeId,
}

impl fmt::Display for TypeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_declarator(f, self.typ, String::new())
    }
}

impl TypeDisplay<'_> {
    fn write_declarator(
        &self,
        f: &mut fmt::Formatter<'_>,
        typ: TypeId,
        declarator: String,
    ) -> fmt::Result {
        use TypeKind::*;
        let types = self.types;
        let base = match types.kind(typ) {
            Int => "int",
            Long => "long",
            UInt => "unsigned int",
            ULong => "unsigned long",
            Double => "double",
            Pointer(element) => {
                let declarator = if matches!(types.kind(*element), Array(..) | Func(..)) {
                    format!("(*{declarator})")
                } else {
                    format!("*{declarator}")
                };
                return self.write_declarator(f, *element, declarator);
            }
            Array(element, len) => {
                return self.write_declarator(f, *element, format!("{declarator}[{len}]"));
            }
            Func(params, ret) => {
                let params = if params.is_empty() {
                    "void".to_string()
                } else {
                    params
                        .iter()
                        .map(|param| types.display(*param).to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                return self.write_declarator(f, *ret, format!("{declarator}({params})"));
            }
        };
        if declarator.is_empty() {
            write!(f, "{base}")
        } else {
            write!(f, "{base} {declarator}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_types_share_ids() {
        let mut types = Types::new();
        assert_eq!(types.intern(TypeKind::Int), types.int);

        let ptr = types.intern(TypeKind::Pointer(types.int));
        assert_eq!(ptr, types.intern(TypeKind::Pointer(types.int)));
        assert_ne!(ptr, types.intern(TypeKind::Pointer(types.long)));

        let array = types.intern(TypeKind::Array(ptr, 3));
        assert_eq!(array, types.intern(TypeKind::Array(ptr, 3)));
        assert_ne!(array, types.intern(TypeKind::Array(ptr, 4)));

        let func = types.intern(TypeKind::Func(vec![ptr, array], types.long));
        assert_eq!(
            func,
            types.intern(TypeKind::Func(vec![ptr, array], types.long))
        );
        assert_ne!(
            func,
            types.intern(TypeKind::Func(vec![array, ptr], types.long))
        );
        assert_ne!(
            func,
            types.intern(TypeKind::Func(vec![ptr, array], types.int))
        );
    }

    #[test]
    fn ids_survive_arena_growth() {
        let mut types = Types::new();
        let ptr = types.intern(TypeKind::Pointer(types.int));
        for len in 1..1024 {
            types.intern(TypeKind::Array(types.long, len));
        }
        assert_eq!(types.kind(ptr), &TypeKind::Pointer(types.int));
        assert_eq!(ptr, types.intern(TypeKind::Pointer(types.int)));
    }

    #[test]
    fn nested_array_layout_uses_element_types() {
        let mut types = Types::new();
        let row = types.intern(TypeKind::Array(types.int, 3));
        let matrix = types.intern(TypeKind::Array(row, 2));
        let ptr = types.intern(TypeKind::Pointer(row));
        assert_eq!(types.size(row), Some(12));
        assert_eq!(types.alignment(row), Some(4));
        assert_eq!(types.size(matrix), Some(24));
        assert_eq!(types.alignment(matrix), Some(16));
        assert_eq!(types.to_asm_type(matrix), AsmType::ByteArray(24, 16));
        assert_eq!(types.size(ptr), Some(8));
        assert_eq!(types.pointer_scale(ptr), 12);

        let oversized = types.intern(TypeKind::Array(row, usize::MAX));
        assert_eq!(types.size(oversized), None);
        assert_eq!(types.alignment(oversized), None);
        let func = types.intern(TypeKind::Func(vec![], types.int));
        assert_eq!(types.size(func), None);
        assert_eq!(types.alignment(func), None);
    }

    #[test]
    fn display_resolves_nested_types() {
        let mut types = Types::new();
        let array = types.intern(TypeKind::Array(types.int, 3));
        let ptr = types.intern(TypeKind::Pointer(array));
        let int_ptr = types.intern(TypeKind::Pointer(types.int));
        let ptr_array = types.intern(TypeKind::Array(int_ptr, 3));
        let matrix = types.intern(TypeKind::Array(array, 2));
        let func = types.intern(TypeKind::Func(vec![ptr, types.double], types.long));
        let func_ptr = types.intern(TypeKind::Pointer(func));
        for (typ, expected) in [
            (types.uint, "unsigned int"),
            (types.ulong, "unsigned long"),
            (array, "int [3]"),
            (ptr, "int (*)[3]"),
            (ptr_array, "int *[3]"),
            (matrix, "int [2][3]"),
            (func, "long (int (*)[3], double)"),
            (func_ptr, "long (*)(int (*)[3], double)"),
        ] {
            assert_eq!(types.display(typ).to_string(), expected);
        }
    }
}
